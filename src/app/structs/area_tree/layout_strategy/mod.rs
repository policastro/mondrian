pub mod golden_ratio;
pub mod mono_axis;
pub mod two_step;

use crate::app::structs::{direction::Direction, orientation::Orientation};
use enum_dispatch::enum_dispatch;
use mono_axis::MonoAxisHorizontal;
use mono_axis::MonoAxisVertical;

use super::layout_strategy::golden_ratio::GoldenRatio;
use super::layout_strategy::mono_axis::MonoAxis;
use super::layout_strategy::two_step::TwoStep;

#[enum_dispatch(LayoutStrategyEnum)]
pub trait LayoutStrategy {
    fn insert_next(&mut self) -> (Direction, Option<Orientation>, Option<u8>);
    fn insert_complete(&mut self) -> (Orientation, u8);
    fn remove_next(&mut self) -> (Option<Orientation>, Option<u8>) {
        (None, None)
    }
    fn remove_complete(&mut self, _removed: bool) {}
}

#[enum_dispatch]
#[derive(Clone, Debug)]
pub enum LayoutStrategyEnum {
    GoldenRatio,
    MonoAxis,
    MonoAxisHorizontal,
    MonoAxisVertical,
    TwoStep,
}

impl Default for LayoutStrategyEnum {
    fn default() -> Self {
        LayoutStrategyEnum::GoldenRatio(GoldenRatio::default())
    }
}
