//! Settings for the finding factory

use rorm::prelude::ForeignModel;
use rorm::Model;
use uuid::Uuid;

use crate::models::FindingDefinition;

/// Settings mapping an identifier to a finding definition
///
/// An identifier is an enum variant which identifies one kind of issue,
/// the finding factory might create a finding for.
///
/// If the finding factory detects an issue it will look up its identifier's finding definition
/// and create a finding using this definition (if it found any).
#[derive(Model)]
pub struct FindingFactoryEntry {
    /// A primary key
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// Identifies the issue a finding could be created for.
    ///
    /// This `String` should only be used in conversion from and to
    /// [`FindingFactoryIdentifier`](crate::modules::finding_factory::schema::FindingFactoryIdentifier).
    /// Invalid strings should be ignored and logged.
    #[rorm(max_length = 255, unique)]
    pub identifier: String,

    /// The finding definition to create a finding with, if the identifier's associated issue is found.
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub finding: ForeignModel<FindingDefinition>,
}
