use crate::app::area_tree::layout_strategy;
use crate::app::configs::deserializers;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct Layout {
    #[serde(deserialize_with = "deserializers::to_tiling_strategy")]
    pub tiling_strategy: String,
    pub paddings: PaddingsConfigs,
    pub half_focalized_paddings: PaddingsConfigs,
    #[serde(deserialize_with = "deserializers::to_u8_max::<120,_>")]
    pub focalized_padding: u8,
    pub strategy: StrategyConfigs,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct PaddingsConfigs {
    #[serde(deserialize_with = "deserializers::to_u8_max::<100,_>")]
    pub tiles: u8,
    #[serde(deserialize_with = "deserializers::to_u8_max::<100,_>")]
    pub borders: u8,
}

impl Default for PaddingsConfigs {
    fn default() -> Self {
        PaddingsConfigs { tiles: 12, borders: 18 }
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
            half_focalized_paddings: PaddingsConfigs::default(),
            focalized_padding: 8,
            strategy: StrategyConfigs::default(),
        }
    }
}
