pub struct TilesManagerConfig {
    pub(super) tiles_padding: i8,
    pub(super) border_padding: i8,
}

impl TilesManagerConfig {
    pub fn new(tiles_padding: u8, border_padding: u8) -> Self {
        let max_i8: u8 = i8::MAX.try_into().expect("max_i8 out of range");
        assert!(tiles_padding <= max_i8 && border_padding <= max_i8);
        Self {
            tiles_padding: i8::try_from(tiles_padding).expect("tiles_padding out of range"),
            border_padding: i8::try_from(border_padding).expect("border_padding out of range"),
        }
    }
}

impl Default for TilesManagerConfig {
    fn default() -> Self {
        Self {
            tiles_padding: 00,
            border_padding: 0,
        }
    }
}
