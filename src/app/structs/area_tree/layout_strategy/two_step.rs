use serde::Deserialize;

use crate::app::structs::{direction::Direction, orientation::Orientation};

use super::LayoutStrategy;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(default)]
pub struct TwoStep {
    #[serde(rename = "first_step")]
    first_dir: Direction,
    #[serde(rename = "second_step")]
    second_dir: Direction,
    #[serde(skip)]
    current_dir: Direction,
}

impl Default for TwoStep {
    fn default() -> Self {
        TwoStep {
            first_dir: Direction::Right,
            second_dir: Direction::Down,
            current_dir: Direction::Right,
        }
    }
}

impl LayoutStrategy for TwoStep {
    fn insert_next(&mut self) -> (Direction, Option<Orientation>, Option<u8>) {
        self.current_dir = match self.current_dir == self.first_dir {
            true => self.second_dir,
            false => self.first_dir,
        };
        (self.current_dir, None, None)
    }

    fn insert_complete(&mut self) -> (Orientation, u8) {
        let orientation = match self.current_dir {
            Direction::Right | Direction::Left => Orientation::Vertical,
            Direction::Down | Direction::Up => Orientation::Horizontal,
        };
        let result = (orientation, 50);
        self.current_dir = self.second_dir;
        result
    }
}
