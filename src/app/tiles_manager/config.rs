pub struct TilesManagerConfig {
    tiles_padding: i8,
    border_padding: i8,
}

impl TilesManagerConfig {
    pub fn new(tiles_padding: u8, border_padding: u8) -> Self {
        let max_i8: u8 = i8::MAX.try_into().expect("max_i8 out of range");
        assert!(tiles_padding <= max_i8 && border_padding <= max_i8);
        // TODO -4 is a magic number
        Self {
            tiles_padding: i8::try_from(tiles_padding).expect("tiles_padding out of range"),
            border_padding: i8::try_from(border_padding).expect("border_padding out of range"),
        }
    }

    pub fn get_border_pad(&self) -> i8 {
        self.border_padding - self.tiles_padding - 2 // TODO Note: 2 is a magic number
    }

    pub fn get_tile_pady(&self) -> i8 {
        self.tiles_padding - 5 // TODO Note: 5 is a magic number
    }

    pub fn get_tile_padx(&self) -> i8 {
        self.tiles_padding - 8 // TODO Note: 8 is a magic number
    }

    pub fn get_tile_pad_xy(&self) -> (i8, i8) {
        (self.get_tile_padx(), self.get_tile_pady())
    }
}

impl Default for TilesManagerConfig {
    fn default() -> Self {
        Self {
            tiles_padding: 0,
            border_padding: 0,
        }
    }
}
