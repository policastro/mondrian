use super::orientation::Orientation;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Area {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Area {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Area {
        Area { x, y, width, height }
    }

    pub fn from_array(area: [u32; 4]) -> Area {
        Area::new(area[0], area[1], area[2], area[3])
    }

    pub fn empty() -> Area {
        Area::new(0, 0, 0, 0)
    }

    pub fn get_center(&self) -> (u32, u32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    pub fn contains(&self, point: (u32, u32)) -> bool {
        point.0 >= self.x && point.0 <= self.x + self.width && point.1 >= self.y && point.1 <= self.y + self.height
    }

    pub fn get_origin(&self) -> (u32, u32) {
        (self.x, self.y)
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn get_shift(&self, other: &Area) -> (i32, i32, i32, i32) {
        (
            self.x as i32 - other.x as i32,
            self.y as i32 - other.y as i32,
            self.width as i32 - other.width as i32,
            self.height as i32 - other.height as i32,
        )
    }

    pub fn get_bottom_center(&self) -> (u32, u32) {
        (self.x + self.width / 2, self.y + self.height)
    }

    pub fn get_right_center(&self) -> (u32, u32) {
        (self.x + self.width, self.y + self.height / 2)
    }

    pub fn get_left_center(&self) -> (u32, u32) {
        (self.x, self.y + self.height / 2)
    }

    pub fn get_top_center(&self) -> (u32, u32) {
        (self.x + self.width / 2, self.y)
    }

    pub fn split(&self, ratio: u8, orientation: Orientation) -> (Area, Area) {
        let (new_width, new_height) = match orientation {
            Orientation::Vertical => ((self.width * ratio as u32) / 100, self.height),
            Orientation::Horizontal => (self.width, (self.height * ratio as u32) / 100),
        };

        let (delta_w, delta_h) = match orientation {
            Orientation::Vertical => (new_width, 0),
            Orientation::Horizontal => (0, new_height),
        };

        (
            Area::new(self.x, self.y, new_width, new_height),
            Area::new(
                self.x.saturating_add(delta_w),
                self.y.saturating_add(delta_h),
                self.width.saturating_sub(delta_w),
                self.height.saturating_sub(delta_h),
            ),
        )
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

