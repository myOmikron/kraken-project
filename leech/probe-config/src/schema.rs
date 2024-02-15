use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct ProbeFile {
    pub service: String,
    pub prevalence: Prevalence,
    pub probes: Vec<Probe>,
}

#[derive(Debug, Deserialize)]
pub struct Probe {
    pub protocol: Protocol,
    pub alpn: Option<String>,
    pub payload_str: Option<String>,
    #[serde(deserialize_with = "deserialize_b64", default)]
    pub payload_b64: Option<Vec<u8>>,
    #[serde(deserialize_with = "deserialize_hex", default)]
    pub payload_hex: Option<Vec<u8>>,
    pub regex: String,
    pub sub_regex: Option<Vec<String>>,
}

/// The protocol used by a [`Probe`]
#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Protocol {
    Tcp,
    Udp,
    Tls,
}

/// The prevalence for a [`ProbeFile`]'s probes
#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Prevalence {
    Often,
    Average,
    Obscure,
}

pub fn deserialize_b64<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    BASE64_STANDARD
        .decode(&string)
        .map(Some)
        .map_err(|_| serde::de::Error::custom(format_args!("invalid base64: '{string}'")))
}

pub fn deserialize_hex<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    hex::decode(&string)
        .map(Some)
        .map_err(|_| serde::de::Error::custom(format_args!("invalid hex: '{string}'")))
}
