use super::lib::window_animation_player::WindowAnimation;
use crate::app::area_tree::layout_strategy::LayoutStrategyEnum;
use crate::app::config::app_configs::AppConfigs;

pub struct CoreModuleConfigs {
    pub layout_strategy: LayoutStrategyEnum,
    pub tiles_padding: u8,
    pub border_padding: u8,
    pub focalized_padding: u8,
    pub animations_enabled: bool,
    pub animations_duration: u32,
    pub animations_framerate: u8,
    pub animation_type: Option<WindowAnimation>,
}

impl Default for CoreModuleConfigs {
    fn default() -> Self {
        CoreModuleConfigs {
            layout_strategy: LayoutStrategyEnum::default(),
            tiles_padding: 0,
            border_padding: 0,
            focalized_padding: 0,
            animations_enabled: false,
            animations_duration: 500,
            animations_framerate: 60,
            animation_type: None,
        }
    }
}

impl CoreModuleConfigs {
    pub fn get_layout(&self, _display_name: Option<&String>) -> LayoutStrategyEnum {
        // TODO: implement display_name support
        self.layout_strategy.clone()
    }
}

impl From<&AppConfigs> for CoreModuleConfigs {
    fn from(app_configs: &AppConfigs) -> Self {
        CoreModuleConfigs {
            layout_strategy: app_configs.get_layout_strategy(),
            tiles_padding: app_configs.layout.tiles_padding,
            border_padding: app_configs.layout.border_padding,
            focalized_padding: app_configs.layout.focalized_padding,
            animations_enabled: app_configs.layout.animations_enabled,
            animations_duration: app_configs.layout.animations_duration,
            animations_framerate: app_configs.layout.animations_framerate,
            animation_type: app_configs.layout.animation_type.clone(),
        }
    }
}
