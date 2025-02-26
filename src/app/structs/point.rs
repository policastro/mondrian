pub trait Point {
    fn distance(&self, other: (i32, i32)) -> u32;
    fn same(&self, other: (i32, i32)) -> bool;
}

impl Point for (i32, i32) {
    fn distance(&self, other: (i32, i32)) -> u32 {
        let ((x1, y1), (x2, y2)) = (self, other);
        ((x2 - x1).pow(2) + (y2 - y1).pow(2)) as u32
    }

    fn same(&self, other: (i32, i32)) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}
