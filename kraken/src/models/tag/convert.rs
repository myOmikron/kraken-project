use crate::api::handler::common::schema::Color;
use crate::models::convert::FromDb;
use crate::models::convert::IntoDb;

impl FromDb for Color {
    type DbFormat = i32;

    fn from_db(db_format: Self::DbFormat) -> Self {
        let [r, g, b, a] = db_format.to_le_bytes();
        Self { r, g, b, a }
    }
}
impl IntoDb for Color {
    fn into_db(self) -> Self::DbFormat {
        let Self { r, g, b, a } = self;
        i32::from_le_bytes([r, g, b, a])
    }
}
