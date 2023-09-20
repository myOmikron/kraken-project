//! This module holds all model definitions for the database
pub use aggregation::*;
pub use attack::*;
pub use leech::*;
pub use oauth::*;
pub use settings::*;
pub use tag::*;
pub use user::*;
pub use workspace::*;

mod aggregation;
mod attack;
mod leech;
mod oauth;
mod settings;
mod tag;
mod user;
mod workspace;
