//! RFC for connection preface:
//! https://datatracker.ietf.org/doc/html/rfc9113

use std::net::SocketAddr;

use log::{debug, log_enabled, trace};

use super::{probe_tcp, DynResult, LoggableBytes};

pub async fn probe(socket: SocketAddr) -> DynResult<bool> {
    let mut payload = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n".to_vec();
    Head {
        length: 0, // we're sending no settings
        r#type: 4, // SETTINGS frame
        flags: 0,
        stream_id: 0, // reserved id from general connection frames
    }
    .write(&mut payload);

    let data = probe_tcp(socket, &payload).await?;
    trace!(target: "http2", "Got bytes: {:?}", LoggableBytes(&data));
    let Some((head, remaining)) = Head::parse(&data) else {
        return Ok(false);
    };
    trace!(target: "http2", "Parsed head: {:?}", LoggableBytes(&data));

    // The server connection preface consists of a potentially empty SETTINGS frame
    // that MUST be the first frame the server sends in the HTTP/2 connection.
    if log_enabled!(target: "http2", log::Level::Debug) {
        if head.r#type != 4 {
            debug!(target: "http2", "Invalid frame type: {} (expected SETTINGS frame)", head.r#type);
            return Ok(false);
        }
        if head.stream_id != 0 {
            debug!(target: "http2", "Invalid stream id for SETTINGS frame: {}", head.stream_id);
            return Ok(false);
        }
        if head.length % 6 != 0 {
            debug!(target: "http2", "Invalid payload length for SETTINGS frame: {}", head.length);
            return Ok(false);
        }
        if remaining.len() < head.length as usize {
            debug!(target: "http2", "Got less payload bytes than promised (exp: {exp}, got: {got})", exp = head.length, got = remaining.len());
            return Ok(false);
        }
        Ok(true)
    } else {
        Ok(head.r#type == 4
            && head.stream_id == 0
            && head.length % 6 == 0
            && remaining.len() >= head.length as usize)
    }
}

/// http/2 frame head
#[derive(Debug, Copy, Clone)]
struct Head {
    length: u32,
    r#type: u8,
    flags: u8,
    stream_id: u32,
}

impl Head {
    const STREAM_ID_MASK: u32 = 1 << 31;

    /// Parse the http/2 frame head from the beginning of a byte slice
    /// The remaining bytes will be returned next to the parsed head
    fn parse(payload: &[u8]) -> Option<(Self, &[u8])> {
        let (length, payload) = pop::<3>(payload)?;
        let ([r#type], payload) = pop::<1>(payload)?;
        let ([flags], payload) = pop::<1>(payload)?;
        let (stream_id, payload) = pop::<4>(payload)?;

        let length = u32::from_be_bytes([0, length[0], length[1], length[2]]);
        let stream_id = u32::from_be_bytes(stream_id) & !Self::STREAM_ID_MASK;

        Some((
            Head {
                length,
                r#type,
                flags,
                stream_id,
            },
            payload,
        ))
    }

    /// Write the http/2 frame head to the end of a byte vec
    fn write(self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&(self.length.to_be_bytes())[1..]);
        bytes.push(self.r#type);
        bytes.push(self.flags);
        bytes.extend((self.stream_id & !Self::STREAM_ID_MASK).to_be_bytes());
    }
}

/// Pop `N` bytes from a slice and return them in an array as well as the remaining bytes
fn pop<const N: usize>(slice: &[u8]) -> Option<([u8; N], &[u8])> {
    let mut array = [0; N];
    array.copy_from_slice(slice.get(..N)?);
    Some((array, slice.get(N..)?))
}
