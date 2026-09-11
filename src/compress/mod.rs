mod options;
mod report;

pub use options::CompressionMode;
pub use report::CompressionReport;

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader};

use crate::cli::{Args, QualityLevel};
use crate::encode::{encode_image, optimize_png_bytes};
use crate::output::{OutputFormat, resolve_output_path};

#[derive(Debug)]
pub struct ImageMetadata {
    pub icc_profile: Option<Vec<u8>>,
    pub exif: Option<Vec<u8>>,
    pub orientation: image::metadata::Orientation,
}

impl Default for ImageMetadata {
    fn default() -> Self {
        Self {
            icc_profile: None,
            exif: None,
            orientation: image::metadata::Orientation::NoTransforms,
        }
    }
}

pub fn compress_image(args: &Args) -> Result<CompressionReport> {
    let input_path = &args.input;
    if !input_path.is_file() {
        bail!("输入路径不是有效文件 {}", input_path.display());
    }

    let input_data = fs::read(input_path)
        .with_context(|| format!("无法读取输入文件 {}", input_path.display()))?;
    let input_size = input_data.len() as u64;

    let source_format = image::guess_format(&input_data)
        .with_context(|| format!("无法识别图片格式 {}", input_path.display()))?;
    ensure_static_image(&input_data, source_format)?;

    let format_path = args
        .output
        .as_deref()
        .filter(|path| path.extension().is_some())
        .unwrap_or(input_path);
    let output_format = OutputFormat::detect(Some(format_path), source_format)?;
    let output_path = resolve_output_path(input_path, args.output.as_deref(), output_format);

    let (mut image, mut metadata) = decode_image(&input_data, source_format, input_path)?;
    apply_orientation(&mut image, &mut metadata);

    let mode = CompressionMode::from_args(args, input_size)?;
    let result = match mode {
        CompressionMode::Quality(quality) => {
            if output_format == OutputFormat::Png
                && source_format == ImageFormat::Png
                && quality.is_lossless()
            {
                optimize_png_bytes(&input_data, quality)
            } else {
                encode_image(&image, output_format, quality, &metadata)?
            }
        }
        CompressionMode::MaxSize(limit) => {
            fit_to_size_limit(&image, output_format, limit, &metadata)?
        }
    };

    // 只有输出格式与输入格式一致时 才能直接用原始字节兜底 否则会写出错误的文件格式
    let can_reuse_input = OutputFormat::from_image_format(source_format) == Some(output_format);
    let final_data = match mode {
        CompressionMode::MaxSize(limit) if can_reuse_input && input_size <= limit => {
            input_data.clone()
        }
        CompressionMode::MaxSize(limit) => {
            if result.len() as u64 > limit {
                bail!("无法在可接受画质下压缩到目标体积 {}", limit);
            }
            result
        }
        CompressionMode::Quality(_) if can_reuse_input => keep_smaller(&input_data, result),
        CompressionMode::Quality(_) => result,
    };

    if let Some(parent) = output_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("无法创建输出目录 {}", parent.display()))?;
    }

    fs::write(&output_path, &final_data)
        .with_context(|| format!("无法写入输出文件 {}", output_path.display()))?;

    Ok(CompressionReport {
        input_path: input_path.clone(),
        output_path,
        input_size,
        output_size: final_data.len() as u64,
    })
}

fn ensure_static_image(input_data: &[u8], format: ImageFormat) -> Result<()> {
    match format {
        ImageFormat::WebP => {
            if webp::BitstreamFeatures::new(input_data)
                .is_some_and(|features| features.has_animation())
            {
                bail!("暂不支持压缩动态 WebP 图片");
            }
        }
        ImageFormat::Gif => bail!("暂不支持压缩 GIF 图片"),
        _ => {}
    }

    Ok(())
}

fn decode_image(
    input_data: &[u8],
    format: ImageFormat,
    input_path: &Path,
) -> Result<(DynamicImage, ImageMetadata)> {
    let cursor = std::io::Cursor::new(input_data);
    let mut decoder = ImageReader::with_format(cursor, format)
        .into_decoder()
        .with_context(|| format!("无法创建解码器 {}", input_path.display()))?;

    let icc_profile = decoder.icc_profile().ok().flatten();
    let mut exif = decoder.exif_metadata().ok().flatten();
    let orientation = decoder
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);

    if let Some(exif_data) = exif.as_mut() {
        let _ = image::metadata::Orientation::remove_from_exif_chunk(exif_data);
    }

    let image = DynamicImage::from_decoder(decoder)
        .with_context(|| format!("无法解码图片 {}", input_path.display()))?;

    Ok((
        image,
        ImageMetadata {
            icc_profile,
            exif,
            orientation,
        },
    ))
}

fn apply_orientation(image: &mut DynamicImage, metadata: &mut ImageMetadata) {
    if metadata.orientation != image::metadata::Orientation::NoTransforms {
        image.apply_orientation(metadata.orientation);
        metadata.orientation = image::metadata::Orientation::NoTransforms;
    }
}

fn keep_smaller(original: &[u8], compressed: Vec<u8>) -> Vec<u8> {
    if compressed.len() < original.len() {
        compressed
    } else {
        original.to_vec()
    }
}

fn fit_to_size_limit(
    image: &DynamicImage,
    format: OutputFormat,
    limit: u64,
    metadata: &ImageMetadata,
) -> Result<Vec<u8>> {
    if format == OutputFormat::Png {
        return fit_png_to_size_limit(image, limit, metadata);
    }

    let mut best: Option<Vec<u8>> = None;
    for quality in (10..=95).rev().filter(|quality| quality % 5 == 0) {
        let encoded = crate::encode::encode_with_numeric_quality(image, format, quality, metadata)?;
        if encoded.len() as u64 <= limit {
            return Ok(encoded);
        }
        best = Some(encoded);
    }

    if let Some(best) = best
        && best.len() as u64 <= limit
    {
        return Ok(best);
    }

    resize_to_limit(image, format, limit, metadata)
}

fn fit_png_to_size_limit(
    image: &DynamicImage,
    limit: u64,
    metadata: &ImageMetadata,
) -> Result<Vec<u8>> {
    // 先尝试无损优化 只有在无法满足体积目标时才退化为调色板量化
    let lossless = crate::encode::encode_png_lossless(image, 6, metadata)?;
    if lossless.len() as u64 <= limit {
        return Ok(lossless);
    }

    let mut best = None;
    for max_colors in [256, 128, 64, 32, 16, 8, 4, 2] {
        let encoded = crate::encode::encode_png_quantized(image, max_colors, 50, metadata)?;
        if encoded.len() as u64 <= limit {
            return Ok(encoded);
        }
        best = Some(encoded);
    }

    if let Some(best) = best {
        let scale = (limit as f64 / best.len() as f64).sqrt().clamp(0.1, 0.98);
        let resized = resize(image, scale);
        return crate::encode::encode_png_quantized(&resized, 64, 40, metadata);
    }

    encode_image(image, OutputFormat::Png, QualityLevel::Lowest, metadata)
}

fn resize_to_limit(
    image: &DynamicImage,
    format: OutputFormat,
    limit: u64,
    metadata: &ImageMetadata,
) -> Result<Vec<u8>> {
    let mut current = image.clone();

    for _ in 0..8 {
        let encoded = crate::encode::encode_with_numeric_quality(&current, format, 30, metadata)?;
        if encoded.len() as u64 <= limit {
            return Ok(encoded);
        }

        let scale = (limit as f64 / encoded.len() as f64)
            .sqrt()
            .clamp(0.4, 0.95);
        current = resize(&current, scale);
        if current.width() == 1 && current.height() == 1 {
            break;
        }
    }

    let encoded = crate::encode::encode_with_numeric_quality(&current, format, 10, metadata)?;
    Ok(encoded)
}

fn resize(image: &DynamicImage, scale: f64) -> DynamicImage {
    let width = ((image.width() as f64 * scale).round() as u32).max(1);
    let height = ((image.height() as f64 * scale).round() as u32).max(1);
    image.resize(width, height, image::imageops::FilterType::Lanczos3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_original_when_compressed_is_not_smaller() {
        let original = vec![1, 2, 3, 4];
        let compressed = vec![1, 2, 3, 4, 5];
        assert_eq!(keep_smaller(&original, compressed), original);
    }
}
