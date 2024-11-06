use serde::{Deserialize, Serialize};

use crate::app::config::app_configs::AppConfigs;

use super::lib::overlay::OverlayParams;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct OverlaysModuleConfigs {
    pub enabled: bool,
    pub update_while_resizing: bool,
    #[serde(default = "OverlayParams::default_active")]
    pub active: OverlayParams,
    #[serde(default = "OverlayParams::default_inactive")]
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
