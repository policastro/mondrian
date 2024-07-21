use serde::{de::Error, Deserialize, Deserializer};

use crate::app::{
    mondrian_command::MondrianMessage,
    structs::{
        area_tree::layout_strategy::{
            golden_ratio::GoldenRatio, mono_axis::MonoAxis, two_step::TwoStep, LayoutStrategyEnum,
        },
        direction::Direction,
        orientation::Orientation,
    },
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
    pub modules: ExtModulesConfig,
    #[serde(default)]
    pub overlays: ExtOverlaysConfig,
    #[serde(default)]
    pub keybindings: ExtKeybindingsConfig,
}

#[derive(Deserialize, Debug)]
pub struct ExtRulesConfig {
    pub filters: Option<Vec<ExtFilterConfig>>,
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct ExtKeybindingsConfig {
    #[serde(deserialize_with = "deserialize_modifier")]
    pub default_modifier: Vec<String>,
    pub bindings: Vec<ExtBindingConfig>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ExtLayoutConfig {
    #[serde(deserialize_with = "deserialize_tiling_strategy")]
    pub tiling_strategy: String,
    #[serde(deserialize_with = "deserialize_u8_max::<60,_>")]
    pub tiles_padding: u8,
    #[serde(deserialize_with = "deserialize_u8_max::<60,_>")]
    pub border_padding: u8,
    pub golden_ratio: ExtLayoutGoldenRatioConfig,
    pub horizontal: ExtLayoutHorizontalConfig,
    pub vertical: ExtLayoutVerticalConfig,
    pub twostep: ExtLayoutTwoStepConfig,
    pub insert_in_monitor: bool,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct ExtOverlaysConfig {
    pub active: ExtOverlayConfig,
    pub inactive: ExtOverlayConfig,
    pub follow_movements: bool,
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct ExtOverlayConfig {
    pub enabled: bool,
    pub thickness: u8,
    pub color: (u8, u8, u8),
    pub padding: u8,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ExtLayoutGoldenRatioConfig {
    pub clockwise: bool,
    pub vertical: bool,
    #[serde(deserialize_with = "deserialize_u8_max::<100,_>")]
    pub ratio: u8,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ExtLayoutHorizontalConfig {
    pub split_right: bool,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ExtLayoutVerticalConfig {
    pub split_down: bool,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ExtLayoutTwoStepConfig {
    pub first_step: Direction,
    pub second_step: Direction,
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
    pub overlays: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ExtBindingConfig {
    #[serde(default, deserialize_with = "deserialize_modifier_opt")]
    pub modifier: Option<Vec<String>>,
    #[serde(deserialize_with = "deserialize_key")]
    pub key: String,
    pub action: MondrianMessage,
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
            overlays: true,
        }
    }
}

impl Default for ExtOverlayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            thickness: 4,
            color: (0, 150, 148),
            padding: 0,
        }
    }
}

impl Default for ExtKeybindingsConfig {
    fn default() -> Self {
        ExtKeybindingsConfig {
            default_modifier: vec!["ALT".to_string()],
            bindings: vec![],
        }
    }
}

impl Default for ExtLayoutVerticalConfig {
    fn default() -> Self {
        ExtLayoutVerticalConfig { split_down: true }
    }
}

impl Default for ExtLayoutGoldenRatioConfig {
    fn default() -> Self {
        ExtLayoutGoldenRatioConfig {
            clockwise: true,
            vertical: false,
            ratio: 50,
        }
    }
}

impl Default for ExtLayoutHorizontalConfig {
    fn default() -> Self {
        ExtLayoutHorizontalConfig { split_right: true }
    }
}

impl Default for ExtLayoutTwoStepConfig {
    fn default() -> Self {
        ExtLayoutTwoStepConfig {
            first_step: Direction::Right,
            second_step: Direction::Down,
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
            twostep: ExtLayoutTwoStepConfig::default(),
            insert_in_monitor: false,
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

impl From<ExtLayoutGoldenRatioConfig> for LayoutStrategyEnum {
    fn from(golden_ratio: ExtLayoutGoldenRatioConfig) -> LayoutStrategyEnum {
        let axis = match golden_ratio.vertical {
            true => Orientation::Vertical,
            false => Orientation::Horizontal,
        };
        GoldenRatio::new(golden_ratio.clockwise, axis, golden_ratio.ratio).into()
    }
}

impl From<ExtLayoutHorizontalConfig> for LayoutStrategyEnum {
    fn from(horizontal: ExtLayoutHorizontalConfig) -> LayoutStrategyEnum {
        let direction = match horizontal.split_right {
            true => Direction::Right,
            false => Direction::Left,
        };

        MonoAxis::new(Orientation::Horizontal, direction).into()
    }
}

impl From<ExtLayoutVerticalConfig> for LayoutStrategyEnum {
    fn from(vertical: ExtLayoutVerticalConfig) -> LayoutStrategyEnum {
        let direction = match vertical.split_down {
            true => Direction::Down,
            false => Direction::Up,
        };

        MonoAxis::new(Orientation::Vertical, direction).into()
    }
}

impl From<ExtLayoutTwoStepConfig> for LayoutStrategyEnum {
    fn from(twostep: ExtLayoutTwoStepConfig) -> LayoutStrategyEnum {
        TwoStep::new(twostep.first_step, twostep.second_step).into()
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

/// Deserialization functions
fn deserialize_u8_max<'de, const L: u8, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let v: u8 = u8::deserialize(deserializer)?;
    match v <= L {
        true => Ok(v),
        false => Err(D::Error::custom(format!("Value must be less than {L}"))),
    }
}

fn deserialize_modifier<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let valid_modifiers = ["ALT", "CTRL", "SHIFT", "WIN"];

    let s: String = String::deserialize(deserializer)?;
    let keys = s.trim().split('+').map(|key| key.trim().to_uppercase());
    let is_valid = keys.clone().all(|key| valid_modifiers.contains(&key.as_str()));
    match is_valid {
        true => Ok(keys.collect::<Vec<String>>()),
        false => Err(D::Error::custom(format!("Invalid modifier: {}", s))),
    }
}

// TODO I can probably merge this with deserialize_modifier
fn deserialize_modifier_opt<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let valid_modifiers = ["ALT", "CTRL", "SHIFT", "WIN"];

    let s: Option<String> = Option::deserialize(deserializer)?;
    let s = match s {
        Some(s) => s,
        None => return Ok(None),
    };
    let keys = s.trim().split('+').map(|key| key.trim().to_uppercase());
    let is_valid = keys.clone().all(|key| valid_modifiers.contains(&key.as_str()));
    match is_valid {
        true => Ok(Some(keys.collect::<Vec<String>>())),
        false => Err(D::Error::custom(format!("Invalid modifier: {}", s))),
    }
}

fn deserialize_key<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let is_char = regex::Regex::new(r"^[A-Za-z\d]$").unwrap();
    let is_dir = regex::Regex::new(r"^\b(?i)left|right|up|down\b$").unwrap();
    let s: String = String::deserialize(deserializer)?;
    let is_valid = is_char.is_match(&s.to_uppercase()) || is_dir.is_match(&s.to_uppercase());
    match is_valid {
        true => Ok(s.to_uppercase()),
        false => Err(D::Error::custom(format!("Invalid key: {}", s))),
    }
}

fn deserialize_tiling_strategy<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let valid = ["golden_ratio", "horizontal", "vertical", "twostep"];
    let s: String = String::deserialize(deserializer)?;
    match valid.contains(&s.to_lowercase().as_str()) {
        true => Ok(s.to_lowercase()),
        false => Err(D::Error::custom(format!(
            "Invalid tiling strategy: {}, valid options are {}",
            s,
            valid.join(", ")
        ))),
    }
}
