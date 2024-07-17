use crate::app::config::app_configs::AppConfigs;

use super::lib::color::Color;

pub struct OverlayConfig {
    pub thickness: u8,
    pub color: Color,
    pub padding: u8,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        OverlayConfig {
            thickness: 1,
            color: Color::default(),
            padding: 0,
        }
    }
}

impl From<&AppConfigs> for OverlayConfig {
    fn from(app_configs: &AppConfigs) -> Self {
        OverlayConfig {
            thickness: app_configs.overlay_thickness,
            color: app_configs.overlay_color,
            padding: app_configs.overlay_padding,
        }
    }
}
