use std::convert::From;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
    PeerRequest = 0x1,
    PeerResponse = 0x2,
    ArchiveRequest = 0x3,
    ArchiveResponse = 0x4,
    NotificationMessage = 0x5,
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            0x1 => MessageType::PeerRequest,
            0x2 => MessageType::PeerResponse,
            0x3 => MessageType::ArchiveRequest,
            0x4 => MessageType::ArchiveResponse,
            0x5 => MessageType::NotificationMessage,
            _ => panic!("Invalid message type: {value}"),
        }
    }
}

impl MessageType {
    pub fn is_valid_message(value: u8) -> bool {
        (0x1..=0x5).contains(&value)
    }

}

#[derive(Debug, Clone)]
pub struct Chat {
    pub message: String,
    pub verification_code: [u8; 16],
    pub md5_hash: [u8; 16],
}

impl Chat {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.message.len() as u8);
        bytes.extend_from_slice(self.message.as_bytes());
        bytes.extend_from_slice(&self.verification_code);
        bytes.extend_from_slice(&self.md5_hash);
        bytes
    }

    pub fn from_bytes(data: &[u8]) -> Option<(Self, usize)> {
        if data.is_empty() {
            return None;
        }

        let msg_len = data[0] as usize;
        if data.len() < 1 + msg_len + 32 {
            return None;
        }

        let message = String::from_utf8(data[1..1 + msg_len].to_vec()).ok()?;
        let mut verification_code = [0u8; 16];
        let mut md5_hash = [0u8; 16];

        verification_code.copy_from_slice(&data[1 + msg_len..1 + msg_len + 16]);
        md5_hash.copy_from_slice(&data[1 + msg_len + 16..1 + msg_len + 32]);

        Some((
            Chat {
                message,
                verification_code,
                md5_hash,
            },
            1 + msg_len + 32,
        ))
    }
}
