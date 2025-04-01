use crate::app::area_tree::layout_strategy::LayoutStrategyEnum;
use crate::app::configs::core::WindowBehavior;
use crate::app::configs::core::WindowRule;
use crate::app::configs::general::FloatingWinsConfigs;
use crate::app::configs::MonitorConfigs;
use crate::app::structs::win_matcher::WinMatcher;
use crate::modules::tiles_manager::configs::CoreModuleConfigs;
use crate::modules::tiles_manager::lib::window_animation_player::WindowAnimation;
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashMap;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TilesManagerConfig {
    tiles_padding: i16,
    borders_padding: i16,
    focalized_padding: i16,
    half_focalized_borders_pad: i16,
    half_focalized_tiles_pad: i16,
    layout_strategy: LayoutStrategyEnum,
    monitors_configs: HashMap<String, MonitorConfigs>,
    pub ignore_filter: WinMatcher,
    pub rules: Vec<WindowRule>,
    pub animations_duration: u32,
    pub animations_framerate: u8,
    pub animation_type: Option<WindowAnimation>,
    pub history_based_navigation: bool,
    pub floating_wins: FloatingWinsConfigs,
}

impl TilesManagerConfig {
    pub fn get_animations(&self) -> Option<WindowAnimation> {
        self.animation_type.clone()
    }

    pub fn get_animation_duration(&self) -> u32 {
        self.animations_duration
    }

    pub fn get_framerate(&self) -> u8 {
        self.animations_framerate
    }

    pub fn get_layout_strategy(&self, monitor_name: &str) -> LayoutStrategyEnum {
        self.monitors_configs
            .get(monitor_name)
            .map(|c| c.layout_strategy.clone())
            .unwrap_or(self.layout_strategy.clone())
    }

    pub fn get_focalized_padding(&self, monitor_name: &str) -> i16 {
        self.monitors_configs
            .get(monitor_name)
            .map(|c| c.focalized_padding as i16)
            .unwrap_or(self.focalized_padding)
    }

    pub fn get_borders_padding(&self, monitor_name: &str) -> i16 {
        self.monitors_configs
            .get(monitor_name)
            .map(|c| c.borders_padding as i16)
            .unwrap_or(self.borders_padding)
    }

    pub fn get_tiles_padding(&self, monitor_name: &str) -> i16 {
        self.monitors_configs
            .get(monitor_name)
            .map(|c| c.tiles_padding as i16)
            .unwrap_or(self.tiles_padding)
    }

    pub fn get_half_focalized_borders_pad(&self, monitor_name: &str) -> i16 {
        self.monitors_configs
            .get(monitor_name)
            .map(|c| c.half_focalized_borders_pad as i16)
            .unwrap_or(self.half_focalized_borders_pad)
    }

    pub fn get_half_focalized_tiles_pad(&self, monitor_name: &str) -> i16 {
        self.monitors_configs
            .get(monitor_name)
            .map(|c| c.half_focalized_tiles_pad as i16)
            .unwrap_or(self.half_focalized_tiles_pad)
    }

    pub fn get_half_focalized_tiles_pad_xy(&self, monitor_name: &str) -> (i16, i16) {
        let pad = self.get_half_focalized_tiles_pad(monitor_name);
        (pad, pad)
    }

    pub fn get_tiles_padding_xy(&self, monitor_name: &str) -> (i16, i16) {
        let tiles_padding = self.get_tiles_padding(monitor_name);
        (tiles_padding, tiles_padding)
    }
}

impl From<&CoreModuleConfigs> for TilesManagerConfig {
    fn from(configs: &CoreModuleConfigs) -> Self {
        let animation_type = match &configs.animations_enabled {
            true => configs.animation_type.clone(),
            false => None,
        };

        Self {
            tiles_padding: configs.tiles_padding as i16,
            borders_padding: configs.border_padding as i16,
            focalized_padding: configs.focalized_padding as i16,
            half_focalized_borders_pad: configs.half_focalized_borders_pad as i16,
            half_focalized_tiles_pad: configs.half_focalized_tiles_pad as i16,
            ignore_filter: configs.ignore_filter.clone(),
            rules: configs.rules.clone(),
            layout_strategy: configs.layout_strategy.clone(),
            animations_duration: configs.animations_duration,
            animations_framerate: configs.animations_framerate,
            animation_type,
            history_based_navigation: configs.history_based_navigation,
            floating_wins: configs.floating_wins.clone(),
            monitors_configs: configs.monitors_configs.clone(),
        }
    }
}

pub trait Rules {
    fn is_floating(&self, window: WindowRef) -> bool;
    fn preferred_monitor(&self, window: WindowRef) -> Option<String>;
}

impl Rules for Vec<WindowRule> {
    fn is_floating(&self, window: WindowRef) -> bool {
        is_floating(self, window)
    }

    fn preferred_monitor(&self, window: WindowRef) -> Option<String> {
        preferred_monitor(self, window)
    }
}

fn find_matches(rules: &[WindowRule], window: WindowRef) -> impl Iterator<Item = &WindowRule> {
    rules.iter().filter(move |r| r.filter.matches(window))
}

fn is_floating(rules: &[WindowRule], window: WindowRef) -> bool {
    find_matches(rules, window).any(|r| matches!(r.behavior, WindowBehavior::Float))
}

fn preferred_monitor(rules: &[WindowRule], window: WindowRef) -> Option<String> {
    find_matches(rules, window).find_map(|r| match &r.behavior {
        WindowBehavior::Insert { monitor } => Some(monitor.clone()),
        _ => None,
    })
}
