use crate::api::handler::domains::schema::DomainCertainty;
use crate::api::handler::hosts::schema::HostCertainty;
use crate::api::handler::hosts::schema::OsType;
use crate::api::handler::http_services::schema::HttpServiceCertainty;
use crate::api::handler::ports::schema::PortCertainty;
use crate::api::handler::ports::schema::PortProtocol;
use crate::api::handler::services::schema::ServiceCertainty;
use crate::models::convert::FromDb;
use crate::models::convert::IntoDb;

impl FromDb for DomainCertainty {
    type DbFormat = super::DomainCertainty;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::DomainCertainty::Unverified => DomainCertainty::Unverified,
            super::DomainCertainty::Verified => DomainCertainty::Verified,
        }
    }
}
impl IntoDb for DomainCertainty {
    fn into_db(self) -> Self::DbFormat {
        match self {
            DomainCertainty::Unverified => super::DomainCertainty::Unverified,
            DomainCertainty::Verified => super::DomainCertainty::Verified,
        }
    }
}

impl FromDb for HostCertainty {
    type DbFormat = super::HostCertainty;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::HostCertainty::Historical => HostCertainty::Historical,
            super::HostCertainty::SupposedTo => HostCertainty::SupposedTo,
            super::HostCertainty::Verified => HostCertainty::Verified,
        }
    }
}
impl IntoDb for HostCertainty {
    fn into_db(self) -> Self::DbFormat {
        match self {
            HostCertainty::Historical => super::HostCertainty::Historical,
            HostCertainty::SupposedTo => super::HostCertainty::SupposedTo,
            HostCertainty::Verified => super::HostCertainty::Verified,
        }
    }
}

impl FromDb for OsType {
    type DbFormat = super::OsType;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::OsType::Unknown => OsType::Unknown,
            super::OsType::Linux => OsType::Linux,
            super::OsType::Windows => OsType::Windows,
            super::OsType::Apple => OsType::Apple,
            super::OsType::Android => OsType::Android,
            super::OsType::FreeBSD => OsType::FreeBSD,
        }
    }
}
impl IntoDb for OsType {
    fn into_db(self) -> Self::DbFormat {
        match self {
            OsType::Unknown => super::OsType::Unknown,
            OsType::Linux => super::OsType::Linux,
            OsType::Windows => super::OsType::Windows,
            OsType::Apple => super::OsType::Apple,
            OsType::Android => super::OsType::Android,
            OsType::FreeBSD => super::OsType::FreeBSD,
        }
    }
}

impl FromDb for PortCertainty {
    type DbFormat = super::PortCertainty;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::PortCertainty::Historical => PortCertainty::Historical,
            super::PortCertainty::SupposedTo => PortCertainty::SupposedTo,
            super::PortCertainty::Verified => PortCertainty::Verified,
        }
    }
}
impl IntoDb for PortCertainty {
    fn into_db(self) -> Self::DbFormat {
        match self {
            PortCertainty::Historical => super::PortCertainty::Historical,
            PortCertainty::SupposedTo => super::PortCertainty::SupposedTo,
            PortCertainty::Verified => super::PortCertainty::Verified,
        }
    }
}

impl FromDb for PortProtocol {
    type DbFormat = super::PortProtocol;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::PortProtocol::Unknown => PortProtocol::Unknown,
            super::PortProtocol::Tcp => PortProtocol::Tcp,
            super::PortProtocol::Udp => PortProtocol::Udp,
            super::PortProtocol::Sctp => PortProtocol::Sctp,
        }
    }
}
impl IntoDb for PortProtocol {
    fn into_db(self) -> Self::DbFormat {
        match self {
            PortProtocol::Unknown => super::PortProtocol::Unknown,
            PortProtocol::Tcp => super::PortProtocol::Tcp,
            PortProtocol::Udp => super::PortProtocol::Udp,
            PortProtocol::Sctp => super::PortProtocol::Sctp,
        }
    }
}

impl FromDb for ServiceCertainty {
    type DbFormat = super::ServiceCertainty;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::ServiceCertainty::Historical => ServiceCertainty::Historical,
            super::ServiceCertainty::SupposedTo => ServiceCertainty::SupposedTo,
            super::ServiceCertainty::MaybeVerified => ServiceCertainty::MaybeVerified,
            super::ServiceCertainty::DefinitelyVerified => ServiceCertainty::DefinitelyVerified,
            super::ServiceCertainty::UnknownService => ServiceCertainty::UnknownService,
        }
    }
}
impl IntoDb for ServiceCertainty {
    fn into_db(self) -> Self::DbFormat {
        match self {
            ServiceCertainty::Historical => super::ServiceCertainty::Historical,
            ServiceCertainty::SupposedTo => super::ServiceCertainty::SupposedTo,
            ServiceCertainty::MaybeVerified => super::ServiceCertainty::MaybeVerified,
            ServiceCertainty::DefinitelyVerified => super::ServiceCertainty::DefinitelyVerified,
            ServiceCertainty::UnknownService => super::ServiceCertainty::UnknownService,
        }
    }
}

impl FromDb for HttpServiceCertainty {
    type DbFormat = super::HttpServiceCertainty;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::HttpServiceCertainty::Historical => HttpServiceCertainty::Historical,
            super::HttpServiceCertainty::SupposedTo => HttpServiceCertainty::SupposedTo,
            super::HttpServiceCertainty::Verified => HttpServiceCertainty::Verified,
        }
    }
}
impl IntoDb for HttpServiceCertainty {
    fn into_db(self) -> Self::DbFormat {
        match self {
            HttpServiceCertainty::Historical => super::HttpServiceCertainty::Historical,
            HttpServiceCertainty::SupposedTo => super::HttpServiceCertainty::SupposedTo,
            HttpServiceCertainty::Verified => super::HttpServiceCertainty::Verified,
        }
    }
}
