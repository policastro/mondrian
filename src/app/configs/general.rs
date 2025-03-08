use crate::app::configs::deserializers;
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
    pub animations: AnimationsConfigs,
    pub floating_wins: FloatingWinsConfigs,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct AnimationsConfigs {
    pub enabled: bool,

    #[serde(deserialize_with = "deserializers::to_u32_minmax::<100,10000,_>")]
    pub duration: u32,

    #[serde(deserialize_with = "deserializers::to_u8_minmax::<10,240,_>")]
    pub framerate: u8,

    #[serde(rename = "type")]
    pub animation_type: WindowAnimation,
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
            animations: AnimationsConfigs::default(),
            floating_wins: FloatingWinsConfigs::default(),
        }
    }
}

impl Default for AnimationsConfigs {
    fn default() -> Self {
        AnimationsConfigs {
            enabled: true,
            duration: 300,
            framerate: 60,
            animation_type: WindowAnimation::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum FloatingWinsSizeStrategy {
    Preserve,
    Fixed,
    Relative,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct FloatingWinsConfigs {
    pub topmost: bool,
    pub size: FloatingWinsSizeStrategy,
    #[serde(deserialize_with = "deserialize_size_ratio")]
    pub size_ratio: (f32, f32),
    #[serde(deserialize_with = "deserialize_size_fixed")]
    pub size_fixed: (u16, u16),
}

impl Default for FloatingWinsConfigs {
    fn default() -> Self {
        FloatingWinsConfigs {
            topmost: true,
            size: FloatingWinsSizeStrategy::Relative,
            size_ratio: (0.5, 0.5),
            size_fixed: (700, 400),
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
