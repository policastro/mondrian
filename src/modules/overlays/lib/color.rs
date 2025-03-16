use core::fmt;
use regex::Regex;
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
    alpha: u8,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub fn solid(red: u8, green: u8, blue: u8) -> Self {
        Color::new(red, green, blue, 255)
    }

    pub fn get_argb(&self) -> u32 {
        ((self.alpha as u32) << 24) | ((self.red as u32) << 16) | ((self.green as u32) << 8) | (self.blue as u32)
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
                formatter.write_str("a tuple (R, G, B)/(R, G, B, A) or a hex string (#rrggbb/#rrggbbaa)")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Color, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                let r = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let g = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let b = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let a = seq
                    .next_element()
                    .unwrap_or(Some(255u8))
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                Ok(Color::new(r, g, b, a))
            }

            fn visit_str<E>(self, v: &str) -> Result<Color, E>
            where
                E: de::Error,
            {
                let hex_color_regex = Regex::new(r"^#?([A-Fa-f0-9]{6}|[A-Fa-f0-9]{8})$").unwrap();
                if !hex_color_regex.is_match(v) {
                    return Err(E::invalid_value(de::Unexpected::Str(v), &self));
                }

                let base_index = if v.starts_with('#') { 1 } else { 0 };
                let r = u8::from_str_radix(&v[base_index..base_index + 2], 16);
                let g = u8::from_str_radix(&v[base_index + 2..base_index + 4], 16);
                let b = u8::from_str_radix(&v[base_index + 4..base_index + 6], 16);
                let a = u8::from_str_radix(v.get(base_index + 6..base_index + 8).unwrap_or("FF"), 16);
                let (r, g, b, a) = (
                    r.map_err(E::custom)?,
                    g.map_err(E::custom)?,
                    b.map_err(E::custom)?,
                    a.map_err(E::custom)?,
                );

                Ok(Color::new(r, g, b, a))
            }
        }

        deserializer.deserialize_any(ColorVisitor)
    }
}
