# How to implement a new attack

## Leech's code

### 1. The actual logic
- Create a new rust module under `leech/src/modules`
- It should expose a single async function to run your attack
- Figure out if your attack should be streamed or not

!!! example

    ```rust
    pub struct PortGuesserSettings {
        pub addresses: Vec<IpNetwork>,
        pub num_ports: u32,
    }
    pub struct PortGuesserResult {
        pub host: IpAddr,
        pub port: u16,
    }
    pub type PortGuesserError = SendError<PortGuesserResult>;
    
    pub async fn port_guesser(
        settings: PortGuesserSettings,
        tx: Sender<PortGuesserResult>, // <- this attack is streamed so we "return" our results through a channel
    ) -> Result<(), PortGuesserError> {
        let mut rng = StdRng::from_entropy();
        for network in settings.addresses {
            for addr in network.iter() {
                for _ in 0..settings.num_ports {
                    tx.send(PortGuesserResult {
                        host: addr,
                        port: rng.gen_range(1..=u16::MAX),
                    })
                    .await?;
                }
            }
        }
        Ok(())
    }
    ```

### 2. Expose it on the cli
- Create a new `RunCommand` variant in `leech/src/main.rs`
- Add the associated match arm to execute your function in `fn main()`

    (the compiler will tell you where)

!!! example

    ```rust
    #[derive(Subcommand)]
    pub enum RunCommand {
        // ...
    
        PortGuesser {
            #[clap(required(true))]
            targets: Vec<String>,
    
            #[clap(short, long, default_value_t = 4)]
            num_ports: u32,
        },
    }

    // inside main
    match command {
        // ...

        RunCommand::PortGuesser { targets, num_ports } => {
            let addresses = targets
                .iter()
                .map(|s| IpNetwork::from_str(s))
                .collect::<Result<_, _>>()?;
            let (tx, mut rx) = mpsc::channel(1);
            task::spawn(port_guesser(
                PortGuesserSettings {
                    addresses,
                    num_ports,
                },
                tx,
            ));
            while let Some(res) = rx.recv().await {
                info!("I guess {}:{} is open", res.host, res.port);
            }
        }
    }
    ```

### 3. Expose it over grpc
- Create two new message types in `proto/attacks.proto`, a `...Request` and a `...Response`
- The request should contain your function's arguments and the response should contain its results
- Extend `ReqAttackService` with a new call using your request and response messages
- Extend the `PushAttackRequest.response` with your new response

    (make it repeated if your attack is streamed)

- Extend the `AnyAttackResponse.response` with your new response

    (never make it repeated)

- Implement the new method on `ReqAttackService`, the compiler will now complain about

    It should convert the `...Request` object to your functions arguments and its result to `...Response`.

    When your attack is streamed, you might find the function `stream_attack` helpful to construct the return value.

!!! example

    ```proto
    message PortGuesserRequest {
      string attack_uuid = 1;
      repeated attacks.shared.NetOrAddress targets = 2;
      uint32 num_ports = 3;
    }
    
    message PortGuesserResponse {
      attacks.shared.Address host = 1;
      uint32 port = 2;
    }

    service ReqAttackService {
      // ...

      rpc PortGuesser(PortGuesserRequest) returns (stream PortGuesserResponse);
    }
    ```

    ```proto
    message PushAttackRequest {
      // ...

      oneof response {
        // ...

        RepeatedPortGuesserResponse port_guesser = 9;
        // ^
        // | if the attack is not streamed, just use `PortGuesserResponse` here
      }
    }

    message RepeatedPortGuesserResponse {
      repeated PortGuesserResponse responses = 1;
    }
    ```

    ```proto
    message AnyAttackResponse {
      // ...
      
      oneof response {
        // ...
        
        PortGuesserResponse port_guesser = 8;
      }
    }
    ```

    ```rust
    impl ReqAttackService for Attacks {
        // ...
      
        type PortGuesserStream =
            Pin<Box<dyn Stream<Item = Result<PortGuesserResponse, Status>> + Send>>;
    
        async fn port_guesser(
            &self,
            request: Request<PortGuesserRequest>,
        ) -> Result<Response<Self::PortGuesserStream>, Status> {
            let req = request.into_inner();
    
            let attack_uuid = Uuid::parse_str(&req.attack_uuid)
                .map_err(|_| Status::invalid_argument("attack_uuid has to be an Uuid"))?;
    
            let settings = PortGuesserSettings {
                addresses: req.targets.into_iter().map(|el| el.into()).collect(),
                num_ports: req.num_ports,
            };
    
            self.stream_attack(
                attack_uuid,
                |tx| async move {
                    port_guesser(settings, tx)
                        .await
                        .map_err(|_| Status::unknown(""))
                },
                |value| PortGuesserResponse {
                    host: Some(value.host.into()),
                    port: value.port as u32,
                },
                any_attack_response::Response::PortGuesser,
            )
        }
    }
    ```

!!! info
    kraken won't compile anymore

## Kraken's code

### 4. DB models
- Add your new attack type to `AttackType` in `kraken/src/models/attack/mod.rs`
  and `SourceType` in `kraken/src/models/aggregation/mod.rs`
- Add models in `kraken/src/models/attack/mod.rs` to store your attack's raw results

    (A raw result is the `...Response` message you defined in step 3)

    Don't forget to create an insertable patch as well!

!!! warning

    Please consult your project lead on how to migrate the `AttackType` and `SourceType` enums

!!! example

    ```rust
    #[derive(DbEnum)]
    pub enum SourceType {
        // ...
    
        PortGuesser,
    }
    
    #[derive(DbEnum)]
    pub enum AttackType {
        // ...
    
        PortGuesser,
    }
    ```
    ```rust
    #[derive(Model)]
    pub struct PortGuesserResult {
        #[rorm(primary_key)]
        pub uuid: Uuid,
    
        #[rorm(on_delete = "Cascade", on_update = "Cascade")]
        pub attack: ForeignModel<Attack>,
    
        #[rorm(auto_create_time)]
        pub created_at: DateTime<Utc>,
    
        pub host: IpNetwork,
    
        pub port: i32,
    }

    #[derive(Patch)]
    #[rorm(model = "PortGuesserResult")]
    pub(crate) struct PortGuesserResultInsert {
        pub(crate) uuid: Uuid,
        pub(crate) attack: ForeignModel<Attack>,
        pub(crate) host: IpNetwork,
        pub(crate) port: i32,
    }
    ```

### 5. Handle your grpc messages
- Implement `HandleAttackResponse<...Response>` for `AttackContext`

    This is done in an attack specific submodule in `kraken/src/modules/attacks`.
  
    The `handle_response` method, you will have to implement, should:
  
    - Validate and convert your `...Response` message
    - Notify the user via `self.send_ws`
      
        (you might need to create a new websocket message type)
  
    - Store the raw result
    - Update the aggregations via `GLOBAL.aggregator`
    - Connect the raw result with the aggregated models using the `AggregationSource` m2m relation
  
- Extend the 3 match statements in `kraken/src/rpc/server.rs` using your `AttackType` variant and `handle_response` impl.

!!! example

    ```rust
    impl HandleAttackResponse<PortGuesserResponse> for AttackContext {
        async fn handle_response(&self, response: PortGuesserResponse) -> Result<(), AttackError> {
            let PortGuesserResponse {
                host: Some(host),
                port,
            } = response
            else {
                return Err(AttackError::Malformed("Missing `host`"));
            };
            let host = IpNetwork::from(IpAddr::try_from(host)?);
    
            let source_uuid = insert!(&GLOBAL.db, PortGuesserResultInsert)
                .return_primary_key()
                .single(&PortGuesserResultInsert {
                    uuid: Uuid::new_v4(),
                    attack: ForeignModelByField::Key(self.attack_uuid),
                    host,
                    port: port as i32,
                })
                .await?;
    
            let host_uuid = GLOBAL
                .aggregator
                .aggregate_host(self.workspace.uuid, host, HostCertainty::SupposedTo)
                .await?;
            let port_uuid = GLOBAL
                .aggregator
                .aggregate_port(
                    self.workspace.uuid,
                    host_uuid,
                    port as u16,
                    PortProtocol::Tcp,
                    PortCertainty::SupposedTo,
                )
                .await?;
    
            insert!(&GLOBAL.db, AggregationSource)
                .return_nothing()
                .bulk([
                    AggregationSource {
                        uuid: Uuid::new_v4(),
                        workspace: ForeignModelByField::Key(self.workspace.uuid),
                        source_type: SourceType::PortGuesser,
                        source_uuid,
                        aggregated_table: AggregationTable::Host,
                        aggregated_uuid: host_uuid,
                    },
                    AggregationSource {
                        uuid: Uuid::new_v4(),
                        workspace: ForeignModelByField::Key(self.workspace.uuid),
                        source_type: SourceType::PortGuesser,
                        source_uuid,
                        aggregated_table: AggregationTable::Port,
                        aggregated_uuid: port_uuid,
                    },
                ])
                .await?;
    
            Ok(())
        }
    }
    ```
    ```rust
    // three match expressions in kraken/src/rpc/server.rs
    
    match &response {
        // ...
        push_attack_request::Response::PortGuesser(_) => AttackType::PortGuesser,
    },
    
    let result = match response {
        // ...
        push_attack_request::Response::PortGuesser(repeated) => {
            attack.handle_vec_response(repeated.responses).await
        }
    };
    
    let result: Result<(), _> = match response {
        // ...
        any_attack_response::Response::PortGuesser(response) => {
            attack_context.handle_response(response).await
        }
    };
    ```

### 6. Write schemas and handler to retrieve raw results

Add utoipa pagination structs to `kraken/src/api/handler/common/schema.rs`.

Write an http handler in `kraken/src/api/handler/attack_results/handler.rs` to query your raw attack results
(the model you created in step 4).

We are doing this step now, because you'll have to write new schemas for the handler's response,
which we'll also need in the next step.

!!! example

    ```rust
    // kraken/src/api/handler/common/schema.rs
    pub use utoipa_fix::{
        ..., PortGuesserResultsPage, ...
    }

    ...

    #[allow(missing_docs)]
    mod utoipa_fix {
        use crate::api::handler::attack_results::schema::{
            ..., PortGuesserResult, ...
        }

        #[derive(Serialize, Deserialize, Default, ToSchema, Clone)]
        #[aliases(
            ...
            PortGuesserResultsPage = Page<PortGuesserResult>,
        )]
        pub struct Page<T> {
            ...
        }
        ...
    }
    ...
    ```

    ```rust
    // kraken/src/api/handler/attack_results/handler.rs

    #[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
    pub struct SimplePortGuesserResult {
        pub uuid: Uuid,
        pub attack: Uuid,
        pub created_at: DateTime<Utc>,
        #[schema(value_type = String, example = "127.0.0.1")]
        pub address: IpNetwork,
        pub port: u16,
    }
    ```
    ```rust
    #[utoipa::path(
        tag = "Attacks",
        context_path = "/api/v1",
        responses(
            (status = 200, description = "Returns attack's results", body = PortGuesserResultsPage),
            (status = 400, description = "Client error", body = ApiErrorResponse),
            (status = 500, description = "Server error", body = ApiErrorResponse),
        ),
        params(PathUuid, PageParams),
        security(("api_key" = []))
    )]
    #[get("/attacks/{uuid}/portGuesserResults")]
    pub async fn get_port_guesser_results(
        path: Path<PathUuid>,
        page_params: Query<PageParams>,
        SessionUser(user_uuid): SessionUser,
    ) -> ApiResult<Json<PortGuesserResultsPage>> {
        let mut tx = GLOBAL.db.start_transaction().await?;
    
        let attack_uuid = path.uuid;
        let (limit, offset) = get_page_params(page_params.0).await?;
    
        if !Attack::has_access(&mut tx, attack_uuid, user_uuid).await? {
            return Err(ApiError::MissingPrivileges);
        }
    
        let (total,) = query!(&mut tx, (PortGuesserResult::F.uuid.count(),))
            .condition(PortGuesserResult::F.attack.equals(attack_uuid))
            .one()
            .await?;
    
        let items = query!(&mut tx, PortGuesserResult)
            .condition(PortGuesserResult::F.attack.equals(attack_uuid))
            .limit(limit)
            .offset(offset)
            .stream()
            .map_ok(|x| SimplePortGuesserResult {
                uuid: x.uuid,
                attack: *x.attack.key(),
                created_at: x.created_at,
                address: x.host,
                port: x.port as u16,
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
    ```

### 7. Handle the new `SourceType`
- Add a new `usize` field to `SimpleAggregationSource` in `kraken/src/api/handler/aggregation_source/schema.rs`
- Increment this field in `SimpleAggregationSource::add` in `kraken/src/api/handler/aggregation_source/utils.rs`
- Add a new variant to `SourceAttackResult` in `kraken/src/api/handler/aggregation_source/schema.rs`
- Add the logic required to query your attack type's raw results for a specific aggregated model
  (`FullAggregationSource::query` in `kraken/src/api/handler/aggregation_source/utils.rs`)

!!! example

    ```rust
    pub struct SimpleAggregationSource {
        // ...
        
        pub port_guesser: usize,
    }
    ```
    ```rust
    match source_type {
        // ...
        
        SourceType::PortGuesser => self.port_guesser += 1,
    }
    ```
    ```rust
    pub enum SourceAttackResult {
      // ...
    
      PortGuesser(Vec<SimplePortGuesserResult>),
    }
    ```
    ```rust
    type Results<T> = HashMap<Uuid, Vec<T>>;
    // ...
    let mut port_guesser: Results<SimplePortGuesserResult> = Results::new();
    
    // ...
    
    match source_type {
        // ...
    
        SourceType::PortGuesser => {
            let mut stream = query!(&mut *tx, PortGuesserResult)
                .condition(field_in(PortGuesserResult::F.uuid, uuids))
                .stream();
            while let Some(result) = stream.try_next().await? {
                port_guesser.entry(*result.attack.key()).or_default().push(
                    SimplePortGuesserResult {
                        uuid: result.uuid,
                        attack: *result.attack.key(),
                        created_at: result.created_at,
                        address: result.host,
                        port: result.port as u16,
                    },
                );
            }
        }
    }
    
    // ...
    
    match attack_type {
        // ...
    
        AttackType::PortGuesser => SourceAttackResult::PortGuesser(
            port_guesser.remove(&uuid).unwrap_or_default(),
        ),
    }
    ```

!!! info
    kraken should compile again

    frontend's data view will break

### 8. Expose it over the API
- Add a method to `AttackContext` in your submodule of `kraken/src/modules/attacks` which runs the attack

    This method should be a thin wrapper which converts the attack's params to grpc and handle the response.
    
    (Use `handle_response` you implemented in step 5 or `handle_streamed_response` if you're attack is streamed)

- Add a function in `kraken/src/modules/attacks/mod.rs` which calls this method in a new task

- Add an endpoint in `kraken/src/api/handler/attacks/handler.rs` which calls this function

!!! example

    ```rust
    impl AttackContext {
        pub async fn port_guesser(
            &self,
            mut leech: LeechClient,
            params: PortGuesserParams,
        ) -> Result<(), AttackError> {
            let targets =
                DomainOrNetwork::resolve(self.workspace.uuid, self.user.uuid, &leech, &params.targets)
                    .await?;
    
            self.handle_streamed_response(leech.port_guesser(PortGuesserRequest {
                attack_uuid: self.attack_uuid.to_string(),
                targets: targets.into_iter().map(From::from).collect(),
                num_ports: params.num_ports,
            }))
            .await
        }
    }
    ```
    
    ```rust
    pub struct PortGuesserParams {
        pub targets: Vec<DomainOrNetwork>,
        pub num_ports: u32,
    }
    pub async fn start_port_guesser(
        workspace: Uuid,
        user: Uuid,
        leech: LeechClient,
        params: PortGuesserParams,
    ) -> Result<(Uuid, JoinHandle<()>), InsertAttackError> {
        let ctx = AttackContext::new(workspace, user, AttackType::PortGuesser).await?;
        Ok((
            ctx.attack_uuid,
            tokio::spawn(async move {
                ctx.set_started().await;
                let result = ctx.port_guesser(leech, params).await;
                ctx.set_finished(result).await;
            }),
        ))
    }
    ```
    
    ```rust
    #[derive(Deserialize, Serialize, Debug, ToSchema)]
    pub struct PortGuesserRequest {
      pub leech_uuid: Option<Uuid>,
      #[schema(value_type = Vec<String>)]
      pub targets: Vec<DomainOrNetwork>,
      pub num_ports: u32,
      pub workspace_uuid: Uuid,
    }
    ```
    
    ```rust
    
    #[utoipa::path(
        tag = "Attacks",
        context_path = "/api/v1",
        responses(
            (status = 202, description = "Attack scheduled", body = UuidResponse),
            (status = 400, description = "Client error", body = ApiErrorResponse),
            (status = 500, description = "Server error", body = ApiErrorResponse)
        ),
        request_body = PortGuesserRequest,
        security(("api_key" = []))
    )]
    #[post("/attacks/portGuesser")]
    pub async fn port_guesser(
        req: Json<PortGuesserRequest>,
        SessionUser(user_uuid): SessionUser,
    ) -> ApiResult<HttpResponse> {
        let PortGuesserRequest {
            leech_uuid,
            targets,
            num_ports,
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
    
        let (attack_uuid, _) = start_port_guesser(
            workspace_uuid,
            user_uuid,
            client,
            PortGuesserParams { targets, num_ports },
        )
        .await?;
    
        Ok(HttpResponse::Accepted().json(UuidResponse { uuid: attack_uuid }))
    }
    ```

## Frontend's code

### 9. Expose API functions and structs

In `kraken/src/api/server.rs` register your endpoints:

!!! example

    ```rust
        ...
        .service(attacks::handler::service_detection)
        .service(attacks::handler::udp_service_detection)
        .service(attacks::handler::dns_resolution)
        .service(attacks::handler::dns_txt_scan)
        .service(attacks::handler::port_guesser)
        ...
        .service(attack_results::handler::get_service_detection_results)
        .service(attack_results::handler::get_udp_service_detection_results)
        .service(attack_results::handler::get_dns_resolution_results)
        .service(attack_results::handler::get_dns_txt_scan_results)
        .service(attack_results::handler::get_port_guesser_results)
    ```

In `kraken/src/api/swagger.rs` register all your endpoints and custom data structures:

!!! example

    ```rust
        ...
        attacks::handler::dns_resolution,
        attacks::handler::dns_txt_scan,
        attacks::handler::port_guesser,
        ...
        attack_results::handler::get_dns_resolution_results,
        attack_results::handler::get_dns_txt_scan_results,
        attack_results::handler::get_port_guesser_results,
        ...
        attacks::schema::DnsResolutionRequest,
        attacks::schema::DnsTxtScanRequest,
        attacks::schema::PortGuesserRequest,
        ...
        attack_results::schema::SimpleDnsTxtScanResult,
        attack_results::schema::FullDnsTxtScanResult,
        attack_results::schema::PortGuesserResult,

        ...
        // as well as new types you expose inside the Request/Result types:
        models::DnsTxtScanSpfType,
        models::DnsTxtScanServiceHintType,
        models::DnsTxtScanSummaryType,
    ```

### 10. Make it build again

- Run `yarn gen-api`

    Don't forget to download the newest `openapi.json` first!

- Add an entry in the `ATTACKS` object in `kraken_frontend/src/utils/attack-resolver.ts`

!!! example

    ```ts
    export const ATTACKS: AttackResolver = {
        // ...
        PortGuesser: { abbreviation: "PG", long: "Port guesser", key: "portGuesser" },
    }
    ```

- Wrap the API endpoint in `kraken_frontend/src/api/api.ts`

!!! example

    ```ts
    import {
        ...
        PortGuesserRequest,
        ...
    } from "./generated";
    ...
    portGuesser: (attack: PortGuesserRequest) => handleError(attacks.portGuesser({ portGuesserRequest: attack })),
    ```

### 11. Make it usable

TODO