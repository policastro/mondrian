use super::LayoutStrategy;
use super::TreeOperation;
use crate::app::configs::deserializers;
use crate::app::structs::direction::Direction;
use crate::app::structs::orientation::Orientation;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(default)]
pub struct TwoStep {
    #[serde(rename = "first_step")]
    first_dir: Direction,
    #[serde(rename = "second_step")]
    second_dir: Direction,
    #[serde(deserialize_with = "deserializers::to_u8_minmax::<10,90,_>")]
    ratio: u8,
    #[serde(skip)]
    current_dir: Direction,
    #[serde(skip)]
    count: u8,
}

impl Default for TwoStep {
    fn default() -> Self {
        TwoStep {
            first_dir: Direction::Right,
            second_dir: Direction::Down,
            current_dir: Direction::Right,
            ratio: 50,
            count: 0,
        }
    }
}

impl LayoutStrategy for TwoStep {
    fn init(&mut self, curr_count: u8, _operation: TreeOperation) {
        self.current_dir = self.second_dir;
        self.count = curr_count;
    }

    fn next(&mut self) -> (Direction, Option<Orientation>, Option<u8>) {
        self.current_dir = match self.current_dir == self.first_dir {
            true => self.second_dir,
            false => self.first_dir,
        };
        (self.current_dir, None, None)
    }

    fn complete(&mut self) -> (Orientation, u8) {
        let orientation = match self.current_dir {
            Direction::Right | Direction::Left => Orientation::Vertical,
            Direction::Down | Direction::Up => Orientation::Horizontal,
        };
        (orientation, if self.count == 1 { self.ratio } else { 50 })
    }
}
