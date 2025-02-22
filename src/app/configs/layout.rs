use crate::app::area_tree::layout_strategy;
use crate::app::area_tree::layout_strategy::golden_ratio::GoldenRatio;
use crate::app::area_tree::layout_strategy::mono_axis::MonoAxisHorizontal;
use crate::app::area_tree::layout_strategy::mono_axis::MonoAxisVertical;
use crate::app::area_tree::layout_strategy::squared::Squared;
use crate::app::area_tree::layout_strategy::two_step::TwoStep;
use crate::app::configs::deserializers;
use crate::modules::tiles_manager::lib::window_animation_player::WindowAnimation;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct Layout {
    #[serde(deserialize_with = "to_tiling_strategy")]
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
