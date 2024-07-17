use crate::app::structs::{direction::Direction, orientation::Orientation};

use super::LayoutStrategy;

#[derive(Clone, Copy, Debug)]
pub struct TwoStep {
    first_direction: Direction,
    second_direction: Direction,
    current_direction: Direction,
}

impl TwoStep {
    pub fn new(first_step: Direction, second_step: Direction) -> TwoStep {
        TwoStep {
            first_direction: first_step,
            second_direction: second_step,
            current_direction: first_step,
        }
    }
}

impl LayoutStrategy for TwoStep {
    fn reset(&mut self) {
        self.current_direction = self.first_direction;
    }

    fn next(&mut self) -> (Direction, Orientation) {
        let prev_direction = self.current_direction;
        self.current_direction = match self.current_direction == self.first_direction {
            true => self.second_direction,
            false => self.first_direction,
        };

        (
            prev_direction,
            match prev_direction {
                Direction::Right | Direction::Left => Orientation::Vertical,
                Direction::Down | Direction::Up => Orientation::Horizontal,
            },
        )
    }

    fn get_initial_params(&self) -> (Orientation, u8) {
        (
            match self.first_direction {
                Direction::Right | Direction::Left => Orientation::Vertical,
                Direction::Down | Direction::Up => Orientation::Horizontal,
            },
            50,
        )
    }
}
