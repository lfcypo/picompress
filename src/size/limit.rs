use anyhow::{Result, bail};

pub fn parse_size_limit(value: &str) -> Result<u64> {
    let input = value.trim();
    if input.is_empty() {
        bail!("大小参数不能为空");
    }

    let split_at = input
        .find(|ch: char| !ch.is_ascii_digit() && ch != '.')
        .unwrap_or(input.len());
    let (number, unit) = input.split_at(split_at);
    if number.is_empty() {
        bail!("大小参数缺少数字 {value}");
    }

    let number: f64 = number.parse()?;
    if !number.is_finite() || number <= 0.0 {
        bail!("大小参数必须大于 0 {value}");
    }

    let multiplier = match unit.trim().to_ascii_lowercase().as_str() {
        "" | "b" => 1.0,
        "k" | "kb" | "kib" => 1024.0,
        "m" | "mb" | "mib" => 1024.0 * 1024.0,
        "g" | "gb" | "gib" => 1024.0 * 1024.0 * 1024.0,
        other => bail!("不支持的大小单位 {other}"),
    };

    let bytes = number * multiplier;
    if bytes > u64::MAX as f64 {
        bail!("大小参数过大 {value}");
    }

    Ok(bytes.floor() as u64)
}
