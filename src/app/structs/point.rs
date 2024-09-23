pub trait Point {
    fn with_offset(&self, offset_x: i32, offset_y: i32) -> (i32, i32);
    fn distance(&self, other: (i32, i32)) -> u32;
}

impl Point for (i32, i32) {
    fn with_offset(&self, offset_x: i32, offset_y: i32) -> (i32, i32) {
        (self.0.saturating_add(offset_x), self.1.saturating_add(offset_y))
    }

    fn distance(&self, other: (i32, i32)) -> u32 {
        let (x1, y1) = self;
        let (x2, y2) = other;
        ((x2 - x1).pow(2) + (y2 - y1).pow(2)) as u32
    }
}
