use crate::app::{configs::AppConfigs, structs::win_matcher};

pub struct EventMonitorModuleConfigs {
    pub default_insert_in_monitor: bool,
    pub default_free_move_in_monitor: bool,
    pub detect_maximized_windows: bool,
    pub filter: Option<win_matcher::WinMatcher>,
}

impl Default for EventMonitorModuleConfigs {
    fn default() -> Self {
        EventMonitorModuleConfigs {
            default_insert_in_monitor: false,
            default_free_move_in_monitor: false,
            detect_maximized_windows: true,
            filter: None,
        }
    }
}

impl From<&AppConfigs> for EventMonitorModuleConfigs {
    fn from(app_configs: &AppConfigs) -> Self {
        EventMonitorModuleConfigs {
            default_insert_in_monitor: app_configs.layout.insert_in_monitor,
            default_free_move_in_monitor: app_configs.layout.free_move_in_monitor,
            detect_maximized_windows: app_configs.advanced.detect_maximized_windows,
            filter: app_configs.get_filters(),
        }
    }
}
