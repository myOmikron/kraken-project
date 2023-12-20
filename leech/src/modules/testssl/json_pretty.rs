//! Struct defining `testssl.sh`'s `--json-pretty` output

use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};

/// The entire output file
///
/// This struct's fields are found in [`fileout_pretty_json_banner`] and [`fileout_json_footer`].
///
/// [`fileout_pretty_json_banner`]: https://github.com/drwetter/testssl.sh/blob/68dec54cc5aedf856a83425cb4cd475a3766fad5/testssl.sh#L896
/// [`fileout_json_footer`]: https://github.com/drwetter/testssl.sh/blob/68dec54cc5aedf856a83425cb4cd475a3766fad5/testssl.sh#L774
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct File {
    /// The command line arguments which lead to this file
    #[serde(rename = "Invocation")]
    pub invocation: String,

    /// Combination of the executing computer's hostname and `testssl.sh`'s file location
    ///
    /// `{hostname}:{path}`
    pub at: String,

    /// The version of `testssl.sh`
    pub version: String,

    /// The version of openssl
    pub openssl: String,

    /// Unix epoch as string
    pub start_time: String,

    /// List of scans
    pub scan_result: Vec<ScanResult>,

    /// Time it took to scan in seconds
    pub scan_time: ScanTime,
}

/// Header: https://github.com/drwetter/testssl.sh/blob/68dec54cc5aedf856a83425cb4cd475a3766fad5/testssl.sh#L863
/// Sections: https://github.com/drwetter/testssl.sh/blob/68dec54cc5aedf856a83425cb4cd475a3766fad5/testssl.sh#L783
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub target_host: String,
    pub ip: String,
    pub port: String,
    #[serde(rename = "rDNS")]
    pub rdns: String,
    pub service: String,
    pub hostname: Option<String>,

    #[serde(default)]
    pub pretest: Vec<Finding>,
    #[serde(default)]
    pub single_cipher: Vec<Finding>, //

    /// Which tls protocols are supported
    #[serde(default)]
    pub protocols: Vec<Finding>,

    /// Server implementation bugs and [GREASE](https://www.ietf.org/archive/id/draft-ietf-tls-grease-01.txt)
    #[serde(default)]
    pub grease: Vec<Finding>,

    /// Which cipher suites are supported
    #[serde(default)]
    pub ciphers: Vec<Finding>,

    /// Checks robust (perfect) forward secrecy key exchange
    #[serde(default)]
    pub pfs: Vec<Finding>,

    /// The server's preferences
    #[serde(default)]
    pub server_preferences: Vec<Finding>,

    /// The server's defaults
    #[serde(default)]
    pub server_defaults: Vec<Finding>,

    /// The http header set by the server
    #[serde(default)]
    pub header_response: Vec<Finding>,

    /// List of several vulnerabilities
    #[serde(default)]
    pub vulnerabilities: Vec<Finding>,

    /// Which concrete ciphers are supported
    ///
    /// Depending on the option `testssl` is invoked with,
    /// this is either a list of all ciphers or a list of all cipher per tls protocol.
    #[serde(default)]
    pub cipher_tests: Vec<Finding>,

    /// Which browser is able to establish a connection
    #[serde(default)]
    pub browser_simulations: Vec<Finding>,
}

/// This struct's fields are found in [`fileout_json_finding`].
///
/// [`fileout_json_finding`]: https://github.com/drwetter/testssl.sh/blob/68dec54cc5aedf856a83425cb4cd475a3766fad5/testssl.sh#L873
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Finding {
    pub id: String,
    pub severity: Severity,
    pub finding: String,

    pub cve: Option<String>,
    pub cwe: Option<String>,
    pub hint: Option<String>,
}

/// Different levels has been taken from [`show_finding`]
///
/// [`show_finding`]: https://github.com/drwetter/testssl.sh/blob/68dec54cc5aedf856a83425cb4cd475a3766fad5/testssl.sh#L473
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
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

/// The time it took to scan in seconds is either a [`NonZeroU64`] or the string `"Scan interrupted"`
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ScanTime {
    /// Time it took to scan in seconds
    Normal(
        /// Time it took to scan in seconds
        NonZeroU64,
    ),

    /// Always `"Scan interrupted"`
    Error(
        /// Always `"Scan interrupted"`
        String,
    ),
}
