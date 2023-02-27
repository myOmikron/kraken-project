//! The types for deserializing responses from crt.sh

use serde::{Deserialize, Serialize};

/// An entry returned from crt.sh
#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    /// The ID of the issuer CA
    pub issuer_ca_id: i64,
    /// The name of the issuer
    pub issuer_name: String,
    /// The common name of the certificate
    pub common_name: String,
    /// The values of the certificate
    pub name_value: String,
    /// Internal id of crt.sh
    pub id: i64,
    /// The timestamp this record was created
    pub entry_timestamp: Option<chrono::NaiveDateTime>,
    /// The start date of the certificate
    pub not_before: Option<chrono::NaiveDateTime>,
    /// The end date of the certificate
    pub not_after: Option<chrono::NaiveDateTime>,
    /// The serial number of the certificate
    pub serial_number: String,
}
