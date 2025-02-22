pub(crate) mod advanced;
pub(crate) mod core;
pub(crate) mod deserializers;
pub(crate) mod layout;
pub(crate) mod modules;

use advanced::Advanced;
use core::Core;
use core::RuleConfig;
use layout::Layout;
use modules::Modules;
use serde::Deserialize;
use serde::Serialize;

use super::area_tree::layout_strategy::LayoutStrategyEnum;
use super::structs::win_matcher::WinMatcher;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AppConfigs {
    #[serde(default)]
    pub layout: Layout,
    #[serde(default)]
    pub core: Core,
    #[serde(default)]
    pub modules: Modules,
    #[serde(default)]
    pub advanced: Advanced,
}

impl AppConfigs {
    pub fn get_filters(&self) -> Option<WinMatcher> {
        let mut base_filters = vec![RuleConfig {
            exename: Some("mondrian.exe".to_owned()),
            classname: None,
            title: None,
        }];
        base_filters.extend(self.core.rules.clone());
        Some(WinMatcher::from(&base_filters))
    }

    pub fn get_layout_strategy(&self) -> LayoutStrategyEnum {
        let app_layout_strategy: LayoutStrategyEnum = match self.layout.tiling_strategy.as_str() {
            "horizontal" => self.layout.horizontal.into(),
            "vertical" => self.layout.vertical.into(),
            "twostep" => self.layout.twostep.into(),
            "squared" => self.layout.squared.clone().into(),
            _ => self.layout.golden_ratio.into(),
        };

        app_layout_strategy
    }
}

/// Defaults
impl Default for AppConfigs {
    fn default() -> Self {
        AppConfigs {
            layout: Layout::default(),
            core: Core::default(),
            modules: Modules::default(),
            advanced: Advanced::default(),
        }
    }
}
