//! Certificate transparency can be used for subdomain enumerations
//!
//! For technical information, see [here](https://certificate.transparency.dev/)

use std::str::FromStr;
use std::time::Duration;

use itertools::Itertools;
use log::{error, info};
use rorm::{Database, DatabaseConfiguration, DatabaseDriver};
use tokio::time::sleep;
use url::Url;

use crate::modules::certificate_transparency::crt_sh_db::get_query;
use crate::modules::certificate_transparency::crt_sh_types::CertLogEntry;
use crate::modules::certificate_transparency::error::CertificateTransparencyError;

pub mod crt_sh_db;
pub mod crt_sh_types;
pub mod error;

const CT_URI: &str = "https://crt.sh";

/// Settings for a certificate transparency search request
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
