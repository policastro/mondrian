use super::lib::window_animation_player::WindowAnimation;
use crate::app::area_tree::layout_strategy::LayoutStrategyEnum;
use crate::app::configs::core::WindowRule;
use crate::app::configs::general::FloatingWinsConfigs;
use crate::app::configs::AppConfigs;
use crate::app::configs::MonitorConfigs;
use crate::app::structs::win_matcher::WinMatcher;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct CoreModuleConfigs {
    pub layout_strategy: LayoutStrategyEnum,
    pub tiles_padding: u8,
    pub border_padding: u8,
    pub half_focalized_borders_pad: u8,
    pub half_focalized_tiles_pad: u8,
    pub focalized_padding: u8,
    pub animations_enabled: bool,
    pub animations_duration: u32,
    pub animations_framerate: u8,
    pub animation_type: Option<WindowAnimation>,
    pub move_cursor_on_focus: bool,
    pub ignore_filter: WinMatcher,
    pub rules: Vec<WindowRule>,
    pub history_based_navigation: bool,
    pub floating_wins: FloatingWinsConfigs,
    pub monitors_configs: HashMap<String, MonitorConfigs>,
}

impl Default for CoreModuleConfigs {
    fn default() -> Self {
        CoreModuleConfigs {
            layout_strategy: LayoutStrategyEnum::default(),
            tiles_padding: 0,
            border_padding: 0,
            half_focalized_borders_pad: 0,
            half_focalized_tiles_pad: 0,
            focalized_padding: 0,
            animations_enabled: false,
            animations_duration: 500,
            animations_framerate: 60,
            animation_type: None,
            move_cursor_on_focus: false,
            ignore_filter: WinMatcher::default(),
            rules: Vec::new(),
            history_based_navigation: false,
            floating_wins: FloatingWinsConfigs::default(),
            monitors_configs: HashMap::new(),
        }
    }
}

impl From<&AppConfigs> for CoreModuleConfigs {
    fn from(app_configs: &AppConfigs) -> Self {
        CoreModuleConfigs {
            layout_strategy: app_configs.get_layout_strategy(),
            tiles_padding: app_configs.layout.paddings.tiles,
            border_padding: app_configs.layout.paddings.borders,
            half_focalized_borders_pad: app_configs.layout.half_focalized_paddings.borders,
            half_focalized_tiles_pad: app_configs.layout.half_focalized_paddings.tiles,
            focalized_padding: app_configs.layout.focalized_padding,
            animations_enabled: app_configs.general.animations.enabled,
            animations_duration: app_configs.general.animations.duration,
            animations_framerate: app_configs.general.animations.framerate,
            animation_type: match app_configs.general.animations.enabled {
                true => Some(app_configs.general.animations.animation_type.clone()),
                false => None,
            },
            move_cursor_on_focus: app_configs.general.move_cursor_on_focus,
            ignore_filter: app_configs.get_ignore_filter(),
            rules: app_configs.get_rules(),
            history_based_navigation: app_configs.general.history_based_navigation,
            floating_wins: app_configs.general.floating_wins.clone(),
            monitors_configs: app_configs.get_monitors_configs(),
        }
    }
}
