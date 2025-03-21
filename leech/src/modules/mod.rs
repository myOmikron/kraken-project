//! This module holds all available reconnaissance and exploitation
//! modules of the leech.

use std::error::Error;
use std::fmt::Debug;

use kraken_proto::any_attack_response;
use kraken_proto::push_attack_request;
use prost::Message;
use tokio::sync::mpsc::Sender;
use tonic::Status;

pub mod bruteforce_subdomains;
pub mod certificate_transparency;
pub mod dehashed;
pub mod dns;
pub mod host_alive;
pub mod os_detection;
pub mod service_detection;
pub mod testssl;
pub mod whois;

/// Everything which is need to implement an attack.
///
/// This trait should be implemented on a unit struct (ex: `struct MyAttack;`).
/// It bundles the various types and functions required to implement an attack.
///
/// An attack's core functionality is the [`execute`] function and its associated types.
#[tonic::async_trait]
pub trait Attack {
    /// A struct containing the parameters to run the attack with
    type Settings: Debug + Send + 'static;
    /// The attack's output
    type Output: Debug + Send + 'static;
    /// The error type produced by the attack
    type Error: Error + Send + 'static;

    /// Execute the attack
    ///
    /// This function contains the actual attack logic.
    /// It takes the attack's settings and returns its output.
    async fn execute(settings: Self::Settings) -> Result<Self::Output, Self::Error>;

    /// Attack's setting's grpc representation
    type Request: Message + Send + 'static;
    /// Convert the attack's request to its settings
    fn decode_settings(request: Self::Request) -> Result<Self::Settings, Status>;

    /// Attack's output' grpc representation
    type Response: Message + Send + 'static;
    /// Convert the attack's output to its response
    fn encode_output(output: Self::Output) -> Self::Response;

    /// Print an output to stdout for a cli use to see
    fn print_output(output: &Self::Output);

    /// Wrap the attack's response in its [`push_attack_request::Response`] variant used for pushing it to the kraken.
    ///
    /// This function shouldn't do more logic than simply wrapping `response` in an enum variant.
    fn wrap_for_push(response: Self::Response) -> push_attack_request::Response;
}

/// Everything which is need to implement an attack which streams its results.
///
/// This trait should be implemented on a unit struct (ex: `struct MyAttack;`).
/// It bundles the various types and functions required to implement an attack.
///
/// An attack's core functionality is the [`execute`] function and its associated types.
#[tonic::async_trait]
pub trait StreamedAttack {
    /// A struct containing the parameters to run the attack with
    type Settings: Debug + Send + 'static;
    /// The attack's output
    ///
    /// For streamed attacks, this is a single item send over the stream
    type Output: Debug + Send + 'static;
    /// The error type produced by the attack
    type Error: Error + Send + 'static;

    /// Execute the attack
    ///
    /// This function contains the actual attack logic.
    /// It takes the attack's settings and a sender to stream the outputs over.
    ///
    /// ## Notes for implementors
    /// [`Sender::send`] only errors if the channel was closed by the receiver. This is irreversible and a stop condition.
    async fn execute(
        settings: Self::Settings,
        sender: Sender<Self::Output>,
    ) -> Result<(), Self::Error>;

    /// Attack's setting's grpc representation
    type Request: Message + Send + 'static;
    /// Get the attack's uuid from the request
    fn get_attack_uuid(request: &Self::Request) -> &str;
    /// Convert the attack's request to its settings
    fn decode_settings(request: Self::Request) -> Result<Self::Settings, Status>;

    /// Attack's output' grpc representation
    type Response: Message + Send + 'static;
    /// Convert the attack's output to its response
    fn encode_output(output: Self::Output) -> Self::Response;

    /// Print an output to stdout for a cli use to see
    fn print_output(output: &Self::Output);

    /// Wrap the attack's response in its [`any_attack_response::Response`] variant used for storing them to the backlog.
    ///
    /// This function shouldn't do more logic than simply wrapping `response` in an enum variant.
    fn wrap_for_backlog(response: Self::Response) -> any_attack_response::Response;

    /// Wrap the attack's responses in its [`push_attack_request::Response`] variant used for pushing them to the kraken.
    ///
    /// This function shouldn't do more logic than simply wrapping `responses` in an enum variant and the `Repeated...` intermediate.
    fn wrap_for_push(responses: Vec<Self::Response>) -> push_attack_request::Response;
}
