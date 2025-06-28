use crate::core::message::MessageType;
use std::collections::HashSet;

#[derive(Debug)]
pub struct PeerList {
    peers: HashSet<u32>,
}

impl PeerList {
    pub fn new() -> Self {
        PeerList { peers: HashSet::new() }
    }

    pub fn add_peer(&mut self, ip: u32) {
        self.peers.insert(ip);
    }

    pub fn remove_peer(&mut self, ip: u32) {
        self.peers.remove(&ip);
    }
    
    pub fn add_and_get_new_peers(&mut self, new_ips: Vec<u32>) -> Vec<u32> {
        let mut truly_new_peers = Vec::new();
        for ip in new_ips {
            if self.peers.insert(ip) {
                truly_new_peers.push(ip);
            }
        }
        truly_new_peers
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let count = self.peers.len() as u32;
        
        bytes.push(MessageType::PeerList as u8);
        bytes.extend_from_slice(&count.to_be_bytes());

        for &ip in &self.peers {
            bytes.extend_from_slice(&ip.to_be_bytes());
        }
        bytes
    }

    pub fn get_ips(&self) -> Vec<u32> {
        self.peers.iter().cloned().collect()
    }
}