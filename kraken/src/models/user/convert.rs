use crate::api::handler::users::schema::UserPermission;
use crate::models::convert::FromDb;
use crate::models::convert::IntoDb;

impl FromDb for UserPermission {
    type DbFormat = super::UserPermission;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::UserPermission::ReadOnly => UserPermission::ReadOnly,
            super::UserPermission::Default => UserPermission::Default,
            super::UserPermission::Admin => UserPermission::Admin,
        }
    }
}
impl IntoDb for UserPermission {
    fn into_db(self) -> Self::DbFormat {
        match self {
            UserPermission::ReadOnly => super::UserPermission::ReadOnly,
            UserPermission::Default => super::UserPermission::Default,
            UserPermission::Admin => super::UserPermission::Admin,
        }
    }
}
