use crate::app::{
    config::{
        app_configs::AppConfigs,
        win_matcher::{self},
    },
    structs::area_tree::layout_strategy::LayoutStrategyEnum,
};

pub struct CoreModuleConfigs {
    pub refresh_time: u64,
    pub detect_maximized_windows: bool,
    pub filter: Option<win_matcher::WinMatcher>,
    pub layout_strategy: LayoutStrategyEnum,
    pub tiles_padding: u8,
    pub border_padding: u8,
    pub focalized_padding: u8,
    pub insert_in_monitor: bool,
}

impl Default for CoreModuleConfigs {
    fn default() -> Self {
        CoreModuleConfigs {
            refresh_time: 50,
            detect_maximized_windows: true,
            filter: None,
            layout_strategy: LayoutStrategyEnum::default(),
            tiles_padding: 0,
            border_padding: 0,
            focalized_padding: 0,
            insert_in_monitor: false,
        }
    }
}

impl CoreModuleConfigs {
    pub fn get_layout(&self, _display_name: Option<&String>) -> LayoutStrategyEnum {
        //TODO implement display_name support
        self.layout_strategy.clone()
    }
}

impl From<&AppConfigs> for CoreModuleConfigs {
    fn from(app_configs: &AppConfigs) -> Self {
        CoreModuleConfigs {
            refresh_time: app_configs.advanced.refresh_time,
            detect_maximized_windows: app_configs.advanced.detect_maximized_windows,
            filter: app_configs.get_filters(),
            layout_strategy: app_configs.get_layout_strategy(),
            tiles_padding: app_configs.layout.tiles_padding,
            border_padding: app_configs.layout.border_padding,
            focalized_padding: app_configs.layout.focalized_padding,
            insert_in_monitor: app_configs.layout.insert_in_monitor,
        }
    }
}
