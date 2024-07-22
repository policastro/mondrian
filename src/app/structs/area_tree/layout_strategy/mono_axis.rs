use serde::Deserialize;

use crate::app::structs::{direction::Direction, orientation::Orientation};

use super::LayoutStrategy;

#[derive(Clone, Copy, Debug)]
pub struct MonoAxis {
    axis: Orientation,
    current_direction: Direction,
    count: u8,
    curr_count: u8,
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
            count: 0,
            curr_count: 0,
        }
    }

    fn get_current_ratio(&self, divisor: u8) -> u8 {
        let ratio = 100 / divisor;
        match self.current_direction {
            Direction::Right | Direction::Down => ratio,
            Direction::Left | Direction::Up => 100 - ratio,
        }
    }
}

impl LayoutStrategy for MonoAxis {
    fn insert_next(&mut self) -> (Direction, Option<Orientation>, Option<u8>) {
        let ratio = self.get_current_ratio(self.curr_count + 1);
        self.curr_count -= u8::min(1, self.count);
        (self.current_direction, None, Some(ratio))
    }

    fn insert_complete(&mut self) -> (Orientation, u8) {
        self.count += 1;
        self.curr_count = self.count;
        (self.axis, 50)
    }

    fn remove_next(&mut self) -> (Option<Orientation>, Option<u8>) {
        self.curr_count -= u8::min(1, self.count);
        let ratio = self.get_current_ratio(self.curr_count);
        (None, Some(ratio))
    }

    fn remove_complete(&mut self, removed: bool) {
        if removed {
            self.count -= 1;
        }
        self.curr_count = self.count;
    }
}

impl Default for MonoAxis {
    fn default() -> Self {
        MonoAxis {
            axis: Orientation::Vertical,
            current_direction: Direction::Down,
            count: 0,
            curr_count: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
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
    fn insert_next(&mut self) -> (Direction, Option<Orientation>, Option<u8>) {
        self.layout.insert_next()
    }

    fn insert_complete(&mut self) -> (Orientation, u8) {
        self.layout.insert_complete()
    }

    fn remove_next(&mut self) -> (Option<Orientation>, Option<u8>) {
        self.layout.remove_next()
    }

    fn remove_complete(&mut self, removed: bool) {
        self.layout.remove_complete(removed);
    }
}

impl Default for MonoAxisVertical {
    fn default() -> Self {
        MonoAxisVertical::new(true)
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
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
    fn insert_next(&mut self) -> (Direction, Option<Orientation>, Option<u8>) {
        self.layout.insert_next()
    }

    fn insert_complete(&mut self) -> (Orientation, u8) {
        self.layout.insert_complete()
    }

    fn remove_next(&mut self) -> (Option<Orientation>, Option<u8>) {
        self.layout.remove_next()
    }

    fn remove_complete(&mut self, removed: bool) {
        self.layout.remove_complete(removed);
    }
}

impl Default for MonoAxisHorizontal {
    fn default() -> Self {
        MonoAxisHorizontal::new(true)
    }
}
