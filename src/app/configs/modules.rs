use crate::modules::keybindings::configs::KeybindingsModuleConfigs;
use crate::modules::overlays::configs::OverlaysModuleConfigs;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Modules {
    pub keybindings: KeybindingsModuleConfigs,
    pub overlays: OverlaysModuleConfigs,
}
