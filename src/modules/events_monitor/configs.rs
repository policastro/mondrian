use crate::app::{configs::AppConfig, structs::win_matcher};

pub struct EventMonitorModuleConfigs {
    pub default_insert_in_monitor: bool,
    pub default_free_move_in_monitor: bool,
    pub detect_maximized_windows: bool,
    pub filter: Option<win_matcher::WinMatcher>,
    pub delayed_filter: Vec<(win_matcher::WinMatcher, u32)>,
}

impl Default for EventMonitorModuleConfigs {
    fn default() -> Self {
        EventMonitorModuleConfigs {
            default_insert_in_monitor: false,
            default_free_move_in_monitor: false,
            detect_maximized_windows: true,
            filter: None,
            delayed_filter: vec![],
        }
    }
}

impl From<&AppConfig> for EventMonitorModuleConfigs {
    fn from(app_configs: &AppConfig) -> Self {
        EventMonitorModuleConfigs {
            default_insert_in_monitor: app_configs.insert_in_monitor,
            default_free_move_in_monitor: app_configs.free_move_in_monitor,
            detect_maximized_windows: app_configs.detect_maximized_windows,
            filter: Some(app_configs.ignore_filter.clone()),
            delayed_filter: app_configs.delayed_filter.clone(),
        }
    }
}
