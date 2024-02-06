//! OS detection for a host, based on an ever-growing collection of probes

use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use tokio::task::JoinSet;

use crate::modules::os_detection::errors::{OsDetectionError, RawTcpError, TcpFingerprintError};
use crate::modules::os_detection::syn_scan::find_open_and_closed_port;
use crate::modules::os_detection::tcp_fingerprint::{fingerprint_tcp, TcpFingerprint};
use crate::modules::service_detection::DetectServiceSettings;

pub mod errors;
mod syn_scan;
pub mod tcp_fingerprint;

/// Various known linux distribution names. Version is stored outside of this enum, e.g. in the tuple in
/// OperatingSystemInfo.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
#[allow(missing_docs)]
pub enum LinuxDistro {
    /// used with other/unknown distros but with detected extra version information
    Generic,
    ArchLinux,
    CentOS,
    Debian,
    Ubuntu,
    Fedora,
    OpenSUSE,
    Oracle,
    RHEL,
}

/// Various known Windows / Windows Server versions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
#[allow(missing_docs)]
pub enum WindowsVersion {
    /// used with other/unknown distros but with detected extra version information
    Generic,
    WindowsXP,
    WindowsVista,
    Windows7,
    Windows8,
    Windows8_1,
    Windows10,
    Windows11,
    Server2000,
    Server2003,
    Server2008,
    Server2012,
    Server2016,
    Server2019,
    Server2022,
    Server2025,
}

/// Information about a detected operating system.
#[derive(Debug, Clone, PartialEq)]
pub enum OperatingSystemInfo {
    /// Unknown OS, but possibly containing human-readable hints
    Unknown {
        /// List of human-readable OS strings / hints to the running OS
        hint: HashSet<String>,
    },
    /// Linux distros
    Linux {
        /// List of detected Linux distros along with additional version information strings
        distro: HashSet<(LinuxDistro, Option<String>)>,
        /// Detected Linux kernel versions, if any.
        kernel_version: HashSet<String>,
        /// List of human-readable extra hints
        hint: HashSet<String>,
    },
    /// BSD variants
    BSD {
        /// List of detected BSD version information strings
        version: HashSet<String>,
        /// List of human-readable extra hints
        hint: HashSet<String>,
    },
    /// Android
    Android {
        /// List of detected Android version information strings
        version: HashSet<String>,
        /// List of human-readable extra hints
        hint: HashSet<String>,
    },
    /// Apple OSX
    OSX {
        /// List of detected OSX version information strings
        version: HashSet<String>,
        /// List of human-readable extra hints
        hint: HashSet<String>,
    },
    /// Apple iOS
    IOS {
        /// List of detected IOS version information strings
        version: HashSet<String>,
        /// List of human-readable extra hints
        hint: HashSet<String>,
    },
    /// Microsoft Windows
    Windows {
        /// List of detected WindowsVersion types along with additional optional Windows version information
        version: HashSet<(WindowsVersion, Option<String>)>,
        /// List of human-readable extra hints
        hint: HashSet<String>,
    },
}

/// Maps to just the type of `OperatingSystemInfo`
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
#[allow(missing_docs)]
pub enum OperatingSystemInfoKind {
    Unknown,
    Linux,
    BSD,
    Android,
    OSX,
    IOS,
    Windows,
}

impl OperatingSystemInfo {
    /// Constructs `OperatingSystemInfo::Unknown`
    pub fn unknown(hint: Option<String>) -> OperatingSystemInfo {
        OperatingSystemInfo::Unknown {
            hint: match hint {
                None => HashSet::new(),
                Some(hint) => HashSet::from([hint]),
            },
        }
    }

    /// Adds the `extra_hints` parameter to the `hint` set that is present in every OS info type.
    pub fn with_hints(self, extra_hints: HashSet<String>) -> OperatingSystemInfo {
        match self {
            OperatingSystemInfo::Unknown { hint } => OperatingSystemInfo::Unknown {
                hint: HashSet::from_iter(hint.into_iter().chain(extra_hints.into_iter())),
            },
            OperatingSystemInfo::Linux {
                distro,
                kernel_version,
                hint,
            } => OperatingSystemInfo::Linux {
                distro,
                kernel_version,
                hint: HashSet::from_iter(hint.into_iter().chain(extra_hints.into_iter())),
            },
            OperatingSystemInfo::BSD { version, hint } => OperatingSystemInfo::BSD {
                version,
                hint: HashSet::from_iter(hint.into_iter().chain(extra_hints.into_iter())),
            },
            OperatingSystemInfo::Android { version, hint } => OperatingSystemInfo::Android {
                version,
                hint: HashSet::from_iter(hint.into_iter().chain(extra_hints.into_iter())),
            },
            OperatingSystemInfo::OSX { version, hint } => OperatingSystemInfo::OSX {
                version,
                hint: HashSet::from_iter(hint.into_iter().chain(extra_hints.into_iter())),
            },
            OperatingSystemInfo::IOS { version, hint } => OperatingSystemInfo::IOS {
                version,
                hint: HashSet::from_iter(hint.into_iter().chain(extra_hints.into_iter())),
            },
            OperatingSystemInfo::Windows { version, hint } => OperatingSystemInfo::Windows {
                version,
                hint: HashSet::from_iter(hint.into_iter().chain(extra_hints.into_iter())),
            },
        }
    }

    /// Constructs `OperatingSystemInfo::Linux`
    pub fn linux(
        distro: Option<(LinuxDistro, Option<String>)>,
        kernel: Option<String>,
        hint: Option<String>,
    ) -> OperatingSystemInfo {
        OperatingSystemInfo::Linux {
            distro: match distro {
                None => HashSet::new(),
                Some(distro) => HashSet::from([distro]),
            },
            kernel_version: match kernel {
                None => HashSet::new(),
                Some(kernel) => HashSet::from([kernel]),
            },
            hint: match hint {
                None => HashSet::new(),
                Some(hint) => HashSet::from([hint]),
            },
        }
    }

    /// Constructs `OperatingSystemInfo::BSD`
    pub fn bsd(version: Option<String>, hint: Option<String>) -> OperatingSystemInfo {
        OperatingSystemInfo::BSD {
            version: match version {
                None => HashSet::new(),
                Some(version) => HashSet::from([version]),
            },
            hint: match hint {
                None => HashSet::new(),
                Some(hint) => HashSet::from([hint]),
            },
        }
    }

    /// Constructs `OperatingSystemInfo::Android`
    pub fn android(version: Option<String>, hint: Option<String>) -> OperatingSystemInfo {
        OperatingSystemInfo::Android {
            version: match version {
                None => HashSet::new(),
                Some(version) => HashSet::from([version]),
            },
            hint: match hint {
                None => HashSet::new(),
                Some(hint) => HashSet::from([hint]),
            },
        }
    }

    /// Constructs `OperatingSystemInfo::OSX`
    pub fn osx(version: Option<String>, hint: Option<String>) -> OperatingSystemInfo {
        OperatingSystemInfo::OSX {
            version: match version {
                None => HashSet::new(),
                Some(version) => HashSet::from([version]),
            },
            hint: match hint {
                None => HashSet::new(),
                Some(hint) => HashSet::from([hint]),
            },
        }
    }

    /// Constructs `OperatingSystemInfo::IOS`
    pub fn ios(version: Option<String>, hint: Option<String>) -> OperatingSystemInfo {
        OperatingSystemInfo::IOS {
            version: match version {
                None => HashSet::new(),
                Some(version) => HashSet::from([version]),
            },
            hint: match hint {
                None => HashSet::new(),
                Some(hint) => HashSet::from([hint]),
            },
        }
    }

    /// Constructs `OperatingSystemInfo::Windows`
    pub fn windows(
        version: Option<(WindowsVersion, Option<String>)>,
        hint: Option<String>,
    ) -> OperatingSystemInfo {
        OperatingSystemInfo::Windows {
            version: match version {
                None => HashSet::new(),
                Some(version) => HashSet::from([version]),
            },
            hint: match hint {
                None => HashSet::new(),
                Some(hint) => HashSet::from([hint]),
            },
        }
    }

    /// Returns just the kind of operating system without additional extra infos.
    pub fn kind(&self) -> OperatingSystemInfoKind {
        match self {
            OperatingSystemInfo::Unknown { .. } => OperatingSystemInfoKind::Unknown,
            OperatingSystemInfo::Linux { .. } => OperatingSystemInfoKind::Linux,
            OperatingSystemInfo::BSD { .. } => OperatingSystemInfoKind::BSD,
            OperatingSystemInfo::Android { .. } => OperatingSystemInfoKind::Android,
            OperatingSystemInfo::OSX { .. } => OperatingSystemInfoKind::OSX,
            OperatingSystemInfo::IOS { .. } => OperatingSystemInfoKind::IOS,
            OperatingSystemInfo::Windows { .. } => OperatingSystemInfoKind::Windows,
        }
    }
}

impl Default for OperatingSystemInfo {
    fn default() -> Self {
        OperatingSystemInfo::Unknown {
            hint: HashSet::new(),
        }
    }
}

fn aggregate_os_results(infos: &[OperatingSystemInfo]) -> Option<OperatingSystemInfo> {
    if infos.is_empty() {
        return None;
    }

    let mut combined = infos[0].clone();
    for info in &infos[1..] {
        let (lhs, rhs) = if &combined.kind() < &info.kind() {
            (combined, info.clone())
        } else {
            (info.clone(), combined)
        };
        combined = match (lhs, rhs) {
            (
                OperatingSystemInfo::Unknown { hint: lhint },
                OperatingSystemInfo::Unknown { hint: rhint },
            ) => OperatingSystemInfo::Unknown {
                hint: HashSet::from_iter(lhint.into_iter().chain(rhint.into_iter())),
            },
            (OperatingSystemInfo::Unknown { hint }, defined) => defined.with_hints(hint),
            (_, _) => return None,
        };
    }

    Some(combined)
}

/// Calls a bunch of OS detection methods to try to find out the operating system running on the given host IP address.
pub async fn os_detection(ip_addr: IpAddr) -> Result<OperatingSystemInfo, OsDetectionError> {
    let (opened_port, _closed_port) =
        find_open_and_closed_port(ip_addr, Duration::from_millis(1000), 32).await?;

    let mut tasks = JoinSet::new();

    tasks.spawn(os_detect_tcp_fingerprint(SocketAddr::new(
        ip_addr,
        opened_port,
    )));
    tasks.spawn(os_detect_ssh(ip_addr));

    let mut found = Vec::new();

    while let Some(result) = tasks.join_next().await {
        found.push(result??);
    }

    return aggregate_os_results(&found).ok_or(OsDetectionError::Ambiguous(found));
}

/// Opens half a TCP connection and reads implementation-defined characteristics from the initial SYN/ACK.
///
/// Calls `fingerprint_tcp` and looks up matching fingerprints from a hardcoded fingerprint database.
async fn os_detect_tcp_fingerprint(
    addr: SocketAddr,
) -> Result<OperatingSystemInfo, OsDetectionError> {
    match fingerprint_tcp(addr, Duration::from_millis(4000)).await {
        Ok(fingerprint) => fingerprint_os_lookup(fingerprint),
        Err(err) => match err {
            TcpFingerprintError::ConnectionTimeout => Ok(OperatingSystemInfo::default()),
            TcpFingerprintError::RawTcpError(err) => Err(OsDetectionError::RawTcpError(err)),
        },
    }
}

fn fingerprint_os_lookup(
    fingerprint: TcpFingerprint,
) -> Result<OperatingSystemInfo, OsDetectionError> {
    let known = [
        (
            OperatingSystemInfo::windows(None, None),
            "8:3:28:80:a:*:8:5b4:64312",
        ),
        (
            OperatingSystemInfo::linux(None, None, None),
            "8:2:28:40:a:*:7:*:31642",
        ),
        (
            OperatingSystemInfo::bsd(Some(String::from("OpenBSD")), None),
            "8:3:2c:40:b:4000:6:5b4:611314112",
        ),
        (
            OperatingSystemInfo::bsd(Some(String::from("FreeBSD")), None),
            "8:2:*:*:*:*:*:5b4:*",
        ),
        (
            OperatingSystemInfo::bsd(Some(String::from("NetBSD")), None),
            "8:2:28:40:a:*:3:5b4:64312",
        ),
    ];

    let fingerprint = &fingerprint.to_string();
    for (os, pattern) in known {
        let mut matches = true;
        for (expected, actual) in pattern.split(':').zip(fingerprint.split(':')).into_iter() {
            if expected != "*" && expected != actual {
                matches = false;
                break;
            }
        }

        if matches {
            return Ok(os.with_hints(HashSet::from([
                "TCP fingerprint: ".to_owned() + &*fingerprint.to_string()
            ])));
        }
    }

    // TODO
    Ok(OperatingSystemInfo::unknown(Some(
        "TCP fingerprint: ".to_owned() + &*fingerprint.to_string(),
    )))
}

/// Opens a TLS connection on ip_addr with port 22 and reads out the SSH banner.
async fn os_detect_ssh(ip_addr: IpAddr) -> Result<OperatingSystemInfo, OsDetectionError> {
    let settings = DetectServiceSettings {
        socket: SocketAddr::new(ip_addr, 22),
        timeout: Duration::from_millis(4000),
        always_run_everything: true,
    };
    let Ok(result) = settings.probe_tls(b"", None).await else {
        // TOOD: might want to return differently if the error is a specific one, but right now it's a dynamic error
        // without proper error information for us to match on.
        return Ok(OperatingSystemInfo::default());
    };
    match result {
        Ok(data) => {
            return if data.starts_with(b"SSH-") {
                Ok(os_detect_ssh_header(&data))
            } else {
                Ok(OperatingSystemInfo::default())
            }
        }
        // TLS error: ignore
        Err(_) => Ok(OperatingSystemInfo::default()),
    }
}

fn os_detect_ssh_header(header: &[u8]) -> OperatingSystemInfo {
    // TODO
    OperatingSystemInfo::unknown(Some(String::from_utf8_lossy(header).to_string()))
}
