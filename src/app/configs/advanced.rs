use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct Advanced {
    pub detect_maximized_windows: bool,
}

impl Default for Advanced {
    fn default() -> Self {
        Advanced {
            detect_maximized_windows: true,
        }
    }
}
