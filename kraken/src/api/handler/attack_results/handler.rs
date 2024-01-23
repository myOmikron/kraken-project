use std::collections::HashMap;

use actix_web::get;
use actix_web::web::{Json, Path, Query};
use futures::{StreamExt, TryStreamExt};
use log::error;
use rorm::{query, FieldAccess, Model};
use uuid::Uuid;

use crate::api::extractors::SessionUser;
use crate::api::handler::attack_results::schema::{
    DnsTxtScanEntry, FullDnsTxtScanResult, FullQueryCertificateTransparencyResult,
    FullServiceDetectionResult, FullUdpServiceDetectionResult, SimpleBruteforceSubdomainsResult,
    SimpleDnsResolutionResult, SimpleHostAliveResult, SimpleQueryUnhashedResult,
    SimpleTcpPortScanResult,
};
use crate::api::handler::common::error::{ApiError, ApiResult};
use crate::api::handler::common::schema::{
    BruteforceSubdomainsResultsPage, DnsResolutionResultsPage, DnsTxtScanResultsPage,
    HostAliveResultsPage, Page, PageParams, PathUuid, QueryCertificateTransparencyResultsPage,
    QueryUnhashedResultsPage, ServiceDetectionResultsPage, TcpPortScanResultsPage,
    UdpServiceDetectionResultsPage,
};
use crate::api::handler::common::utils::get_page_params;
use crate::chan::global::GLOBAL;
use crate::models::{
    Attack, BruteforceSubdomainsResult, CertificateTransparencyResult,
    CertificateTransparencyValueName, DehashedQueryResult, DnsResolutionResult,
    DnsTxtScanAttackResult, DnsTxtScanServiceHintEntry, DnsTxtScanSpfEntry, HostAliveResult,
    ServiceCertainty, ServiceDetectionName, ServiceDetectionResult, TcpPortScanResult,
    UdpServiceDetectionName, UdpServiceDetectionResult,
};

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
) -> ApiResult<Json<BruteforceSubdomainsResultsPage>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let attack_uuid = path.uuid;
    let (limit, offset) = get_page_params(page_params.0).await?;

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
) -> ApiResult<Json<TcpPortScanResultsPage>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let uuid = path.uuid;
    let (limit, offset) = get_page_params(page_params.0).await?;

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
) -> ApiResult<Json<QueryCertificateTransparencyResultsPage>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let attack_uuid = path.uuid;
    let (limit, offset) = get_page_params(page_params.0).await?;

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
) -> ApiResult<Json<QueryUnhashedResultsPage>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let attack_uuid = path.uuid;
    let (limit, offset) = get_page_params(page_params.0).await?;

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
) -> ApiResult<Json<HostAliveResultsPage>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let attack_uuid = path.uuid;
    let (limit, offset) = get_page_params(page_params.0).await?;

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
) -> ApiResult<Json<ServiceDetectionResultsPage>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let attack_uuid = path.uuid;
    let (limit, offset) = get_page_params(page_params.0).await?;

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
                    ServiceCertainty::MaybeVerified => names.remove(&x.uuid).ok_or_else(|| {
                        error!(
                            "Inconsistent database: ServiceDetectionResult {uuid} has \
                            Certainty::Maybe but no ServiceDetectionName were found",
                            uuid = x.uuid
                        );
                        ApiError::InternalServerError
                    })?,
                    ServiceCertainty::DefinitelyVerified => names
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
                    _ => vec![],
                },
                host: x.host,
                port: x.port as u16,
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

/// Retrieve UDP service detection results by the attack's id
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns attack's results", body = UdpServiceDetectionResultsPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid, PageParams),
    security(("api_key" = []))
)]
#[get("/attacks/{uuid}/udpServiceDetectionResults")]
pub async fn get_udp_service_detection_results(
    path: Path<PathUuid>,
    page_params: Query<PageParams>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<UdpServiceDetectionResultsPage>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let attack_uuid = path.uuid;
    let (limit, offset) = get_page_params(page_params.0).await?;

    if !Attack::has_access(&mut tx, attack_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let (total,) = query!(&mut tx, (UdpServiceDetectionResult::F.uuid.count(),))
        .condition(UdpServiceDetectionResult::F.attack.equals(attack_uuid))
        .one()
        .await?;

    let mut names: HashMap<Uuid, Vec<String>> = HashMap::new();
    query!(
        &mut tx,
        (
            UdpServiceDetectionName::F.result,
            UdpServiceDetectionName::F.name
        )
    )
    .condition(UdpServiceDetectionName::F.result.attack.equals(attack_uuid))
    .stream()
    .try_for_each(|(result, name)| {
        names.entry(*result.key()).or_default().push(name);
        async { Ok(()) }
    })
    .await?;

    let items = query!(&mut tx, UdpServiceDetectionResult)
        .condition(UdpServiceDetectionResult::F.attack.equals(attack_uuid))
        .limit(limit)
        .offset(offset)
        .stream()
        .map(|x| {
            let x = x?;
            Ok::<_, ApiError>(FullUdpServiceDetectionResult {
                uuid: x.uuid,
                attack: *x.attack.key(),
                created_at: x.created_at,
                certainty: x.certainty,
                service_names: match x.certainty {
                    ServiceCertainty::MaybeVerified => names.remove(&x.uuid).ok_or_else(|| {
                        error!(
                            "Inconsistent database: UdpServiceDetectionResult {uuid} has \
                            Certainty::Maybe but no UdpServiceDetectionName were found",
                            uuid = x.uuid
                        );
                        ApiError::InternalServerError
                    })?,
                    ServiceCertainty::DefinitelyVerified => names
                        .remove(&x.uuid)
                        .ok_or_else(|| {
                            error!(
                                "Inconsistent database: UdpServiceDetectionResult {uuid} has \
                                Certainty::Definitely but no UdpServiceDetectionName were found",
                                uuid = x.uuid
                            );
                            ApiError::InternalServerError
                        })
                        .and_then(|names| {
                            if names.len() > 1 {
                                error!(
                                    "Inconsistent database: UdpServiceDetectionResult {uuid} has \
                                    Certainty::Definitely but multiple UdpServiceDetectionNames were found",
                                    uuid = x.uuid
                                );
                                Err(ApiError::InternalServerError)
                            } else {
                                Ok(names)
                            }
                        })?,
                    _ => vec![],
                },
                host: x.host,
                port: x.port as u16,
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
) -> ApiResult<Json<DnsResolutionResultsPage>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let attack_uuid = path.uuid;
    let (limit, offset) = get_page_params(page_params.0).await?;

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

/// Retrieve a DNS TXT scan's results by the attack's id
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns attack's results", body = DnsTxtScanResultsPage),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid, PageParams),
    security(("api_key" = []))
)]
#[get("/attacks/{uuid}/dnsTxtScanResults")]
pub async fn get_dns_txt_scan_results(
    path: Path<PathUuid>,
    page_params: Query<PageParams>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<DnsTxtScanResultsPage>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let attack_uuid = path.uuid;
    let (limit, offset) = get_page_params(page_params.0).await?;

    if !Attack::has_access(&mut tx, attack_uuid, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let (total,) = query!(&mut tx, (DnsTxtScanAttackResult::F.uuid.count(),))
        .condition(DnsTxtScanAttackResult::F.attack.equals(attack_uuid))
        .one()
        .await?;

    let mut items: Vec<FullDnsTxtScanResult> = query!(&mut tx, DnsTxtScanAttackResult)
        .condition(DnsTxtScanAttackResult::F.attack.equals(attack_uuid))
        .limit(limit)
        .offset(offset)
        // TODO: aggregate / join with ServiceHint and Spf entries
        .stream()
        .map_ok(|x| FullDnsTxtScanResult {
            uuid: x.uuid,
            attack: *x.attack.key(),
            domain: x.domain,
            created_at: x.created_at,
            collection_type: x.collection_type,
            entries: vec![],
        })
        .try_collect()
        .await?;

    // TODO: this could probably be better represented using a JOIN

    for item in items.iter_mut() {
        let uuid = item.uuid;
        let entries1: Vec<DnsTxtScanEntry> = query!(&mut tx, DnsTxtScanServiceHintEntry)
            .condition(DnsTxtScanServiceHintEntry::F.collection.equals(uuid))
            .stream()
            .map_ok(|s| DnsTxtScanEntry::ServiceHint {
                uuid: s.uuid,
                created_at: s.created_at,
                rule: s.rule,
                txt_type: s.txt_type,
            })
            .try_collect()
            .await?;
        let entries2: Vec<DnsTxtScanEntry> = query!(&mut tx, DnsTxtScanSpfEntry)
            .condition(DnsTxtScanSpfEntry::F.collection.equals(uuid))
            .stream()
            .map_ok(|s| DnsTxtScanEntry::Spf {
                uuid: s.uuid,
                created_at: s.created_at,
                rule: s.rule,
                spf_type: s.spf_type,
                spf_ip: s.spf_ip,
                spf_domain: s.spf_domain,
                spf_domain_ipv4_cidr: s.spf_domain_ipv4_cidr,
                spf_domain_ipv6_cidr: s.spf_domain_ipv6_cidr,
            })
            .try_collect()
            .await?;

        item.entries = [entries1, entries2].concat();
    }

    tx.commit().await?;

    Ok(Json(Page {
        items,
        limit,
        offset,
        total: total as u64,
    }))
}
