use super::lib::color::Color;
use super::lib::overlay::OverlayParams;
use crate::app::configs::deserializers;
use crate::app::configs::AppConfig;
use crate::app::mondrian_message::WindowTileState;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct OverlaysModuleConfigs {
    pub enabled: bool,
    pub update_while_dragging: bool,
    pub update_while_animating: bool,

    #[serde(deserialize_with = "deserializers::to_u8_max::<100,_>")]
    pub thickness: u8,

    #[serde(deserialize_with = "deserializers::to_u8_max::<100,_>")]
    pub border_radius: u8,

    #[serde(deserialize_with = "deserializers::to_u8_max::<30,_>")]
    pub padding: u8,

    #[serde(
        default = "ExtOverlayParams::default_active",
        deserialize_with = "deserialize_active"
    )]
    active: ExtOverlayParams,

    #[serde(
        default = "ExtOverlayParams::default_inactive",
        deserialize_with = "deserialize_inactive"
    )]
    inactive: ExtOverlayParams,

    #[serde(
        default = "ExtOverlayParams::default_focalized",
        deserialize_with = "deserialize_focalized"
    )]
    focalized: ExtOverlayParams,

    #[serde(
        default = "ExtOverlayParams::default_floating",
        deserialize_with = "deserialize_floating"
    )]
    floating: ExtOverlayParams,

    #[serde(
        default = "ExtOverlayParams::default_half_focalized",
        deserialize_with = "deserialize_half_focalized"
    )]
    half_focalized: ExtOverlayParams,
}

impl Default for OverlaysModuleConfigs {
    fn default() -> Self {
        OverlaysModuleConfigs {
            enabled: true,
            update_while_dragging: true,
            update_while_animating: true,
            thickness: 4,
            border_radius: 15,
            padding: 0,
            active: ExtOverlayParams::default_active(),
            inactive: ExtOverlayParams::default_inactive(),
            focalized: ExtOverlayParams::default_focalized(),
            floating: ExtOverlayParams::default_floating(),
            half_focalized: ExtOverlayParams::default_half_focalized(),
        }
    }
}

impl From<&AppConfig> for OverlaysModuleConfigs {
    fn from(app_configs: &AppConfig) -> Self {
        app_configs.modules.overlays.clone()
    }
}

impl OverlaysModuleConfigs {
    pub(crate) fn get_hidden(&self) -> OverlayParams {
        OverlayParams::new(
            true,
            Color::new(0, 0, 0, 0),
            self.thickness,
            self.border_radius,
            self.padding,
        )
    }

    pub(crate) fn get_active(&self) -> Option<OverlayParams> {
        let overlay_params = self.create_overlay_params(&self.active);
        match &self.active.enabled {
            true => Some(overlay_params),
            false => None,
        }
    }

    pub(crate) fn get_by_tile_state(&self, tile_state: &WindowTileState) -> Option<OverlayParams> {
        match tile_state {
            WindowTileState::Focalized => self.get_focalized(),
            WindowTileState::Floating => self.get_floating(),
            WindowTileState::HalfFocalized => self.get_half_focalized(),
            WindowTileState::Normal | WindowTileState::Maximized => None,
        }
    }

    pub(crate) fn get_inactive(&self) -> Option<OverlayParams> {
        let overlay_params = self.create_overlay_params(&self.inactive);
        match &self.inactive.enabled {
            true => Some(overlay_params),
            false => None,
        }
    }

    pub(crate) fn get_focalized(&self) -> Option<OverlayParams> {
        let overlay_params = self.create_overlay_params(&self.focalized);
        match &self.focalized.enabled {
            true => Some(overlay_params),
            false => None,
        }
    }

    pub(crate) fn get_half_focalized(&self) -> Option<OverlayParams> {
        let overlay_params = self.create_overlay_params(&self.half_focalized);
        match &self.half_focalized.enabled {
            true => Some(overlay_params),
            false => None,
        }
    }

    pub(crate) fn get_floating(&self) -> Option<OverlayParams> {
        let overlay_params = self.create_overlay_params(&self.floating);
        match &self.floating.enabled {
            true => Some(overlay_params),
            false => None,
        }
    }

    fn create_overlay_params(&self, ext_overlay_params: &ExtOverlayParams) -> OverlayParams {
        OverlayParams::new(
            ext_overlay_params.enabled,
            ext_overlay_params.color,
            self.thickness,
            self.border_radius,
            self.padding,
        )
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.active.enabled
            || self.inactive.enabled
            || self.focalized.enabled
            || self.floating.enabled
            || self.half_focalized.enabled
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
struct ExtOptOverlayParams {
    pub enabled: Option<bool>,
    pub color: Option<Color>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct ExtOverlayParams {
    pub enabled: bool,
    pub color: Color,
}

impl ExtOverlayParams {
    pub(crate) fn new(enabled: bool, color: Color) -> ExtOverlayParams {
        ExtOverlayParams { enabled, color }
    }

    pub(crate) fn default_active() -> ExtOverlayParams {
        ExtOverlayParams::new(true, Color::solid(155, 209, 229))
    }

    pub(crate) fn default_inactive() -> ExtOverlayParams {
        ExtOverlayParams::new(true, Color::solid(156, 156, 156))
    }

    pub(crate) fn default_focalized() -> ExtOverlayParams {
        ExtOverlayParams::new(true, Color::solid(234, 153, 153))
    }

    pub(crate) fn default_floating() -> ExtOverlayParams {
        ExtOverlayParams::new(true, Color::solid(220, 198, 224))
    }

    fn default_half_focalized() -> ExtOverlayParams {
        ExtOverlayParams::new(true, Color::solid(220, 242, 215))
    }
}

fn deserialize_active<'de, D>(de: D) -> Result<ExtOverlayParams, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_overlay_params(de, ExtOverlayParams::default_active())
}

fn deserialize_inactive<'de, D>(de: D) -> Result<ExtOverlayParams, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_overlay_params(de, ExtOverlayParams::default_inactive())
}

fn deserialize_focalized<'de, D>(de: D) -> Result<ExtOverlayParams, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_overlay_params(de, ExtOverlayParams::default_focalized())
}

fn deserialize_floating<'de, D>(de: D) -> Result<ExtOverlayParams, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_overlay_params(de, ExtOverlayParams::default_floating())
}

fn deserialize_half_focalized<'de, D>(de: D) -> Result<ExtOverlayParams, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_overlay_params(de, ExtOverlayParams::default_half_focalized())
}

fn deserialize_overlay_params<'de, D>(de: D, base: ExtOverlayParams) -> Result<ExtOverlayParams, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let ext: ExtOptOverlayParams = serde::Deserialize::deserialize(de)?;
    Ok(ExtOverlayParams {
        enabled: ext.enabled.unwrap_or(base.enabled),
        color: ext.color.unwrap_or(base.color),
    })
}
