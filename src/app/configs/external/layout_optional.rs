use crate::app::configs::deserializers;
use crate::app::structs::paddings::Paddings;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct LayoutOptional {
    #[serde(deserialize_with = "deserializers::to_opt_tiling_strategy")]
    pub tiling_strategy: Option<String>,
    pub paddings: PaddingsOptionalConfigs,
    pub half_focalized_paddings: PaddingsOptionalConfigs,
    #[serde(deserialize_with = "deserializers::to_opt_paddings_max::<140,_>")]
    pub focalized_padding: Option<Paddings>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct PaddingsOptionalConfigs {
    #[serde(deserialize_with = "deserializers::to_opt_u8_max::<140,_>")]
    pub tiles: Option<u8>,
    #[serde(deserialize_with = "deserializers::to_opt_paddings_max::<140,_>")]
    pub borders: Option<Paddings>,
}
