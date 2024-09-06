use std::collections::hash_map;
use std::collections::HashMap;
use std::collections::HashSet;

use futures::TryStreamExt;
use rorm::and;
use rorm::conditions::Condition;
use rorm::conditions::DynamicCollection;
use rorm::db::Executor;
use rorm::insert;
use rorm::prelude::ForeignModelByField;
use rorm::query;
use rorm::Database;
use rorm::FieldAccess;
use rorm::Model;
use uuid::Uuid;

use crate::chan::ws_manager::schema::AggregationType;
use crate::models::Finding;
use crate::models::FindingAffected;
use crate::models::FindingDefinition;
use crate::models::FindingDefinitionCategoryRelation;
use crate::models::FindingDetails;
use crate::models::FindingFactoryEntry;
use crate::models::FindingFindingCategoryRelation;
use crate::models::InsertFinding;
use crate::modules::finding_factory::schema::FindingFactoryIdentifier;

/// The finding factory provides a simple api to create findings in automations.
///
/// It collects what findings should be created with what affected
/// and performs all database operations in [`FindingFactory::process`] consuming itself.
///
/// The finding factory's API uses an enum [`FindingFactoryIdentifier`] instead of [`FindingDefinition`] uuids
/// to allow code to hard code them while being dynamically configurable at runtime.
#[derive(Debug, Default)]
pub struct FindingFactory {
    issues: HashMap<FindingFactoryIdentifier, HashSet<(Uuid, AggregationType)>>,
}

impl FindingFactory {
    /// Constructs a new empty `FindingFactory`
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an aggregated model and its issue to the factory
    pub fn add(&mut self, uuid: Uuid, table: AggregationType, issue: FindingFactoryIdentifier) {
        self.issues.entry(issue).or_default().insert((uuid, table));
    }

    /// Adds a host and its issue to the factory
    ///
    /// This is a convenience method wrapping [`FindingFactory::add`].
    pub fn add_host(&mut self, host: Uuid, issue: FindingFactoryIdentifier) {
        self.add(host, AggregationType::Host, issue);
    }

    /// Adds a service and its issue to the factory
    ///
    /// This is a convenience method wrapping [`FindingFactory::add`].
    pub fn add_service(&mut self, service: Uuid, issue: FindingFactoryIdentifier) {
        self.add(service, AggregationType::Service, issue);
    }

    /// Consumes the factory, processes the collected issues and creates findings
    pub async fn process(
        mut self,
        executor: &Database,
        workspace_uuid: Uuid,
    ) -> Result<(), rorm::Error> {
        let mut guard = executor.ensure_transaction().await?;

        if self.issues.is_empty() {
            return Ok(());
        }

        let mut definitions = query!(
            guard.get_transaction(),
            (
                FindingFactoryEntry::F.identifier,
                FindingFactoryEntry::F.finding,
            )
        )
        .condition(DynamicCollection::or(
            self.issues
                .keys()
                .map(|id| FindingFactoryEntry::F.identifier.equals(id.to_string()))
                .collect(),
        ))
        .stream()
        .try_fold(HashMap::new(), move |mut map, (identifier, finding)| {
            if let Some(finding) = finding {
                if let Ok(identifier) = identifier.parse::<FindingFactoryIdentifier>() {
                    if let Some(affected) = self.issues.remove(&identifier) {
                        match map.entry(*finding.key()) {
                            hash_map::Entry::Vacant(entry) => {
                                entry.insert((affected, Vec::new()));
                            }
                            hash_map::Entry::Occupied(mut entry) => {
                                entry.get_mut().0.extend(affected);
                            }
                        }
                    }
                }
            }

            async move { Ok(map) }
        })
        .await?;

        let mut findings_to_attach = {
            let mut findings_to_attach = HashMap::new();
            let condition = {
                let mut hosts = Vec::new();
                let mut ports = Vec::new();
                let mut services = Vec::new();
                let mut domains = Vec::new();
                let mut http_services = Vec::new();

                for (definition, (affected, _)) in &definitions {
                    for (object_uuid, object_table) in affected {
                        match object_table {
                            AggregationType::Host => hosts.push(and![
                                FindingAffected::F.host.equals(*object_uuid),
                                FindingAffected::F.finding.definition.equals(*definition)
                            ]),
                            AggregationType::Port => ports.push(and![
                                FindingAffected::F.port.equals(*object_uuid),
                                FindingAffected::F.finding.definition.equals(*definition)
                            ]),
                            AggregationType::Service => services.push(and![
                                FindingAffected::F.service.equals(*object_uuid),
                                FindingAffected::F.finding.definition.equals(*definition)
                            ]),
                            AggregationType::Domain => domains.push(and![
                                FindingAffected::F.domain.equals(*object_uuid),
                                FindingAffected::F.finding.definition.equals(*definition)
                            ]),
                            AggregationType::HttpService => http_services.push(and![
                                FindingAffected::F.http_service.equals(*object_uuid),
                                FindingAffected::F.finding.definition.equals(*definition)
                            ]),
                        }
                    }
                }

                let hosts = (!hosts.is_empty()).then(|| DynamicCollection::or(hosts).boxed());
                let ports = (!ports.is_empty()).then(|| DynamicCollection::or(ports).boxed());
                let services =
                    (!services.is_empty()).then(|| DynamicCollection::or(services).boxed());
                let domains = (!domains.is_empty()).then(|| DynamicCollection::or(domains).boxed());
                let http_services = (!http_services.is_empty())
                    .then(|| DynamicCollection::or(http_services).boxed());

                DynamicCollection::or(
                    [hosts, ports, services, domains, http_services]
                        .into_iter()
                        .flatten()
                        .collect(),
                )
            };
            let mut stream = query!(
                guard.get_transaction(),
                (
                    FindingAffected::F.host,
                    FindingAffected::F.port,
                    FindingAffected::F.service,
                    FindingAffected::F.domain,
                    FindingAffected::F.http_service,
                    FindingAffected::F.finding,
                    FindingAffected::F.finding.definition
                )
            )
            .condition(condition)
            .stream();
            while let Some((host, port, service, domain, http_service, finding, definition)) =
                stream.try_next().await?
            {
                if let Some((affected, _)) = definitions.remove(definition.key()) {
                    findings_to_attach.insert(*finding.key(), affected);
                }

                if let Some(affected) = findings_to_attach.get_mut(finding.key()) {
                    let host = host.map(|fm| (*fm.key(), AggregationType::Host));
                    let port = port.map(|fm| (*fm.key(), AggregationType::Port));
                    let service = service.map(|fm| (*fm.key(), AggregationType::Service));
                    let domain = domain.map(|fm| (*fm.key(), AggregationType::Domain));
                    let http_service =
                        http_service.map(|fm| (*fm.key(), AggregationType::HttpService));
                    for (object_uuid, object_table) in [host, port, service, domain, http_service]
                        .into_iter()
                        .flatten()
                    {
                        affected.remove(&(object_uuid, object_table));
                    }
                }
            }
            findings_to_attach
        };

        let findings_to_create = async {
            if definitions.is_empty() {
                return Ok::<_, rorm::Error>(Vec::new());
            }

            let mut stream = query!(
                guard.get_transaction(),
                (
                    FindingDefinitionCategoryRelation::F.definition,
                    FindingDefinitionCategoryRelation::F.category,
                )
            )
            .condition(DynamicCollection::or(
                definitions
                    .keys()
                    .map(|uuid| {
                        FindingDefinitionCategoryRelation::F
                            .definition
                            .equals(*uuid)
                    })
                    .collect(),
            ))
            .stream();
            while let Some((definition, category)) = stream.try_next().await? {
                if let Some(definition) = definitions.get_mut(definition.key()) {
                    definition.1.push(category);
                }
            }
            drop(stream);

            let mut findings_to_create = Vec::new();
            let mut stream = query!(
                guard.get_transaction(),
                (FindingDefinition::F.uuid, FindingDefinition::F.severity,)
            )
            .condition(DynamicCollection::or(
                definitions
                    .keys()
                    .map(|uuid| FindingDefinition::F.uuid.equals(*uuid))
                    .collect(),
            ))
            .stream();
            while let Some((definition, severity)) = stream.try_next().await? {
                if let Some((affected, categories)) = definitions.remove(&definition) {
                    findings_to_create.push((definition, severity, categories, affected));
                }
            }
            Ok(findings_to_create)
        }
        .await?;

        let inserted_details = insert!(guard.get_transaction(), FindingDetails)
            .return_primary_key()
            .bulk(findings_to_create.iter().map(|_| FindingDetails {
                uuid: Uuid::new_v4(),
                export_details: String::new(),
                user_details: String::new(),
                tool_details: Some("Auto generated by kraken™".to_string()),
                screenshot: None,
                log_file: None,
            }))
            .await?;

        let inserted_findings = insert!(guard.get_transaction(), Finding)
            .return_primary_key()
            .bulk(
                findings_to_create
                    .iter()
                    .zip(inserted_details.into_iter())
                    .map(
                        |((definition, severity, _, _), details_uuid)| InsertFinding {
                            uuid: Uuid::new_v4(),
                            definition: ForeignModelByField::Key(*definition),
                            severity: *severity,
                            details: ForeignModelByField::Key(details_uuid),
                            workspace: ForeignModelByField::Key(workspace_uuid),
                        },
                    ),
            )
            .await?;

        insert!(guard.get_transaction(), FindingFindingCategoryRelation)
            .return_nothing()
            .bulk(
                findings_to_create
                    .iter()
                    .zip(inserted_findings.iter())
                    .flat_map(|((_, _, categories, _), finding_uuid)| {
                        categories
                            .iter()
                            .cloned()
                            .map(|category| FindingFindingCategoryRelation {
                                uuid: Uuid::new_v4(),
                                finding: ForeignModelByField::Key(*finding_uuid),
                                category,
                            })
                    }),
            )
            .await?;

        for ((_, _, _, affected), finding_uuid) in findings_to_create
            .into_iter()
            .zip(inserted_findings.into_iter())
        {
            match findings_to_attach.entry(finding_uuid) {
                hash_map::Entry::Vacant(entry) => {
                    entry.insert(affected);
                }
                hash_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().extend(affected);
                }
            }
        }

        FindingAffected::insert_simple_bulk(
            guard.get_transaction(),
            findings_to_attach
                .into_iter()
                .flat_map(|(finding_uuid, affected)| {
                    affected
                        .into_iter()
                        .map(move |(object_uuid, object_table)| {
                            (
                                ForeignModelByField::Key(finding_uuid),
                                object_uuid,
                                object_table,
                                ForeignModelByField::Key(workspace_uuid),
                            )
                        })
                }),
        )
        .await?;

        guard.commit().await
    }
}
