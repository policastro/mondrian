use super::lib::tm::configs::TilesManagerConfig;
use crate::app::configs::AppConfig;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CoreModuleConfigs {
    pub move_cursor_on_focus: bool,
    pub history_based_navigation: bool,
    pub tm_configs: TilesManagerConfig,
}

impl From<&AppConfig> for CoreModuleConfigs {
    fn from(configs: &AppConfig) -> Self {
        CoreModuleConfigs {
            move_cursor_on_focus: configs.move_cursor_on_focus,
            history_based_navigation: configs.history_based_navigation,
            tm_configs: TilesManagerConfig::from(configs),
        }
    }
}
