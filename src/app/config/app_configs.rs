use std::path::PathBuf;

use crate::{
    app::structs::area_tree::layout_strategy::LayoutStrategyEnum, modules::overlay::lib::color::Color,
};

use super::{
    ext_configs::{ExtBindingConfig, ExtConfig, ExtFilterConfig, ExtLayoutConfig},
    filters::window_match_filter::WinMatchAnyFilters,
};

#[derive(Debug, Clone)]
pub struct AppConfigs {
    pub filter: Option<WinMatchAnyFilters>,
    pub layout_strategy: LayoutStrategyEnum,
    pub tiles_padding: u8,
    pub border_padding: u8,
    pub refresh_time: u64,
    pub overlay_enabled: bool,
    pub overlay_thickness: u8,
    pub overlay_color: Color,
    pub overlay_padding: u8,
    pub keybinds_enabled: bool,
    pub bindings: Vec<ExtBindingConfig>,
    pub insert_in_monitor: bool,
}

impl AppConfigs {
    pub fn from_file(path: &PathBuf) -> Result<AppConfigs, toml::de::Error> {
        let file_content = std::fs::read_to_string(path).expect("Something went wrong reading the file");

        let cfg: ExtConfig = toml::from_str(&file_content)?;

        let filter = Self::extract_filters(&cfg);
        let layout_strategy = Self::extract_tiling_layout(&cfg.layout);
        let refresh_time = cfg.advanced.refresh_time;
        let tiles_padding = cfg.layout.tiles_padding;
        let border_padding = cfg.layout.border_padding;
        let overlay_enabled = cfg.modules.overlay;
        let overlay_thickness = cfg.overlay.thickness;
        let overlay_color = Color::from(cfg.overlay.color);
        let overlay_padding = cfg.overlay.padding;
        let keybinds_enabled = cfg.modules.keybindings;
        let default_mod = cfg.keybindings.default_modifier;
        let bindings = cfg
            .keybindings
            .bindings
            .into_iter()
            .map(|b| ExtBindingConfig {
                modifier: b.modifier.or(Some(default_mod.clone())),
                ..b
            })
            .collect();
        let insert_in_monitor = cfg.layout.insert_in_monitor;

        Ok(AppConfigs {
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
            bindings,
            insert_in_monitor,
        })
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

    fn extract_tiling_layout(configs: &ExtLayoutConfig) -> LayoutStrategyEnum {
        let app_layout_strategy: LayoutStrategyEnum = match configs.tiling_strategy.as_str() {
            "horizontal" => LayoutStrategyEnum::from(configs.horizontal.clone()),
            "vertical" => LayoutStrategyEnum::from(configs.vertical.clone()),
            "twostep" => LayoutStrategyEnum::from(configs.twostep.clone()),
            _ => LayoutStrategyEnum::from(configs.golden_ratio.clone()),
        };

        app_layout_strategy
    }
}
