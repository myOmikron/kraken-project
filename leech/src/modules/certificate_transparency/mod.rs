//! Certificate transparency can be used for subdomain enumerations
//!
//! For technical information, see [here](https://certificate.transparency.dev/)

use std::collections::BTreeSet;
use std::str::FromStr;
use std::time::Duration;

use chrono::Datelike;
use chrono::Timelike;
use itertools::Itertools;
use kraken_proto::push_attack_request;
use kraken_proto::shared::CertEntry;
use kraken_proto::CertificateTransparencyRequest;
use kraken_proto::CertificateTransparencyResponse;
use log::error;
use log::info;
use prost_types::Timestamp;
use rorm::Database;
use rorm::DatabaseConfiguration;
use rorm::DatabaseDriver;
use tokio::time::sleep;
use tonic::Status;
use url::Url;

use crate::modules::certificate_transparency::crt_sh_db::get_query;
use crate::modules::certificate_transparency::crt_sh_types::CertLogEntry;
use crate::modules::certificate_transparency::error::CertificateTransparencyError;
use crate::modules::Attack;

pub struct CertificateTransparency;
#[tonic::async_trait]
impl Attack for CertificateTransparency {
    type Settings = CertificateTransparencySettings;
    type Output = Vec<CertLogEntry>;
    type Error = CertificateTransparencyError;
    async fn execute(settings: Self::Settings) -> Result<Self::Output, Self::Error> {
        query_ct_api(settings).await
    }

    type Request = CertificateTransparencyRequest;
    fn decode_settings(request: Self::Request) -> Result<Self::Settings, Status> {
        Ok(CertificateTransparencySettings {
            target: request.target,
            include_expired: request.include_expired,
            max_retries: request.max_retries,
            retry_interval: Duration::from_millis(request.retry_interval),
        })
    }

    type Response = CertificateTransparencyResponse;
    fn encode_output(output: Self::Output) -> Self::Response {
        CertificateTransparencyResponse {
            entries: output
                .into_iter()
                .map(|cert_entry| CertEntry {
                    issuer_name: cert_entry.issuer_name,
                    common_name: cert_entry.common_name,
                    value_names: cert_entry.name_value,
                    not_before: cert_entry.not_before.map(|nb| {
                        Timestamp::date_time_nanos(
                            nb.year() as i64,
                            nb.month() as u8,
                            nb.day() as u8,
                            nb.hour() as u8,
                            nb.minute() as u8,
                            nb.second() as u8,
                            nb.nanosecond(),
                        )
                        .unwrap()
                    }),
                    not_after: cert_entry.not_after.map(|na| {
                        Timestamp::date_time_nanos(
                            na.year() as i64,
                            na.month() as u8,
                            na.day() as u8,
                            na.hour() as u8,
                            na.minute() as u8,
                            na.second() as u8,
                            na.nanosecond(),
                        )
                        .unwrap()
                    }),
                    serial_number: cert_entry.serial_number,
                })
                .collect(),
        }
    }

    fn print_output(output: &Self::Output) {
        let values = BTreeSet::from_iter(output.iter().flat_map(|entry| {
            [&entry.common_name]
                .into_iter()
                .chain(entry.name_value.iter())
        }));
        for value in values {
            info!("{value}");
        }
    }

    fn wrap_for_push(response: Self::Response) -> push_attack_request::Response {
        push_attack_request::Response::CertificateTransparency(response)
    }
}

pub mod crt_sh_db;
pub mod crt_sh_types;
pub mod error;

const CT_URI: &str = "https://crt.sh";

/// Settings for a certificate transparency search request
#[derive(Debug)]
pub struct CertificateTransparencySettings {
    /// The target domain to query
    pub target: String,
    /// Also include already expired certificates
    pub include_expired: bool,
    /// The number of times the connection should be retried if it failed.
    pub max_retries: u32,
    /// The interval to wait in between the retries
    pub retry_interval: Duration,
}

/// Query the crt.sh certificate transparency api.
///
/// **Parameters**:
/// - `settings`: [CertificateTransparencySettings]
pub async fn query_ct_api(
    settings: CertificateTransparencySettings,
) -> Result<Vec<CertLogEntry>, CertificateTransparencyError> {
    let mut uri = Url::from_str(CT_URI).unwrap();

    let query = if settings.include_expired {
        format!("q={}&output=json", &settings.target)
    } else {
        format!("q={}&output=json&exclude=expired", &settings.target)
    };
    uri.set_query(Some(&query));

    info!("Requesting information about: {}", &settings.target);

    for idx in 0..=settings.max_retries {
        match reqwest::get(uri.clone()).await {
            Ok(r) => {
                if r.status() == 200 {
                    let results = r.text().await?;

                    let entries = serde_json::from_str(&results)
                        .map_err(CertificateTransparencyError::DeserializeError)?;

                    return Ok(entries);
                }
            }
            Err(err) => {
                if idx != settings.max_retries {
                    error!("Error requesting {CT_URI}: {err}, retrying in 500ms");
                    sleep(settings.retry_interval).await;
                } else {
                    error!("Error requesting {CT_URI}: {err}");
                    return Err(CertificateTransparencyError::CouldntFetchData);
                }
            }
        };
    }

    unreachable!("Loop exits")
}

/// Query the crt.sh certificate transparency api.
///
/// **Parameters**:
/// - `settings`: [CertificateTransparencySettings]
pub async fn query_ct_db(settings: CertificateTransparencySettings) {
    let mut db_config = DatabaseConfiguration::new(DatabaseDriver::Postgres {
        name: "certwatch".to_string(),
        host: "crt.sh".to_string(),
        port: 5432,
        user: "guest".to_string(),
        password: "".to_string(),
    });
    db_config.disable_logging = Some(true);
    let db = Database::connect(db_config).await.unwrap();

    let rows = match db
        .raw_sql(
            get_query(&settings.target, settings.include_expired).as_str(),
            None,
            None,
        )
        .await
    {
        Ok(rows) => rows,
        Err(err) => {
            println!("Error querying data: {err}");
            return;
        }
    };

    let entries: Vec<String> = rows
        .into_iter()
        .map(|row| row.get(2).unwrap())
        .sorted()
        .dedup()
        .collect();

    println!("{entries:#?}");
}
