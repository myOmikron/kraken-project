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
    pub scan_result: Vec<Service>,

    /// Time it took to scan in seconds
    pub scan_time: ScanTime,
}

/// A service's scan results or an error
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)] // the error variant should be the cold path
pub enum Service {
    /// A service's scan results
    Result(ScanResult),

    /// Some error prevented a service from being scanned
    Error(Finding),
}

/// A service's scan results
///
/// Header: https://github.com/drwetter/testssl.sh/blob/68dec54cc5aedf856a83425cb4cd475a3766fad5/testssl.sh#L863
/// Sections: https://github.com/drwetter/testssl.sh/blob/68dec54cc5aedf856a83425cb4cd475a3766fad5/testssl.sh#L783
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    /// The original user target this result belongs to
    pub target_host: String,

    /// The scanned ip address
    pub ip: String,

    /// The scanned port
    pub port: String,

    /// The ip address' rDNS name
    #[serde(rename = "rDNS")]
    pub rdns: String,

    /// The detected service
    pub service: String,

    /// TODO: not found yet in the wild
    pub hostname: Option<String>,

    /// Some sanity checks which can't be disabled
    #[serde(default)]
    pub pretest: Vec<Finding>,

    /// The results of a single cipher check
    ///
    /// [`run_testssl`](super::run_testssl) doesn't expose the necessary option.
    #[serde(default)]
    pub single_cipher: Vec<Finding>,

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

impl ScanResult {
    /// Iterate over all sections
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &[Finding])> {
        let Self {
            target_host: _,
            ip: _,
            port: _,
            rdns: _,
            service: _,
            hostname: _,
            pretest,
            single_cipher,
            protocols,
            grease,
            ciphers,
            pfs,
            server_preferences,
            server_defaults,
            header_response,
            vulnerabilities,
            cipher_tests,
            browser_simulations,
        } = self;
        [
            ("pretest", pretest.as_slice()),
            ("single_cipher", single_cipher.as_slice()),
            ("protocols", protocols.as_slice()),
            ("grease", grease.as_slice()),
            ("ciphers", ciphers.as_slice()),
            ("pfs", pfs.as_slice()),
            ("server_preferences", server_preferences.as_slice()),
            ("server_defaults", server_defaults.as_slice()),
            ("header_response", header_response.as_slice()),
            ("vulnerabilities", vulnerabilities.as_slice()),
            ("cipher_tests", cipher_tests.as_slice()),
            ("browser_simulations", browser_simulations.as_slice()),
        ]
        .into_iter()
    }
}

/// Either a test's result or a log message
///
/// Which one it is might be determined by the [`Severity`]
///
/// This struct's fields are found in [`fileout_json_finding`].
///
/// [`fileout_json_finding`]: https://github.com/drwetter/testssl.sh/blob/68dec54cc5aedf856a83425cb4cd475a3766fad5/testssl.sh#L873
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Finding {
    /// The test's id
    pub id: String,

    /// The test result's severity or the log message's log level
    pub severity: Severity,

    /// The test result or a log message
    pub finding: String,

    /// An CVE associated with the test
    pub cve: Option<String>,

    /// An CWE associated with the test
    pub cwe: Option<String>,

    /// An hint on how to fix the problem
    ///
    /// Not completely implemented yet in `testssl.sh`
    pub hint: Option<String>,
}

/// Either a test result's severity or a log message's log level
///
/// Different levels has been taken from [`show_finding`]
///
/// [`show_finding`]: https://github.com/drwetter/testssl.sh/blob/68dec54cc5aedf856a83425cb4cd475a3766fad5/testssl.sh#L473
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    /// A debug level log message
    Debug,
    /// An info level log message
    Info,
    /// A warning level log message
    Warn,
    /// An error level log message
    Fatal,

    /// The test's result doesn't pose an issue
    Ok,
    /// The test's result pose a low priority issue
    Low,
    /// The test's result pose a medium priority issue
    Medium,
    /// The test's result pose a high priority issue
    High,
    /// The test's result pose a critical priority issue
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
