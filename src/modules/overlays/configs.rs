use serde::Deserialize;

use crate::app::config::app_configs::AppConfigs;

use super::lib::overlay::OverlayParams;

#[derive(Clone, Debug, Deserialize)]
pub struct OverlaysModuleConfigs {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
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
        if self.active.enabled {
            Some(self.active)
        } else {
            None
        }
    }

    pub(crate) fn get_inactive(&self) -> Option<OverlayParams> {
        if self.inactive.enabled {
            Some(self.inactive)
        } else {
            None
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
