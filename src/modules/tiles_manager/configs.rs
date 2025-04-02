use super::lib::tm::configs::TilesManagerConfig;
use crate::app::configs::AppConfigs;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CoreModuleConfigs {
    pub move_cursor_on_focus: bool,
    pub history_based_navigation: bool,
    pub tm_configs: TilesManagerConfig,
}

impl From<&AppConfigs> for CoreModuleConfigs {
    fn from(configs: &AppConfigs) -> Self {
        CoreModuleConfigs {
            move_cursor_on_focus: configs.general.move_cursor_on_focus,
            history_based_navigation: configs.general.history_based_navigation,
            tm_configs: TilesManagerConfig::from(configs),
        }
    }
}
