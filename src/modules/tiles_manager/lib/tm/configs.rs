use crate::app::area_tree::layout_strategy::LayoutStrategyEnum;
use crate::app::configs::floating::FloatingWinsConfig;
use crate::app::configs::rules::WindowBehavior;
use crate::app::configs::rules::WindowRule;
use crate::app::configs::AnimationsConfig;
use crate::app::configs::AppConfig;
use crate::app::configs::MonitorConfig;
use crate::app::structs::paddings::Paddings;
use crate::app::structs::win_matcher::WinMatcher;
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashMap;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TilesManagerConfig {
    tiles_padding: i16,
    borders_padding: Paddings,
    focalized_padding: Paddings,
    half_focalized_borders_pad: Paddings,
    half_focalized_tiles_pad: i16,
    layout_strategy: LayoutStrategyEnum,
    monitors_configs: HashMap<String, MonitorConfig>,
    pub ignore_filter: WinMatcher,
    pub rules: Vec<WindowRule>,
    pub animation: AnimationsConfig,
    pub history_based_navigation: bool,
    pub floating_wins: FloatingWinsConfig,
}

impl TilesManagerConfig {
    pub fn get_layout_strategy(&self, monitor_name: &str) -> LayoutStrategyEnum {
        self.monitors_configs
            .get(monitor_name)
            .map(|c| c.layout_strategy.clone())
            .unwrap_or(self.layout_strategy.clone())
    }

    pub fn get_focalized_padding(&self, monitor_name: &str) -> Paddings {
        self.monitors_configs
            .get(monitor_name)
            .map(|c| c.focalized_padding)
            .unwrap_or(self.focalized_padding)
    }

    pub fn get_borders_padding(&self, monitor_name: &str) -> Paddings {
        self.monitors_configs
            .get(monitor_name)
            .map(|c| c.borders_padding)
            .unwrap_or(self.borders_padding)
    }

    pub fn get_tiles_padding(&self, monitor_name: &str) -> i16 {
        self.monitors_configs
            .get(monitor_name)
            .map(|c| c.tiles_padding as i16)
            .unwrap_or(self.tiles_padding)
    }

    pub fn get_half_focalized_borders_pad(&self, monitor_name: &str) -> Paddings {
        self.monitors_configs
            .get(monitor_name)
            .map(|c| c.half_focalized_borders_pad)
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

impl From<&AppConfig> for TilesManagerConfig {
    fn from(config: &AppConfig) -> Self {
        Self {
            layout_strategy: config.layout_strategy.clone(),
            tiles_padding: config.tiles_pad as i16,
            borders_padding: config.borders_pads,
            half_focalized_borders_pad: config.half_focalized_borders_pads,
            half_focalized_tiles_pad: config.half_focalized_tiles_pad as i16,
            focalized_padding: config.focalized_pads,
            animation: config.animations.clone(),
            ignore_filter: config.ignore_filter.clone(),
            rules: config.rules.clone(),
            history_based_navigation: config.history_based_navigation,
            floating_wins: config.floating_wins_config,
            monitors_configs: config.monitors_config.clone(),
        }
    }
}

pub trait Rules {
    fn get_floating_config(&self, window: WindowRef) -> Option<FloatingWinsConfig>;
    fn preferred_monitor(&self, window: WindowRef) -> Option<String>;
}

impl Rules for Vec<WindowRule> {
    fn get_floating_config(&self, window: WindowRef) -> Option<FloatingWinsConfig> {
        get_floating_config(self, window)
    }

    fn preferred_monitor(&self, window: WindowRef) -> Option<String> {
        preferred_monitor(self, window)
    }
}

fn find_matches(rules: &[WindowRule], window: WindowRef) -> impl Iterator<Item = &WindowRule> {
    rules.iter().filter(move |r| r.filter.matches(window))
}

fn get_floating_config(rules: &[WindowRule], window: WindowRef) -> Option<FloatingWinsConfig> {
    find_matches(rules, window).find_map(|r| match &r.behavior {
        WindowBehavior::Float { config } => Some(*config),
        _ => None,
    })
}

fn preferred_monitor(rules: &[WindowRule], window: WindowRef) -> Option<String> {
    find_matches(rules, window).find_map(|r| match &r.behavior {
        WindowBehavior::Insert { monitor } => Some(monitor.clone()),
        _ => None,
    })
}
