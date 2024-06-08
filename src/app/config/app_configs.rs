use std::path::PathBuf;

use crate::{app::structs::area_tree::layout::layout_strategy::AreaTreeLayoutStrategyEnum, modules::overlay::lib::color::Color};

use super::{
    ext_configs::{ExtConfig, ExtFilterConfig, ExtLayoutConfig},
    filters::window_match_filter::WinMatchAnyFilters,
};

#[derive(Debug, Clone)]
pub struct AppConfigs {
    pub filter: Option<WinMatchAnyFilters>,
    pub layout_strategy: AreaTreeLayoutStrategyEnum,
    pub tiles_padding: u8,
    pub border_padding: u8,
    pub refresh_time: u64,
    pub overlay_enabled: bool,
    pub overlay_thickness: u8,
    pub overlay_color: Color,
    pub overlay_padding: u8,
    pub keybinds_enabled: bool,
}

impl AppConfigs {
    pub fn from_file(path: &PathBuf) -> AppConfigs {
        let file_content = std::fs::read_to_string(path).expect("Something went wrong reading the file");
        let configs: ExtConfig = toml::from_str(&file_content).unwrap();

        let filter = Self::extract_filters(&configs);
        let layout_strategy = Self::extract_tiling_layout(&configs.layout);
        let refresh_time = configs.advanced.refresh_time;
        let tiles_padding = configs.layout.tiles_padding;
        let border_padding = configs.layout.border_padding;
        let overlay_enabled = configs.modules.overlay;
        let overlay_thickness = configs.overlay.thickness;
        let overlay_color = Color::from(configs.overlay.color);
        let overlay_padding = configs.overlay.padding;
        let keybinds_enabled = configs.modules.keybindings;

        AppConfigs {
            filter,
            layout_strategy,
            refresh_time,
            tiles_padding,
            border_padding,
            overlay_enabled,
            overlay_thickness,
            overlay_color,
            overlay_padding,
            keybinds_enabled,
        }
    }

    fn extract_filters(configs: &ExtConfig) -> Option<WinMatchAnyFilters> {
        // Needed to prevent the tray icon and overlay from being filtered
        let mut base_filters = vec![ExtFilterConfig {
            exename: Some("mondrian.exe".to_owned()),
            classname: None,
            title: None,
        }];

        let cfg_filters = configs
            .rules
            .as_ref()
            .map(|r| r.filters.clone().unwrap_or_default())
            .unwrap_or_default();

        base_filters.extend(cfg_filters);

        let app_filter: Option<WinMatchAnyFilters> = Some(WinMatchAnyFilters::from(&base_filters));

        app_filter
    }

    fn extract_tiling_layout(configs: &ExtLayoutConfig) -> AreaTreeLayoutStrategyEnum {
        let app_layout_strategy: AreaTreeLayoutStrategyEnum = match configs.tiling_strategy.as_str() {
            "horizontal" => AreaTreeLayoutStrategyEnum::from(configs.horizontal.clone()),
            "vertical" => AreaTreeLayoutStrategyEnum::from(configs.vertical.clone()),
            _ => AreaTreeLayoutStrategyEnum::from(configs.golden_ratio.clone()),
        };

        app_layout_strategy
    }
}
