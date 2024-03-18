use actix_web::delete;
use actix_web::get;
use actix_web::post;
use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::HttpResponse;
use futures::TryStreamExt;
use log::debug;
use rorm::conditions::Condition;
use rorm::conditions::DynamicCollection;
use rorm::query;
use rorm::FieldAccess;
use rorm::Model;

use crate::api::extractors::SessionUser;
use crate::api::handler::attacks::schema::BruteforceSubdomainsRequest;
use crate::api::handler::attacks::schema::DnsResolutionRequest;
use crate::api::handler::attacks::schema::DnsTxtScanRequest;
use crate::api::handler::attacks::schema::HostsAliveRequest;
use crate::api::handler::attacks::schema::ListAttacks;
use crate::api::handler::attacks::schema::OsDetectionRequest;
use crate::api::handler::attacks::schema::QueryCertificateTransparencyRequest;
use crate::api::handler::attacks::schema::QueryDehashedRequest;
use crate::api::handler::attacks::schema::ServiceDetectionRequest;
use crate::api::handler::attacks::schema::SimpleAttack;
use crate::api::handler::attacks::schema::UdpServiceDetectionRequest;
use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::common::schema::UuidResponse;
use crate::api::handler::users::schema::SimpleUser;
use crate::api::handler::workspaces::schema::SimpleWorkspace;
use crate::chan::global::GLOBAL;
use crate::models::Attack;
use crate::models::User;
use crate::models::UserPermission;
use crate::models::WordList;
use crate::models::Workspace;
use crate::models::WorkspaceMember;
use crate::modules::attacks::start_bruteforce_subdomains;
use crate::modules::attacks::start_certificate_transparency;
use crate::modules::attacks::start_dehashed_query;
use crate::modules::attacks::start_dns_resolution;
use crate::modules::attacks::start_dns_txt_scan;
use crate::modules::attacks::start_host_alive;
use crate::modules::attacks::start_os_detection;
use crate::modules::attacks::start_service_detection;
use crate::modules::attacks::start_udp_service_detection;
use crate::modules::attacks::BruteforceSubdomainsParams;
use crate::modules::attacks::CertificateTransparencyParams;
use crate::modules::attacks::DehashedQueryParams;
use crate::modules::attacks::DnsResolutionParams;
use crate::modules::attacks::DnsTxtScanParams;
use crate::modules::attacks::HostAliveParams;
use crate::modules::attacks::OsDetectionParams;
use crate::modules::attacks::ServiceDetectionParams;
use crate::modules::attacks::UdpServiceDetectionParams;

/// Bruteforce subdomains through a DNS wordlist attack
///
/// Enumerate possible subdomains by querying a DNS server with constructed domains.
/// See [OWASP](https://owasp.org/www-community/attacks/Brute_force_attack) for further information.
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 202, description = "Attack scheduled", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = BruteforceSubdomainsRequest,
    security(("api_key" = []))
)]
#[post("/attacks/bruteforceSubdomains")]
pub async fn bruteforce_subdomains(
    req: Json<BruteforceSubdomainsRequest>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let BruteforceSubdomainsRequest {
        leech_uuid,
        domain,
        wordlist_uuid,
        concurrent_limit,
        workspace_uuid,
    } = req.into_inner();

    let (wordlist_path,) = query!(&GLOBAL.db, (WordList::F.path,))
        .condition(WordList::F.uuid.equals(wordlist_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    let client = if let Some(leech_uuid) = leech_uuid {
        GLOBAL.leeches.get_leech(&leech_uuid)?
    } else {
        GLOBAL.leeches.random_leech()?
    };

    let (attack_uuid, _) = start_bruteforce_subdomains(
        workspace_uuid,
        user_uuid,
        client,
        BruteforceSubdomainsParams {
            target: domain,
            wordlist_path,
            concurrent_limit,
        },
    )
    .await?;

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: attack_uuid }))
}

/// Check if hosts are reachable
///
/// Just an ICMP scan for now to see which targets respond.
///
/// All intervals are interpreted in milliseconds. E.g. a `timeout` of 3000 means 3 seconds.
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 202, description = "Attack scheduled", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = HostsAliveRequest,
    security(("api_key" = []))
)]
#[post("/attacks/hostsAlive")]
pub async fn hosts_alive_check(
    req: Json<HostsAliveRequest>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let HostsAliveRequest {
        leech_uuid,
        targets,
        timeout,
        concurrent_limit,
        workspace_uuid,
    } = req.into_inner();

    let leech = if let Some(leech_uuid) = leech_uuid {
        GLOBAL.leeches.get_leech(&leech_uuid)?
    } else {
        GLOBAL.leeches.random_leech()?
    };

    let (attack_uuid, _) = start_host_alive(
        workspace_uuid,
        user_uuid,
        leech,
        HostAliveParams {
            targets,
            timeout,
            concurrent_limit,
        },
    )
    .await?;

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: attack_uuid }))
}

/// Tries to find out the operating system of the remote host.
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 202, description = "Attack scheduled", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = OsDetectionRequest,
    security(("api_key" = []))
)]
#[post("/attacks/osDetection")]
pub async fn os_detection(
    req: Json<OsDetectionRequest>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let OsDetectionRequest {
        leech_uuid,
        targets,
        fingerprint_port,
        ssh_port,
        fingerprint_timeout,
        ssh_connect_timeout,
        ssh_timeout,
        port_ack_timeout,
        port_parallel_syns,
        concurrent_limit,
        workspace_uuid,
    } = req.into_inner();

    let leech = if let Some(leech_uuid) = leech_uuid {
        GLOBAL.leeches.get_leech(&leech_uuid)?
    } else {
        GLOBAL.leeches.random_leech()?
    };

    let (attack_uuid, _) = start_os_detection(
        workspace_uuid,
        user_uuid,
        leech,
        OsDetectionParams {
            targets,
            fingerprint_port,
            ssh_port,
            fingerprint_timeout,
            ssh_connect_timeout,
            ssh_timeout,
            port_ack_timeout,
            port_parallel_syns,
            concurrent_limit,
        },
    )
    .await?;

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: attack_uuid }))
}

/// Query a certificate transparency log collector.
///
/// For further information, see [the explanation](https://certificate.transparency.dev/).
///
/// Certificate transparency can be used to find subdomains or related domains.
///
/// `retry_interval` is specified in milliseconds.
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 202, description = "Attack scheduled", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = QueryCertificateTransparencyRequest,
    security(("api_key" = []))
)]
#[post("/attacks/queryCertificateTransparency")]
pub async fn query_certificate_transparency(
    req: Json<QueryCertificateTransparencyRequest>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let QueryCertificateTransparencyRequest {
        target,
        include_expired,
        max_retries,
        retry_interval,
        workspace_uuid,
    } = req.into_inner();

    let client = GLOBAL.leeches.random_leech()?;

    let (attack_uuid, _) = start_certificate_transparency(
        workspace_uuid,
        user_uuid,
        client,
        CertificateTransparencyParams {
            target,
            include_expired,
            max_retries,
            retry_interval,
        },
    )
    .await?;

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: attack_uuid }))
}

/// Query the [dehashed](https://dehashed.com/) API.
/// It provides email, password, credit cards and other types of information from leak-databases.
///
/// Note that you are only able to query the API if you have bought access and have a running
/// subscription saved in kraken.
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 202, description = "Attack scheduled", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = QueryDehashedRequest,
    security(("api_key" = []))
)]
#[post("/attacks/queryDehashed")]
pub async fn query_dehashed(
    req: Json<QueryDehashedRequest>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let QueryDehashedRequest {
        query,
        workspace_uuid,
    } = req.into_inner();

    let sender = {
        match GLOBAL.dehashed.try_read()?.as_ref() {
            None => return Err(ApiError::DehashedNotAvailable),
            Some(scheduler) => scheduler.retrieve_sender(),
        }
    };

    let (attack_uuid, _) = start_dehashed_query(
        workspace_uuid,
        user_uuid,
        sender,
        DehashedQueryParams { query },
    )
    .await?;

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: attack_uuid }))
}

/// Perform service detection on a ip and port combination
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 202, description = "Attack scheduled", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = ServiceDetectionRequest,
    security(("api_key" = []))
)]
#[post("/attacks/serviceDetection")]
pub async fn service_detection(
    req: Json<ServiceDetectionRequest>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let ServiceDetectionRequest {
        leech_uuid,
        targets,
        ports,
        connect_timeout,
        receive_timeout,
        retry_interval,
        max_retries,
        concurrent_limit,
        skip_icmp_check,
        workspace_uuid,
    } = req.into_inner();

    let client = if let Some(leech_uuid) = leech_uuid {
        GLOBAL.leeches.get_leech(&leech_uuid)?
    } else {
        GLOBAL.leeches.random_leech()?
    };

    let (attack_uuid, _) = start_service_detection(
        workspace_uuid,
        user_uuid,
        client,
        ServiceDetectionParams {
            targets,
            ports,
            connect_timeout,
            receive_timeout,
            max_retries,
            retry_interval,
            concurrent_limit,
            skip_icmp_check,
        },
    )
    .await?;

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: attack_uuid }))
}

/// Perform UDP service detection on an ip on a list of ports.
///
/// All intervals are interpreted in milliseconds. E.g. a `timeout` of 3000 means 3 seconds.
///
/// Set `max_retries` to 0 if you don't want to try a port more than 1 time.
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 202, description = "Attack scheduled", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = UdpServiceDetectionRequest,
    security(("api_key" = []))
)]
#[post("/attacks/udpServiceDetection")]
pub async fn udp_service_detection(
    req: Json<UdpServiceDetectionRequest>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let UdpServiceDetectionRequest {
        leech_uuid,
        targets,
        ports,
        retry_interval,
        max_retries,
        timeout,
        concurrent_limit,
        workspace_uuid,
    } = req.into_inner();

    let client = if let Some(leech_uuid) = leech_uuid {
        GLOBAL.leeches.get_leech(&leech_uuid)?
    } else {
        GLOBAL.leeches.random_leech()?
    };

    let (attack_uuid, _) = start_udp_service_detection(
        workspace_uuid,
        user_uuid,
        client,
        UdpServiceDetectionParams {
            targets,
            ports,
            timeout,
            concurrent_limit,
            max_retries,
            retry_interval,
        },
    )
    .await?;

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: attack_uuid }))
}

/// Perform domain name resolution
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 202, description = "Attack scheduled", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = DnsResolutionRequest,
    security(("api_key" = []))
)]
#[post("/attacks/dnsResolution")]
pub async fn dns_resolution(
    req: Json<DnsResolutionRequest>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let DnsResolutionRequest {
        leech_uuid,
        targets,
        concurrent_limit,
        workspace_uuid,
    } = req.into_inner();

    if targets.is_empty() {
        return Err(ApiError::EmptyTargets);
    }

    let client = if let Some(leech_uuid) = leech_uuid {
        GLOBAL.leeches.get_leech(&leech_uuid)?
    } else {
        GLOBAL.leeches.random_leech()?
    };

    let (attack_uuid, _) = start_dns_resolution(
        workspace_uuid,
        user_uuid,
        client,
        DnsResolutionParams {
            targets,
            concurrent_limit,
        },
    )
    .await?;

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: attack_uuid }))
}

/// Perform DNS TXT scanning & parsing
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 202, description = "Attack scheduled", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    request_body = DnsTxtScanRequest,
    security(("api_key" = []))
)]
#[post("/attacks/dnsTxtScan")]
pub async fn dns_txt_scan(
    req: Json<DnsTxtScanRequest>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let DnsTxtScanRequest {
        leech_uuid,
        targets,
        workspace_uuid,
    } = req.into_inner();

    if targets.is_empty() {
        return Err(ApiError::EmptyTargets);
    }

    let client = if let Some(leech_uuid) = leech_uuid {
        GLOBAL.leeches.get_leech(&leech_uuid)?
    } else {
        GLOBAL.leeches.random_leech()?
    };

    let (attack_uuid, _) = start_dns_txt_scan(
        workspace_uuid,
        user_uuid,
        client,
        DnsTxtScanParams { targets },
    )
    .await?;

    Ok(HttpResponse::Accepted().json(UuidResponse { uuid: attack_uuid }))
}

/// Retrieve an attack by id
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns the attack", body = SimpleAttack),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/attacks/{uuid}")]
pub async fn get_attack(
    req: Path<PathUuid>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<SimpleAttack>> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let attack = query!(
        &mut tx,
        (
            Attack::F.uuid,
            Attack::F.workspace.uuid,
            Attack::F.workspace.name,
            Attack::F.workspace.description,
            Attack::F.workspace.owner as SimpleUser,
            Attack::F.workspace.created_at,
            Attack::F.attack_type,
            Attack::F.finished_at,
            Attack::F.created_at,
            Attack::F.started_by as SimpleUser,
            Attack::F.error,
        )
    )
    .condition(Attack::F.uuid.equals(req.uuid))
    .optional()
    .await?
    .ok_or(ApiError::InvalidUuid)?;

    let attack = if Attack::has_access(&mut tx, req.uuid, user_uuid).await? {
        let (
            uuid,
            w_uuid,
            w_name,
            w_description,
            w_owner,
            w_created_at,
            attack_type,
            finished_at,
            created_at,
            started_by,
            error,
        ) = attack;
        Ok(SimpleAttack {
            uuid,
            workspace: SimpleWorkspace {
                uuid: w_uuid,
                name: w_name,
                description: w_description,
                owner: w_owner,
                created_at: w_created_at,
            },
            attack_type,
            started_by,
            finished_at,
            created_at,
            error,
        })
    } else {
        Err(ApiError::MissingPrivileges)
    };

    tx.commit().await?;

    Ok(Json(attack?))
}

/// Retrieve all attacks the user has access to
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieve a list of all attacks the user has access to", body = ListAttacks),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    security(("api_key" = []))
)]
#[get("/attacks")]
pub async fn get_all_attacks(
    SessionUser(session_user): SessionUser,
) -> ApiResult<Json<ListAttacks>> {
    let db = &GLOBAL.db;

    let mut tx = db.start_transaction().await?;

    let mut workspaces: Vec<_> = query!(&mut tx, (Workspace::F.uuid,))
        .condition(Workspace::F.owner.equals(session_user))
        .stream()
        .map_ok(|(x,)| Attack::F.workspace.equals(x).boxed())
        .try_collect()
        .await?;
    let workspace_members: Vec<_> = query!(&mut tx, (WorkspaceMember::F.workspace,))
        .condition(WorkspaceMember::F.member.equals(session_user))
        .stream()
        .map_ok(|(x,)| Attack::F.workspace.equals(*x.key()).boxed())
        .try_collect()
        .await?;
    workspaces.extend(workspace_members);

    let attacks = if workspaces.is_empty() {
        vec![]
    } else {
        query!(
            &mut tx,
            (
                Attack::F.uuid,
                Attack::F.attack_type,
                Attack::F.error,
                Attack::F.created_at,
                Attack::F.finished_at,
                Attack::F.started_by as SimpleUser,
                Attack::F.workspace.uuid,
                Attack::F.workspace.name,
                Attack::F.workspace.description,
                Attack::F.workspace.created_at,
                Attack::F.workspace.owner as SimpleUser
            )
        )
        .condition(DynamicCollection::or(workspaces))
        .stream()
        .map_ok(
            |(
                uuid,
                attack_type,
                error,
                created_at,
                finished_at,
                started_by,
                w_uuid,
                w_name,
                w_description,
                w_created_at,
                w_owner,
            )| SimpleAttack {
                uuid,
                attack_type,
                error,
                created_at,
                finished_at,
                started_by,
                workspace: SimpleWorkspace {
                    uuid: w_uuid,
                    name: w_name,
                    description: w_description,
                    created_at: w_created_at,
                    owner: w_owner,
                },
            },
        )
        .try_collect()
        .await?
    };

    tx.commit().await?;

    Ok(Json(ListAttacks { attacks }))
}

/// Query all attacks of a workspace
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Retrieve a list of all attacks of a workspace", body = ListAttacks),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}/attacks")]
pub async fn get_workspace_attacks(
    path: Path<PathUuid>,

    SessionUser(user_uuid): SessionUser,
) -> ApiResult<Json<ListAttacks>> {
    let workspace = path.uuid;

    let mut tx = GLOBAL.db.start_transaction().await?;

    if !Workspace::is_user_member_or_owner(&mut tx, workspace, user_uuid).await? {
        return Err(ApiError::MissingPrivileges);
    }

    let attacks = query!(
        &mut tx,
        (
            Attack::F.uuid,
            Attack::F.attack_type,
            Attack::F.error,
            Attack::F.created_at,
            Attack::F.finished_at,
            Attack::F.started_by as SimpleUser,
            Attack::F.workspace.uuid,
            Attack::F.workspace.name,
            Attack::F.workspace.description,
            Attack::F.workspace.created_at,
            Attack::F.workspace.owner as SimpleUser
        )
    )
    .condition(Attack::F.workspace.equals(workspace))
    .all()
    .await?
    .into_iter()
    .map(
        |(
            uuid,
            attack_type,
            error,
            created_at,
            finished_at,
            started_by,
            w_uuid,
            w_name,
            w_description,
            w_created_at,
            w_owner,
        )| SimpleAttack {
            uuid,
            attack_type,
            started_by,
            created_at,
            finished_at,
            error,
            workspace: SimpleWorkspace {
                uuid: w_uuid,
                name: w_name,
                description: w_description,
                created_at: w_created_at,
                owner: w_owner,
            },
        },
    )
    .collect::<Vec<_>>();

    tx.commit().await?;

    Ok(Json(ListAttacks { attacks }))
}

/// Delete an attack and its results
#[utoipa::path(
    tag = "Attacks",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Attack was deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse)
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[delete("/attacks/{uuid}")]
pub async fn delete_attack(
    req: Path<PathUuid>,
    SessionUser(user_uuid): SessionUser,
) -> ApiResult<HttpResponse> {
    let mut tx = GLOBAL.db.start_transaction().await?;

    let user = query!(&mut tx, User)
        .condition(User::F.uuid.equals(user_uuid))
        .optional()
        .await?
        .ok_or(ApiError::SessionCorrupt)?;

    let attack = query!(&mut tx, Attack)
        .condition(Attack::F.uuid.equals(req.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if user.permission == UserPermission::Admin || *attack.started_by.key() == user.uuid {
        debug!("Attack {} got deleted by {}", attack.uuid, user.username);

        rorm::delete!(&mut tx, Attack).single(&attack).await?;
    } else {
        debug!(
            "User {} does not has the privileges to delete the attack {}",
            user.username, attack.uuid
        );

        return Err(ApiError::MissingPrivileges);
    }

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}
