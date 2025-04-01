use super::{direction::Direction, orientation::Orientation, point::Point};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Area {
    pub x: i32,
    pub y: i32,
    pub width: u16,
    pub height: u16,
}

impl Default for Area {
    fn default() -> Self {
        Area::new(0, 0, 0, 0)
    }
}

impl Area {
    pub fn new(x: i32, y: i32, width: u16, height: u16) -> Area {
        Area { x, y, width, height }
    }

    pub fn get_center(&self) -> (i32, i32) {
        (self.x + i32::from(self.width / 2), self.y + i32::from(self.height / 2))
    }

    pub fn distance(&self, point: (i32, i32)) -> u32 {
        self.get_center().distance(point)
    }

    pub fn contains(&self, point: (i32, i32)) -> bool {
        self.contains_x(point.0) && self.contains_y(point.1)
    }

    pub fn contains_x(&self, x: i32) -> bool {
        x >= self.x && x <= self.x + i32::from(self.width)
    }

    pub fn contains_y(&self, y: i32) -> bool {
        y >= self.y && y <= self.y + i32::from(self.height)
    }

    pub fn get_origin(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn get_shift(&self, other: &Area) -> (i32, i32, i32, i32) {
        (
            self.x - other.x,
            self.y - other.y,
            self.width as i32 - other.width as i32,
            self.height as i32 - other.height as i32,
        )
    }

    pub fn calc_area(&self) -> u32 {
        u32::from(self.width) * u32::from(self.height)
    }

    pub fn get_ne_corner(&self) -> (i32, i32) {
        (self.x + i32::from(self.width), self.y)
    }

    pub fn get_nw_corner(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn get_sw_corner(&self) -> (i32, i32) {
        (self.x, self.y + i32::from(self.height))
    }

    pub fn get_se_corner(&self) -> (i32, i32) {
        (self.x + i32::from(self.width), self.y + i32::from(self.height))
    }

    pub fn get_all_corners(&self) -> [(i32, i32); 4] {
        [
            self.get_nw_corner(),
            self.get_ne_corner(),
            self.get_sw_corner(),
            self.get_se_corner(),
        ]
    }

    pub fn get_center_in_direction(&self, direction: Direction) -> (i32, i32) {
        match direction {
            Direction::Down => self.get_bottom_center(),
            Direction::Left => self.get_left_center(),
            Direction::Right => self.get_right_center(),
            Direction::Up => self.get_top_center(),
        }
    }

    pub fn get_bottom_center(&self) -> (i32, i32) {
        (self.x + i32::from(self.width / 2), self.y + i32::from(self.height))
    }

    pub fn get_right_center(&self) -> (i32, i32) {
        (self.x + i32::from(self.width), self.y + i32::from(self.height / 2))
    }

    pub fn get_left_center(&self) -> (i32, i32) {
        (self.x, self.y + i32::from(self.height / 2))
    }

    pub fn get_top_center(&self) -> (i32, i32) {
        (self.x + i32::from(self.width / 2), self.y)
    }

    pub fn get_edge(&self, direction: Direction) -> i32 {
        match direction {
            Direction::Down => self.get_bottom_edge(),
            Direction::Left => self.get_left_edge(),
            Direction::Right => self.get_right_edge(),
            Direction::Up => self.get_top_edge(),
        }
    }

    pub fn get_right_edge(&self) -> i32 {
        self.x + i32::from(self.width)
    }

    pub fn get_bottom_edge(&self) -> i32 {
        self.y + i32::from(self.height)
    }

    pub fn get_left_edge(&self) -> i32 {
        self.x
    }

    pub fn get_top_edge(&self) -> i32 {
        self.y
    }

    pub fn get_bottom_corners(&self) -> [(i32, i32); 2] {
        [self.get_sw_corner(), self.get_se_corner()]
    }

    pub fn get_right_corners(&self) -> [(i32, i32); 2] {
        [self.get_ne_corner(), self.get_se_corner()]
    }

    pub fn get_left_corners(&self) -> [(i32, i32); 2] {
        [self.get_nw_corner(), self.get_sw_corner()]
    }

    pub fn get_top_corners(&self) -> [(i32, i32); 2] {
        [self.get_nw_corner(), self.get_ne_corner()]
    }

    pub fn get_corners(&self, direction: Direction) -> [(i32, i32); 2] {
        match direction {
            Direction::Down => self.get_bottom_corners(),
            Direction::Right => self.get_right_corners(),
            Direction::Left => self.get_left_corners(),
            Direction::Up => self.get_top_corners(),
        }
    }

    pub fn split(&self, ratio: u8, orientation: Orientation) -> (Area, Area) {
        let (new_width, new_height) = match orientation {
            Orientation::Vertical => (Area::get_percent(self.width, ratio), self.height),
            Orientation::Horizontal => (self.width, Area::get_percent(self.height, ratio)),
        };

        let (delta_w, delta_h) = match orientation {
            Orientation::Vertical => (new_width, 0),
            Orientation::Horizontal => (0, new_height),
        };

        (
            Area::new(self.x, self.y, new_width, new_height),
            Area::new(
                self.x.saturating_add(i32::from(delta_w)),
                self.y.saturating_add(i32::from(delta_h)),
                self.width.saturating_sub(delta_w),
                self.height.saturating_sub(delta_h),
            ),
        )
    }

    pub fn shift(&self, shift: (i16, i16, i16, i16)) -> Area {
        Area::new(
            self.x + i32::from(shift.0),
            self.y + i32::from(shift.1),
            (self.width as i16 + shift.2).max(0) as u16,
            (self.height as i16 + shift.3).max(0) as u16,
        )
    }

    pub fn pad(&self, padding_x: (i16, i16), padding_y: (i16, i16)) -> Area {
        let new_width = Self::add_to_dimension(self.width, padding_x.0 + padding_x.1);
        let new_height = Self::add_to_dimension(self.height, padding_y.0 + padding_y.1);

        Area::new(
            self.x + i32::from(padding_x.0),
            self.y + i32::from(padding_y.0),
            new_width,
            new_height,
        )
    }

    pub fn pad_full(&self, padding: i16) -> Area {
        self.pad((padding, padding), (padding, padding))
    }

    pub fn pad_xy(&self, (px, py): (i16, i16)) -> Area {
        self.pad((px, px), (py, py))
    }

    pub fn overlaps_y(&self, y1: i32, y2: i32) -> bool {
        let (y1, y2) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
        self.y + self.height as i32 >= y1 && self.y <= y2
    }

    pub fn overlaps_x(&self, x1: i32, x2: i32) -> bool {
        let (x1, x2) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        self.x + self.width as i32 >= x1 && self.x <= x2
    }

    pub fn clamp(&self, other: &Area) -> Area {
        Area::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.width.min(other.width),
            self.height.min(other.height),
        )
    }

    fn get_percent(value: u16, percent: u8) -> u16 {
        assert!(percent <= 100);
        (f32::from(value) * (f32::from(percent) / 100.0)).round() as u16
    }

    fn add_to_dimension(value: u16, delta: i16) -> u16 {
        match delta > 0 {
            true => value.saturating_sub(delta.unsigned_abs()),
            false => value.saturating_add(delta.unsigned_abs()),
        }
    }
}

impl From<(i32, i32, u16, u16)> for Area {
    fn from(val: (i32, i32, u16, u16)) -> Self {
        Area {
            x: val.0,
            y: val.1,
            width: val.2,
            height: val.3,
        }
    }
}

impl From<(i32, i32, i32, i32)> for Area {
    fn from(val: (i32, i32, i32, i32)) -> Self {
        Area {
            x: val.0,
            y: val.1,
            width: val.2.max(0) as u16,
            height: val.3.max(0) as u16,
        }
    }
}

impl From<Area> for (i32, i32, i32, i32) {
    fn from(val: Area) -> Self {
        (val.x, val.y, val.width as i32, val.height as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::{Area, Orientation};

    #[test]
    fn test_area() {
        let area1 = Area::new(0, 0, 100, 100);
        let area2 = Area {
            x: 0,
            y: 0,
            width: 100,
            height: 100,
        };

        assert_eq!(area1, area2);
    }

    #[test]
    fn test_split() {
        let area = Area::new(0, 0, 100, 100);
        let (area_v1, area_v2) = area.split(50, Orientation::Vertical);
        let (area_h1, area_h2) = area.split(50, Orientation::Horizontal);

        assert_eq!(area_v1, Area::new(0, 0, 50, 100));
        assert_eq!(area_v2, Area::new(50, 0, 50, 100));
        assert_eq!(area_h1, Area::new(0, 0, 100, 50));
        assert_eq!(area_h2, Area::new(0, 50, 100, 50));
    }

    #[test]
    fn test_contains() {
        let area = Area::new(0, 0, 100, 100);

        assert!(area.contains((0, 0)));
        assert!(area.contains((50, 50)));
        assert!(area.contains((100, 100)));
        assert!(!area.contains((101, 101)));
    }

    #[test]
    fn test_get_center() {
        let area1 = Area::new(0, 0, 100, 100);
        let area2 = Area::new(0, 0, 99, 99);

        assert_eq!(area1.get_center(), (50, 50));
        assert_eq!(area2.get_center(), (49, 49));
    }
}
