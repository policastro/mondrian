use crate::app::structs::{direction::Direction, orientation::Orientation};
use enum_dispatch::enum_dispatch;

use crate::app::structs::area_tree::layout::golden_ration_layout::GoldenRatioLayoutStrategy;
use crate::app::structs::area_tree::layout::mono_axis_layout::MonoAxisLayoutStrategy;

#[enum_dispatch]
#[derive(Clone, Debug)]
pub enum AreaTreeLayoutStrategyEnum {
    GoldenRatioLayoutStrategy,
    MonoAxisLayoutStrategy,
}

impl Default for AreaTreeLayoutStrategyEnum {
    fn default() -> Self {
        AreaTreeLayoutStrategyEnum::GoldenRatioLayoutStrategy(GoldenRatioLayoutStrategy::new(
            true,
            Orientation::Horizontal,
        ))
    }
}

#[enum_dispatch(AreaTreeLayoutStrategyEnum)]
pub trait AreaTreeLayoutStrategy {
    fn reset(&mut self);
    fn get_initial_params(&self) -> (Orientation, u8);
    fn next(&mut self) -> (Direction, Orientation);
}
