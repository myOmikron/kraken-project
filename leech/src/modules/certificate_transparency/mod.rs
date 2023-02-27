//! Certificate transparency can be used for subdomain enumerations
//!
//! For technical information, see [here](https://certificate.transparency.dev/)

use std::str::FromStr;
use std::time::Duration;

use itertools::Itertools;
use rorm::{Database, DatabaseConfiguration, DatabaseDriver};
use tokio::time::sleep;
use url::Url;

use crate::modules::certificate_transparency::crt_sh_db::get_query;
use crate::modules::certificate_transparency::crt_sh_types::Entry;

pub mod crt_sh_db;
pub mod crt_sh_types;

const CT_URI: &str = "https://crt.sh";

/// Settings for a certificate transparency search request
pub struct CertificateTransparencySettings {
    /// The target domain to query
    pub target: String,
    /// Also include already expired certificates
    pub include_expired: bool,
}

/// Query the crt.sh certificate transparency api.
///
/// **Parameters**:
/// - `settings`: [CertificateTransparencySettings]
pub async fn query_ct_api(settings: CertificateTransparencySettings) {
    let mut uri = Url::from_str(CT_URI).unwrap();

    let query = if settings.include_expired {
        format!("q={}&output=json", &settings.target)
    } else {
        format!("q={}&output=json&exclude=expired", &settings.target)
    };
    uri.set_query(Some(&query));

    println!("Requesting information about: {}", &settings.target);

    let mut res = None;
    for _ in 0..3 {
        match reqwest::get(uri.clone()).await {
            Ok(r) => {
                if r.status() == 200 {
                    res = Some(r);
                    break;
                }
            }
            Err(err) => {
                println!("Error requesting {CT_URI}: {err}");
                sleep(Duration::from_millis(500)).await;
            }
        };
    }

    let Some(res) = res else {
        return;
    };

    let results = match res.text().await {
        Ok(res) => res,
        Err(err) => {
            println!("{err}");
            return;
        }
    };

    let parsed: Vec<Entry> = match serde_json::from_str(&results) {
        Err(err) => {
            println!("{err}");
            println!("Received: {results}");
            return;
        }
        Ok(v) => v,
    };

    let mut v = vec![];
    for x in parsed {
        v.push(x.common_name);
        v.extend(
            x.name_value
                .split('\n')
                .map(|s| s.to_owned())
                .collect::<Vec<_>>(),
        )
    }
    v.sort();
    v.dedup();

    for x in v {
        println!("{x}");
    }
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
