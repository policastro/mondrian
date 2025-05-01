use crate::app::configs::deserializers;
use crate::app::configs::FloatingWinsConfig;
use crate::app::configs::FloatingWinsSizeStrategy;
use crate::modules::tiles_manager::lib::window_animation_player::WindowAnimation;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct General {
    pub history_based_navigation: bool,
    pub move_cursor_on_focus: bool,
    pub auto_reload_configs: bool,
    pub detect_maximized_windows: bool,
    pub insert_in_monitor: bool,
    pub free_move_in_monitor: bool,
    pub animations: ExtAnimationsConfig,
    pub floating_wins: ExtFloatingWinsConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct ExtAnimationsConfig {
    pub enabled: bool,

    #[serde(deserialize_with = "deserializers::to_u32_minmax::<100,10000,_>")]
    pub duration: u32,

    #[serde(deserialize_with = "deserializers::to_u8_minmax::<10,240,_>")]
    pub framerate: u8,

    #[serde(rename = "type")]
    pub animation_type: WindowAnimation,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum FloatingWinsSizeStrategyLabel {
    Preserve,
    Fixed,
    Relative,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct ExtFloatingWinsConfig {
    pub topmost: bool,
    pub size: FloatingWinsSizeStrategyLabel,
    #[serde(deserialize_with = "deserialize_size_ratio")]
    pub size_ratio: (f32, f32),
    #[serde(deserialize_with = "deserialize_size_fixed")]
    pub size_fixed: (u16, u16),
}

impl Default for General {
    fn default() -> Self {
        General {
            history_based_navigation: false,
            move_cursor_on_focus: false,
            auto_reload_configs: true,
            detect_maximized_windows: true,
            insert_in_monitor: true,
            free_move_in_monitor: false,
            animations: ExtAnimationsConfig::default(),
            floating_wins: ExtFloatingWinsConfig::default(),
        }
    }
}

impl Default for ExtAnimationsConfig {
    fn default() -> Self {
        ExtAnimationsConfig {
            enabled: true,
            duration: 300,
            framerate: 60,
            animation_type: WindowAnimation::default(),
        }
    }
}

impl Default for ExtFloatingWinsConfig {
    fn default() -> Self {
        ExtFloatingWinsConfig {
            topmost: true,
            size: FloatingWinsSizeStrategyLabel::Relative,
            size_ratio: (0.5, 0.5),
            size_fixed: (700, 400),
        }
    }
}

impl From<ExtFloatingWinsConfig> for FloatingWinsConfig {
    fn from(value: ExtFloatingWinsConfig) -> Self {
        FloatingWinsConfig {
            topmost: value.topmost,
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

fn deserialize_size_ratio<'de, D>(deserializer: D) -> Result<(f32, f32), D::Error>
where
    D: serde::Deserializer<'de>,
{
    let (w, h) = serde::Deserialize::deserialize(deserializer)?;
    if w < 0.1 || h < 0.1 || w > 1.0 || h > 1.0 {
        return Err(serde::de::Error::custom("Width and height must be between 0.1 and 1.0"));
    }
    Ok((w, h))
}

fn deserialize_size_fixed<'de, D>(deserializer: D) -> Result<(u16, u16), D::Error>
where
    D: serde::Deserializer<'de>,
{
    let (w, h) = serde::Deserialize::deserialize(deserializer)?;
    if w < 100 || h < 100 || w > 10000 || h > 10000 {
        return Err(serde::de::Error::custom(
            "Width and height must be between 100 and 10000",
        ));
    }
    Ok((w, h))
}
