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
