pub(crate) mod core;
pub(crate) mod deserializers;
pub(crate) mod general;
pub(crate) mod layout;
pub(crate) mod modules;
pub(crate) mod monitors;

use super::area_tree::layout_strategy::LayoutStrategyEnum;
use super::structs::win_matcher::WinMatcher;
use core::Core;
use core::WindowBehavior;
use core::WindowRule;
use general::General;
use layout::Layout;
use modules::Modules;
use monitors::ExtMonitorConfigs;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct AppConfigs {
    pub general: General,
    pub layout: Layout,
    pub core: Core,
    pub modules: Modules,
    monitors: HashMap<String, ExtMonitorConfigs>,
}

impl AppConfigs {
    pub fn get_ignore_filter(&self) -> WinMatcher {
        let mut filter = vec![WinMatcher::Exename("mondrian.exe".to_owned())];
        let mut rules: Vec<WinMatcher> = self
            .core
            .rules
            .iter()
            .filter(|r| matches!(r.behavior, WindowBehavior::Ignore))
            .map(|r| r.filter.clone())
            .collect();

        filter.append(&mut rules);
        match filter.len() == 1 {
            true => filter[0].clone(),
            false => WinMatcher::any(filter.into_iter()),
        }
    }

    pub fn get_rules(&self) -> Vec<WindowRule> {
        self.core
            .rules
            .iter()
            .filter(|r| !matches!(r.behavior, WindowBehavior::Ignore))
            .cloned()
            .collect()
    }

    pub fn get_layout_strategy(&self) -> LayoutStrategyEnum {
        self.string_to_layout_strategy(&self.layout.tiling_strategy)
    }

    pub fn get_monitors_configs(&self) -> HashMap<String, MonitorConfigs> {
        self.monitors
            .keys()
            .map(|k| (k.clone(), self.get_monitor_configs(k)))
            .collect()
    }

    fn get_monitor_configs(&self, monitor_name: &str) -> MonitorConfigs {
        self.monitors
            .get(monitor_name)
            .map(|c| MonitorConfigs {
                layout_strategy: c
                    .layout
                    .tiling_strategy
                    .as_ref()
                    .map(|s| self.string_to_layout_strategy(s))
                    .unwrap_or(self.get_layout_strategy()),
                tiles_padding: c.layout.paddings.tiles.unwrap_or(self.layout.paddings.tiles),
                borders_padding: c.layout.paddings.borders.unwrap_or(self.layout.paddings.borders),
                focalized_padding: c.layout.focalized_padding.unwrap_or(self.layout.focalized_padding),
                half_focalized_borders_pad: c
                    .layout
                    .half_focalized_paddings
                    .borders
                    .unwrap_or(self.layout.half_focalized_paddings.borders),
                half_focalized_tiles_pad: c
                    .layout
                    .half_focalized_paddings
                    .tiles
                    .unwrap_or(self.layout.half_focalized_paddings.tiles),
            })
            .unwrap_or(MonitorConfigs {
                layout_strategy: self.get_layout_strategy(),
                tiles_padding: self.layout.paddings.tiles,
                borders_padding: self.layout.paddings.borders,
                focalized_padding: self.layout.focalized_padding,
                half_focalized_borders_pad: self.layout.half_focalized_paddings.borders,
                half_focalized_tiles_pad: self.layout.half_focalized_paddings.tiles,
            })
    }

    fn string_to_layout_strategy(&self, s: &str) -> LayoutStrategyEnum {
        match s {
            "horizontal" => self.layout.strategy.horizontal.into(),
            "vertical" => self.layout.strategy.vertical.into(),
            "twostep" => self.layout.strategy.twostep.into(),
            "squared" => self.layout.strategy.squared.clone().into(),
            _ => self.layout.strategy.golden_ratio.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MonitorConfigs {
    pub layout_strategy: LayoutStrategyEnum,
    pub tiles_padding: u8,
    pub borders_padding: u8,
    pub focalized_padding: u8,
    pub half_focalized_borders_pad: u8,
    pub half_focalized_tiles_pad: u8,
}
