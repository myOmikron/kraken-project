//! Binary for debugging not yet handled ssl errors
//!
//! Pass an url which yields the unhandled error as first argument.

use std::env;

use reqwest::redirect::Policy;
use reqwest::ClientBuilder;
use ssl_errors::NativeTlsError;
use ssl_errors::ReqwestError;

#[cfg(not(feature = "bin"))]
compile_error!("Missing async runtime");

#[tokio::main]
async fn main() -> Result<(), ()> {
    let Some(url) = env::args().nth(1) else {
        eprintln!("Expected url as argument");
        return Err(());
    };
    let client = ClientBuilder::new()
        .redirect(Policy::none())
        .build()
        .unwrap();
    if let Err(error) = client.get(url).send().await {
        let error = ReqwestError::new(&error);
        if let ReqwestError::Tls(NativeTlsError::OpenSsl(error_stack)) = error {
            for (i, error) in error_stack.errors().iter().enumerate() {
                println!("SSL error #{i}:");
                println!("Reason = {}", error.reason_code());
                println!("{error:#?}");
                println!();
            }
        }
        println!("{error:#?}");
    }
    Ok(())
}
