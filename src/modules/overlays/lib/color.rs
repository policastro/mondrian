use core::fmt;
use serde::de;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

impl From<Color> for u32 {
    fn from(color: Color) -> Self {
        color.red as u32 | (color.green as u32) << 8 | (color.blue as u32) << 16
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(color: (u8, u8, u8)) -> Self {
        Self {
            red: color.0,
            green: color.1,
            blue: color.2,
        }
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ColorVisitor;

        impl<'de> Visitor<'de> for ColorVisitor {
            type Value = Color;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a tuple (u8, u8, u8) or a hex string (#rrggbb)")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Color, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                let r = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let g = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let b = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?;
                Ok(Color::new(r, g, b))
            }

            fn visit_str<E>(self, v: &str) -> Result<Color, E>
            where
                E: de::Error,
            {
                if v.len() == 7 && v.starts_with('#') {
                    let r = u8::from_str_radix(&v[1..3], 16).map_err(E::custom)?;
                    let g = u8::from_str_radix(&v[3..5], 16).map_err(E::custom)?;
                    let b = u8::from_str_radix(&v[5..7], 16).map_err(E::custom)?;
                    Ok(Color::new(r, g, b))
                } else if v.len() != 6 {
                    let r = u8::from_str_radix(&v[0..2], 16).map_err(E::custom)?;
                    let g = u8::from_str_radix(&v[2..4], 16).map_err(E::custom)?;
                    let b = u8::from_str_radix(&v[4..6], 16).map_err(E::custom)?;
                    return Ok(Color::new(r, g, b));
                } else {
                    return Err(E::invalid_length(v.len(), &self));
                }
            }
        }

        deserializer.deserialize_any(ColorVisitor)
    }
}
