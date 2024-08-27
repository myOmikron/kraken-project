use crate::api::handler::attack_results::schema::DnsRecordType;
use crate::api::handler::attack_results::schema::DnsTxtScanServiceHintType;
use crate::api::handler::attack_results::schema::DnsTxtScanSpfType;
use crate::api::handler::attack_results::schema::DnsTxtScanSummaryType;
use crate::api::handler::attack_results::schema::TestSSLSection;
use crate::api::handler::attack_results::schema::TestSSLSeverity;
use crate::api::handler::attacks::schema::AttackType;
use crate::models::convert::FromDb;
use crate::models::convert::IntoDb;

impl FromDb for AttackType {
    type DbFormat = super::AttackType;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::AttackType::Undefined => AttackType::Undefined,
            super::AttackType::BruteforceSubdomains => AttackType::BruteforceSubdomains,
            super::AttackType::TcpPortScan => AttackType::Undefined,
            super::AttackType::QueryCertificateTransparency => {
                AttackType::QueryCertificateTransparency
            }
            super::AttackType::QueryUnhashed => AttackType::QueryUnhashed,
            super::AttackType::HostAlive => AttackType::HostAlive,
            super::AttackType::ServiceDetection => AttackType::ServiceDetection,
            super::AttackType::UdpServiceDetection => AttackType::UdpServiceDetection,
            super::AttackType::DnsResolution => AttackType::DnsResolution,
            super::AttackType::DnsTxtScan => AttackType::DnsTxtScan,
            super::AttackType::UdpPortScan => AttackType::UdpPortScan,
            super::AttackType::ForcedBrowsing => AttackType::ForcedBrowsing,
            super::AttackType::OSDetection => AttackType::OSDetection,
            super::AttackType::VersionDetection => AttackType::VersionDetection,
            super::AttackType::AntiPortScanningDetection => AttackType::AntiPortScanningDetection,
            super::AttackType::TestSSL => AttackType::TestSSL,
        }
    }
}
impl IntoDb for AttackType {
    fn into_db(self) -> Self::DbFormat {
        match self {
            AttackType::Undefined => super::AttackType::Undefined,
            AttackType::BruteforceSubdomains => super::AttackType::BruteforceSubdomains,
            AttackType::QueryCertificateTransparency => {
                super::AttackType::QueryCertificateTransparency
            }
            AttackType::QueryUnhashed => super::AttackType::QueryUnhashed,
            AttackType::HostAlive => super::AttackType::HostAlive,
            AttackType::ServiceDetection => super::AttackType::ServiceDetection,
            AttackType::UdpServiceDetection => super::AttackType::UdpServiceDetection,
            AttackType::DnsResolution => super::AttackType::DnsResolution,
            AttackType::DnsTxtScan => super::AttackType::DnsTxtScan,
            AttackType::UdpPortScan => super::AttackType::UdpPortScan,
            AttackType::ForcedBrowsing => super::AttackType::ForcedBrowsing,
            AttackType::OSDetection => super::AttackType::OSDetection,
            AttackType::VersionDetection => super::AttackType::VersionDetection,
            AttackType::AntiPortScanningDetection => super::AttackType::AntiPortScanningDetection,
            AttackType::TestSSL => super::AttackType::TestSSL,
        }
    }
}

impl FromDb for DnsRecordType {
    type DbFormat = super::DnsRecordType;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::DnsRecordType::A => DnsRecordType::A,
            super::DnsRecordType::Aaaa => DnsRecordType::Aaaa,
            super::DnsRecordType::Caa => DnsRecordType::Caa,
            super::DnsRecordType::Cname => DnsRecordType::Cname,
            super::DnsRecordType::Mx => DnsRecordType::Mx,
            super::DnsRecordType::Tlsa => DnsRecordType::Tlsa,
            super::DnsRecordType::Txt => DnsRecordType::Txt,
        }
    }
}
impl IntoDb for DnsRecordType {
    fn into_db(self) -> Self::DbFormat {
        match self {
            DnsRecordType::A => super::DnsRecordType::A,
            DnsRecordType::Aaaa => super::DnsRecordType::Aaaa,
            DnsRecordType::Caa => super::DnsRecordType::Caa,
            DnsRecordType::Cname => super::DnsRecordType::Cname,
            DnsRecordType::Mx => super::DnsRecordType::Mx,
            DnsRecordType::Tlsa => super::DnsRecordType::Tlsa,
            DnsRecordType::Txt => super::DnsRecordType::Txt,
        }
    }
}

impl FromDb for DnsTxtScanSummaryType {
    type DbFormat = super::DnsTxtScanSummaryType;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::DnsTxtScanSummaryType::ServiceHints => DnsTxtScanSummaryType::ServiceHints,
            super::DnsTxtScanSummaryType::Spf => DnsTxtScanSummaryType::Spf,
        }
    }
}
impl IntoDb for DnsTxtScanSummaryType {
    fn into_db(self) -> Self::DbFormat {
        match self {
            DnsTxtScanSummaryType::ServiceHints => super::DnsTxtScanSummaryType::ServiceHints,
            DnsTxtScanSummaryType::Spf => super::DnsTxtScanSummaryType::Spf,
        }
    }
}

impl FromDb for DnsTxtScanServiceHintType {
    type DbFormat = super::DnsTxtScanServiceHintType;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::DnsTxtScanServiceHintType::HasGoogleAccount => {
                DnsTxtScanServiceHintType::HasGoogleAccount
            }
            super::DnsTxtScanServiceHintType::HasGlobalsignAccount => {
                DnsTxtScanServiceHintType::HasGlobalsignAccount
            }
            super::DnsTxtScanServiceHintType::HasGlobalsignSMime => {
                DnsTxtScanServiceHintType::HasGlobalsignSMime
            }
            super::DnsTxtScanServiceHintType::HasDocusignAccount => {
                DnsTxtScanServiceHintType::HasDocusignAccount
            }
            super::DnsTxtScanServiceHintType::HasAppleAccount => {
                DnsTxtScanServiceHintType::HasAppleAccount
            }
            super::DnsTxtScanServiceHintType::HasFacebookAccount => {
                DnsTxtScanServiceHintType::HasFacebookAccount
            }
            super::DnsTxtScanServiceHintType::HasHubspotAccount => {
                DnsTxtScanServiceHintType::HasHubspotAccount
            }
            super::DnsTxtScanServiceHintType::HasMSDynamics365 => {
                DnsTxtScanServiceHintType::HasMSDynamics365
            }
            super::DnsTxtScanServiceHintType::HasStripeAccount => {
                DnsTxtScanServiceHintType::HasStripeAccount
            }
            super::DnsTxtScanServiceHintType::HasOneTrustSso => {
                DnsTxtScanServiceHintType::HasOneTrustSso
            }
            super::DnsTxtScanServiceHintType::HasBrevoAccount => {
                DnsTxtScanServiceHintType::HasBrevoAccount
            }
            super::DnsTxtScanServiceHintType::OwnsAtlassianAccounts => {
                DnsTxtScanServiceHintType::OwnsAtlassianAccounts
            }
            super::DnsTxtScanServiceHintType::OwnsZoomAccounts => {
                DnsTxtScanServiceHintType::OwnsZoomAccounts
            }
            super::DnsTxtScanServiceHintType::EmailProtonMail => {
                DnsTxtScanServiceHintType::EmailProtonMail
            }
        }
    }
}
impl IntoDb for DnsTxtScanServiceHintType {
    fn into_db(self) -> Self::DbFormat {
        match self {
            DnsTxtScanServiceHintType::HasGoogleAccount => {
                super::DnsTxtScanServiceHintType::HasGoogleAccount
            }
            DnsTxtScanServiceHintType::HasGlobalsignAccount => {
                super::DnsTxtScanServiceHintType::HasGlobalsignAccount
            }
            DnsTxtScanServiceHintType::HasGlobalsignSMime => {
                super::DnsTxtScanServiceHintType::HasGlobalsignSMime
            }
            DnsTxtScanServiceHintType::HasDocusignAccount => {
                super::DnsTxtScanServiceHintType::HasDocusignAccount
            }
            DnsTxtScanServiceHintType::HasAppleAccount => {
                super::DnsTxtScanServiceHintType::HasAppleAccount
            }
            DnsTxtScanServiceHintType::HasFacebookAccount => {
                super::DnsTxtScanServiceHintType::HasFacebookAccount
            }
            DnsTxtScanServiceHintType::HasHubspotAccount => {
                super::DnsTxtScanServiceHintType::HasHubspotAccount
            }
            DnsTxtScanServiceHintType::HasMSDynamics365 => {
                super::DnsTxtScanServiceHintType::HasMSDynamics365
            }
            DnsTxtScanServiceHintType::HasStripeAccount => {
                super::DnsTxtScanServiceHintType::HasStripeAccount
            }
            DnsTxtScanServiceHintType::HasOneTrustSso => {
                super::DnsTxtScanServiceHintType::HasOneTrustSso
            }
            DnsTxtScanServiceHintType::HasBrevoAccount => {
                super::DnsTxtScanServiceHintType::HasBrevoAccount
            }
            DnsTxtScanServiceHintType::OwnsAtlassianAccounts => {
                super::DnsTxtScanServiceHintType::OwnsAtlassianAccounts
            }
            DnsTxtScanServiceHintType::OwnsZoomAccounts => {
                super::DnsTxtScanServiceHintType::OwnsZoomAccounts
            }
            DnsTxtScanServiceHintType::EmailProtonMail => {
                super::DnsTxtScanServiceHintType::EmailProtonMail
            }
        }
    }
}

impl FromDb for DnsTxtScanSpfType {
    type DbFormat = super::DnsTxtScanSpfType;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::DnsTxtScanSpfType::All => DnsTxtScanSpfType::All,
            super::DnsTxtScanSpfType::Include => DnsTxtScanSpfType::Include,
            super::DnsTxtScanSpfType::A => DnsTxtScanSpfType::A,
            super::DnsTxtScanSpfType::Mx => DnsTxtScanSpfType::Mx,
            super::DnsTxtScanSpfType::Ptr => DnsTxtScanSpfType::Ptr,
            super::DnsTxtScanSpfType::Ip => DnsTxtScanSpfType::Ip,
            super::DnsTxtScanSpfType::Exists => DnsTxtScanSpfType::Exists,
            super::DnsTxtScanSpfType::Redirect => DnsTxtScanSpfType::Redirect,
            super::DnsTxtScanSpfType::Explanation => DnsTxtScanSpfType::Explanation,
            super::DnsTxtScanSpfType::Modifier => DnsTxtScanSpfType::Modifier,
        }
    }
}
impl IntoDb for DnsTxtScanSpfType {
    fn into_db(self) -> Self::DbFormat {
        match self {
            DnsTxtScanSpfType::All => super::DnsTxtScanSpfType::All,
            DnsTxtScanSpfType::Include => super::DnsTxtScanSpfType::Include,
            DnsTxtScanSpfType::A => super::DnsTxtScanSpfType::A,
            DnsTxtScanSpfType::Mx => super::DnsTxtScanSpfType::Mx,
            DnsTxtScanSpfType::Ptr => super::DnsTxtScanSpfType::Ptr,
            DnsTxtScanSpfType::Ip => super::DnsTxtScanSpfType::Ip,
            DnsTxtScanSpfType::Exists => super::DnsTxtScanSpfType::Exists,
            DnsTxtScanSpfType::Redirect => super::DnsTxtScanSpfType::Redirect,
            DnsTxtScanSpfType::Explanation => super::DnsTxtScanSpfType::Explanation,
            DnsTxtScanSpfType::Modifier => super::DnsTxtScanSpfType::Modifier,
        }
    }
}

impl FromDb for TestSSLSeverity {
    type DbFormat = super::TestSSLSeverity;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::TestSSLSeverity::Debug => Self::Debug,
            super::TestSSLSeverity::Info => Self::Info,
            super::TestSSLSeverity::Warn => Self::Warn,
            super::TestSSLSeverity::Fatal => Self::Fatal,
            super::TestSSLSeverity::Ok => Self::Ok,
            super::TestSSLSeverity::Low => Self::Low,
            super::TestSSLSeverity::Medium => Self::Medium,
            super::TestSSLSeverity::High => Self::High,
            super::TestSSLSeverity::Critical => Self::Critical,
        }
    }
}
impl IntoDb for TestSSLSeverity {
    fn into_db(self) -> Self::DbFormat {
        match self {
            Self::Debug => super::TestSSLSeverity::Debug,
            Self::Info => super::TestSSLSeverity::Info,
            Self::Warn => super::TestSSLSeverity::Warn,
            Self::Fatal => super::TestSSLSeverity::Fatal,
            Self::Ok => super::TestSSLSeverity::Ok,
            Self::Low => super::TestSSLSeverity::Low,
            Self::Medium => super::TestSSLSeverity::Medium,
            Self::High => super::TestSSLSeverity::High,
            Self::Critical => super::TestSSLSeverity::Critical,
        }
    }
}

impl FromDb for TestSSLSection {
    type DbFormat = super::TestSSLSection;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::TestSSLSection::Pretest => Self::Pretest,
            super::TestSSLSection::Protocols => Self::Protocols,
            super::TestSSLSection::Grease => Self::Grease,
            super::TestSSLSection::Ciphers => Self::Ciphers,
            super::TestSSLSection::Pfs => Self::Pfs,
            super::TestSSLSection::ServerPreferences => Self::ServerPreferences,
            super::TestSSLSection::ServerDefaults => Self::ServerDefaults,
            super::TestSSLSection::HeaderResponse => Self::HeaderResponse,
            super::TestSSLSection::Vulnerabilities => Self::Vulnerabilities,
            super::TestSSLSection::CipherTests => Self::CipherTests,
            super::TestSSLSection::BrowserSimulations => Self::BrowserSimulations,
        }
    }
}

impl IntoDb for TestSSLSection {
    fn into_db(self) -> Self::DbFormat {
        match self {
            Self::Pretest => super::TestSSLSection::Pretest,
            Self::Protocols => super::TestSSLSection::Protocols,
            Self::Grease => super::TestSSLSection::Grease,
            Self::Ciphers => super::TestSSLSection::Ciphers,
            Self::Pfs => super::TestSSLSection::Pfs,
            Self::ServerPreferences => super::TestSSLSection::ServerPreferences,
            Self::ServerDefaults => super::TestSSLSection::ServerDefaults,
            Self::HeaderResponse => super::TestSSLSection::HeaderResponse,
            Self::Vulnerabilities => super::TestSSLSection::Vulnerabilities,
            Self::CipherTests => super::TestSSLSection::CipherTests,
            Self::BrowserSimulations => super::TestSSLSection::BrowserSimulations,
        }
    }
}
