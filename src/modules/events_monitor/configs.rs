use crate::app::config::app_configs::AppConfigs;
use crate::app::config::win_matcher;

pub struct EventMonitorModuleConfigs {
    pub detect_maximized_windows: bool,
    pub filter: Option<win_matcher::WinMatcher>,
}

impl Default for EventMonitorModuleConfigs {
    fn default() -> Self {
        EventMonitorModuleConfigs {
            detect_maximized_windows: true,
            filter: None,
        }
    }
}

impl From<&AppConfigs> for EventMonitorModuleConfigs {
    fn from(app_configs: &AppConfigs) -> Self {
        EventMonitorModuleConfigs {
            detect_maximized_windows: app_configs.advanced.detect_maximized_windows,
            filter: app_configs.get_filters(),
        }
    }
}
