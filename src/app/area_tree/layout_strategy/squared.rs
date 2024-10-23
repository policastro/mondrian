use serde::Deserialize;

use crate::app::structs::{direction::Direction, orientation::Orientation};

use super::{LayoutStrategy, TreeOperation};

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Squared {
    #[serde(skip)]
    current_dir: Direction,
    #[serde(skip)]
    count: u8,
    #[serde(skip)]
    path: Vec<Direction>,
}

impl Default for Squared {
    fn default() -> Self {
        Squared {
            current_dir: Direction::Up,
            count: 0,
            path: get_next_path(0),
        }
    }
}

impl LayoutStrategy for Squared {
    fn init(&mut self, curr_count: u8, _operation: TreeOperation) {
        self.count = curr_count;
        self.path = get_next_path(self.count);
    }

    fn next(&mut self) -> (Direction, Option<Orientation>, Option<u8>) {
        self.current_dir = self.path.pop().unwrap_or(self.current_dir);
        (self.current_dir, None, None)
    }

    fn complete(&mut self) -> (Orientation, u8) {
        let orientation = match (self.count + 1) % 3 == 2 {
            true => Orientation::Vertical,
            false => Orientation::Horizontal,
        };
        (orientation, 50)
    }
}

const SUFFIXES: [&str; 4] = ["0", "1", "01", "10"];
fn get_next_path(count: u8) -> Vec<Direction> {
    if count == 0 {
        return SUFFIXES[0].chars().map(map_char_to_dir).collect();
    }

    let factor = match count {
        0..=3 => 0,
        4..=15 => 1,
        16..=63 => 2,
        _ => 3,
    };

    let corrective = u8::from(count > 3);
    let offset = (count - 4u8.pow(factor)) / 3;
    let suffix = SUFFIXES[((count + offset + corrective) % 4) as usize];
    let mut path = format!("{:0>8b}{}", offset, suffix)
        .chars()
        .skip(8 - (factor * 2) as usize)
        .map(map_char_to_dir)
        .collect::<Vec<Direction>>();
    path.reverse();
    path
}

fn map_char_to_dir(c: char) -> Direction {
    match c {
        '0' => Direction::Left,
        '1' => Direction::Right,
        _ => unreachable!(),
    }
}
