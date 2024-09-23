pub mod golden_ratio;
pub mod mono_axis;
pub mod squared;
pub mod two_step;

use crate::app::structs::{direction::Direction, orientation::Orientation};
use enum_dispatch::enum_dispatch;
use mono_axis::MonoAxisHorizontal;
use mono_axis::MonoAxisVertical;
use squared::Squared;

use super::layout_strategy::golden_ratio::GoldenRatio;
use super::layout_strategy::mono_axis::MonoAxis;
use super::layout_strategy::two_step::TwoStep;

#[derive(Debug, Clone, Copy)]
pub enum TreeOperation {
    Insert,
    Remove,
}

#[enum_dispatch(LayoutStrategyEnum)]
pub trait LayoutStrategy {
    fn init(&mut self, curr_count: u8, operation: TreeOperation);
    fn next(&mut self) -> (Direction, Option<Orientation>, Option<u8>);
    fn complete(&mut self) -> (Orientation, u8);
}

#[enum_dispatch]
#[derive(Clone, Debug)]
pub enum LayoutStrategyEnum {
    GoldenRatio,
    MonoAxis,
    MonoAxisHorizontal,
    MonoAxisVertical,
    TwoStep,
    Squared,
}

impl Default for LayoutStrategyEnum {
    fn default() -> Self {
        LayoutStrategyEnum::GoldenRatio(GoldenRatio::default())
    }
}
