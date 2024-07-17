use crate::app::structs::{direction::Direction, orientation::Orientation};

use super::LayoutStrategy;

#[derive(Clone, Copy, Debug)]
pub struct GoldenRatio {
    current_direction: Direction,
    clockwise: bool,
    first_split: Orientation,
    ratio: u8,
}

impl GoldenRatio {
    pub fn new(clockwise: bool, first_split: Orientation, ratio: u8) -> GoldenRatio {
        assert!(ratio > 0 && ratio < 100);
        GoldenRatio {
            current_direction: match first_split {
                Orientation::Horizontal => Direction::Up,
                Orientation::Vertical => Direction::Right,
            },
            clockwise,
            first_split,
            ratio,
        }
    }
}

impl Default for GoldenRatio {
    fn default() -> Self {
        GoldenRatio::new(true, Orientation::Horizontal, 50)
    }
}

impl LayoutStrategy for GoldenRatio {
    fn reset(&mut self) {
        self.current_direction = match self.first_split {
            Orientation::Horizontal => Direction::Up,
            Orientation::Vertical => Direction::Right,
        };
    }

    fn next(&mut self) -> (Direction, Orientation) {
        self.current_direction = match self.current_direction {
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Up => Direction::Right,
        };

        if !self.clockwise {
            self.current_direction = self.current_direction.opposite();
        }

        (
            self.current_direction,
            match self.current_direction {
                Direction::Right | Direction::Left => Orientation::Vertical,
                Direction::Up | Direction::Down => Orientation::Horizontal,
            },
        )
    }

    fn get_initial_params(&self) -> (Orientation, u8) {
        (
            Orientation::Vertical,
            if self.clockwise { self.ratio } else { 100 - self.ratio },
        )
    }
}
