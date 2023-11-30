use crate::app::structs::{direction::Direction, orientation::Orientation};

use super::layout_strategy::AreaTreeLayoutStrategy;

#[derive(Clone, Copy)]
pub struct GoldenRatioLayoutStrategy {
    current_direction: Direction,
}

impl GoldenRatioLayoutStrategy {
    pub fn new() -> GoldenRatioLayoutStrategy {
        GoldenRatioLayoutStrategy {
            current_direction: Direction::Up,
        }
    }
}

impl AreaTreeLayoutStrategy for GoldenRatioLayoutStrategy {
    fn reset(&mut self) {
        self.current_direction = Direction::Up;
    }

    fn next(&mut self) -> (Direction, Orientation) {
        self.current_direction = match self.current_direction {
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Up => Direction::Right,
        };
        (
            self.current_direction,
            match self.current_direction {
                Direction::Right | Direction::Left => Orientation::Vertical,
                Direction::Up | Direction::Down => Orientation::Horizontal,
            },
        )
    }

    fn get_initial_params(self) -> (Orientation, u8) {
        (Orientation::Vertical, 50)
    }
}
