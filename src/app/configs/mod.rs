pub(crate) mod core;
pub(crate) mod deserializers;
pub(crate) mod general;
pub(crate) mod layout;
pub(crate) mod modules;
pub(crate) mod monitors;

use core::Core;
use core::RuleConfig;
use general::General;
use layout::Layout;
use modules::Modules;
use monitors::ExtMonitorConfigs;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

use super::area_tree::layout_strategy::LayoutStrategyEnum;
use super::structs::win_matcher::WinMatcher;

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
    pub fn get_filters(&self) -> Option<WinMatcher> {
        let mut base_filters = vec![RuleConfig {
            exename: Some("mondrian.exe".to_owned()),
            classname: None,
            title: None,
        }];
        base_filters.extend(self.core.ignore_rules.clone());
        Some(WinMatcher::from(&base_filters))
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
                focalized_padding: c.layout.paddings.focalized.unwrap_or(self.layout.paddings.focalized),
            })
            .unwrap_or(MonitorConfigs {
                layout_strategy: self.get_layout_strategy(),
                tiles_padding: self.layout.paddings.tiles,
                borders_padding: self.layout.paddings.borders,
                focalized_padding: self.layout.paddings.focalized,
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
}
