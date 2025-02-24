use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;

pub fn to_u8_max<'de, const MAX: u8, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    to_u8_minmax::<'de, { u8::MIN }, MAX, D>(deserializer)
}

pub fn to_u8_minmax<'de, const MIN: u8, const MAX: u8, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let v: u8 = u8::deserialize(deserializer)?;
    match v >= MIN && v <= MAX {
        true => Ok(v),
        false => Err(D::Error::custom(format!(
            "value must be between {MIN} and {MAX} (inclusive)"
        ))),
    }
}

pub fn to_u32_minmax<'de, const MIN: u32, const MAX: u32, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let v: u32 = u32::deserialize(deserializer)?;
    match v >= MIN && v <= MAX {
        true => Ok(v),
        false => Err(D::Error::custom(format!(
            "value must be between {MIN} and {MAX} (inclusive)"
        ))),
    }
}
