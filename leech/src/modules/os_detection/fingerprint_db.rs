use std::str::FromStr;

use once_cell::sync::Lazy;

use crate::modules::os_detection::LinuxDistro;
use crate::modules::os_detection::OperatingSystemInfo;
use crate::modules::os_detection::WindowsVersion;

const FINGERPRINT_DB_RAW: &str = include_str!("../../../res/tcp_fingerprints_db.txt");

// can't run parse_fingerprint_db at compile time, would require an extra build step to build ahead
// of time and require rust source code generation, which is error-prone. So we just do this once at
// program startup instead.
pub static FINGERPRINT_DB: Lazy<Vec<FingerprintPattern>> =
    Lazy::new(|| parse_fingerprint_db(FINGERPRINT_DB_RAW));

pub struct FingerprintPattern {
    /// The fingerprint pattern to match
    pub pattern: String,
    ///
    pub os: OperatingSystemInfo,
}

impl FingerprintPattern {
    pub fn matches(&self, fingerprint: &str) -> bool {
        for (expected, actual) in self.pattern.split(':').zip(fingerprint.split(':')) {
            if expected != "*" && expected != actual {
                return false;
            }
        }
        true
    }
}

fn parse_fingerprint_db(db: &str) -> Vec<FingerprintPattern> {
    return db
        .lines()
        .filter(|&s| !s.trim().is_empty() && !s.trim().starts_with('#'))
        .map(|s| s.splitn(3, '|'))
        .map(|mut p| {
            (
                p.next().expect("missing fingerprint").trim(),
                p.next().expect("missing OS").trim(),
                match p.next() {
                    None => None,
                    Some(v) => {
                        if v.trim().is_empty() {
                            None
                        } else {
                            Some(String::from(v.trim()))
                        }
                    }
                },
            )
        })
        .map(|(fingerprint, os, extra)| FingerprintPattern {
            pattern: String::from(fingerprint),
            os: match os {
                "Linux" => OperatingSystemInfo::linux(
                    extra.map(|distro| {
                        (
                            LinuxDistro::from_str(&distro)
                                .unwrap_or_else(|_| panic!("invalid linux distro '{}'", distro)),
                            None,
                        )
                    }),
                    None,
                    None,
                ),
                "BSD" => OperatingSystemInfo::bsd(extra, None),
                "Android" => OperatingSystemInfo::android(extra, None),
                "OSX" => OperatingSystemInfo::osx(extra, None),
                "IOS" => OperatingSystemInfo::ios(extra, None),
                "Windows" => OperatingSystemInfo::windows(
                    extra.map(|v| {
                        (
                            WindowsVersion::from_str(&v)
                                .unwrap_or_else(|_| panic!("invalid windows version '{}'", v)),
                            None,
                        )
                    }),
                    None,
                ),
                _ => panic!("unknown OS type: {}", os),
            },
        })
        .collect();
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_valid_fingerprint_db() {
        // evaluate Lazy, will give us errors if there is anything wrong with the fingerprint here.
        assert!(FINGERPRINT_DB.len() > 0);
    }
}
