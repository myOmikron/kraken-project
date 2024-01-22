use itertools::Itertools;

use super::{DetectServiceSettings, DynResult};

pub async fn probe(settings: &DetectServiceSettings) -> DynResult<bool> {
    let response = settings.probe_tcp(b"anything").await?;
}

pub struct SessionPacket<'b> {
    r#type: u8,
    flags: u8,
    length: u16,
    trailer: &'b [u8],
}

impl<'b> SessionPacket<'b> {
    pub fn parse(data: &'b [u8]) -> Option<Self> {
        let (&r#type, data) = data.split_first()?;
        let (&flags, data) = data.split_first()?;
        let (&l1, data) = data.split_first()?;
        let (&l2, data) = data.split_first()?;
        let length = u16::from_le_bytes([l1, l2]);
        (data.len() == length as usize).then_some(Self {
            r#type,
            flags,
            length,
            trailer: data,
        })
    }

    /// Checks if type and flags are set correctly
    pub fn is_valid(&self) -> bool {
        [0x00, 0x81, 0x82, 0x83, 0x84, 0x85].contains(&self.r#type) && (self.flags >> 1) == 0
    }
}
