use std::path::PathBuf;

#[derive(Debug)]
pub struct CompressionReport {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub input_size: u64,
    pub output_size: u64,
}

impl CompressionReport {
    pub fn saved_percent(&self) -> f64 {
        if self.input_size == 0 {
            return 0.0;
        }

        let saved = self.input_size.saturating_sub(self.output_size);
        saved as f64 / self.input_size as f64 * 100.0
    }
}
