//! OS detection for a host, based on an ever-growing collection of probes

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use futures::stream::{self, TryStreamExt};
use futures::StreamExt;

use crate::modules::os_detection::errors::OsDetectionError;
use crate::modules::service_detection::DetectServiceSettings;

pub mod errors;
pub mod tcp_fingerprint;

/// Contains how many probably OS-specific services have been matched.
/// If the confidence of a matching probe is high, it adds 10 to its corresponding OS field, if it's low, it adds 1.
/// This is aggregated from possibly many matching probes.
///
/// From a single probe multiple matches can be set as well, for example for POSIX specified things that are observed on
/// multiple operating systems.
#[derive(Default, Copy, Clone, Debug)]
pub struct OperatingSystemInfo {
    /// Linux-specific probe matches.
    pub linux_matches: u16,
    /// Linux Debian-specific probe matches.
    pub linux_debian_matches: u16,
    /// Linux Ubuntu-specific probe matches.
    pub linux_ubuntu_matches: u16,
    /// ArchLinux-specific probe matches.
    pub linux_arch_matches: u16,
    /// Linux RHEL-specific probe matches.
    pub linux_rhel_matches: u16,
    /// Linux openSUSE-specific probe matches.
    pub linux_suse_matches: u16,
    /// Apple-specific probe matches.
    pub apple_matches: u16,
    /// FreeBSD-specific probe matches.
    pub free_bsd_matches: u16,
    /// Android-specific probe matches.
    pub android_matches: u16,
    /// Microsoft® ⊞™ Windows®-specific probe matches.
    pub windows_matches: u16,
}

impl OperatingSystemInfo {
    fn linux(confidence: u16) -> OperatingSystemInfo {
        OperatingSystemInfo {
            linux_matches: confidence,
            linux_debian_matches: confidence,
            linux_ubuntu_matches: confidence,
            linux_arch_matches: confidence,
            linux_rhel_matches: confidence,
            linux_suse_matches: confidence,
            ..OperatingSystemInfo::default()
        }
    }

    fn linux_debian(confidence: u16) -> OperatingSystemInfo {
        OperatingSystemInfo {
            linux_matches: confidence,
            linux_debian_matches: confidence,
            ..OperatingSystemInfo::default()
        }
    }

    fn linux_ubuntu(confidence: u16) -> OperatingSystemInfo {
        OperatingSystemInfo {
            linux_matches: confidence,
            linux_ubuntu_matches: confidence,
            ..OperatingSystemInfo::default()
        }
    }

    fn linux_arch(confidence: u16) -> OperatingSystemInfo {
        OperatingSystemInfo {
            linux_matches: confidence,
            linux_arch_matches: confidence,
            ..OperatingSystemInfo::default()
        }
    }

    fn linux_rhel(confidence: u16) -> OperatingSystemInfo {
        OperatingSystemInfo {
            linux_matches: confidence,
            linux_rhel_matches: confidence,
            ..OperatingSystemInfo::default()
        }
    }

    fn linux_suse(confidence: u16) -> OperatingSystemInfo {
        OperatingSystemInfo {
            linux_matches: confidence,
            linux_suse_matches: confidence,
            ..OperatingSystemInfo::default()
        }
    }

    fn apple(confidence: u16) -> OperatingSystemInfo {
        OperatingSystemInfo {
            apple_matches: confidence,
            ..OperatingSystemInfo::default()
        }
    }

    fn free_bsd(confidence: u16) -> OperatingSystemInfo {
        OperatingSystemInfo {
            free_bsd_matches: confidence,
            ..OperatingSystemInfo::default()
        }
    }

    fn windows(confidence: u16) -> OperatingSystemInfo {
        OperatingSystemInfo {
            windows_matches: confidence,
            ..OperatingSystemInfo::default()
        }
    }

    fn android(confidence: u16) -> OperatingSystemInfo {
        OperatingSystemInfo {
            android_matches: confidence,
            ..OperatingSystemInfo::default()
        }
    }
}

pub async fn os_detection(ip_addr: IpAddr) -> Result<OperatingSystemInfo, OsDetectionError> {
    return stream::iter(vec![os_detect_ssh(ip_addr)])
        .buffer_unordered(8)
        .try_fold(OperatingSystemInfo::default(), |a, b| async move {
            Ok(OperatingSystemInfo {
                linux_matches: a.linux_matches + b.linux_matches,
                linux_debian_matches: a.linux_debian_matches + b.linux_debian_matches,
                linux_ubuntu_matches: a.linux_ubuntu_matches + b.linux_ubuntu_matches,
                linux_arch_matches: a.linux_arch_matches + b.linux_arch_matches,
                linux_rhel_matches: a.linux_rhel_matches + b.linux_rhel_matches,
                linux_suse_matches: a.linux_suse_matches + b.linux_suse_matches,
                apple_matches: a.apple_matches + b.apple_matches,
                free_bsd_matches: a.free_bsd_matches + b.free_bsd_matches,
                windows_matches: a.windows_matches + b.windows_matches,
                android_matches: a.android_matches + b.android_matches,
            })
        })
        .await;
}

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
    return OperatingSystemInfo::default();
}
