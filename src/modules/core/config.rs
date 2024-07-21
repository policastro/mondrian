use crate::app::{
    config::{app_configs::AppConfigs, filters::window_match_filter::WinMatchAnyFilters},
    structs::area_tree::layout_strategy::LayoutStrategyEnum,
};

pub struct CoreModuleConfigs {
    pub refresh_time: u64,
    pub filter: Option<WinMatchAnyFilters>,
    pub layout_strategy: LayoutStrategyEnum,
    pub tiles_padding: u8,
    pub border_padding: u8,
    pub insert_in_monitor: bool,
}

impl Default for CoreModuleConfigs {
    fn default() -> Self {
        CoreModuleConfigs {
            refresh_time: 50,
            filter: None,
            layout_strategy: LayoutStrategyEnum::default(),
            tiles_padding: 0,
            border_padding: 0,
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
            refresh_time: app_configs.refresh_time,
            filter: app_configs.filter.clone(),
            layout_strategy: app_configs.layout_strategy.clone(),
            tiles_padding: app_configs.tiles_padding,
            border_padding: app_configs.border_padding,
            insert_in_monitor: app_configs.insert_in_monitor,
        }
    }
}
