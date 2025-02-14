use super::orientation::Orientation;

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

    pub fn contains(&self, point: (i32, i32)) -> bool {
        point.0 >= self.x
            && point.0 <= self.x + i32::from(self.width)
            && point.1 >= self.y
            && point.1 <= self.y + i32::from(self.height)
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

    pub fn get_area(&self) -> u32 {
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

    pub fn get_corners(&self) -> [(i32, i32); 4] {
        [
            self.get_nw_corner(),
            self.get_ne_corner(),
            self.get_sw_corner(),
            self.get_se_corner(),
        ]
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
