pub(crate) mod core;
pub(crate) mod deserializers;
pub(crate) mod general;
pub(crate) mod layout;
pub(crate) mod modules;

use core::Core;
use core::RuleConfig;
use general::General;
use layout::Layout;
use modules::Modules;
use serde::Deserialize;
use serde::Serialize;

use super::area_tree::layout_strategy::LayoutStrategyEnum;
use super::structs::win_matcher::WinMatcher;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct AppConfigs {
    pub general: General,
    pub layout: Layout,
    pub core: Core,
    pub modules: Modules,
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
        let app_layout_strategy: LayoutStrategyEnum = match self.layout.tiling_strategy.as_str() {
            "horizontal" => self.layout.strategy.horizontal.into(),
            "vertical" => self.layout.strategy.vertical.into(),
            "twostep" => self.layout.strategy.twostep.into(),
            "squared" => self.layout.strategy.squared.clone().into(),
            _ => self.layout.strategy.golden_ratio.into(),
        };

        app_layout_strategy
    }
}

/// Defaults
impl Default for AppConfigs {
    fn default() -> Self {
        AppConfigs {
            general: General::default(),
            layout: Layout::default(),
            core: Core::default(),
            modules: Modules::default(),
        }
    }
}
