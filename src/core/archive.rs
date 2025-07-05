use crate::logger;

use super::message::{Chat, MessageType};
use rand::{self, Rng};

#[derive(Debug, Clone)]
pub struct Archive {
    pub chats: Vec<Chat>,
}

impl Archive {
    pub fn new() -> Self {
        Archive { chats: Vec::new() }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(MessageType::ArchiveResponse as u8);

        let count = self.chats.len() as u32;
        bytes.extend_from_slice(&count.to_be_bytes());

        for chat in &self.chats {
            bytes.extend_from_slice(&chat.to_bytes());
        }

        bytes
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 5 || data[0] != MessageType::ArchiveResponse as u8 {
            return None;
        }

        let count = u32::from_be_bytes([data[1], data[2], data[3], data[4]]) as usize;
        let mut chats = Vec::with_capacity(count);
        let mut offset = 5;

        for _ in 0..count {
            if let Some((chat, size)) = Chat::from_bytes(&data[offset..]) {
                chats.push(chat);
                offset += size;
            } else {
                return None;
            }
        }
        Some(Archive { chats })
    }

    pub fn is_valid(&self) -> bool {
        if self.chats.is_empty() {
            return true;
        }

        for i in 0..self.chats.len() {
            if !self.validate_chat_at_index(i) {
                return false;
            }
        }

        true
    }

    fn validate_chat_at_index(&self, index: usize) -> bool {
        if index >= self.chats.len() {
            return false;
        }

        let chat = &self.chats[index];
        if chat.md5_hash[0] != 0 || chat.md5_hash[1] != 0 {
            return false;
        }

        let start_index = index.saturating_sub(19);
        let mut data_to_hash = Vec::new();

        for j in start_index..=index {
            let chat_bytes = self.chats[j].to_bytes();

            if j == index {
                data_to_hash.extend_from_slice(&chat_bytes[..chat_bytes.len() - 16]);
            } else {
                data_to_hash.extend_from_slice(&chat_bytes);
            }
        }

        let calculated_hash = md5::compute(&data_to_hash);

        calculated_hash.0 == chat.md5_hash
    }

    pub fn add_message(&mut self, message: String) -> bool {
        if !Self::is_valid_message(&message) {
            println!(
                "Erro: Mensagem inválida. Deve conter entre 1 e 255 caracteres ASCII (32-126)."
            );
            return false;
        }

        logger::info(&format!(
            "Minerando código de verificação para a mensagem: '{}'...",
            message
        ));

        let mut rng = rand::rng();

        loop {
            let mut verification_code = [0u8; 16];
            rng.fill(&mut verification_code);

            let temp_chat = Chat {
                message: message.clone(),
                verification_code,
                md5_hash: [0u8; 16],
            };

            let start_index = self.chats.len().saturating_sub(19);
            let mut data_to_hash = Vec::new();

            for i in start_index..self.chats.len() {
                data_to_hash.extend_from_slice(&self.chats[i].to_bytes());
            }

            let temp_bytes = temp_chat.to_bytes();
            data_to_hash.extend_from_slice(&temp_bytes[..temp_bytes.len() - 16]);

            let calculated_hash = md5::compute(&data_to_hash).0;

            if calculated_hash[0] == 0 && calculated_hash[1] == 0 {
                let final_chat = Chat {
                    message,
                    verification_code,
                    md5_hash: calculated_hash,
                };
                
                self.chats.push(final_chat);

                let verification_code_str = verification_code
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>();

                let md5_hash_str = calculated_hash
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>();

                logger::info(&format!(
                    "Código de verificação minerado: {}",
                    verification_code_str
                ));

                logger::info(&format!("Hash MD5 da mensagem: {}", md5_hash_str));
                return true;
            }
        }
    }

    fn is_valid_message(message: &str) -> bool {
        !message.is_empty()
            && message.len() <= 255
            && message.chars().all(|c| c.is_ascii_graphic() || c == ' ')
    }

    pub fn len(&self) -> usize {
        self.chats.len()
    }
}
