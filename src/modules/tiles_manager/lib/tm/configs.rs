use crate::app::area_tree::layout_strategy::LayoutStrategyEnum;
use crate::app::configs::general::FloatingWinsConfigs;
use crate::app::structs::win_matcher::WinMatcher;
use crate::modules::tiles_manager::configs::CoreModuleConfigs;
use crate::modules::tiles_manager::lib::window_animation_player::WindowAnimation;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TilesManagerConfig {
    tiles_padding: i8,
    borders_padding: i8,
    focalized_padding: i8,
    pub filter: WinMatcher,
    pub layout_strategy: LayoutStrategyEnum,
    pub animations_duration: u32,
    pub animations_framerate: u8,
    pub animation_type: Option<WindowAnimation>,
    pub history_based_navigation: bool,
    pub floating_wins: FloatingWinsConfigs,
}

impl TilesManagerConfig {
    pub fn get_focalized_pad(&self) -> i16 {
        i16::from(self.focalized_padding)
    }

    pub fn get_borders_pad(&self) -> i16 {
        i16::from(self.borders_padding - self.tiles_padding)
    }

    pub fn get_tile_pady(&self) -> i16 {
        i16::from(self.tiles_padding)
    }

    pub fn get_tile_padx(&self) -> i16 {
        i16::from(self.tiles_padding)
    }

    pub fn get_tile_pad_xy(&self) -> (i16, i16) {
        (self.get_tile_padx(), self.get_tile_pady())
    }

    pub fn get_animations(&self) -> Option<WindowAnimation> {
        self.animation_type.clone()
    }

    pub fn get_animation_duration(&self) -> u32 {
        self.animations_duration
    }

    pub fn get_framerate(&self) -> u8 {
        self.animations_framerate
    }
}

impl From<&CoreModuleConfigs> for TilesManagerConfig {
    fn from(configs: &CoreModuleConfigs) -> Self {
        let animation_type = match &configs.animations_enabled {
            true => configs.animation_type.clone(),
            false => None,
        };

        Self {
            tiles_padding: configs.tiles_padding as i8,
            borders_padding: configs.border_padding as i8,
            focalized_padding: configs.focalized_padding as i8,
            filter: configs.filter.clone(),
            layout_strategy: configs.layout_strategy.clone(),
            animations_duration: configs.animations_duration,
            animations_framerate: configs.animations_framerate,
            animation_type,
            history_based_navigation: configs.history_based_navigation,
            floating_wins: configs.floating_wins.clone(),
        }
    }
}
