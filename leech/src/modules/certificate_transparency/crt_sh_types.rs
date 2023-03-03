//! The types for deserializing responses from crt.sh

use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_name_value<'de, D>(de: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(de)?;
    Ok(s.split('\n').map(|s| s.to_string()).collect())
}

/// An entry returned from crt.sh
#[derive(Serialize, Deserialize, Debug)]
pub struct CertLogEntry {
    /// The ID of the issuer CA
    pub issuer_ca_id: i64,
    /// The name of the issuer
    pub issuer_name: String,
    /// The common name of the certificate
    pub common_name: String,
    /// The values of the certificate
    #[serde(deserialize_with = "deserialize_name_value")]
    pub name_value: Vec<String>,
    /// The timestamp this record was created
    pub entry_timestamp: Option<chrono::NaiveDateTime>,
    /// The start date of the certificate
    pub not_before: Option<chrono::NaiveDateTime>,
    /// The end date of the certificate
    pub not_after: Option<chrono::NaiveDateTime>,
    /// The serial number of the certificate
    pub serial_number: String,
}
