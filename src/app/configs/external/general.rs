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
    pub animations: AnimationsConfig,
    pub floating_wins: FloatingWinsConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct AnimationsConfig {
    pub enabled: bool,

    #[serde(deserialize_with = "deserializers::to_u32_minmax::<100,10000,_>")]
    pub duration: u32,

    #[serde(deserialize_with = "deserializers::to_u8_minmax::<10,240,_>")]
    pub framerate: u8,

    #[serde(rename = "type")]
    pub animation_type: WindowAnimation,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum FloatingWinsSizeStrategyLabel {
    Preserve,
    Fixed,
    Relative,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct FloatingWinsConfig {
    pub topmost: bool,
    pub size: FloatingWinsSizeStrategyLabel,
    #[serde(deserialize_with = "deserializers::deserialize_size_ratio")]
    pub size_ratio: (f32, f32),
    #[serde(deserialize_with = "deserializers::deserialize_size_fixed")]
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
            animations: AnimationsConfig::default(),
            floating_wins: FloatingWinsConfig::default(),
        }
    }
}

impl Default for AnimationsConfig {
    fn default() -> Self {
        AnimationsConfig {
            enabled: true,
            duration: 300,
            framerate: 60,
            animation_type: WindowAnimation::default(),
        }
    }
}

impl Default for FloatingWinsConfig {
    fn default() -> Self {
        FloatingWinsConfig {
            topmost: true,
            size: FloatingWinsSizeStrategyLabel::Relative,
            size_ratio: (0.5, 0.5),
            size_fixed: (700, 400),
        }
    }
}
