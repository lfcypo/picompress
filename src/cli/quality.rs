use clap::ValueEnum;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum QualityLevel {
    Lowest,
    Low,
    Medium,
    High,
    Highest,
}

impl QualityLevel {
    pub fn lossy_quality(self) -> u8 {
        match self {
            Self::Lowest => 30,
            Self::Low => 50,
            Self::Medium => 75,
            Self::High => 88,
            Self::Highest => 95,
        }
    }

    pub fn is_lossless(self) -> bool {
        matches!(self, Self::Medium | Self::High | Self::Highest)
    }
}
