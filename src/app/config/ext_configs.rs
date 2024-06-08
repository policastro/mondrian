use serde::Deserialize;

use crate::app::structs::{
    area_tree::layout::{
        golden_ration_layout::GoldenRatioLayoutStrategy, layout_strategy::AreaTreeLayoutStrategyEnum,
        mono_axis_layout::MonoAxisLayoutStrategy,
    },
    direction::Direction,
    orientation::Orientation,
};

use super::filters::{
    window_filter_type::WindowFilterType,
    window_match_filter::{WinMatchAllFilters, WinMatchAnyFilters},
};

#[derive(Deserialize, Debug)]
pub struct ExtConfig {
    pub rules: Option<ExtRulesConfig>,
    #[serde(default)]
    pub layout: ExtLayoutConfig,
    #[serde(default)]
    pub advanced: AdvancedConfig,
    #[serde(default)]
    pub overlay: ExtOverlayConfig,
    #[serde(default)]
    pub modules: ExtModulesConfig,
}

#[derive(Deserialize, Debug)]
pub struct ExtRulesConfig {
    pub filters: Option<Vec<ExtFilterConfig>>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ExtLayoutConfig {
    pub tiling_strategy: String,
    pub tiles_padding: u8,
    pub border_padding: u8,
    pub golden_ratio: ExtLayoutGoldenRatioConfig,
    pub horizontal: ExtLayoutHorizontalConfig,
    pub vertical: ExtLayoutVerticalConfig,
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct ExtOverlayConfig {
    pub thickness: u8,
    pub color: (u8, u8, u8),
    pub padding: u8,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ExtLayoutGoldenRatioConfig {
    pub clockwise: bool,
    pub vertical: bool,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ExtLayoutHorizontalConfig {
    pub start_right: bool,
    pub toogle: bool,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ExtLayoutVerticalConfig {
    pub start_down: bool,
    pub toogle: bool,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AdvancedConfig {
    pub refresh_time: u64,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ExtModulesConfig {
    pub keybindings: bool,
    pub overlay: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ExtFilterConfig {
    pub classname: Option<String>,
    pub exename: Option<String>,
    pub title: Option<String>,
}


/// Defaults
impl Default for AdvancedConfig {
    fn default() -> Self {
        AdvancedConfig { refresh_time: 50 }
    }
}

impl Default for ExtModulesConfig {
    fn default() -> Self {
        ExtModulesConfig {
            keybindings: true,
            overlay: true,
        }
    }
}

impl Default for ExtOverlayConfig {
    fn default() -> Self {
        ExtOverlayConfig {
            thickness: 4,
            color: (0, 150, 148),
            padding: 0,
        }
    }
}

impl Default for ExtLayoutVerticalConfig {
    fn default() -> Self {
        ExtLayoutVerticalConfig {
            start_down: true,
            toogle: false,
        }
    }
}

impl Default for ExtLayoutGoldenRatioConfig {
    fn default() -> Self {
        ExtLayoutGoldenRatioConfig {
            clockwise: true,
            vertical: false,
        }
    }
}

impl Default for ExtLayoutHorizontalConfig {
    fn default() -> Self {
        ExtLayoutHorizontalConfig {
            start_right: true,
            toogle: false,
        }
    }
}

impl Default for ExtLayoutConfig {
    fn default() -> Self {
        ExtLayoutConfig {
            tiling_strategy: "golden_ratio".to_string(),
            tiles_padding: 4,
            border_padding: 4,
            golden_ratio: ExtLayoutGoldenRatioConfig::default(),
            horizontal: ExtLayoutHorizontalConfig::default(),
            vertical: ExtLayoutVerticalConfig::default(),
        }
    }
}

/// From implementations
impl From<&ExtFilterConfig> for WinMatchAllFilters {
    fn from(filters: &ExtFilterConfig) -> Self {
        let mut window_filters: Vec<WindowFilterType> = Vec::new();

        if let Some(exename) = &filters.exename {
            window_filters.push(WindowFilterType::Exename(exename.clone()));
        }

        if let Some(classname) = &filters.classname {
            window_filters.push(WindowFilterType::Classname(classname.clone()));
        }

        if let Some(title) = &filters.title {
            window_filters.push(WindowFilterType::Title(title.clone()));
        }

        if window_filters.is_empty() {
            panic!("The filter must specify at least one field between 'exename', 'classname' and 'title'.")
        }

        WinMatchAllFilters::new(window_filters)
    }
}

impl From<ExtLayoutGoldenRatioConfig> for AreaTreeLayoutStrategyEnum {
    fn from(golden_ratio: ExtLayoutGoldenRatioConfig) -> AreaTreeLayoutStrategyEnum {
        let axis = match golden_ratio.vertical {
            true => Orientation::Vertical,
            false => Orientation::Horizontal,
        };
        GoldenRatioLayoutStrategy::new(golden_ratio.clockwise, axis).into()
    }
}

impl From<ExtLayoutHorizontalConfig> for AreaTreeLayoutStrategyEnum {
    fn from(horizontal: ExtLayoutHorizontalConfig) -> AreaTreeLayoutStrategyEnum {
        let direction = match horizontal.start_right {
            true => Direction::Right,
            false => Direction::Left,
        };

        MonoAxisLayoutStrategy::new(Orientation::Horizontal, direction, horizontal.toogle).into()
    }
}

impl From<ExtLayoutVerticalConfig> for AreaTreeLayoutStrategyEnum {
    fn from(vertical: ExtLayoutVerticalConfig) -> AreaTreeLayoutStrategyEnum {
        let direction = match vertical.start_down {
            true => Direction::Down,
            false => Direction::Up,
        };

        MonoAxisLayoutStrategy::new(Orientation::Horizontal, direction, vertical.toogle).into()
    }
}

impl From<&Vec<ExtFilterConfig>> for WinMatchAnyFilters {
    fn from(filters: &Vec<ExtFilterConfig>) -> Self {
        WinMatchAnyFilters::new(
            filters
                .iter()
                .map(WinMatchAllFilters::from)
                .collect::<Vec<WinMatchAllFilters>>(),
        )
    }
}
