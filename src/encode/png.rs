use std::borrow::Cow;

use anyhow::{Context, Result};
use image::DynamicImage;
use imagequant::{Attributes, RGBA};
use oxipng::{Options, optimize_from_memory};

use crate::cli::QualityLevel;
use crate::compress::ImageMetadata;

pub fn optimize_png_bytes(input: &[u8], quality: QualityLevel) -> Vec<u8> {
    let level = match quality {
        QualityLevel::Lowest | QualityLevel::Low => 3,
        QualityLevel::Medium => 4,
        QualityLevel::High => 5,
        QualityLevel::Highest => 6,
    };

    optimize_from_memory(input, &Options::from_preset(level)).unwrap_or_else(|_| input.to_vec())
}

pub fn encode_png_lossless(
    image: &DynamicImage,
    preset: u8,
    metadata: &ImageMetadata,
) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let info = png_info(image.width(), image.height(), metadata);
    let mut encoder = png::Encoder::with_info(&mut buffer, info).context("PNG 编码器初始化失败")?;

    match image {
        DynamicImage::ImageLuma8(data) => {
            encoder.set_color(png::ColorType::Grayscale);
            encoder.set_depth(png::BitDepth::Eight);
            write_png_data(encoder, data.as_raw())?;
        }
        DynamicImage::ImageLumaA8(data) => {
            encoder.set_color(png::ColorType::GrayscaleAlpha);
            encoder.set_depth(png::BitDepth::Eight);
            write_png_data(encoder, data.as_raw())?;
        }
        DynamicImage::ImageRgb8(data) => {
            encoder.set_color(png::ColorType::Rgb);
            encoder.set_depth(png::BitDepth::Eight);
            write_png_data(encoder, data.as_raw())?;
        }
        DynamicImage::ImageRgba8(data) => {
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            write_png_data(encoder, data.as_raw())?;
        }
        DynamicImage::ImageLuma16(data) => {
            encoder.set_color(png::ColorType::Grayscale);
            encoder.set_depth(png::BitDepth::Sixteen);
            write_png_data(encoder, &to_big_endian(data.as_raw()))?;
        }
        DynamicImage::ImageLumaA16(data) => {
            encoder.set_color(png::ColorType::GrayscaleAlpha);
            encoder.set_depth(png::BitDepth::Sixteen);
            write_png_data(encoder, &to_big_endian(data.as_raw()))?;
        }
        DynamicImage::ImageRgb16(data) => {
            encoder.set_color(png::ColorType::Rgb);
            encoder.set_depth(png::BitDepth::Sixteen);
            write_png_data(encoder, &to_big_endian(data.as_raw()))?;
        }
        DynamicImage::ImageRgba16(data) => {
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Sixteen);
            write_png_data(encoder, &to_big_endian(data.as_raw()))?;
        }
        _ => {
            let rgba = image.to_rgba8();
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            write_png_data(encoder, rgba.as_raw())?;
        }
    }

    Ok(optimize_from_memory(&buffer, &Options::from_preset(preset)).unwrap_or(buffer))
}

pub fn encode_png_quantized(
    image: &DynamicImage,
    max_colors: u32,
    quality: u8,
    metadata: &ImageMetadata,
) -> Result<Vec<u8>> {
    let rgba = image.to_rgba8();
    let (width, height) = (rgba.width() as usize, rgba.height() as usize);

    let pixels: Vec<RGBA> = rgba
        .as_raw()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|pixel| RGBA::new(pixel[0], pixel[1], pixel[2], pixel[3]))
        .collect();

    let mut attributes = Attributes::new();
    attributes
        .set_max_colors(max_colors.clamp(2, 256))
        .context("无法设置 PNG 调色板颜色数")?;
    attributes
        .set_quality(0, quality)
        .context("无法设置 PNG 量化质量")?;
    attributes
        .set_speed(speed_for(quality))
        .context("无法设置 PNG 量化速度")?;

    let mut quant_image = attributes
        .new_image(pixels, width, height, 0.0)
        .context("无法创建 PNG 量化输入")?;
    let mut result = attributes
        .quantize(&mut quant_image)
        .context("PNG 量化失败")?;
    result
        .set_dithering_level(dithering_for(quality))
        .context("无法设置 PNG 抖动强度")?;

    let (palette, indices) = result
        .remapped(&mut quant_image)
        .context("PNG 调色板映射失败")?;

    if palette.is_empty() || palette.len() > 256 {
        anyhow::bail!("PNG 调色板颜色数超出限制");
    }

    let mut palette_bytes = Vec::with_capacity(palette.len() * 3);
    for color in &palette {
        palette_bytes.extend_from_slice(&[color.r, color.g, color.b]);
    }
    let transparency = build_transparency(&palette, &indices);

    let mut buffer = Vec::new();
    let info = png_info(rgba.width(), rgba.height(), metadata);
    let mut encoder = png::Encoder::with_info(&mut buffer, info).context("PNG 编码器初始化失败")?;
    encoder.set_color(png::ColorType::Indexed);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_palette(palette_bytes);
    if let Some(transparency) = transparency {
        encoder.set_trns(transparency);
    }
    write_png_data(encoder, &indices)?;

    Ok(optimize_png_bytes(&buffer, QualityLevel::Medium))
}

fn png_info(width: u32, height: u32, metadata: &ImageMetadata) -> png::Info<'static> {
    let mut info = png::Info::with_size(width, height);

    if let Some(icc_profile) = &metadata.icc_profile {
        info.icc_profile = Some(Cow::Owned(icc_profile.clone()));
    }
    if let Some(exif) = &metadata.exif {
        info.exif_metadata = Some(Cow::Owned(exif.clone()));
    }

    info
}

fn to_big_endian(samples: &[u16]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(samples.len() * 2);
    for sample in samples {
        bytes.extend_from_slice(&sample.to_be_bytes());
    }
    bytes
}

fn write_png_data(encoder: png::Encoder<'_, &mut Vec<u8>>, data: &[u8]) -> Result<()> {
    let mut writer = encoder.write_header().context("PNG 写入头失败")?;
    writer.write_image_data(data).context("PNG 写入数据失败")?;
    Ok(())
}

fn build_transparency(palette: &[RGBA], indices: &[u8]) -> Option<Vec<u8>> {
    let max_index = indices.iter().copied().max().unwrap_or(0) as usize;
    let has_alpha = palette
        .iter()
        .take(max_index + 1)
        .any(|color| color.a != 255);
    if !has_alpha {
        return None;
    }

    Some(
        palette
            .iter()
            .take(max_index + 1)
            .map(|color| color.a)
            .collect(),
    )
}

fn speed_for(quality: u8) -> i32 {
    match quality {
        0..=40 => 10,
        41..=60 => 8,
        61..=70 => 6,
        71..=85 => 4,
        _ => 2,
    }
}

fn dithering_for(quality: u8) -> f32 {
    match quality {
        0..=40 => 0.4,
        41..=60 => 0.6,
        61..=70 => 0.8,
        _ => 1.0,
    }
}
