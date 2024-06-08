use crate::app::structs::{direction::Direction, orientation::Orientation};

use super::layout_strategy::AreaTreeLayoutStrategy;

#[derive(Clone, Copy, Debug)]
pub struct MonoAxisLayoutStrategy {
    axis: Orientation,
    //intial_direction: Direction,
    current_direction: Direction,
    //toogle: bool,
}

impl MonoAxisLayoutStrategy {
    pub fn new(axis: Orientation, direction: Direction, _toogle: bool) -> MonoAxisLayoutStrategy {
        MonoAxisLayoutStrategy {
            axis: axis.opposite(),
            //intial_direction: direction, // TODO fix
            current_direction: direction,
            //toogle,
        }
    }
}

impl AreaTreeLayoutStrategy for MonoAxisLayoutStrategy {
    fn reset(&mut self) {}

    fn next(&mut self) -> (Direction, Orientation) {
        (self.current_direction, self.axis)
    }

    fn get_initial_params(&self) -> (Orientation, u8) {
        (self.axis, 50)
    }
}
