use anyhow::{Context, Result};
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ExtendedColorType, ImageEncoder};

use crate::compress::ImageMetadata;

pub fn encode_jpeg(image: &DynamicImage, quality: u8, metadata: &ImageMetadata) -> Result<Vec<u8>> {
    let rgb = image.to_rgb8();
    let mut output = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut output, quality.clamp(1, 100));

    if let Some(icc_profile) = &metadata.icc_profile {
        encoder
            .set_icc_profile(icc_profile.clone())
            .map_err(|error| anyhow::anyhow!("JPEG ICC 配置失败 {error}"))?;
    }
    if let Some(exif) = &metadata.exif {
        encoder
            .set_exif_metadata(exif.clone())
            .map_err(|error| anyhow::anyhow!("JPEG EXIF 配置失败 {error}"))?;
    }

    encoder
        .write_image(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            ExtendedColorType::Rgb8,
        )
        .context("JPEG 编码失败")?;

    Ok(output)
}
