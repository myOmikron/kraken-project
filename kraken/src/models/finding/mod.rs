use chrono::DateTime;
use chrono::Utc;
use rorm::field;
use rorm::prelude::BackRef;
use rorm::prelude::ForeignModel;
use rorm::DbEnum;
use rorm::Model;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[cfg(feature = "bin")]
pub(crate) use crate::models::finding::patches::InsertFindingDefinition;
use crate::models::Domain;
use crate::models::Host;
use crate::models::Port;
use crate::models::Service;

#[cfg(feature = "bin")]
mod patches;

/// The severity of a finding
#[rustfmt::skip]
#[derive(Copy, Clone, Debug, DbEnum, Deserialize, Serialize, ToSchema, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum FindingSeverity {
    /// Severity was set to okay
    Okay,
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// The model represents a finding entry in the knowledge base
#[derive(Model, Clone)]
pub struct FindingDefinition {
    /// The primary key of the finding
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The name of the finding
    #[rorm(unique, max_length = 255)]
    pub name: String,

    /// The summary of the finding
    #[rorm(max_length = 1024)]
    pub summary: String,

    /// The severity of the finding
    pub severity: FindingSeverity,

    /// The CVE Identifier for this finding
    #[rorm(max_length = 255)]
    pub cve: Option<String>,

    /// The description of the finding
    #[rorm(max_length = 65535)]
    pub description: String,

    /// The impact of the finding
    #[rorm(max_length = 65535)]
    pub impact: String,

    /// The remediation of the finding
    #[rorm(max_length = 65535)]
    pub remediation: String,

    /// The references of the finding
    #[rorm(max_length = 65535)]
    pub references: String,

    /// The point in time this finding was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// The instance of a finding
#[derive(Model)]
pub struct Finding {
    /// The primary key of the finding
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The relation to the definition of the finding
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub definition: ForeignModel<FindingDefinition>,

    /// The severity of this specific instance of the finding
    pub severity: FindingSeverity,

    /// The affected aggregations for this finding
    pub affected: BackRef<field!(FindingAffected::F.finding)>,

    /// The relation to details for this finding
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub details: ForeignModel<FindingDetails>,

    /// The point in time this finding was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// The affected aggregations of the [Finding]
///
/// At least one of `domain`, `host`, `port` or `service` must be set
#[derive(Model)]
pub struct FindingAffected {
    /// The primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The relation to the finding
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub finding: ForeignModel<Finding>,

    /// Related aggregation
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub domain: Option<ForeignModel<Domain>>,
    /// Related aggregation
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub host: Option<ForeignModel<Host>>,
    /// Related aggregation
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub port: Option<ForeignModel<Port>>,
    /// Related aggregation
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub service: Option<ForeignModel<Service>>,

    /// The details of this affected finding
    #[rorm(on_update = "Cascade", on_delete = "SetNull")]
    pub details: Option<ForeignModel<FindingDetails>>,

    /// The point in time this model was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// The details that can be attached to a [Finding] or to a [FindingAffected].
#[derive(Model)]
pub struct FindingDetails {
    /// The primary key of the finding details
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// Notes about the finding provided by the user
    ///
    /// May be used for documenting command invocation or other information
    /// that are provided by the user
    #[rorm(max_length = 65535)]
    pub user_details: Option<String>,

    /// Details of the finding that comes from the attack module
    ///
    /// This field should only be read-only for the user
    #[rorm(max_length = 65535)]
    pub tool_details: Option<String>,

    /// The path to the file relative to the media directory from the config
    #[rorm(max_length = 1024)]
    pub screenshot: Option<String>,

    /// The path to the file relative to the media directory from the config
    #[rorm(max_length = 1024)]
    pub log_file: Option<String>,
}
