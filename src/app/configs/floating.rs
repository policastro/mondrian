use super::external::{self, general::FloatingWinsSizeStrategyLabel};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloatingWinsSizeStrategy {
    Preserve,
    Fixed { w: u16, h: u16 },
    Relative { w: f32, h: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatingWinsConfig {
    pub topmost: bool,
    pub centered: bool,
    pub strategy: FloatingWinsSizeStrategy,
}

impl Default for FloatingWinsConfig {
    fn default() -> Self {
        FloatingWinsConfig {
            topmost: true,
            centered: true,
            strategy: FloatingWinsSizeStrategy::Relative { w: 0.5, h: 0.5 },
        }
    }
}

impl From<external::general::FloatingWinsConfig> for FloatingWinsConfig {
    fn from(value: external::general::FloatingWinsConfig) -> Self {
        FloatingWinsConfig {
            topmost: value.topmost,
            centered: value.centered,
            strategy: match value.size {
                FloatingWinsSizeStrategyLabel::Preserve => FloatingWinsSizeStrategy::Preserve,
                FloatingWinsSizeStrategyLabel::Fixed => FloatingWinsSizeStrategy::Fixed {
                    w: value.size_fixed.0,
                    h: value.size_fixed.1,
                },
                FloatingWinsSizeStrategyLabel::Relative => FloatingWinsSizeStrategy::Relative {
                    w: value.size_ratio.0,
                    h: value.size_ratio.1,
                },
            },
        }
    }
}
