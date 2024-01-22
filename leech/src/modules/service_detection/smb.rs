use std::borrow::Cow;

pub fn create_connection_message() -> Vec<u8> {
    let mut bytes = Vec::new();
    SMBMessage {
        header: SMBHeader {
            command: SMB_COM_NEGOTIATE,
            status: SMBStatus::EMPTY,
            flags_1: 0,
            flags_2: 0,
            security_features: [0; 8],
            pid: 1, // arbitrary chosen
            tid: 0, // uninit
            uid: 0, // uninit
            mid: 1, // arbitrary chosen
        },
        params: Cow::Borrowed(&[]),
        data: Cow::Borrowed(b""), // list of (\x02 + cstr)
    }
    .push(&mut bytes);
    bytes
}

struct SMBMessage<'a> {
    header: SMBHeader,

    /// Must be an even number of bytes and less than 512
    params: Cow<'a, [u8]>,

    /// Must be less than 65536
    data: Cow<'a, [u8]>,
}
impl<'a> SMBMessage<'a> {
    fn push(&self, bytes: &mut Vec<u8>) {
        self.header.push(bytes);

        assert_eq!(
            self.params.len() % 2,
            0,
            "Parameters are counted in word i.e. two bytes"
        );
        let word_count = self.params.len() / 2;
        bytes.push(u8::try_from(word_count).expect("Word count is stored in a single byte"));
        bytes.extend_from_slice(&self.params);

        bytes.extend_from_slice(
            &u16::try_from(self.data.len())
                .expect("Byte count is stored in two bytes")
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&self.data);
    }

    fn parse(&self, bytes: &'a [u8]) -> Option<Self> {
        None
    }
}

struct SMBHeader {
    command: SMBCommand,        // 72
    status: SMBStatus,          // 0
    flags_1: u8,                // 08
    flags_2: u16,               // 01 40
    security_features: [u8; 8], // 0

    /// Process identifier
    ///
    /// The PID is assigned by the client.
    pid: u32, // 00 00 40 06

    /// Tree identifier
    ///
    /// TIDs are generated on servers.
    tid: u16, // 00 00

    /// User identifier
    ///
    /// UIDs are generated on servers.
    uid: u16, // 00 00

    /// Multiplex identifier
    ///
    /// The MID is assigned by the client.
    mid: u16, // 00 01
}
impl SMBHeader {
    fn push(&self, bytes: &mut Vec<u8>) {
        let (pid_high, pid_low) = self.pid.to_le_bytes().split_at(2);

        bytes.extend_from_slice(b"\xffSMB"); // protocol
        bytes.push(self.command);
        self.status.push(bytes);
        bytes.push(self.flags_1);
        bytes.extend_from_slice(&self.flags_2.to_le_bytes());
        bytes.extend_from_slice(&self.pid.to_le_bytes()[..2]);
        bytes.extend_from_slice(&self.security_features);
        bytes.extend_from_slice(&[0, 0]); // reserved
        bytes.extend_from_slice(&self.tid.to_le_bytes());
        bytes.extend_from_slice(&self.pid.to_le_bytes()[2..]);
        bytes.extend_from_slice(&self.uid.to_le_bytes());
        bytes.extend_from_slice(&self.mid.to_le_bytes());
    }
}

type SMBCommand = u8;
const SMB_COM_NEGOTIATE: SMBCommand = 0x72;

struct SMBStatus {
    error_class: u8,
    error_code: u16,
}
impl SMBStatus {
    const EMPTY: Self = Self {
        error_class: 0,
        error_code: 0,
    };

    fn push(&self, bytes: &mut Vec<u8>) {
        bytes.push(self.error_class);
        bytes.push(0); // reserved
        bytes.extend_from_slice(&self.error_code.to_le_bytes());
    }
}
