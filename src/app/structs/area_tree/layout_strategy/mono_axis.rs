use crate::app::structs::{direction::Direction, orientation::Orientation};

use super::LayoutStrategy;

#[derive(Clone, Copy, Debug)]
pub struct MonoAxis {
    axis: Orientation,
    current_direction: Direction,
}

impl MonoAxis {
    pub fn new(axis: Orientation, direction: Direction) -> MonoAxis {
        match axis {
            Orientation::Horizontal => assert!(direction == Direction::Right || direction == Direction::Left),
            Orientation::Vertical => assert!(direction == Direction::Down || direction == Direction::Up),
        }
        MonoAxis {
            axis: axis.opposite(),
            current_direction: direction,
        }
    }
}

impl LayoutStrategy for MonoAxis {
    fn reset(&mut self) {}

    fn next(&mut self) -> (Direction, Orientation) {
        (self.current_direction, self.axis)
    }

    fn get_initial_params(&self) -> (Orientation, u8) {
        (self.axis, 50)
    }
}
