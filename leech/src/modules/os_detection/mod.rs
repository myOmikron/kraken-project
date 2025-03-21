//! OS detection for a host, based on an ever-growing collection of probes

use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::num::NonZeroUsize;
use std::time::Duration;

use ipnetwork::IpNetwork;
use itertools::Itertools;
use kraken_proto::any_attack_response;
use kraken_proto::push_attack_request;
use kraken_proto::shared::Address;
use kraken_proto::shared::OperatingSystem;
use kraken_proto::OsDetectionRequest;
use kraken_proto::OsDetectionResponse;
use kraken_proto::RepeatedOsDetectionResponse;
use log::debug;
use strum_macros::EnumString;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinSet;
use tokio::time::timeout;
use tonic::async_trait;
use tonic::Status;

use crate::modules::os_detection::errors::OsDetectionError;
use crate::modules::os_detection::errors::TcpFingerprintError;
use crate::modules::os_detection::fingerprint_db::FINGERPRINT_DB;
use crate::modules::os_detection::syn_scan::find_open_and_closed_port;
use crate::modules::os_detection::tcp_fingerprint::fingerprint_tcp;
use crate::modules::os_detection::tcp_fingerprint::TcpFingerprint;
use crate::modules::service_detection::tcp::OneShotTcpSettings;
use crate::modules::service_detection::tcp::ProbeTcpResult;
use crate::modules::StreamedAttack;
use crate::utils::IteratorExt;

pub mod errors;
mod fingerprint_db;
mod syn_scan;
pub mod tcp_fingerprint;

pub struct OsDetection;
#[async_trait]
impl StreamedAttack for OsDetection {
    type Settings = OsDetectionSettings;
    type Output = OsDetectionResult;
    type Error = OsDetectionError;

    async fn execute(
        settings: Self::Settings,
        sender: Sender<Self::Output>,
    ) -> Result<(), Self::Error> {
        os_detection(settings, sender).await
    }

    type Request = OsDetectionRequest;

    fn get_attack_uuid(request: &Self::Request) -> &str {
        &request.attack_uuid
    }

    fn decode_settings(request: Self::Request) -> Result<Self::Settings, Status> {
        if request.targets.is_empty() {
            return Err(Status::invalid_argument("no targets specified"));
        }

        let addresses: Vec<_> = request
            .targets
            .into_iter()
            .map(IpNetwork::try_from)
            .collect::<Result<_, _>>()?;

        let fingerprint_port = match request.fingerprint_port {
            None => None,
            Some(p) => Some(
                u16::try_from(p)
                    .map_err(|_| Status::invalid_argument("`fingerprint_port` out of range"))?,
            ),
        };

        let ssh_port = match request.ssh_port {
            None => None,
            Some(p) => Some(
                u16::try_from(p)
                    .map_err(|_| Status::invalid_argument("`ssh_port` out of range"))?,
            ),
        };

        Ok(OsDetectionSettings {
            addresses,
            fingerprint_port,
            fingerprint_timeout: Duration::from_millis(request.fingerprint_timeout),
            ssh_port,
            ssh_connect_timeout: Duration::from_millis(request.ssh_connect_timeout),
            ssh_timeout: Duration::from_millis(request.ssh_timeout),
            port_ack_timeout: Duration::from_millis(request.port_ack_timeout),
            port_parallel_syns: request.port_parallel_syns as usize,
            concurrent_limit: request.concurrent_limit,
        })
    }

    type Response = OsDetectionResponse;

    fn encode_output(OsDetectionResult { address, os }: Self::Output) -> Self::Response {
        let host = Some(Address::from(address));
        match os {
            OperatingSystemInfo::Unknown { hint } => OsDetectionResponse {
                host,
                os: OperatingSystem::Unknown as _,
                hints: hint.iter().cloned().collect(),
                versions: Vec::new(),
            },
            OperatingSystemInfo::Linux {
                distro,
                kernel_version,
                hint,
            } => OsDetectionResponse {
                host,
                os: OperatingSystem::Linux as _,
                hints: if kernel_version.is_empty() {
                    hint.iter().cloned().collect()
                } else {
                    hint.iter()
                        .cloned()
                        .chain(vec![format!(
                            "Kernel {}",
                            kernel_version.iter().join(" OR ")
                        )])
                        .collect()
                },
                versions: distro
                    .iter()
                    .map(|(distro, v)| match v {
                        None => format!("{distro:?}"),
                        Some(v) => format!("{distro:?} {v}"),
                    })
                    .collect(),
            },
            OperatingSystemInfo::BSD { version, hint } => OsDetectionResponse {
                host,
                os: OperatingSystem::Bsd as _,
                hints: hint.iter().cloned().collect(),
                versions: version.iter().cloned().collect(),
            },
            OperatingSystemInfo::Android { version, hint } => OsDetectionResponse {
                host,
                os: OperatingSystem::Android as _,
                hints: hint.iter().cloned().collect(),
                versions: version.iter().cloned().collect(),
            },
            OperatingSystemInfo::OSX { version, hint } => OsDetectionResponse {
                host,
                os: OperatingSystem::Osx as _,
                hints: hint.iter().cloned().collect(),
                versions: version.iter().cloned().collect(),
            },
            OperatingSystemInfo::IOS { version, hint } => OsDetectionResponse {
                host,
                os: OperatingSystem::Ios as _,
                hints: hint.iter().cloned().collect(),
                versions: version.iter().cloned().collect(),
            },
            OperatingSystemInfo::Windows { version, hint } => OsDetectionResponse {
                host,
                os: OperatingSystem::Windows as _,
                hints: hint.iter().cloned().collect(),
                versions: version
                    .iter()
                    .map(|(ver, v)| match v {
                        None => format!("{ver}"),
                        Some(v) => format!("{ver} {v}"),
                    })
                    .collect(),
            },
        }
    }

    fn print_output(output: &Self::Output) {
        println!("OS detection result:");
        println!("- likely OS: {}", output.os);
        let hints = output.os.hints();
        if !hints.is_empty() {
            println!("- hints:");
            for hint in hints {
                println!("\t- {hint}");
            }
        }
    }

    fn wrap_for_backlog(response: Self::Response) -> any_attack_response::Response {
        any_attack_response::Response::OsDetection(response)
    }

    fn wrap_for_push(responses: Vec<Self::Response>) -> push_attack_request::Response {
        push_attack_request::Response::OsDetection(RepeatedOsDetectionResponse { responses })
    }
}

#[derive(Debug, Clone)]
pub struct OsDetectionResult {
    /// The address that the os was detected on.
    pub address: IpAddr,
    /// The os that was detected
    pub os: OperatingSystemInfo,
}

/// Various known linux distribution names. Version is stored outside of this enum, e.g. in the tuple in
/// OperatingSystemInfo.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash, EnumString)]
#[allow(missing_docs)]
pub enum LinuxDistro {
    // independent non-android linux distros
    Alpine,
    ArchLinux,
    ClearLinuxOS,
    Debian,
    Elive,
    Gentoo,
    GnuGuixSystem,
    GoboLinux,
    KaOS,
    Mageia,
    NixOS,
    OpenSUSE,
    PuppyLinux,
    SliTaz,
    TinyCoreLinux,
    VoidLinux,
    // non-independent popular linux distros (distrowatch)
    LinuxMint,
    EndeavourOS,
    Manjaro,
    Ubuntu,
    Fedora,
    PopOs,
    Zorin,
    // server distros
    CentOS,
    RHEL,
    OracleLinux,
    RockyLinux,
    AlmaLinux,
    EuroLinux,
}

/// Various known Windows / Windows Server versions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash, EnumString)]
#[allow(missing_docs)]
pub enum WindowsVersion {
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

/// OS detection settings.
#[derive(Debug)]
pub struct OsDetectionSettings {
    /// The host IP address to scan
    pub addresses: Vec<IpNetwork>,

    /// Optionally set a fixed TCP fingerprint port. This port must be open and accept connections.
    pub fingerprint_port: Option<u16>,

    /// The timeout for TCP fingerprinting
    pub fingerprint_timeout: Duration,

    /// If set, perform OS detection via SSH header on this port.
    pub ssh_port: Option<u16>,

    /// The timeout how long to wait at most for the SSH connection to be established.
    /// Has no effect if `ssh_port` is `None`.
    pub ssh_connect_timeout: Duration,

    /// The total timeout for SSH detection. Must be more than `ssh_connect_timeout`.
    /// Has no effect if `ssh_port` is `None`.
    pub ssh_timeout: Duration,

    /// The timeout for a SYN/ACK response on a port for guessing open ports.
    /// Has no effect if `fingerprint_port` is set, since that will just be used instead.
    pub port_ack_timeout: Duration,

    /// The amount of parallel SYN requests to try out finding ports.
    /// Has no effect if `fingerprint_port` is set, since that will just be used instead.
    pub port_parallel_syns: usize,

    /// Maximum of concurrent tasks that should be spawned
    ///
    /// 0 means, that there should be no limit.
    pub concurrent_limit: u32,
}

/// Calls a bunch of OS detection methods to try to find out the operating system running on the given host IP address.
pub async fn os_detection(
    settings: OsDetectionSettings,
    tx: Sender<OsDetectionResult>,
) -> Result<(), OsDetectionError> {
    let OsDetectionSettings {
        addresses,
        fingerprint_timeout,
        fingerprint_port,
        ssh_connect_timeout,
        ssh_timeout,
        ssh_port,
        port_ack_timeout,
        port_parallel_syns,
        concurrent_limit,
    } = settings;

    addresses
        .into_iter()
        .flat_map(|network| network.into_iter())
        .try_for_each_concurrent(
            NonZeroUsize::new(concurrent_limit as usize),
            |ip_addr| async move {
                if ssh_timeout <= ssh_connect_timeout {
                    return Err(OsDetectionError::InvalidSetting(String::from(
                        "`ssh_timeout` must be larger than `ssh_connect_timeout`",
                    )));
                }
                if port_parallel_syns == 0 {
                    return Err(OsDetectionError::InvalidSetting(String::from(
                        "`port_parallel_syns` must be non-zero",
                    )));
                }

                let (opened_port, _) = match fingerprint_port {
                    None => {
                        find_open_and_closed_port(ip_addr, port_ack_timeout, port_parallel_syns)
                            .await?
                    }
                    Some(p) => (p, 1),
                };

                let mut tasks = JoinSet::new();

                tasks.spawn(os_detect_tcp_fingerprint(
                    SocketAddr::new(ip_addr, opened_port),
                    fingerprint_timeout,
                ));
                if let Some(ssh_port) = ssh_port {
                    tasks.spawn(os_detect_ssh(
                        ip_addr,
                        ssh_port,
                        ssh_timeout - ssh_connect_timeout,
                        ssh_timeout,
                    ));
                }

                let mut found = Vec::new();

                while let Some(result) = tasks.join_next().await {
                    debug!("Found OS detection partial result: {result:?}");
                    found.push(result??);
                }

                let os = aggregate_os_results(&found).ok_or(OsDetectionError::Ambiguous(found))?;

                let _ = tx
                    .send(OsDetectionResult {
                        address: ip_addr,
                        os,
                    })
                    .await;
                Ok(())
            },
        )
        .await?;
    Ok(())
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
    let fingerprint = &fingerprint.to_string();
    for known in FINGERPRINT_DB.iter() {
        if known.matches(fingerprint) {
            return Ok(known
                .os
                .clone()
                .with_hints(HashSet::from([
                    "TCP fingerprint: ".to_owned() + &*fingerprint.to_string()
                ])));
        }
    }

    Ok(OperatingSystemInfo::unknown(Some(
        "TCP fingerprint: ".to_owned() + &*fingerprint.to_string(),
    )))
}

/// Opens a TLS connection on ip_addr with port 22 and reads out the SSH banner.
async fn os_detect_ssh(
    ip_addr: IpAddr,
    port: u16,
    recv_timeout: Duration,
    total_timeout: Duration,
) -> Result<OperatingSystemInfo, OsDetectionError> {
    let settings = OneShotTcpSettings {
        socket: SocketAddr::new(ip_addr, port),
        recv_timeout,
        connect_timeout: total_timeout - recv_timeout,
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

    let ProbeTcpResult::Ok(data) = result else {
        // TOOD: might want to return differently if the error is a specific one, but right now it's a dynamic error
        // without proper error information for us to match on.
        return Ok(OperatingSystemInfo::default());
    };

    if data.starts_with(b"SSH-") {
        if let Some(end) = data.iter().find_position(|&&c| c == b'\r' || c == b'\n') {
            return Ok(os_detect_ssh_header(&data[0..end.0]));
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
