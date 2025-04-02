use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq)]
pub struct Paddings {
    pub left: u8,
    pub right: u8,
    pub top: u8,
    pub bottom: u8,
}

impl Paddings {
    pub fn new(top: u8, right: u8, bottom: u8, left: u8) -> Self {
        Paddings {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn full(padding: u8) -> Self {
        Paddings::new(padding, padding, padding, padding)
    }
}
