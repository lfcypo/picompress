use std::path::PathBuf;

use clap::{ArgGroup, Parser};

use super::QualityLevel;

#[derive(Debug, Parser)]
#[command(
    name = "picompress",
    version,
    about = "压缩常见格式图片并尽量保留画面内容",
    group(
        ArgGroup::new("size").args(["quality", "max_size"]).multiple(false)
    )
)]
pub struct Args {
    /// 待压缩图片路径
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,

    /// 输出质量档位
    #[arg(short, long, value_enum, value_name = "LEVEL")]
    pub quality: Option<QualityLevel>,

    /// 输出最大体积 例如 1MB 或 200KB
    #[arg(long = "max-size", value_name = "SIZE", conflicts_with = "quality")]
    pub max_size: Option<String>,

    /// 输出文件路径 默认在原图旁生成压缩结果
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<PathBuf>,
}
