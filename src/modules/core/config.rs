use crate::app::{
    config::{app_configs::AppConfigs, filters::window_match_filter::WinMatchAnyFilters},
    structs::area_tree::layout::layout_strategy::AreaTreeLayoutStrategyEnum,
};

pub struct CoreConfigs {
    pub refresh_time: u64,
    pub filter: Option<WinMatchAnyFilters>,
    pub layout_strategy: AreaTreeLayoutStrategyEnum,
    pub tiles_padding: u8,
    pub border_padding: u8,
}

impl Default for CoreConfigs {
    fn default() -> Self {
        CoreConfigs {
            refresh_time: 50,
            filter: None,
            layout_strategy: AreaTreeLayoutStrategyEnum::default(),
            tiles_padding: 0,
            border_padding: 0,
        }
    }
}

impl CoreConfigs {
    pub fn get_layout(&self, _display_name: Option<&String>) -> AreaTreeLayoutStrategyEnum {
        //TODO implement display_name support
        self.layout_strategy.clone()
    }
}

impl From<&AppConfigs> for CoreConfigs {
    fn from(app_configs: &AppConfigs) -> Self {
        CoreConfigs {
            refresh_time: app_configs.refresh_time,
            filter: app_configs.filter.clone(),
            layout_strategy: app_configs.layout_strategy.clone(),
            tiles_padding: app_configs.tiles_padding,
            border_padding: app_configs.border_padding,
        }
    }
}
