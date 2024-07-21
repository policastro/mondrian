use crate::app::config::app_configs::AppConfigs;

use super::lib::overlay::OverlayParams;

#[derive(Clone, Debug)]
pub struct OverlaysModuleConfigs {
    active: OverlayParams,
    inactive: OverlayParams,
    active_enabled: bool,
    inactive_enabled: bool,
    follow_movements: bool,
}

impl Default for OverlaysModuleConfigs {
    fn default() -> Self {
        OverlaysModuleConfigs {
            active: OverlayParams::default(),
            inactive: OverlayParams::default(),
            active_enabled: true,
            inactive_enabled: true,
            follow_movements: true,
        }
    }
}

impl From<&AppConfigs> for OverlaysModuleConfigs {
    fn from(app_configs: &AppConfigs) -> Self {
        OverlaysModuleConfigs {
            active: app_configs.active_overlay,
            inactive: app_configs.inactive_overlay,
            active_enabled: app_configs.active_overlay_enabled,
            inactive_enabled: app_configs.inactive_overlay_enabled,
            follow_movements: app_configs.overlay_follow_movements,
        }
    }
}

impl OverlaysModuleConfigs {
    pub(crate) fn get_active(&self) -> Option<OverlayParams> {
        if self.active_enabled {
            Some(self.active)
        } else {
            None
        }
    }

    pub(crate) fn get_inactive(&self) -> Option<OverlayParams> {
        if self.inactive_enabled {
            Some(self.inactive)
        } else {
            None
        }
    }

    pub(crate) fn get_active_enabled(&self) -> bool {
        self.active_enabled
    }

    pub(crate) fn get_inactive_enabled(&self) -> bool {
        self.inactive_enabled
    }

    pub(crate) fn get_follow_movements(&self) -> bool {
        self.follow_movements && self.active_enabled
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.get_active_enabled() || self.get_inactive_enabled()
    }
}
