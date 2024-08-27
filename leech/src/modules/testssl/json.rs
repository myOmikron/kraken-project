//! Struct defining `testssl.sh`'s `--json` output

use serde::Deserialize;
use serde::Serialize;

/// The entire output file
#[allow(dead_code)]
pub type File = Vec<Finding>;

/// This struct's fields are found in [`fileout_json_finding`].
///
/// [`fileout_json_finding`]: https://github.com/drwetter/testssl.sh/blob/68dec54cc5aedf856a83425cb4cd475a3766fad5/testssl.sh#L844
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Finding {
    pub id: String,
    pub severity: Severity,
    pub finding: String,

    pub ip: String,
    pub port: String,

    pub cve: Option<String>,
    pub cwe: Option<String>,
    pub hint: Option<String>,
}

/// Different levels has been taken from [`show_finding`]
///
/// [`show_finding`]: https://github.com/drwetter/testssl.sh/blob/68dec54cc5aedf856a83425cb4cd475a3766fad5/testssl.sh#L473
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Debug,
    Info,
    Warn,
    Fatal,

    Ok,
    Low,
    Medium,
    High,
    Critical,
}
