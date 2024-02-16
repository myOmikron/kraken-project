use std::fmt;

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct ProbeFile {
    pub service: String,
    pub prevalence: Prevalence,
    pub probes: Vec<Probe>,
}

#[derive(Debug, Deserialize)]
pub struct Probe {
    /// Is this probe applicable to a tcp connection?
    ///
    /// This field is an optional yaml 1.1 boolean which defaults to `false`
    #[serde(default, deserialize_with = "deserialize_bool_11")]
    pub tcp: bool,

    /// Is this probe applicable to an udp connection?
    ///
    /// This field is an optional yaml 1.1 boolean which defaults to `false`
    #[serde(default, deserialize_with = "deserialize_bool_11")]
    pub udp: bool,

    /// Is this probe applicable to a tls over tcp connection?
    ///
    /// This field is an optional yaml 1.1 boolean which defaults to `false`
    #[serde(default, deserialize_with = "deserialize_bool_11")]
    pub tls: bool,

    /// An optional protocol to request during ALPN while establishing a tls connection
    pub alpn: Option<String>,

    pub payload_str: Option<String>,
    #[serde(deserialize_with = "deserialize_b64", default)]
    pub payload_b64: Option<Vec<u8>>,
    #[serde(deserialize_with = "deserialize_hex", default)]
    pub payload_hex: Option<Vec<u8>>,

    pub regex: String,
    pub sub_regex: Option<Vec<String>>,
}

/// The prevalence for a [`ProbeFile`]'s probes
#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Prevalence {
    Often,
    Average,
    Obscure,
}

/// Deserializes a binary blob from a string containing base64
pub fn deserialize_b64<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    BASE64_STANDARD
        .decode(&string)
        .map(Some)
        .map_err(|_| Error::custom(format_args!("invalid base64: '{string}'")))
}

/// Deserializes a binary blob from a string containing hex code
pub fn deserialize_hex<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    hex::decode(&string)
        .map(Some)
        .map_err(|_| Error::custom(format_args!("invalid hex: '{string}'")))
}

/// Deserializes a boolean using yaml 1.1 syntax instead of the restricted yaml 1.2 syntax
///
/// I.e. `yes`, `no` and so are valid booleans
pub fn deserialize_bool_11<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    struct Bool11Visitor;
    impl<'de> Visitor<'de> for Bool11Visitor {
        type Value = bool;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "a yaml 1.1 boolean")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match v {
                "true" | "True" | "TRUE" | "y" | "Y" | "yes" | "Yes" | "YES" | "on" | "On"
                | "ON" => Ok(true),
                "false" | "False" | "FALSE" | "n" | "N" | "no" | "No" | "NO" | "off" | "Off"
                | "OFF" => Ok(false),
                _ => Err(E::invalid_type(Unexpected::Str(v), &self)),
            }
        }
    }
    deserializer.deserialize_str(Bool11Visitor)
}
