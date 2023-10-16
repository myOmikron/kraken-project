//! Endpoints for retrieving a specific

use std::collections::HashMap;

use actix_web::get;
use actix_web::web::{Data, Json, Path, Query};
use chrono::{DateTime, Utc};
use futures::{StreamExt, TryStreamExt};
use ipnetwork::IpNetwork;
use log::error;
use rorm::prelude::*;
use rorm::{query, Database};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::extractors::SessionUser;
use crate::api::handler::{
    get_page_params, ApiError, ApiResult, BruteforceSubdomainsResultsPage,
    DnsResolutionResultsPage, HostAliveResultsPage, Page, PageParams, PathUuid,
    QueryCertificateTransparencyResultsPage, QueryUnhashedResultsPage, ServiceDetectionResultsPage,
    TcpPortScanResultsPage,
};
use crate::models::{
    Attack, BruteforceSubdomainsResult, Certainty, CertificateTransparencyResult,
    CertificateTransparencyValueName, DehashedQueryResult, DnsRecordType, DnsResolutionResult,
    HostAliveResult, ServiceDetectionName, ServiceDetectionResult, TcpPortScanResult,
};

/// A simple representation of a bruteforce subdomains result
#[derive(Serialize, ToSchema)]
pub struct SimpleBruteforceSubdomainsResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// The source address
    pub source: String,

    /// The destination address
    pub destination: String,

    /// The type of DNS record
    #[schema(inline)]
    pub dns_record_type: DnsRecordType,
}

/// Retrieve a bruteforce subdomains' results by the attack's id
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns attack's results", body = BruteforceSubdomainsResultsPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid, PageParams),
    security(("api_key" = []))
)]
#[get("/attacks/{uuid}/bruteforceSubdomainsResults")]
pub async fn get_bruteforce_subdomains_results(
    path: Path<PathUuid>,
    page_params: Query<PageParams>,
    SessionUser(user_uuid): SessionUser,
    db: Data<Database>,
) -> ApiResult<Json<BruteforceSubdomainsResultsPage>> {
    let mut tx = db.start_transaction().await?;

    let attack_uuid = path.uuid;
    let (limit, offset) = get_page_params(page_params).await?;

    if !Attack::has_access(&mut tx, attack_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let (total,) = query!(&mut tx, (BruteforceSubdomainsResult::F.uuid.count(),))
        .condition(BruteforceSubdomainsResult::F.attack.equals(attack_uuid))
        .one()
        .await?;

    let items = query!(&mut tx, BruteforceSubdomainsResult)
        .condition(BruteforceSubdomainsResult::F.attack.equals(attack_uuid))
        .limit(limit)
        .offset(offset)
        .stream()
        .map_ok(|x| SimpleBruteforceSubdomainsResult {
            uuid: x.uuid,
            attack: *x.attack.key(),
            source: x.source,
            destination: x.destination,
            dns_record_type: x.dns_record_type,
            created_at: x.created_at,
        })
        .try_collect()
        .await?;

    tx.commit().await?;

    Ok(Json(Page {
        items,
        limit,
        offset,
        total: total as u64,
    }))
}

/// A simple representation of a tcp port scan result
#[derive(Serialize, ToSchema)]
pub struct SimpleTcpPortScanResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// The ip address a port was found on
    #[schema(value_type = String, example = "127.0.0.1")]
    pub address: IpNetwork,

    /// The found port
    pub port: u16,
}

/// Retrieve a tcp port scan's results by the attack's id
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns attack's results", body = TcpPortScanResultsPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid, PageParams),
    security(("api_key" = []))
)]
#[get("/attacks/{uuid}/tcpPortScanResults")]
pub async fn get_tcp_port_scan_results(
    path: Path<PathUuid>,
    page_params: Query<PageParams>,
    SessionUser(user_uuid): SessionUser,
    db: Data<Database>,
) -> ApiResult<Json<TcpPortScanResultsPage>> {
    let mut tx = db.start_transaction().await?;

    let uuid = path.uuid;
    let (limit, offset) = get_page_params(page_params).await?;

    if !Attack::has_access(&mut tx, uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let (total,) = query!(&mut tx, (TcpPortScanResult::F.uuid.count(),))
        .condition(TcpPortScanResult::F.attack.equals(uuid))
        .one()
        .await?;

    let items = query!(&mut tx, TcpPortScanResult)
        .condition(TcpPortScanResult::F.attack.equals(uuid))
        .limit(limit)
        .offset(offset)
        .stream()
        .map_ok(|result| SimpleTcpPortScanResult {
            uuid: result.uuid,
            attack: *result.attack.key(),
            created_at: result.created_at,
            address: result.address,
            port: result.port as u16,
        })
        .try_collect()
        .await?;

    tx.commit().await?;

    Ok(Json(Page {
        items,
        limit,
        offset,
        total: total as u64,
    }))
}

/// A simple representation of a query certificate transparency result
#[derive(Serialize, ToSchema)]
pub struct FullQueryCertificateTransparencyResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// The name of the issuer
    pub issuer_name: String,

    /// The common name of the certificate
    pub common_name: String,

    /// The values of the certificate
    pub value_names: Vec<String>,

    /// The start date of the certificate
    pub not_before: Option<DateTime<Utc>>,

    /// The end date of the certificate
    pub not_after: Option<DateTime<Utc>>,

    /// The serial number of the certificate
    pub serial_number: String,
}

/// Retrieve a query certificate transparency's results by the attack's id
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns attack's results", body = QueryCertificateTransparencyResultsPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid, PageParams),
    security(("api_key" = []))
)]
#[get("/attacks/{uuid}/queryCertificateTransparencyResults")]
pub async fn get_query_certificate_transparency_results(
    path: Path<PathUuid>,
    page_params: Query<PageParams>,
    SessionUser(user_uuid): SessionUser,
    db: Data<Database>,
) -> ApiResult<Json<QueryCertificateTransparencyResultsPage>> {
    let mut tx = db.start_transaction().await?;

    let attack_uuid = path.uuid;
    let (limit, offset) = get_page_params(page_params).await?;

    if !Attack::has_access(&mut tx, attack_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let (total,) = query!(&mut tx, (CertificateTransparencyResult::F.uuid.count(),))
        .condition(CertificateTransparencyResult::F.attack.equals(attack_uuid))
        .one()
        .await?;

    let mut values: HashMap<Uuid, Vec<String>> = HashMap::new();
    query!(
        &mut tx,
        (
            CertificateTransparencyValueName::F.ct_result,
            CertificateTransparencyValueName::F.value_name
        )
    )
    .condition(
        CertificateTransparencyValueName::F
            .ct_result
            .attack
            .equals(attack_uuid),
    )
    .stream()
    .try_for_each(|(result, value)| {
        values.entry(*result.key()).or_default().push(value);
        async { Ok(()) }
    })
    .await?;

    let items = query!(&mut tx, CertificateTransparencyResult)
        .condition(CertificateTransparencyResult::F.attack.equals(attack_uuid))
        .limit(limit)
        .offset(offset)
        .stream()
        .map_ok(|x| FullQueryCertificateTransparencyResult {
            uuid: x.uuid,
            attack: *x.attack.key(),
            created_at: x.created_at,
            issuer_name: x.issuer_name,
            common_name: x.common_name,
            value_names: values.remove(&x.uuid).unwrap_or_default(),
            not_before: x.not_before,
            not_after: x.not_after,
            serial_number: x.serial_number,
        })
        .try_collect()
        .await?;

    tx.commit().await?;

    Ok(Json(Page {
        items,
        limit,
        offset,
        total: total as u64,
    }))
}

/// A simple representation of a query unhashed result
#[derive(Serialize, ToSchema)]
pub struct SimpleQueryUnhashedResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// ID of the entry
    pub dehashed_id: i64,

    /// An email address
    pub email: Option<String>,

    /// An username
    pub username: Option<String>,

    /// A password
    pub password: Option<String>,

    /// An hashed password
    pub hashed_password: Option<String>,

    /// An ip address
    #[schema(value_type = String, example = "127.0.0.1")]
    pub ip_address: Option<IpNetwork>,

    /// A name
    pub name: Option<String>,

    /// A vin
    pub vin: Option<String>,

    /// An address
    pub address: Option<String>,

    /// A phone number
    pub phone: Option<String>,

    /// A database name
    pub database_name: Option<String>,
}

/// Retrieve a query dehashed's results by the attack's id
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns attack's results", body = QueryUnhashedResultsPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid, PageParams),
    security(("api_key" = []))
)]
#[get("/attacks/{uuid}/queryUnhashedResults")]
pub async fn get_query_unhashed_results(
    path: Path<PathUuid>,
    page_params: Query<PageParams>,
    SessionUser(user_uuid): SessionUser,
    db: Data<Database>,
) -> ApiResult<Json<QueryUnhashedResultsPage>> {
    let mut tx = db.start_transaction().await?;

    let attack_uuid = path.uuid;
    let (limit, offset) = get_page_params(page_params).await?;

    if !Attack::has_access(&mut tx, attack_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let (total,) = query!(&mut tx, (DehashedQueryResult::F.uuid.count(),))
        .condition(DehashedQueryResult::F.attack.equals(attack_uuid))
        .one()
        .await?;

    let items = query!(&mut tx, DehashedQueryResult)
        .condition(DehashedQueryResult::F.attack.equals(attack_uuid))
        .limit(limit)
        .offset(offset)
        .stream()
        .map_ok(|x| SimpleQueryUnhashedResult {
            uuid: x.uuid,
            attack: *x.attack.key(),
            created_at: x.created_at,
            dehashed_id: x.dehashed_id,
            email: x.email,
            username: x.username,
            password: x.password,
            hashed_password: x.hashed_password,
            ip_address: x.ip_address,
            name: x.name,
            vin: x.vin,
            address: x.address,
            phone: x.phone,
            database_name: x.database_name,
        })
        .try_collect()
        .await?;

    tx.commit().await?;

    Ok(Json(Page {
        items,
        limit,
        offset,
        total: total as u64,
    }))
}

/// A simple representation of a host alive result
#[derive(Serialize, ToSchema)]
pub struct SimpleHostAliveResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// A host that responded
    #[schema(value_type = String, example = "127.0.0.1")]
    pub host: IpNetwork,
}

/// Retrieve a host alive's results by the attack's id
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns attack's results", body = HostAliveResultsPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid, PageParams),
    security(("api_key" = []))
)]
#[get("/attacks/{uuid}/hostAliveResults")]
pub async fn get_host_alive_results(
    path: Path<PathUuid>,
    page_params: Query<PageParams>,
    SessionUser(user_uuid): SessionUser,
    db: Data<Database>,
) -> ApiResult<Json<HostAliveResultsPage>> {
    let mut tx = db.start_transaction().await?;

    let attack_uuid = path.uuid;
    let (limit, offset) = get_page_params(page_params).await?;

    if !Attack::has_access(&mut tx, attack_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let (total,) = query!(&mut tx, (HostAliveResult::F.uuid.count(),))
        .condition(HostAliveResult::F.attack.equals(attack_uuid))
        .one()
        .await?;

    let items = query!(&mut tx, HostAliveResult)
        .condition(HostAliveResult::F.attack.equals(attack_uuid))
        .limit(limit)
        .offset(offset)
        .stream()
        .map_ok(|x| SimpleHostAliveResult {
            uuid: x.uuid,
            attack: *x.attack.key(),
            created_at: x.created_at,
            host: x.host,
        })
        .try_collect()
        .await?;

    tx.commit().await?;

    Ok(Json(Page {
        items,
        limit,
        offset,
        total: total as u64,
    }))
}

/// A simple representation of a service detection result
#[derive(Serialize, ToSchema)]
pub struct FullServiceDetectionResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// The certainty of the result
    #[schema(inline)]
    pub certainty: Certainty,

    /// The found names of the service
    pub service_names: Vec<String>,

    /// The ip address a port was found on
    #[schema(value_type = String, example = "127.0.0.1")]
    pub host: IpNetwork,

    /// Port number
    pub port: i16,
}

/// Retrieve a detect service's results by the attack's id
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns attack's results", body = ServiceDetectionResultsPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid, PageParams),
    security(("api_key" = []))
)]
#[get("/attacks/{uuid}/serviceDetectionResults")]
pub async fn get_service_detection_results(
    path: Path<PathUuid>,
    page_params: Query<PageParams>,
    SessionUser(user_uuid): SessionUser,
    db: Data<Database>,
) -> ApiResult<Json<ServiceDetectionResultsPage>> {
    let mut tx = db.start_transaction().await?;

    let attack_uuid = path.uuid;
    let (limit, offset) = get_page_params(page_params).await?;

    if !Attack::has_access(&mut tx, attack_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let (total,) = query!(&mut tx, (ServiceDetectionResult::F.uuid.count(),))
        .condition(ServiceDetectionResult::F.attack.equals(attack_uuid))
        .one()
        .await?;

    let mut names: HashMap<Uuid, Vec<String>> = HashMap::new();
    query!(
        &mut tx,
        (ServiceDetectionName::F.result, ServiceDetectionName::F.name)
    )
    .condition(ServiceDetectionName::F.result.attack.equals(attack_uuid))
    .stream()
    .try_for_each(|(result, name)| {
        names.entry(*result.key()).or_default().push(name);
        async { Ok(()) }
    })
    .await?;

    let items = query!(&mut tx, ServiceDetectionResult)
        .condition(ServiceDetectionResult::F.attack.equals(attack_uuid))
        .limit(limit)
        .offset(offset)
        .stream()
        .map(|x| {
            let x = x?;
            Ok::<_, ApiError>(FullServiceDetectionResult {
                uuid: x.uuid,
                attack: *x.attack.key(),
                created_at: x.created_at,
                certainty: x.certainty,
                service_names: match x.certainty {
                    Certainty::Unknown => Vec::new(),
                    Certainty::Maybe => names.remove(&x.uuid).ok_or_else(|| {
                        error!(
                            "Inconsistent database: ServiceDetectionResult {uuid} has \
                            Certainty::Maybe but no ServiceDetectionName were found",
                            uuid = x.uuid
                        );
                        ApiError::InternalServerError
                    })?,
                    Certainty::Definitely => names
                        .remove(&x.uuid)
                        .ok_or_else(|| {
                            error!(
                                "Inconsistent database: ServiceDetectionResult {uuid} has \
                                Certainty::Definitely but no ServiceDetectionName were found",
                                uuid = x.uuid
                            );
                            ApiError::InternalServerError
                        })
                        .and_then(|names| {
                            if names.len() > 1 {
                                error!(
                                    "Inconsistent database: ServiceDetectionResult {uuid} has \
                                    Certainty::Definitely but multiple ServiceDetectionNames were found",
                                    uuid = x.uuid
                                );
                                Err(ApiError::InternalServerError)
                            } else {
                                Ok(names)
                            }
                        })?,
                },
                host: x.host,
                port: x.port,
            })
        })
        .try_collect()
        .await?;

    tx.commit().await?;

    Ok(Json(Page {
        items,
        limit,
        offset,
        total: total as u64,
    }))
}

/// A simple representation of a dns resolution result
#[derive(Serialize, ToSchema)]
pub struct SimpleDnsResolutionResult {
    /// The primary key
    pub uuid: Uuid,

    /// The attack which produced this result
    pub attack: Uuid,

    /// The point in time, this result was produced
    pub created_at: DateTime<Utc>,

    /// The source address
    pub source: String,

    /// The destination address
    pub destination: String,

    /// The type of DNS record
    #[schema(inline)]
    pub dns_record_type: DnsRecordType,
}

/// Retrieve a dns resolution's results by the attack's id
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns attack's results", body = DnsResolutionResultsPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid, PageParams),
    security(("api_key" = []))
)]
#[get("/attacks/{uuid}/dnsResolutionResults")]
pub async fn get_dns_resolution_results(
    path: Path<PathUuid>,
    page_params: Query<PageParams>,
    SessionUser(user_uuid): SessionUser,
    db: Data<Database>,
) -> ApiResult<Json<DnsResolutionResultsPage>> {
    let mut tx = db.start_transaction().await?;

    let attack_uuid = path.uuid;
    let (limit, offset) = get_page_params(page_params).await?;

    if !Attack::has_access(&mut tx, attack_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let (total,) = query!(&mut tx, (DnsResolutionResult::F.uuid.count(),))
        .condition(DnsResolutionResult::F.attack.equals(attack_uuid))
        .one()
        .await?;

    let items = query!(&mut tx, DnsResolutionResult)
        .condition(DnsResolutionResult::F.attack.equals(attack_uuid))
        .limit(limit)
        .offset(offset)
        .stream()
        .map_ok(|x| SimpleDnsResolutionResult {
            uuid: x.uuid,
            attack: *x.attack.key(),
            source: x.source,
            destination: x.destination,
            dns_record_type: x.dns_record_type,
            created_at: x.created_at,
        })
        .try_collect()
        .await?;

    tx.commit().await?;

    Ok(Json(Page {
        items,
        limit,
        offset,
        total: total as u64,
    }))
}
