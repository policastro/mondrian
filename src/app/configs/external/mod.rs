pub(super) mod core;
pub(super) mod general;
pub(super) mod layout;
pub(super) mod layout_optional;
pub(super) mod monitors;
pub(super) mod workspaces;

use super::modules::Modules;
use core::Core;
use general::General;
use layout::Layout;
use monitors::MonitorConfig;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use workspaces::WorkspaceConfig;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub(super) struct AppConfigExternal {
    pub general: General,
    pub layout: Layout,
    pub core: Core,
    pub modules: Modules,
    pub monitors: HashMap<String, MonitorConfig>,
    pub workspaces: HashMap<String, WorkspaceConfig>,
}
