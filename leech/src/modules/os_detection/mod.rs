//! OS detection for a host, based on an ever-growing collection of probes

use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use itertools::Itertools;
use log::debug;
use tokio::task::JoinSet;
use tokio::time::timeout;

use crate::modules::os_detection::errors::{OsDetectionError, TcpFingerprintError};
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

impl Display for WindowsVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowsVersion::Generic => write!(f, "Windows"),
            WindowsVersion::WindowsXP => write!(f, "Windows XP"),
            WindowsVersion::WindowsVista => write!(f, "Windows Vista"),
            WindowsVersion::Windows7 => write!(f, "Windows 7"),
            WindowsVersion::Windows8 => write!(f, "Windows 8"),
            WindowsVersion::Windows8_1 => write!(f, "Windows 8.1"),
            WindowsVersion::Windows10 => write!(f, "Windows 10"),
            WindowsVersion::Windows11 => write!(f, "Windows 11"),
            WindowsVersion::Server2000 => write!(f, "Windows Server 2000"),
            WindowsVersion::Server2003 => write!(f, "Windows Server 2003"),
            WindowsVersion::Server2008 => write!(f, "Windows Server 2008"),
            WindowsVersion::Server2012 => write!(f, "Windows Server 2012"),
            WindowsVersion::Server2016 => write!(f, "Windows Server 2016"),
            WindowsVersion::Server2019 => write!(f, "Windows Server 2019"),
            WindowsVersion::Server2022 => write!(f, "Windows Server 2022"),
            WindowsVersion::Server2025 => write!(f, "Windows Server 2025"),
        }
    }
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
                hint: HashSet::from_iter(hint.into_iter().chain(extra_hints)),
            },
            OperatingSystemInfo::Linux {
                distro,
                kernel_version,
                hint,
            } => OperatingSystemInfo::Linux {
                distro,
                kernel_version,
                hint: HashSet::from_iter(hint.into_iter().chain(extra_hints)),
            },
            OperatingSystemInfo::BSD { version, hint } => OperatingSystemInfo::BSD {
                version,
                hint: HashSet::from_iter(hint.into_iter().chain(extra_hints)),
            },
            OperatingSystemInfo::Android { version, hint } => OperatingSystemInfo::Android {
                version,
                hint: HashSet::from_iter(hint.into_iter().chain(extra_hints)),
            },
            OperatingSystemInfo::OSX { version, hint } => OperatingSystemInfo::OSX {
                version,
                hint: HashSet::from_iter(hint.into_iter().chain(extra_hints)),
            },
            OperatingSystemInfo::IOS { version, hint } => OperatingSystemInfo::IOS {
                version,
                hint: HashSet::from_iter(hint.into_iter().chain(extra_hints)),
            },
            OperatingSystemInfo::Windows { version, hint } => OperatingSystemInfo::Windows {
                version,
                hint: HashSet::from_iter(hint.into_iter().chain(extra_hints)),
            },
        }
    }

    /// Returns the `hint` value that is present in every OS info type.
    pub fn hints(&self) -> &HashSet<String> {
        match self {
            OperatingSystemInfo::Unknown { hint } => hint,
            OperatingSystemInfo::Linux { hint, .. } => hint,
            OperatingSystemInfo::BSD { hint, .. } => hint,
            OperatingSystemInfo::Android { hint, .. } => hint,
            OperatingSystemInfo::OSX { hint, .. } => hint,
            OperatingSystemInfo::IOS { hint, .. } => hint,
            OperatingSystemInfo::Windows { hint, .. } => hint,
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

impl Display for OperatingSystemInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatingSystemInfo::Unknown { .. } => write!(f, "Unknown"),
            OperatingSystemInfo::Linux {
                distro,
                kernel_version,
                ..
            } => {
                write!(f, "Linux")?;
                if !distro.is_empty() {
                    write!(
                        f,
                        " {}",
                        distro
                            .iter()
                            .map(|(distro, v)| match v {
                                None => format!("{distro:?}"),
                                Some(v) => format!("{distro:?} {v}"),
                            })
                            .join(" OR ")
                    )?;
                }
                if !kernel_version.is_empty() {
                    write!(f, " (kernel {})", kernel_version.iter().join(" OR "))?;
                }
                Ok(())
            }
            OperatingSystemInfo::BSD { version, .. } => {
                write!(f, "BSD")?;
                if !version.is_empty() {
                    write!(f, " {}", version.iter().join(" OR "))?;
                }
                Ok(())
            }
            OperatingSystemInfo::Android { version, .. } => {
                write!(f, "Android")?;
                if !version.is_empty() {
                    write!(f, " {}", version.iter().join(" OR "))?;
                }
                Ok(())
            }
            OperatingSystemInfo::OSX { version, .. } => {
                write!(f, "OSX")?;
                if !version.is_empty() {
                    write!(f, " {}", version.iter().join(" OR "))?;
                }
                Ok(())
            }
            OperatingSystemInfo::IOS { version, .. } => {
                write!(f, "iOS")?;
                if !version.is_empty() {
                    write!(f, " {}", version.iter().join(" OR "))?;
                }
                Ok(())
            }
            OperatingSystemInfo::Windows { version, .. } => {
                if version.is_empty() {
                    write!(f, "Windows")
                } else {
                    write!(
                        f,
                        "{}",
                        version
                            .iter()
                            .map(|(ver, v)| match v {
                                None => format!("{ver}"),
                                Some(v) => format!("{ver} {v}"),
                            })
                            .join(" OR ")
                    )
                }
            }
        }
    }
}

fn aggregate_os_results(infos: &[OperatingSystemInfo]) -> Option<OperatingSystemInfo> {
    if infos.is_empty() {
        return None;
    }

    let mut combined = infos[0].clone();
    for info in &infos[1..] {
        let (lhs, rhs) = if combined.kind() < info.kind() {
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
            (
                OperatingSystemInfo::Linux {
                    distro: ldistro,
                    kernel_version: lkernel_version,
                    hint: lhint,
                },
                OperatingSystemInfo::Linux {
                    distro: rdistro,
                    kernel_version: rkernel_version,
                    hint: rhint,
                },
            ) => OperatingSystemInfo::Linux {
                distro: HashSet::from_iter(ldistro.into_iter().chain(rdistro.into_iter())),
                kernel_version: HashSet::from_iter(
                    lkernel_version
                        .into_iter()
                        .chain(rkernel_version.into_iter()),
                ),
                hint: HashSet::from_iter(lhint.into_iter().chain(rhint.into_iter())),
            },
            (
                OperatingSystemInfo::BSD {
                    version: lversion,
                    hint: lhint,
                },
                OperatingSystemInfo::BSD {
                    version: rversion,
                    hint: rhint,
                },
            ) => OperatingSystemInfo::BSD {
                version: HashSet::from_iter(lversion.into_iter().chain(rversion.into_iter())),
                hint: HashSet::from_iter(lhint.into_iter().chain(rhint.into_iter())),
            },
            (
                OperatingSystemInfo::Android {
                    version: lversion,
                    hint: lhint,
                },
                OperatingSystemInfo::Android {
                    version: rversion,
                    hint: rhint,
                },
            ) => OperatingSystemInfo::Android {
                version: HashSet::from_iter(lversion.into_iter().chain(rversion.into_iter())),
                hint: HashSet::from_iter(lhint.into_iter().chain(rhint.into_iter())),
            },
            (
                OperatingSystemInfo::OSX {
                    version: lversion,
                    hint: lhint,
                },
                OperatingSystemInfo::OSX {
                    version: rversion,
                    hint: rhint,
                },
            ) => OperatingSystemInfo::OSX {
                version: HashSet::from_iter(lversion.into_iter().chain(rversion.into_iter())),
                hint: HashSet::from_iter(lhint.into_iter().chain(rhint.into_iter())),
            },
            (
                OperatingSystemInfo::IOS {
                    version: lversion,
                    hint: lhint,
                },
                OperatingSystemInfo::IOS {
                    version: rversion,
                    hint: rhint,
                },
            ) => OperatingSystemInfo::IOS {
                version: HashSet::from_iter(lversion.into_iter().chain(rversion.into_iter())),
                hint: HashSet::from_iter(lhint.into_iter().chain(rhint.into_iter())),
            },
            (
                OperatingSystemInfo::Windows {
                    version: lversion,
                    hint: lhint,
                },
                OperatingSystemInfo::Windows {
                    version: rversion,
                    hint: rhint,
                },
            ) => OperatingSystemInfo::Windows {
                version: HashSet::from_iter(lversion.into_iter().chain(rversion.into_iter())),
                hint: HashSet::from_iter(lhint.into_iter().chain(rhint.into_iter())),
            },
            (_, _) => return None,
        };
    }

    Some(combined)
}

/// Calls a bunch of OS detection methods to try to find out the operating system running on the given host IP address.
pub async fn os_detection(ip_addr: IpAddr) -> Result<OperatingSystemInfo, OsDetectionError> {
    let (opened_port, _closed_port) =
        find_open_and_closed_port(ip_addr, Duration::from_millis(2000), 16).await?;

    let mut tasks = JoinSet::new();

    tasks.spawn(os_detect_tcp_fingerprint(
        SocketAddr::new(ip_addr, opened_port),
        Duration::from_millis(4000),
    ));
    tasks.spawn(os_detect_ssh(
        ip_addr,
        Duration::from_millis(1500),
        Duration::from_millis(4000),
    ));

    let mut found = Vec::new();

    while let Some(result) = tasks.join_next().await {
        debug!("Found OS detection partial result: {result:?}");
        found.push(result??);
    }

    aggregate_os_results(&found).ok_or(OsDetectionError::Ambiguous(found))
}

/// Opens half a TCP connection and reads implementation-defined characteristics from the initial SYN/ACK.
///
/// Calls `fingerprint_tcp` and looks up matching fingerprints from a hardcoded fingerprint database.
async fn os_detect_tcp_fingerprint(
    addr: SocketAddr,
    total_timeout: Duration,
) -> Result<OperatingSystemInfo, OsDetectionError> {
    match fingerprint_tcp(addr, total_timeout).await {
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
            "8:3:28:2:*:8:5b4:64312",
        ),
        (
            OperatingSystemInfo::windows(None, None),
            "8:1:20:2:ffff:8:5b4:411312",
        ),
        (
            OperatingSystemInfo::linux(None, None, None),
            "8:2:28:1:*:7:*:31642",
        ),
        (
            OperatingSystemInfo::linux(None, None, None),
            "8:2:28:1:a9b0:b:*:31642",
        ),
        (
            OperatingSystemInfo::bsd(Some(String::from("OpenBSD")), None),
            "8:3:2c:1:4000:6:5b4:611314112",
        ),
        (
            OperatingSystemInfo::bsd(Some(String::from("FreeBSD")), None),
            "8:2:*:*:*:*:5b4:*",
        ),
        (
            OperatingSystemInfo::bsd(Some(String::from("NetBSD")), None),
            "8:2:28:1:*:3:5b4:64312",
        ),
    ];

    let fingerprint = &fingerprint.to_string();
    for (os, pattern) in known {
        let mut matches = true;
        for (expected, actual) in pattern.split(':').zip(fingerprint.split(':')) {
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
async fn os_detect_ssh(
    ip_addr: IpAddr,
    recv_timeout: Duration,
    total_timeout: Duration,
) -> Result<OperatingSystemInfo, OsDetectionError> {
    let settings = DetectServiceSettings {
        socket: SocketAddr::new(ip_addr, 22),
        timeout: recv_timeout,
        always_run_everything: true,
    };
    let Ok(result) = timeout(
        total_timeout,
        settings.probe_tcp(b"SSH-2.0-OpenSSH_9.6\r\n"),
    )
    .await
    else {
        // timeout
        return Ok(OperatingSystemInfo::default());
    };

    let Ok(result) = result else {
        // TOOD: might want to return differently if the error is a specific one, but right now it's a dynamic error
        // without proper error information for us to match on.
        return Ok(OperatingSystemInfo::default());
    };

    if let Some(data) = result {
        if data.starts_with(b"SSH-") {
            if let Some(end) = data.iter().find_position(|&&c| c == b'\r' || c == b'\n') {
                return Ok(os_detect_ssh_header(&data[0..end.0]));
            }
        }
    }

    Ok(OperatingSystemInfo::default())
}

fn os_detect_ssh_header(header: &[u8]) -> OperatingSystemInfo {
    if &header[0..8] != b"SSH-2.0-" {
        return OperatingSystemInfo::default();
    }

    let hint = Some(String::from_utf8_lossy(header).to_string());
    let header = &header[8..];
    let contains = |needle: &[u8]| header.windows(needle.len()).contains(&needle);

    // TODO: this list should probably be some external text files, there might be a public repository for these strings
    // with a compatible license on the internet as well.
    if header == b"OpenSSH_7.2p2 Ubuntu-4ubuntu2.10" {
        OperatingSystemInfo::linux(
            Some((LinuxDistro::Ubuntu, Some(String::from("16")))),
            None,
            hint,
        )
    } else if header == b"OpenSSH_7.6p1 Ubuntu-4ubuntu0.7" {
        OperatingSystemInfo::linux(
            Some((LinuxDistro::Ubuntu, Some(String::from("18")))),
            None,
            hint,
        )
    } else if header == b"OpenSSH_8.2p1 Ubuntu-4ubuntu0.11" {
        OperatingSystemInfo::linux(
            Some((LinuxDistro::Ubuntu, Some(String::from("20")))),
            None,
            hint,
        )
    } else if header == b"OpenSSH_8.9p1 Ubuntu-3ubuntu0.6" {
        OperatingSystemInfo::linux(
            Some((LinuxDistro::Ubuntu, Some(String::from("22")))),
            None,
            hint,
        )
    } else if contains(b"Ubuntu") || contains(b"ubuntu") {
        OperatingSystemInfo::linux(Some((LinuxDistro::Ubuntu, None)), None, hint)
    } else if contains(b"Debian") || contains(b"debian") {
        if contains(b"+deb7u") {
            OperatingSystemInfo::linux(
                Some((LinuxDistro::Debian, Some(String::from("7")))),
                None,
                hint,
            )
        } else if contains(b"+deb8u") {
            OperatingSystemInfo::linux(
                Some((LinuxDistro::Debian, Some(String::from("8")))),
                None,
                hint,
            )
        } else if contains(b"+deb9u") {
            OperatingSystemInfo::linux(
                Some((LinuxDistro::Debian, Some(String::from("9")))),
                None,
                hint,
            )
        } else if contains(b"+deb10u") {
            OperatingSystemInfo::linux(
                Some((LinuxDistro::Debian, Some(String::from("10")))),
                None,
                hint,
            )
        } else if contains(b"+deb11u") {
            OperatingSystemInfo::linux(
                Some((LinuxDistro::Debian, Some(String::from("11")))),
                None,
                hint,
            )
        } else if contains(b"+deb12u") {
            OperatingSystemInfo::linux(
                Some((LinuxDistro::Debian, Some(String::from("12")))),
                None,
                hint,
            )
        } else {
            OperatingSystemInfo::linux(Some((LinuxDistro::Debian, None)), None, hint)
        }
    } else if contains(b"FreeBSD") {
        OperatingSystemInfo::bsd(Some(String::from("FreeBSD")), hint)
    } else if contains(b"NetBSD") {
        OperatingSystemInfo::bsd(Some(String::from("NetBSD")), hint)
    } else if contains(b"OpenSSH_for_Windows") || contains(b"Windows") || contains(b"windows") {
        OperatingSystemInfo::windows(None, hint)
    } else {
        OperatingSystemInfo::unknown(hint)
    }
}
