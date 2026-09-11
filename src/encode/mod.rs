mod jpeg;
mod png;
mod webp;

pub use jpeg::encode_jpeg;
pub use png::{encode_png_lossless, encode_png_quantized, optimize_png_bytes};
pub use webp::encode_webp;

use anyhow::Result;
use image::DynamicImage;

use crate::cli::QualityLevel;
use crate::compress::ImageMetadata;
use crate::output::OutputFormat;

pub fn encode_image(
    image: &DynamicImage,
    format: OutputFormat,
    quality: QualityLevel,
    metadata: &ImageMetadata,
) -> Result<Vec<u8>> {
    match format {
        OutputFormat::Jpeg => encode_jpeg(image, quality.lossy_quality(), metadata),
        OutputFormat::WebP => encode_webp(image, quality.lossy_quality()),
        OutputFormat::Png if quality.is_lossless() => {
            encode_png_lossless(image, png_preset(quality), metadata)
        }
        OutputFormat::Png => encode_png_quantized(
            image,
            max_colors(quality),
            quality.lossy_quality(),
            metadata,
        ),
    }
}

/// 数值质量编码 用于最大体积搜索
pub fn encode_with_numeric_quality(
    image: &DynamicImage,
    format: OutputFormat,
    quality: u8,
    metadata: &ImageMetadata,
) -> Result<Vec<u8>> {
    match format {
        OutputFormat::Jpeg => encode_jpeg(image, quality, metadata),
        OutputFormat::WebP => encode_webp(image, quality),
        OutputFormat::Png => encode_png_lossless(image, png_preset_from_quality(quality), metadata),
    }
}

fn png_preset(quality: QualityLevel) -> u8 {
    match quality {
        QualityLevel::Lowest | QualityLevel::Low => 2,
        QualityLevel::Medium => 3,
        QualityLevel::High => 4,
        QualityLevel::Highest => 6,
    }
}

fn png_preset_from_quality(quality: u8) -> u8 {
    match quality {
        0..=40 => 2,
        41..=70 => 3,
        71..=85 => 4,
        _ => 6,
    }
}

fn max_colors(quality: QualityLevel) -> u32 {
    match quality {
        QualityLevel::Lowest => 64,
        QualityLevel::Low => 128,
        QualityLevel::Medium | QualityLevel::High | QualityLevel::Highest => 256,
    }
}
