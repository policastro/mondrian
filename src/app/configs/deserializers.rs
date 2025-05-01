use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;

use crate::app::structs::paddings::Paddings;

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

pub fn to_opt_paddings_max<'de, const MAX: u8, D>(deserializer: D) -> Result<Option<Paddings>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: toml::Value = toml::Value::deserialize(deserializer)?;
    get_paddings(&value, MAX).map(Some).map_err(D::Error::custom)
}

pub fn to_paddings_max<'de, const MAX: u8, D>(deserializer: D) -> Result<Paddings, D::Error>
where
    D: Deserializer<'de>,
{
    let value: toml::Value = toml::Value::deserialize(deserializer)?;
    get_paddings(&value, MAX).map_err(D::Error::custom)
}

fn get_paddings(value: &toml::Value, max: u8) -> Result<Paddings, String> {
    match value {
        toml::Value::Integer(n) => {
            if *n < 0 || *n > max as i64 {
                return Err(format!("value must be between 0 and {max} (inclusive)"));
            }
            Ok(Paddings::full(*n as u8))
        }

        toml::Value::Array(arr) => {
            let values: Vec<u8> = arr.iter().filter_map(|x| x.as_integer().map(|x| x as u8)).collect();
            for v in arr.iter() {
                let v = v.as_integer().ok_or("Invalid value".to_string())?;
                let v = u8::try_from(v).map_err(|e| e.to_string())?;
                if v > max {
                    return Err(format!("values must be between 0 and {max} (inclusive)"));
                }
            }

            match values.len() {
                2 => Ok(Paddings::new(values[0], values[1], values[0], values[1])),
                4 => Ok(Paddings::new(values[0], values[1], values[2], values[3])),
                _ => Err(
                    "The array must have  2 (vertical/horizontal) or 4 (top/right/bottom/left) elements".to_string(),
                ),
            }
        }
        _ => Err("Invalid value".to_string()),
    }
}

pub fn deserialize_size_ratio<'de, D>(deserializer: D) -> Result<(f32, f32), D::Error>
where
    D: serde::Deserializer<'de>,
{
    let (w, h) = serde::Deserialize::deserialize(deserializer)?;
    if w < 0.1 || h < 0.1 || w > 1.0 || h > 1.0 {
        return Err(serde::de::Error::custom("Width and height must be between 0.1 and 1.0"));
    }
    Ok((w, h))
}

pub fn deserialize_size_fixed<'de, D>(deserializer: D) -> Result<(u16, u16), D::Error>
where
    D: serde::Deserializer<'de>,
{
    let (w, h) = serde::Deserialize::deserialize(deserializer)?;
    if w < 100 || h < 100 || w > 10000 || h > 10000 {
        return Err(serde::de::Error::custom(
            "Width and height must be between 100 and 10000",
        ));
    }
    Ok((w, h))
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
