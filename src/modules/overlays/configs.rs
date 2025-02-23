use super::lib::color::Color;
use super::lib::overlay::OverlayParams;
use crate::app::configs::deserializers;
use crate::app::configs::AppConfigs;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct OverlaysModuleConfigs {
    pub enabled: bool,
    pub update_while_resizing: bool,
    #[serde(default = "OverlayParams::default_active", deserialize_with = "deserialize_active")]
    pub active: OverlayParams,
    #[serde(
        default = "OverlayParams::default_inactive",
        deserialize_with = "deserialize_inactive"
    )]
    pub inactive: OverlayParams,
}

impl Default for OverlaysModuleConfigs {
    fn default() -> Self {
        OverlaysModuleConfigs {
            enabled: true,
            update_while_resizing: true,
            active: OverlayParams::default_active(),
            inactive: OverlayParams::default_inactive(),
        }
    }
}

impl From<&AppConfigs> for OverlaysModuleConfigs {
    fn from(app_configs: &AppConfigs) -> Self {
        app_configs.modules.overlays.clone()
    }
}

impl OverlaysModuleConfigs {
    pub(crate) fn get_active(&self) -> Option<OverlayParams> {
        match &self.active.enabled {
            true => Some(self.active),
            false => None,
        }
    }

    pub(crate) fn get_inactive(&self) -> Option<OverlayParams> {
        match &self.inactive.enabled {
            true => Some(self.inactive),
            false => None,
        }
    }

    pub(crate) fn get_active_enabled(&self) -> bool {
        self.active.enabled
    }

    pub(crate) fn get_inactive_enabled(&self) -> bool {
        self.inactive.enabled
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.get_active_enabled() || self.get_inactive_enabled()
    }
}

#[derive(Deserialize)]
struct ExtOverlayParams {
    pub enabled: Option<bool>,
    pub color: Option<Color>,
    #[serde(deserialize_with = "deserializers::to_opt_u8_max::<100,_>")]
    pub thickness: Option<u8>,
    #[serde(deserialize_with = "deserializers::to_opt_u8_max::<100,_>")]
    pub border_radius: Option<u8>,
    #[serde(deserialize_with = "deserializers::to_opt_u8_max::<30,_>")]
    pub padding: Option<u8>,
}

fn deserialize_active<'de, D>(de: D) -> Result<OverlayParams, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let ext: ExtOverlayParams = serde::Deserialize::deserialize(de)?;
    Ok(merge_overlay_params(OverlayParams::default_active(), ext))
}

fn deserialize_inactive<'de, D>(de: D) -> Result<OverlayParams, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let ext: ExtOverlayParams = serde::Deserialize::deserialize(de)?;
    Ok(merge_overlay_params(OverlayParams::default_inactive(), ext))
}

fn merge_overlay_params(base: OverlayParams, ext_overlay_params: ExtOverlayParams) -> OverlayParams {
    OverlayParams {
        enabled: ext_overlay_params.enabled.unwrap_or(base.enabled),
        color: ext_overlay_params.color.unwrap_or(base.color),
        thickness: ext_overlay_params.thickness.unwrap_or(base.thickness),
        border_radius: ext_overlay_params.border_radius.unwrap_or(base.border_radius),
        padding: ext_overlay_params.padding.unwrap_or(base.padding),
    }
}
