use anyhow::Result;

use crate::cli::{Args, QualityLevel};
use crate::size::parse_size_limit;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompressionMode {
    Quality(QualityLevel),
    MaxSize(u64),
}

impl CompressionMode {
    pub fn from_args(args: &Args, _input_size: u64) -> Result<Self> {
        if let Some(quality) = args.quality {
            return Ok(Self::Quality(quality));
        }

        if let Some(raw_limit) = &args.max_size {
            return Ok(Self::MaxSize(parse_size_limit(raw_limit)?));
        }

        Ok(Self::Quality(QualityLevel::Medium))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Args;
    use clap::Parser;

    #[test]
    fn defaults_to_medium_quality() {
        let args = Args::try_parse_from(["picompress", "input.png"]).unwrap();
        assert_eq!(
            CompressionMode::from_args(&args, 1024).unwrap(),
            CompressionMode::Quality(QualityLevel::Medium)
        );
    }

    #[test]
    fn parses_max_size() {
        let args =
            Args::try_parse_from(["picompress", "input.png", "--max-size", "200KB"]).unwrap();
        assert_eq!(
            CompressionMode::from_args(&args, 400 * 1024).unwrap(),
            CompressionMode::MaxSize(200 * 1024)
        );
    }

    #[test]
    fn allows_limit_larger_than_input() {
        let args = Args::try_parse_from(["picompress", "input.png", "--max-size", "10MB"]).unwrap();
        assert_eq!(
            CompressionMode::from_args(&args, 1024).unwrap(),
            CompressionMode::MaxSize(10 * 1024 * 1024)
        );
    }
}
