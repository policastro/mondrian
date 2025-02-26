use crate::app::area_tree::layout_strategy;
use crate::app::configs::deserializers;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct Layout {
    #[serde(deserialize_with = "to_tiling_strategy")]
    pub tiling_strategy: String,
    pub paddings: PaddingsConfigs,
    pub strategy: StrategyConfigs,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct PaddingsConfigs {
    #[serde(deserialize_with = "deserializers::to_u8_max::<100,_>")]
    pub tiles: u8,
    #[serde(deserialize_with = "deserializers::to_u8_max::<100,_>")]
    pub borders: u8,
    #[serde(deserialize_with = "deserializers::to_u8_max::<120,_>")]
    pub focalized: u8,
}

impl Default for PaddingsConfigs {
    fn default() -> Self {
        PaddingsConfigs {
            tiles: 12,
            borders: 18,
            focalized: 8,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct StrategyConfigs {
    pub golden_ratio: layout_strategy::golden_ratio::GoldenRatio,
    pub twostep: layout_strategy::two_step::TwoStep,
    pub horizontal: layout_strategy::mono_axis::MonoAxisHorizontal,
    pub vertical: layout_strategy::mono_axis::MonoAxisVertical,
    pub squared: layout_strategy::squared::Squared,
}

impl Default for Layout {
    fn default() -> Self {
        Layout {
            tiling_strategy: "golden_ratio".to_string(),
            paddings: PaddingsConfigs::default(),
            strategy: StrategyConfigs::default(),
        }
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
