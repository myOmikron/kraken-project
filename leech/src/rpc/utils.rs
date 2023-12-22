use std::future::Future;

use futures::stream::BoxStream;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};
use uuid::Uuid;

use crate::backlog::Backlog;
use crate::rpc::rpc_attacks::any_attack_response;

/// Perform an attack which streams its results
///
/// It manages the communication between the attacking task, the grpc output stream and the backlog.
///
/// The `perform_attack` argument is an async closure (called once) which performs the actual attack.
/// It receives a [`mpsc::Sender<Item>`] to stream its results
/// and is expected to produce a [`Result<(), Status>`](Status).
///
/// The [`From`] implementations for the trait bound `GrpcItem: Into<any_attack_response::Response>`
/// are located in `backlog/mod.rs`.
pub(crate) fn stream_attack<Item, GrpcItem, AttackFut>(
    perform_attack: impl FnOnce(mpsc::Sender<Item>) -> AttackFut,
    backlog: Backlog,
    attack_uuid: Uuid,
) -> Result<Response<BoxStream<'static, Result<GrpcItem, Status>>>, Status>
where
    Item: Send + 'static,
    GrpcItem: From<Item> + Into<any_attack_response::Response> + Send + 'static,
    AttackFut: Future<Output = Result<(), Status>> + Send + 'static,
    AttackFut::Output: Send + 'static,
{
    let (from_attack, mut to_middleware) = mpsc::channel::<Item>(16);
    let (from_middleware, to_stream) = mpsc::channel::<Result<GrpcItem, Status>>(1);

    // Spawn attack
    let attack = perform_attack(from_attack);
    let error_from_attack = from_middleware.clone();
    tokio::spawn(async move {
        if let Err(err) = attack.await {
            let _ = error_from_attack.send(Err(err)).await;
        }
    });

    // Spawn middleware
    tokio::spawn({
        async move {
            while let Some(item) = to_middleware.recv().await {
                let grpc_item: GrpcItem = item.into();

                // Try sending the item over the rpc stream
                let result = from_middleware.send(Ok(grpc_item)).await;

                // Failure means the receiver i.e. outgoing stream has been closed and dropped
                if let Err(error) = result {
                    let Ok(grpc_item) = error.0 else {
                        unreachable!("We tried to send an `Ok(_)` above");
                    };

                    // Save this item to the backlog
                    backlog.store(attack_uuid, grpc_item).await;

                    // Drain all remaining items into the backlog, because the stream is gone
                    while let Some(item) = to_middleware.recv().await {
                        let grpc_item: GrpcItem = item.into();
                        backlog.store(attack_uuid, grpc_item).await;
                    }
                    return;
                }
            }
        }
    });

    // Return stream
    Ok(Response::new(Box::pin(ReceiverStream::new(to_stream))))
}
