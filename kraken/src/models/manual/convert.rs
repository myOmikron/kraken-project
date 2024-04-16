use crate::api::handler::hosts::schema::ManualHostCertainty;
use crate::api::handler::http_services::schema::ManualHttpServiceCertainty;
use crate::api::handler::ports::schema::ManualPortCertainty;
use crate::api::handler::services::schema::ManualServiceCertainty;
use crate::models::convert::FromDb;
use crate::models::convert::IntoDb;

impl FromDb for ManualHostCertainty {
    type DbFormat = super::ManualHostCertainty;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::ManualHostCertainty::Historical => ManualHostCertainty::Historical,
            super::ManualHostCertainty::SupposedTo => ManualHostCertainty::SupposedTo,
        }
    }
}
impl IntoDb for ManualHostCertainty {
    fn into_db(self) -> Self::DbFormat {
        match self {
            ManualHostCertainty::Historical => super::ManualHostCertainty::Historical,
            ManualHostCertainty::SupposedTo => super::ManualHostCertainty::SupposedTo,
        }
    }
}

impl FromDb for ManualPortCertainty {
    type DbFormat = super::ManualPortCertainty;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::ManualPortCertainty::Historical => ManualPortCertainty::Historical,
            super::ManualPortCertainty::SupposedTo => ManualPortCertainty::SupposedTo,
        }
    }
}
impl IntoDb for ManualPortCertainty {
    fn into_db(self) -> Self::DbFormat {
        match self {
            ManualPortCertainty::Historical => super::ManualPortCertainty::Historical,
            ManualPortCertainty::SupposedTo => super::ManualPortCertainty::SupposedTo,
        }
    }
}

impl FromDb for ManualServiceCertainty {
    type DbFormat = super::ManualServiceCertainty;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::ManualServiceCertainty::Historical => ManualServiceCertainty::Historical,
            super::ManualServiceCertainty::SupposedTo => ManualServiceCertainty::SupposedTo,
        }
    }
}
impl IntoDb for ManualServiceCertainty {
    fn into_db(self) -> Self::DbFormat {
        match self {
            ManualServiceCertainty::Historical => super::ManualServiceCertainty::Historical,
            ManualServiceCertainty::SupposedTo => super::ManualServiceCertainty::SupposedTo,
        }
    }
}

impl FromDb for ManualHttpServiceCertainty {
    type DbFormat = super::ManualHttpServiceCertainty;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::ManualHttpServiceCertainty::Historical => ManualHttpServiceCertainty::Historical,
            super::ManualHttpServiceCertainty::SupposedTo => ManualHttpServiceCertainty::SupposedTo,
        }
    }
}
impl IntoDb for ManualHttpServiceCertainty {
    fn into_db(self) -> Self::DbFormat {
        match self {
            ManualHttpServiceCertainty::Historical => super::ManualHttpServiceCertainty::Historical,
            ManualHttpServiceCertainty::SupposedTo => super::ManualHttpServiceCertainty::SupposedTo,
        }
    }
}
