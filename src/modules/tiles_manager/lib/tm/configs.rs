use crate::app::area_tree::layout_strategy::LayoutStrategyEnum;
use crate::app::structs::win_matcher::WinMatcher;
use crate::modules::tiles_manager::configs::CoreModuleConfigs;
use crate::modules::tiles_manager::lib::window_animation_player::WindowAnimation;

#[derive(Default)]
pub struct TilesManagerConfig {
    tiles_padding: i8,
    border_padding: i8,
    focalized_padding: i8,
    pub filter: WinMatcher,
    pub layout_strategy: LayoutStrategyEnum,
    pub animations_duration: u32,
    pub animations_framerate: u8,
    pub animation_type: Option<WindowAnimation>,
}

impl TilesManagerConfig {
    pub fn new(
        tiles_padding: u8,
        border_padding: u8,
        focalized_padding: u8,
        filter: WinMatcher,
        layout_strategy: LayoutStrategyEnum,
        animations_configs: AnimationConfigs,
    ) -> Self {
        let max_i8: u8 = i8::MAX.try_into().expect("max_i8 out of range");
        assert!(tiles_padding <= max_i8 && border_padding <= max_i8);
        assert!(animations_configs.framerate > 0);
        Self {
            tiles_padding: i8::try_from(tiles_padding).expect("tiles_padding out of range"),
            border_padding: i8::try_from(border_padding).expect("border_padding out of range"),
            focalized_padding: i8::try_from(focalized_padding).expect("focalized_padding out of range"),
            filter,
            layout_strategy,
            animations_duration: animations_configs.duration,
            animations_framerate: animations_configs.framerate,
            animation_type: animations_configs.anim_type,
        }
    }

    pub fn get_focalized_pad(&self) -> i16 {
        i16::from(self.focalized_padding)
    }

    pub fn get_border_pad(&self) -> i16 {
        i16::from(self.border_padding - self.tiles_padding)
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
        Self::new(
            configs.tiles_padding,
            configs.border_padding,
            configs.focalized_padding,
            configs.filter.clone(),
            configs.layout_strategy.clone(),
            AnimationConfigs {
                duration: configs.animations_duration,
                framerate: configs.animations_framerate,
                anim_type: animation_type,
            },
        )
    }
}

pub struct AnimationConfigs {
    duration: u32,
    framerate: u8,
    anim_type: Option<WindowAnimation>,
}
