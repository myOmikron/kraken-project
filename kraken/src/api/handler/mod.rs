//! All handler for the frontend API are defined here

pub mod aggregation_source;
pub mod api_keys;
pub mod attack_results;
pub mod attacks;
pub mod auth;
pub mod common;
pub mod data_export;
pub mod domains;
pub mod files;
pub mod finding_definitions;
pub mod global_tags;
pub mod hosts;
pub mod leeches;
pub mod oauth;
pub mod oauth_applications;
pub mod oauth_decisions;
pub mod ports;
pub mod services;
pub mod settings;
pub mod users;
#[cfg(feature = "bin")]
pub(crate) mod websocket;
pub mod wordlists;
pub mod workspace_invitations;
pub mod workspace_tags;
pub mod workspaces;
