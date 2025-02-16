use super::win_matcher::WinMatcher;
use crate::app::area_tree::layout_strategy;
use crate::app::area_tree::layout_strategy::golden_ratio::GoldenRatio;
use crate::app::area_tree::layout_strategy::mono_axis::MonoAxisHorizontal;
use crate::app::area_tree::layout_strategy::mono_axis::MonoAxisVertical;
use crate::app::area_tree::layout_strategy::squared::Squared;
use crate::app::area_tree::layout_strategy::two_step::TwoStep;
use crate::app::area_tree::layout_strategy::LayoutStrategyEnum;
use crate::modules::keybindings::configs::KeybindingsModuleConfigs;
use crate::modules::overlays::configs::OverlaysModuleConfigs;
use crate::modules::tiles_manager::lib::window_animation_player::WindowAnimation;
use serde::Deserialize;
use serde::Serialize;

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

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Core {
    pub rules: Vec<RuleConfig>,
    #[serde(default)]
    pub move_cursor_on_focus: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct Layout {
    #[serde(deserialize_with = "deserializers::to_tiling_strategy")]
    pub tiling_strategy: String,
    pub animations_enabled: bool,
    #[serde(deserialize_with = "deserializers::to_u32_minmax::<100,10000,_>")]
    pub animations_duration: u32,
    #[serde(deserialize_with = "deserializers::to_u8_minmax::<10,240,_>")]
    pub animations_framerate: u8,
    pub animation_type: Option<WindowAnimation>,
    #[serde(deserialize_with = "deserializers::to_u8_max::<100,_>")]
    pub tiles_padding: u8,
    #[serde(deserialize_with = "deserializers::to_u8_max::<100,_>")]
    pub border_padding: u8,
    #[serde(deserialize_with = "deserializers::to_u8_max::<120,_>")]
    pub focalized_padding: u8,
    pub insert_in_monitor: bool,
    pub free_move_in_monitor: bool,
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

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct Advanced {
    pub detect_maximized_windows: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Modules {
    pub keybindings: KeybindingsModuleConfigs,
    pub overlays: OverlaysModuleConfigs,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
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
            detect_maximized_windows: true,
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        Layout {
            tiling_strategy: "golden_ratio".to_string(),
            tiles_padding: 12,
            animations_enabled: true,
            animations_duration: 300,
            animations_framerate: 60,
            animation_type: Some(WindowAnimation::default()),
            border_padding: 18,
            focalized_padding: 8,
            golden_ratio: GoldenRatio::default(),
            horizontal: MonoAxisHorizontal::default(),
            vertical: MonoAxisVertical::default(),
            twostep: TwoStep::default(),
            squared: Squared::default(),
            insert_in_monitor: true,
            free_move_in_monitor: false,
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

    pub fn to_u8_max<'de, const MAX: u8, D>(deserializer: D) -> Result<u8, D::Error>
    where
        D: Deserializer<'de>,
    {
        to_u8_minmax::<'de, { u8::MIN }, MAX, D>(deserializer)
    }

    pub fn to_u8_minmax<'de, const MIN: u8, const MAX: u8, D>(deserializer: D) -> Result<u8, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: u8 = u8::deserialize(deserializer)?;
        match v >= MIN && v <= MAX {
            true => Ok(v),
            false => Err(D::Error::custom(format!(
                "value must be between {MIN} and {MAX} (inclusive)"
            ))),
        }
    }

    pub fn to_u32_minmax<'de, const MIN: u32, const MAX: u32, D>(deserializer: D) -> Result<u32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: u32 = u32::deserialize(deserializer)?;
        match v >= MIN && v <= MAX {
            true => Ok(v),
            false => Err(D::Error::custom(format!(
                "value must be between {MIN} and {MAX} (inclusive)"
            ))),
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
