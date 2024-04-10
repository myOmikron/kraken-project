use crate::api::handler::findings::schema::FindingSeverity;
use crate::models::convert::FromDb;
use crate::models::convert::IntoDb;

impl FromDb for FindingSeverity {
    type DbFormat = super::FindingSeverity;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::FindingSeverity::Okay => FindingSeverity::Okay,
            super::FindingSeverity::Low => FindingSeverity::Low,
            super::FindingSeverity::Medium => FindingSeverity::Medium,
            super::FindingSeverity::High => FindingSeverity::High,
            super::FindingSeverity::Critical => FindingSeverity::Critical,
        }
    }
}
impl IntoDb for FindingSeverity {
    fn into_db(self) -> Self::DbFormat {
        match self {
            FindingSeverity::Okay => super::FindingSeverity::Okay,
            FindingSeverity::Low => super::FindingSeverity::Low,
            FindingSeverity::Medium => super::FindingSeverity::Medium,
            FindingSeverity::High => super::FindingSeverity::High,
            FindingSeverity::Critical => super::FindingSeverity::Critical,
        }
    }
}
