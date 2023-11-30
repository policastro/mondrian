use crate::app::structs::{direction::Direction, orientation::Orientation};

use super::layout_strategy::AreaTreeLayoutStrategy;

#[derive(Clone, Copy)]
pub struct MonoAxisLayoutStrategy {
    axis: Orientation,
    direction: Direction,
}

impl MonoAxisLayoutStrategy {
    pub fn new(axis: Orientation, direction: Direction) -> MonoAxisLayoutStrategy {
        MonoAxisLayoutStrategy {
            axis: axis.opposite(),
            direction,
        }
    }
}

impl AreaTreeLayoutStrategy for MonoAxisLayoutStrategy {
    fn reset(&mut self) {}

    fn next(&mut self) -> (Direction, Orientation) {
        return (self.direction, self.axis);
    }

    fn get_initial_params(self) -> (Orientation, u8) {
        (self.axis, 50)
    }
}
