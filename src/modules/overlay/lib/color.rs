#[derive(Debug, Clone, Copy, Default)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

impl From<Color> for u32 {
    fn from(color: Color) -> Self {
        color.red as u32 | (color.green as u32) << 8 | (color.blue as u32) << 16
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(color: (u8, u8, u8)) -> Self {
        Self {
            red: color.0,
            green: color.1,
            blue: color.2,
        }
    }
}
