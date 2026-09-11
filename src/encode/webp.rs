use anyhow::Result;
use image::DynamicImage;
use webp::Encoder;

pub fn encode_webp(image: &DynamicImage, quality: u8) -> Result<Vec<u8>> {
    let quality = quality.clamp(1, 100) as f32;

    if image.color().has_alpha() {
        let rgba = image.to_rgba8();
        let encoder = Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height());
        Ok(encoder.encode(quality).to_vec())
    } else {
        let rgb = image.to_rgb8();
        let encoder = Encoder::from_rgb(rgb.as_raw(), rgb.width(), rgb.height());
        Ok(encoder.encode(quality).to_vec())
    }
}
