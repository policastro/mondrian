use crate::modules::core::config::CoreModuleConfigs;

#[derive(Default)]
pub struct TilesManagerConfig {
    tiles_padding: i8,
    border_padding: i8,
    insert_in_monitor: bool,
}

impl TilesManagerConfig {
    pub fn new(tiles_padding: u8, border_padding: u8, insert_in_monitor: bool) -> Self {
        let max_i8: u8 = i8::MAX.try_into().expect("max_i8 out of range");
        assert!(tiles_padding <= max_i8 && border_padding <= max_i8);
        // TODO -4 is a magic number
        Self {
            tiles_padding: i8::try_from(tiles_padding).expect("tiles_padding out of range"),
            border_padding: i8::try_from(border_padding).expect("border_padding out of range"),
            insert_in_monitor,
        }
    }

    pub fn get_border_pad(&self) -> i16 {
        i16::from(self.border_padding - self.tiles_padding) - 2 // TODO Note: 2 is a magic number
    }

    pub fn get_tile_pady(&self) -> i16 {
        i16::from(self.tiles_padding) - 5 // TODO Note: 5 is a magic number
    }

    pub fn get_tile_padx(&self) -> i16 {
        i16::from(self.tiles_padding) - 8 // TODO Note: 8 is a magic number
    }

    pub fn get_tile_pad_xy(&self) -> (i16, i16) {
        (self.get_tile_padx(), self.get_tile_pady())
    }

    pub fn is_insert_in_monitor(&self, inverted: bool) -> bool {
        match inverted {
            true => !self.insert_in_monitor,
            false => self.insert_in_monitor,
        }
    }
}

impl From<&CoreModuleConfigs> for TilesManagerConfig {
    fn from(configs: &CoreModuleConfigs) -> Self {
        Self::new(configs.tiles_padding, configs.border_padding, configs.insert_in_monitor)
    }
}
