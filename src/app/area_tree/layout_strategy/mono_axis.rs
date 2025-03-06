use serde::{Deserialize, Serialize};

use crate::app::structs::{direction::Direction, orientation::Orientation};

use super::{LayoutStrategy, TreeOperation};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonoAxis {
    axis: Orientation,
    curr_direction: Direction,
    curr_count: u8,
    curr_operation: Option<TreeOperation>,
}

impl MonoAxis {
    pub fn new(axis: Orientation, direction: Direction) -> MonoAxis {
        match axis {
            Orientation::Horizontal => assert!(direction == Direction::Right || direction == Direction::Left),
            Orientation::Vertical => assert!(direction == Direction::Down || direction == Direction::Up),
        }
        MonoAxis {
            axis: axis.opposite(),
            curr_direction: direction,
            curr_count: 0,
            curr_operation: None,
        }
    }

    fn get_current_ratio(&self, divisor: u8) -> u8 {
        let ratio = 100 / divisor;
        match self.curr_direction {
            Direction::Right | Direction::Down => ratio,
            Direction::Left | Direction::Up => 100 - ratio,
        }
    }
}

impl LayoutStrategy for MonoAxis {
    fn init(&mut self, curr_count: u8, operation: TreeOperation) {
        self.curr_count = curr_count;
        self.curr_operation = Some(operation);
    }

    fn next(&mut self) -> (Direction, Option<Orientation>, Option<u8>) {
        self.curr_count -= u8::min(1, self.curr_count);
        let coeff = match self.curr_operation.expect("Init should have been called") {
            TreeOperation::Insert => 2,
            TreeOperation::Remove => 1,
        };
        let ratio = self.get_current_ratio(self.curr_count + coeff);
        (Direction::Right, None, Some(ratio))
    }

    fn complete(&mut self) -> (Orientation, u8) {
        (self.axis, 50)
    }
}

impl Default for MonoAxis {
    fn default() -> Self {
        MonoAxis {
            axis: Orientation::Vertical,
            curr_direction: Direction::Down,
            curr_count: 0,
            curr_operation: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct MonoAxisVertical {
    grow_down: bool,
    #[serde(skip)]
    layout: MonoAxis,
}

impl MonoAxisVertical {
    pub fn new(grow_down: bool) -> Self {
        Self {
            grow_down,
            layout: MonoAxis::new(
                Orientation::Vertical,
                if grow_down { Direction::Down } else { Direction::Up },
            ),
        }
    }
}

impl LayoutStrategy for MonoAxisVertical {
    fn init(&mut self, curr_count: u8, operation: TreeOperation) {
        self.layout.init(curr_count, operation);
    }

    fn next(&mut self) -> (Direction, Option<Orientation>, Option<u8>) {
        self.layout.next()
    }

    fn complete(&mut self) -> (Orientation, u8) {
        self.layout.complete()
    }
}

impl Default for MonoAxisVertical {
    fn default() -> Self {
        MonoAxisVertical::new(true)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct MonoAxisHorizontal {
    grow_right: bool,
    #[serde(skip)]
    layout: MonoAxis,
}

impl MonoAxisHorizontal {
    pub fn new(grow_right: bool) -> Self {
        Self {
            grow_right,
            layout: MonoAxis::new(
                Orientation::Horizontal,
                if grow_right { Direction::Right } else { Direction::Left },
            ),
        }
    }
}

impl LayoutStrategy for MonoAxisHorizontal {
    fn init(&mut self, curr_count: u8, operation: TreeOperation) {
        self.layout.init(curr_count, operation);
    }

    fn next(&mut self) -> (Direction, Option<Orientation>, Option<u8>) {
        self.layout.next()
    }

    fn complete(&mut self) -> (Orientation, u8) {
        self.layout.complete()
    }
}

impl Default for MonoAxisHorizontal {
    fn default() -> Self {
        MonoAxisHorizontal::new(true)
    }
}
