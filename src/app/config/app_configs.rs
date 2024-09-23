use serde::Deserialize;

use crate::{
    app::structs::area_tree::layout_strategy::{
        self,
        golden_ratio::GoldenRatio,
        mono_axis::{MonoAxisHorizontal, MonoAxisVertical},
        squared::Squared,
        two_step::TwoStep,
        LayoutStrategyEnum,
    },
    modules::{keybindings::configs::KeybindingsModuleConfigs, overlays::configs::OverlaysModuleConfigs},
};

use super::win_matcher::WinMatcher;

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Core {
    pub rules: Vec<RuleConfig>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct Layout {
    #[serde(deserialize_with = "deserializers::to_tiling_strategy")]
    pub tiling_strategy: String,
    #[serde(deserialize_with = "deserializers::to_u8_max::<60,_>")]
    pub tiles_padding: u8,
    #[serde(deserialize_with = "deserializers::to_u8_max::<60,_>")]
    pub border_padding: u8,
    #[serde(deserialize_with = "deserializers::to_u8_max::<60,_>")]
    pub focalized_padding: u8,
    pub insert_in_monitor: bool,
    #[serde(default)]
    pub golden_ratio: layout_strategy::golden_ratio::GoldenRatio,
    #[serde(default)]
    pub twostep: layout_strategy::two_step::TwoStep,
    #[serde(default)]
    pub horizontal: layout_strategy::mono_axis::MonoAxisHorizontal,
    #[serde(default)]
    pub vertical: layout_strategy::mono_axis::MonoAxisVertical,
    #[serde(default)]
    pub squared: layout_strategy::squared::Squared,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct Advanced {
    pub refresh_time: u64,
    pub detect_maximized_windows: bool,
}

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Modules {
    pub keybindings: KeybindingsModuleConfigs,
    pub overlays: OverlaysModuleConfigs,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RuleConfig {
    pub classname: Option<String>,
    pub exename: Option<String>,
    pub title: Option<String>,
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

impl Default for Advanced {
    fn default() -> Self {
        Advanced {
            refresh_time: 50,
            detect_maximized_windows: true,
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        Layout {
            tiling_strategy: "golden_ratio".to_string(),
            tiles_padding: 4,
            border_padding: 4,
            focalized_padding: 8,
            golden_ratio: GoldenRatio::default(),
            horizontal: MonoAxisHorizontal::default(),
            vertical: MonoAxisVertical::default(),
            twostep: TwoStep::default(),
            squared: Squared::default(),
            insert_in_monitor: false,
        }
    }
}

/// From implementations
impl From<&Vec<RuleConfig>> for WinMatcher {
    fn from(rules: &Vec<RuleConfig>) -> Self {
        let matchers: Vec<WinMatcher> = rules
            .iter()
            .map(|r| {
                let mut matchers: Vec<WinMatcher> = Vec::new();

                if let Some(exename) = &r.exename {
                    matchers.push(WinMatcher::Exename(exename.clone()));
                }

                if let Some(classname) = &r.classname {
                    matchers.push(WinMatcher::Classname(classname.clone()));
                }

                if let Some(title) = &r.title {
                    matchers.push(WinMatcher::Title(title.clone()));
                }

                if matchers.is_empty() {
                    panic!("The filter must specify at least one field between 'exename', 'classname' and 'title'.")
                }

                WinMatcher::All(matchers)
            })
            .collect();

        WinMatcher::Any(matchers)
    }
}

/// Deserialization functions
pub mod deserializers {
    use serde::{de::Error, Deserialize, Deserializer};

    pub fn to_u8_max<'de, const L: u8, D>(deserializer: D) -> Result<u8, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: u8 = u8::deserialize(deserializer)?;
        match v <= L {
            true => Ok(v),
            false => Err(D::Error::custom(format!("value must be less than {L}"))),
        }
    }

    pub fn to_tiling_strategy<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let valid = ["golden_ratio", "horizontal", "vertical", "twostep", "squared"];
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
}
