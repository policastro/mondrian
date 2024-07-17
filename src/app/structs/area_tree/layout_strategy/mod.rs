pub mod golden_ratio;
pub mod mono_axis;
pub mod two_step;

use crate::app::structs::{direction::Direction, orientation::Orientation};
use enum_dispatch::enum_dispatch;

use super::layout_strategy::golden_ratio::GoldenRatio;
use super::layout_strategy::mono_axis::MonoAxis;
use super::layout_strategy::two_step::TwoStep;

#[enum_dispatch(LayoutStrategyEnum)]
pub trait LayoutStrategy {
    fn reset(&mut self);
    fn get_initial_params(&self) -> (Orientation, u8);
    fn next(&mut self) -> (Direction, Orientation);
}

#[enum_dispatch]
#[derive(Clone, Debug)]
pub enum LayoutStrategyEnum {
    GoldenRatio,
    MonoAxis,
    TwoStep,
}

impl Default for LayoutStrategyEnum {
    fn default() -> Self {
        LayoutStrategyEnum::GoldenRatio(GoldenRatio::default())
    }
}
