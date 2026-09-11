mod cli;
mod compress;
mod encode;
mod output;
mod size;

use anyhow::Result;
use clap::Parser;

use crate::cli::Args;
use crate::compress::compress_image;

fn main() -> Result<()> {
    let args = Args::parse();
    let report = compress_image(&args)?;

    println!(
        "{} -> {}  {} -> {}  saved {} ({:.1}%)",
        report.input_path.display(),
        report.output_path.display(),
        format_size(report.input_size),
        format_size(report.output_size),
        format_size(report.input_size.saturating_sub(report.output_size)),
        report.saved_percent(),
    );

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;

    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
