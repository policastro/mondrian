use std::path::PathBuf;

use crate::{
    app::structs::area_tree::layout_strategy::LayoutStrategyEnum, modules::overlays::lib::{color::Color, overlay::OverlayParams},
};

use super::{
    ext_configs::{ExtBindingConfig, ExtConfig, ExtFilterConfig, ExtLayoutConfig, ExtOverlayConfig},
    filters::window_match_filter::WinMatchAnyFilters,
};

#[derive(Debug, Clone)]
pub struct AppConfigs {
    pub filter: Option<WinMatchAnyFilters>,
    pub layout_strategy: LayoutStrategyEnum,
    pub tiles_padding: u8,
    pub border_padding: u8,
    pub refresh_time: u64,
    pub active_overlay_enabled: bool,
    pub inactive_overlay_enabled: bool,
    pub overlay_follow_movements: bool,
    pub active_overlay: OverlayParams,
    pub inactive_overlay: OverlayParams,
    pub overlays_enabled: bool,
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
        let overlays_enabled = cfg.modules.overlays;
        let active_overlay_enabled = cfg.overlays.active.enabled;
        let inactive_overlay_enabled = cfg.overlays.inactive.enabled;
        let active_overlay = Self::extract_overlay_params(&cfg.overlays.active);
        let inactive_overlay = Self::extract_overlay_params(&cfg.overlays.inactive);
        let overlay_follow_movements = cfg.overlays.follow_movements;
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
            overlays_enabled,
            active_overlay_enabled,
            inactive_overlay_enabled,
            overlay_follow_movements,
            active_overlay,
            inactive_overlay,
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

    pub fn extract_overlay_params(configs: &ExtOverlayConfig) -> OverlayParams {
        let color = Color::new(configs.color.0, configs.color.1, configs.color.2);
        OverlayParams::new(color, configs.thickness, configs.padding)
    }
}
