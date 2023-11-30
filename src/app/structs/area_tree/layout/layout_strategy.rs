use crate::app::structs::{direction::Direction, orientation::Orientation};

pub trait AreaTreeLayoutStrategy {
    fn reset(&mut self);
    fn get_initial_params(self) -> (Orientation, u8);
    fn next(&mut self) -> (Direction, Orientation);
}
