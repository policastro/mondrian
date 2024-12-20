use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Orientation {
    pub fn opposite(&self) -> Orientation {
        match self {
            Orientation::Horizontal => Orientation::Vertical,
            Orientation::Vertical => Orientation::Horizontal,
        }
    }
}
