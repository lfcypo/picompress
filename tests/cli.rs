use std::process::Command;

use image::{DynamicImage, ImageFormat, Rgb, RgbImage};
use tempfile::tempdir;

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_picompress")
}

fn sample_image(width: u32, height: u32) -> DynamicImage {
    DynamicImage::ImageRgb8(RgbImage::from_fn(width, height, |x, y| {
        Rgb([
            (x * 255 / width.max(1)) as u8,
            (y * 255 / height.max(1)) as u8,
            ((x + y) % 256) as u8,
        ])
    }))
}

#[test]
fn compresses_jpeg_default_quality() {
    let directory = tempdir().unwrap();
    let input = directory.path().join("input.jpg");
    sample_image(320, 240)
        .save_with_format(&input, ImageFormat::Jpeg)
        .unwrap();

    let output = directory.path().join("output.jpg");
    let status = Command::new(binary())
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .status()
        .unwrap();

    assert!(status.success());
    assert!(output.is_file());
    assert!(output.metadata().unwrap().len() > 0);
}

#[test]
fn converts_png_to_webp() {
    let directory = tempdir().unwrap();
    let input = directory.path().join("input.png");
    sample_image(256, 256)
        .save_with_format(&input, ImageFormat::Png)
        .unwrap();

    let output = directory.path().join("output.webp");
    let status = Command::new(binary())
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .arg("--quality")
        .arg("high")
        .status()
        .unwrap();

    assert!(status.success());
    let data = std::fs::read(&output).unwrap();
    assert_eq!(image::guess_format(&data).unwrap(), ImageFormat::WebP);
}

#[test]
fn respects_max_size() {
    let directory = tempdir().unwrap();
    let input = directory.path().join("input.png");
    sample_image(512, 512)
        .save_with_format(&input, ImageFormat::Png)
        .unwrap();

    let output = directory.path().join("output.jpg");
    let status = Command::new(binary())
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .arg("--max-size")
        .arg("20KB")
        .status()
        .unwrap();

    assert!(status.success());
    assert!(output.metadata().unwrap().len() <= 20 * 1024);
}

#[test]
fn rejects_conflicting_target_arguments() {
    let directory = tempdir().unwrap();
    let input = directory.path().join("input.png");
    sample_image(32, 32)
        .save_with_format(&input, ImageFormat::Png)
        .unwrap();

    let status = Command::new(binary())
        .arg(&input)
        .arg("--quality")
        .arg("low")
        .arg("--max-size")
        .arg("10KB")
        .status()
        .unwrap();

    assert!(!status.success());
}
