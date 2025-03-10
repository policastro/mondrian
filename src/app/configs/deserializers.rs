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

pub fn to_opt_u8_max<'de, const MAX: u8, D>(deserializer: D) -> Result<Option<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    to_opt_u8_minmax::<'de, { u8::MIN }, MAX, D>(deserializer)
}

pub fn to_opt_u8_minmax<'de, const MIN: u8, const MAX: u8, D>(deserializer: D) -> Result<Option<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: u8 = match Option::deserialize(deserializer)? {
        Some(v) => v,
        None => return Ok(None),
    };

    match v >= MIN && v <= MAX {
        true => Ok(Some(v)),
        false => Err(D::Error::custom(format!(
            "value must be between {MIN} and {MAX} (inclusive)"
        ))),
    }
}

pub fn to_tiling_strategy<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    get_tiling_strategy(&s).map_err(D::Error::custom)
}

pub fn to_opt_tiling_strategy<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = match Option::<String>::deserialize(deserializer)? {
        Some(s) => s,
        None => return Ok(None),
    };
    get_tiling_strategy(&s).map(Some).map_err(D::Error::custom)
}

fn get_tiling_strategy(s: &str) -> Result<String, String> {
    let valid = ["golden_ratio", "horizontal", "vertical", "twostep", "squared"];
    match valid.contains(&s.to_lowercase().as_str()) {
        true => Ok(s.to_lowercase()),
        false => Err(format!(
            "Invalid tiling strategy: {}, valid options are {}",
            s,
            valid.join(", ")
        )),
    }
}
