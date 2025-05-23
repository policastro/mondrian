use super::LayoutStrategy;
use super::TreeOperation;
use crate::app::configs::deserializers;
use crate::app::structs::direction::Direction;
use crate::app::structs::orientation::Orientation;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(default)]
pub struct GoldenRatio {
    #[serde(deserialize_with = "deserializers::to_u8_minmax::<10,90,_>")]
    ratio: u8,
    clockwise: bool,
    #[serde(skip)]
    current_direction: Direction,
    #[serde(
        rename = "vertical",
        deserialize_with = "deserialize_first_split",
        serialize_with = "serialize_first_split"
    )]
    first_split: Orientation,
    #[serde(skip)]
    count: u8,
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
            count: 0,
        }
    }
}

impl Default for GoldenRatio {
    fn default() -> Self {
        GoldenRatio::new(true, Orientation::Horizontal, 50)
    }
}

impl LayoutStrategy for GoldenRatio {
    fn init(&mut self, curr_count: u8, _operation: TreeOperation) {
        self.count = curr_count;
        self.reset();
    }

    fn next(&mut self) -> (Direction, Option<Orientation>, Option<u8>) {
        self.current_direction = match self.current_direction {
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Up => Direction::Right,
        };

        if !self.clockwise {
            self.current_direction = self.current_direction.opposite();
        }

        (self.current_direction, None, None)
    }

    fn complete(&mut self) -> (Orientation, u8) {
        let orientation = match self.current_direction {
            Direction::Right | Direction::Left => Orientation::Vertical,
            Direction::Up | Direction::Down => Orientation::Horizontal,
        };

        (orientation, if self.count == 1 { self.get_first_ratio() } else { 50 })
    }
}

impl GoldenRatio {
    fn get_first_ratio(&self) -> u8 {
        match self.clockwise {
            true => self.ratio,
            false => 100 - self.ratio,
        }
    }

    fn reset(&mut self) {
        self.current_direction = match self.first_split {
            Orientation::Horizontal => Direction::Up,
            Orientation::Vertical => Direction::Right,
        };
    }
}

fn deserialize_first_split<'de, D>(deserializer: D) -> Result<Orientation, D::Error>
where
    D: Deserializer<'de>,
{
    let vertical: bool = bool::deserialize(deserializer)?;
    Ok(match vertical {
        true => Orientation::Vertical,
        false => Orientation::Horizontal,
    })
}

fn serialize_first_split<S>(orientation: &Orientation, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match orientation {
        Orientation::Vertical => serializer.serialize_bool(true),
        Orientation::Horizontal => serializer.serialize_bool(false),
    }
}
