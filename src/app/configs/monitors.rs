use crate::app::configs::deserializers;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct ExtMonitorConfigs {
    pub layout: MonitorLayout,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct MonitorLayout {
    #[serde(deserialize_with = "deserializers::to_opt_tiling_strategy")]
    pub tiling_strategy: Option<String>,
    pub paddings: MonitorPaddingsConfigs,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct MonitorPaddingsConfigs {
    #[serde(deserialize_with = "deserializers::to_opt_u8_max::<100,_>")]
    pub tiles: Option<u8>,
    #[serde(deserialize_with = "deserializers::to_opt_u8_max::<100,_>")]
    pub borders: Option<u8>,
    #[serde(deserialize_with = "deserializers::to_opt_u8_max::<120,_>")]
    pub focalized: Option<u8>,
}
