use std::path::Path;

use serde::de::{Deserialize, Deserializer, Visitor};

use crate::parse::{Annote, AnnoteString};
use crate::{Error, Program};

impl Program {
    pub fn from_json(file: &Path) -> Result<Self, Error> {
        let json = std::fs::read_to_string(file).map_err(Error::Io)?;
        serde_json::from_str(&json).map_err(Error::Serde)
    }
}

impl<'de> Deserialize<'de> for Annote {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AnnoteVisitor;

        impl<'de> Visitor<'de> for AnnoteVisitor {
            type Value = Annote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string containing an annotation")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let a_s = AnnoteString::new(s);
                let a = crate::parse::parse(&a_s);
                match a {
                    Ok((_, annote)) => Ok(annote),
                    Err(e) => Err(serde::de::Error::custom(e)),
                }
            }
        }
        deserializer.deserialize_string(AnnoteVisitor)
    }
}
