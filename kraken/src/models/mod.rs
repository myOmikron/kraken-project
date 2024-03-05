//! This module holds all model definitions for the database
pub use aggregation::*;
pub use attack::*;
pub use file::*;
pub use finding::*;
pub use leech::*;
pub use manual::*;
pub use oauth::*;
pub use search::*;
pub use settings::*;
pub use tag::*;
pub use user::*;
pub use wordlist::*;
pub use workspace::*;

mod aggregation;
mod attack;
mod file;
mod finding;
mod leech;
mod manual;
mod oauth;
mod search;
mod settings;
mod tag;
mod user;
mod wordlist;
mod workspace;
