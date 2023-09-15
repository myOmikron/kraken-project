//! This module holds the whois query code

use std::net::IpAddr;

use log::{debug, warn};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

pub use crate::modules::whois::errors::*;

mod errors;

const URL: &str = "https://rdap.db.ripe.net/ip/";

/// A contact information
#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WhoisEntity {
    /// Unique identifier
    pub handle: String,
    /// The roles of this entry
    pub roles: Vec<String>,
    /// VCard information
    pub vcard_array: Option<serde_json::Value>,
}

/// Whois result from ripe
#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WhoisResult {
    /// The handle of the result, mostly should be start_address - end_address
    pub handle: String,
    /// The name of the result
    pub name: String,
    /// Start starting address
    pub start_address: IpAddr,
    /// The end address
    pub end_address: IpAddr,
    /// Optional contact entities
    pub entities: Option<Vec<WhoisEntity>>,
}

/// Query whois information
pub async fn query_whois(ip_addr: IpAddr) -> Result<WhoisResult, WhoisError> {
    let client = Client::new();

    let res = client.get(format!("{URL}{ip_addr}")).send().await?;
    let status = res.status();
    let text = res.text().await.unwrap();

    if status != StatusCode::OK {
        warn!("Status code {status} found. See debug logs for more information.");
        debug!("{text}");
        return Err(WhoisError::InvalidResponse);
    }

    serde_json::from_str::<WhoisResult>(&text).map_err(WhoisError::DeserializeError)
}
