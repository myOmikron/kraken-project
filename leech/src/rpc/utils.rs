use std::future::Future;

use futures::stream::BoxStream;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};

/// Perform an attack which streams its results
///
/// It manages the communication between the attacking task, the grpc output stream and the backlog.
///
/// ## Arguments
/// - **1st** `perform_attack` is an async closure (called once) which performs the actual attack.
///
///     It receives a [`mpsc::Sender<Item>`] to stream its results
///     and is expected to produce a [`Result<(), Status>`](Status).
///
/// - **2nd** `write_backlog` is an async closure which is used
///     if the stream disconnects unexpectedly to store remaining results to the database.
///
///     It receives the `Item` to store as argument.
///
///     Due to limitations in current rust the produced future has to own its [`Backlog`] instance.
///     Therefore this closure will likely be of the shape:
///     ```no-run
///     # let backlog: Backlog = todo!();
///     move |item| {
///         let backlog = backlog.clone();
///         async move {
///             // backlog.store_...(item).await;
///         }
///     }
///     ```
pub(crate) fn stream_attack<Item, GrpcItem, AttackFut, BacklogFut>(
    perform_attack: impl FnOnce(mpsc::Sender<Item>) -> AttackFut,
    write_backlog: impl Fn(Item) -> BacklogFut + Send + Sync + 'static,
) -> Result<Response<BoxStream<'static, Result<GrpcItem, Status>>>, Status>
where
    Item: Clone + Send + 'static,
    GrpcItem: From<Item> + Send + 'static,
    AttackFut: Future<Output = Result<(), Status>> + Send + 'static,
    AttackFut::Output: Send + 'static,
    BacklogFut: Future + Send,
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
                let grpc_item = item.clone().into();

                // Try sending the item over the rpc stream
                let result = from_middleware.send(Ok(grpc_item)).await;

                // Failure means the receiver i.e. outgoing stream has been closed and dropped
                if result.is_err() {
                    // Save this item to the backlog
                    write_backlog(item).await;

                    // Drain all remaining items into the backlog, because the stream is gone
                    while let Some(item) = to_middleware.recv().await {
                        write_backlog(item).await;
                    }
                    return;
                }
            }
        }
    });

    // Return stream
    Ok(Response::new(Box::pin(ReceiverStream::new(to_stream))))
}
