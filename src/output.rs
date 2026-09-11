use std::path::{Path, PathBuf};

use anyhow::Result;
use image::ImageFormat;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputFormat {
    Jpeg,
    Png,
    WebP,
}

impl OutputFormat {
    /// 根据显式输出路径推断格式 否则回退到源格式 无法输出到源格式时使用 PNG
    pub fn detect(output: Option<&Path>, source: ImageFormat) -> Result<Self> {
        if let Some(path) = output
            && path.extension().is_some()
            && !path.is_dir()
        {
            let extension = path
                .extension()
                .and_then(|extension| extension.to_str())
                .unwrap_or_default();
            let format = ImageFormat::from_extension(extension)
                .ok_or_else(|| anyhow::anyhow!("无法识别输出格式 {extension}"))?;

            return Self::from_image_format(format)
                .ok_or_else(|| anyhow::anyhow!("暂不支持输出 {extension} 格式"));
        }

        Ok(Self::from_image_format(source).unwrap_or(Self::Png))
    }

    pub fn from_image_format(format: ImageFormat) -> Option<Self> {
        match format {
            ImageFormat::Jpeg => Some(Self::Jpeg),
            ImageFormat::Png => Some(Self::Png),
            ImageFormat::WebP => Some(Self::WebP),
            _ => None,
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::WebP => "webp",
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Jpeg => "JPEG",
            Self::Png => "PNG",
            Self::WebP => "WebP",
        };
        formatter.write_str(name)
    }
}

pub fn resolve_output_path(
    input_path: &Path,
    output: Option<&Path>,
    format: OutputFormat,
) -> PathBuf {
    if let Some(output) = output {
        if output.is_dir() {
            return output.join(default_file_name(input_path, format));
        }

        if output.extension().is_none() {
            return output.with_extension(format.extension());
        }

        return output.to_path_buf();
    }

    let parent = input_path.parent().unwrap_or_else(|| Path::new(""));
    parent.join(default_file_name(input_path, format))
}

fn default_file_name(input_path: &Path, format: OutputFormat) -> String {
    let stem = input_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("image");

    format!("{stem}.compressed.{}", format.extension())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_output_adds_compressed_suffix() {
        let path = resolve_output_path(Path::new("/tmp/photo.png"), None, OutputFormat::Png);

        assert_eq!(path, PathBuf::from("/tmp/photo.compressed.png"));
    }

    #[test]
    fn output_directory_gets_generated_name() {
        let directory = tempfile::tempdir().unwrap();
        let path = resolve_output_path(
            Path::new("/tmp/photo.png"),
            Some(directory.path()),
            OutputFormat::WebP,
        );

        assert_eq!(path, directory.path().join("photo.compressed.webp"));
    }

    #[test]
    fn output_without_extension_gets_one() {
        let path = resolve_output_path(
            Path::new("/tmp/photo.png"),
            Some(Path::new("/tmp/result")),
            OutputFormat::WebP,
        );

        assert_eq!(path, PathBuf::from("/tmp/result.webp"));
    }

    #[test]
    fn output_extension_overrides_source_format() {
        let format =
            OutputFormat::detect(Some(Path::new("/tmp/photo.webp")), ImageFormat::Png).unwrap();
        assert_eq!(format, OutputFormat::WebP);
    }

    #[test]
    fn unsupported_output_extension_is_rejected() {
        let result = OutputFormat::detect(Some(Path::new("/tmp/photo.avif")), ImageFormat::Png);
        assert!(result.is_err());
    }

    #[test]
    fn unsupported_source_format_falls_back_to_png() {
        let format = OutputFormat::detect(None, ImageFormat::Bmp).unwrap();
        assert_eq!(format, OutputFormat::Png);
    }
}
