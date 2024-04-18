#![allow(clippy::get_first)]
//! Docs for the message formats:
//! https://www.postgresql.org/docs/current/protocol-message-formats.html

use std::slice;

use log::debug;
use log::trace;

use crate::modules::service_detection::generated::Match;
use crate::modules::service_detection::tcp::OneShotTcpSettings;
use crate::modules::service_detection::DynResult;
use crate::utils::DebuggableBytes;

pub async fn probe_tcp(settings: &OneShotTcpSettings) -> DynResult<Match> {
    let Some(data) = settings.probe_tcp(&create_startup_message()).await? else {
        return Ok(Match::No);
    };
    trace!(target: "postgres", "Got data: {data:x?}");
    Ok(if parse_response(data).is_some() {
        Match::Exact
    } else {
        Match::No
    })
}

// NB: postgres' protocol provides no method to enforce ssl.
// Instead admins could block the auth step when the connection is unencrypted.
// Therefore, no special treatment for ssl should be required.
// https://www.postgresql.org/docs/current/protocol-flow.html#PROTOCOL-FLOW-SSL
pub async fn probe_tls(settings: &OneShotTcpSettings, alpn: Option<&str>) -> DynResult<Match> {
    let Ok(Some(data)) = settings.probe_tls(&create_startup_message(), alpn).await? else {
        return Ok(Match::No);
    };
    trace!(target: "postgres", "Got data: {data:x?}");
    Ok(if parse_response(data).is_some() {
        Match::Exact
    } else {
        Match::No
    })
}

fn create_startup_message() -> Vec<u8> {
    const LENGTH: i32 = 19;

    let mut msg = Vec::new();
    msg.extend(LENGTH.to_be_bytes()); // length (4)
    msg.extend(196608i32.to_be_bytes()); // protocol version (4)
    msg.extend("user".as_bytes()); // parameter name (5)
    msg.push(0);
    msg.extend("admin".as_bytes()); // parameter value (6)
    msg.push(0);
    assert_eq!(msg.len() as i32, LENGTH);

    msg.push(0); // end of message (1)
    msg
}

fn parse_response(response: Vec<u8>) -> Option<()> {
    let code = response.get(0)?;
    if ![b'E', b'R'].contains(code) {
        debug!(target: "postgres", "Unhandled response type: {:?}", DebuggableBytes(slice::from_ref(code)));
        return None;
    }

    // NB postgres 13 did not return a length when given the protocol version of `0`
    let length = parse_i32(response.get(1..)?)?;
    debug!(target: "postgres", "Got error response of length: {length}");

    // TODO parse more from the responses

    (*response.get(length as usize)? == 0).then_some(())
}

fn parse_i32(bytes: &[u8]) -> Option<i32> {
    let array = [
        *bytes.get(0)?,
        *bytes.get(1)?,
        *bytes.get(2)?,
        *bytes.get(3)?,
    ];
    Some(i32::from_be_bytes(array))
}
