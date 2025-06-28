use super::peer::PeerList;
use crate::constants::TCP_PORT;
use crate::core::{archive::Archive, message::MessageType};
use crate::logger;

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

pub struct P2PNode {
    pub peers: Arc<Mutex<PeerList>>,
    pub archive: Arc<RwLock<Archive>>,
}

impl P2PNode {
    pub fn new() -> Self {
        P2PNode {
            peers: Arc::new(Mutex::new(PeerList::new())),
            archive: Arc::new(RwLock::new(Archive::new())),
        }
    }

    pub fn start_listener(&self) {
        let node_arc = Arc::new(self.clone_state());
        thread::spawn(move || {
            let listener =
                TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), TCP_PORT))
                    .expect("Falha ao iniciar o listener TCP");

            logger::info(&format!("Escutando por conexões na porta {}", TCP_PORT));

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let node_clone = Arc::clone(&node_arc);
                        thread::spawn(move || {
                            node_clone.handle_peer_connection(stream);
                        });
                    }
                    Err(e) => logger::error(&format!("Falha ao aceitar conexão: {}", e)),
                }
            }
        });
    }

    pub fn connect_to_peer(&self, peer_addr: &str) {
        let addr = format!("{}:{}", peer_addr, TCP_PORT);
        let node_clone = Arc::new(self.clone_state());
        let peer_addr = peer_addr.to_string();

        thread::spawn(move || {
            match TcpStream::connect(&addr) {
                Ok(stream) => {
                    logger::info(&format!("Conectado com sucesso ao peer: {}", peer_addr));
                    let node_clone_inner = Arc::clone(&node_clone);
                    node_clone_inner.handle_peer_connection(stream);
                }
                Err(e) => logger::error(&format!("Falha ao conectar ao peer {}: {}", peer_addr, e)),
            }
        });
    }

    fn handle_peer_connection(&self, mut stream: TcpStream) {
        let peer_addr = match stream.peer_addr() {
            Ok(addr) => addr,
            Err(_) => {
                logger::error("Conexão terminada antes de identificar o IP do peer");
                return;
            }
        };

        let peer_ip_u32 = if let IpAddr::V4(ipv4) = peer_addr.ip() {
            u32::from(ipv4)
        } else {
            return; // Ignora conexões IPv6
        };

        self.peers.lock().unwrap().add_peer(peer_ip_u32);
        
        logger::debug(&format!("Novo peer conectado: {}", peer_addr));
        let node_clone = self.clone_state();
        let stream_clone = stream.try_clone().expect("Falha ao clonar stream TCP");

        thread::spawn(move || {
            node_clone.peer_requester_thread(stream_clone);
        });

        loop {
            let mut msg_type_buf = [0u8; 1];
            match stream.read_exact(&mut msg_type_buf) {
                Ok(_) => {
                    let msg_type = MessageType::from(msg_type_buf[0]);
                    if !self.handle_message(msg_type, &mut stream) {
                        break;
                    }
                }
                Err(_) => {
                    logger::info(&format!("Peer {} desconectado.", peer_addr));
                    break;
                }
            }
        }

        self.peers.lock().unwrap().remove_peer(peer_ip_u32);
    }

    fn peer_requester_thread(&self, mut stream: TcpStream) {
        let mut counter = 0;

        loop {
            logger::debug("Enviando pedido de lista de peers");
            thread::sleep(Duration::from_secs(5));
            if stream.write_all(&[MessageType::PeerRequest as u8]).is_err() {
                break;
            }

            counter += 1;
            if counter >= 12 {
                logger::debug("Enviando pedido de arquivo de chats");
                if stream
                    .write_all(&[MessageType::ArchiveRequest as u8])
                    .is_err()
                {
                    break;
                }
                counter = 0;
            }
        }
    }

    pub fn publish_archive(&self) {
        let archive_data = { self.archive.read().unwrap().to_bytes() };
        let peer_ips = { self.peers.lock().unwrap().get_ips() };

        if peer_ips.is_empty() {
            return;
        }

        logger::info(&format!(
            "Publicando arquivo atualizado para {} peer(s)",
            peer_ips.len()
        ));

        for ip in peer_ips {
            let ip_addr = Ipv4Addr::from(ip);
            let addr_str = format!("{}:{}", ip_addr, TCP_PORT);
            let data_clone = archive_data.clone();

            thread::spawn(move || {
                let addr: SocketAddr = addr_str.parse().expect("Endereço inválido");
                match TcpStream::connect(addr) {
                    Ok(mut stream) => {
                        logger::debug(&format!("Publicando para {}", addr_str));
                        if let Err(e) = stream.write_all(&data_clone) {
                            logger::warn(&format!("Falha ao publicar para {}: {}", addr_str, e));
                        }
                    }
                    Err(e) => {
                        logger::error(&format!(
                            "Timeout ou erro ao tentar publicar para {}: {}",
                            addr_str, e
                        ));
                    }
                }
            });
        }
    }

    pub fn clone_state(&self) -> Self {
        P2PNode {
            peers: Arc::clone(&self.peers),
            archive: Arc::clone(&self.archive),
        }
    }

    pub fn handle_message(&self, msg_type: MessageType, stream: &mut TcpStream) -> bool {
        match msg_type {
            MessageType::PeerRequest => self.handle_peer_request(stream),
            MessageType::PeerList => self.handle_peer_list(stream),
            MessageType::ArchiveRequest => self.handle_archive_request(stream),
            MessageType::ArchiveResponse => self.handle_archive_response(stream),
        }
    }

    fn handle_peer_request(&self, stream: &mut TcpStream) -> bool {
        logger::debug("Recebido pedido de lista de peers");
        let peer_list = self.peers.lock().unwrap();
        let response = peer_list.to_bytes();
        stream.write_all(&response).is_ok()
    }

    fn handle_archive_request(&self, stream: &mut TcpStream) -> bool {
        logger::debug("Recebido pedido de arquivo de chats");
        let arch = self.archive.read().unwrap();
        if arch.len() > 0 {
            let response = arch.to_bytes();
            stream.write_all(&response).is_ok()
        } else {
            true
        }
    }

    fn handle_peer_list(&self, stream: &mut TcpStream) -> bool {
        logger::debug("Recebido lista de peers");

        let mut count_buf = [0u8; 4];
        if stream.read_exact(&mut count_buf).is_err() {
            return false;
        }

        let count = u32::from_be_bytes(count_buf) as usize;

        let mut received_ips = Vec::with_capacity(count);
        for _ in 0..count {
            let mut ip_buf = [0u8; 4];
            if stream.read_exact(&mut ip_buf).is_err() {
                return false;
            }
            received_ips.push(u32::from_be_bytes(ip_buf));
        }

        let new_peers_to_connect = self
            .peers
            .lock()
            .unwrap()
            .add_and_get_new_peers(received_ips);

        for ip in new_peers_to_connect {
            let ip_addr = Ipv4Addr::from(ip);
            self.connect_to_peer(&ip_addr.to_string());
        }
        true
    }

    fn handle_archive_response(&self, stream: &mut TcpStream) -> bool {
        logger::debug("Recebido resposta de arquivo de chats");
        let mut full_data = vec![MessageType::ArchiveResponse as u8];
        let mut count_buf = [0u8; 4];
        if stream.read_exact(&mut count_buf).is_err() {
            return false;
        }
        full_data.extend_from_slice(&count_buf);

        let count = u32::from_be_bytes(count_buf) as usize;

        for _ in 0..count {
            let mut len_buf = [0u8; 1];
            if stream.read_exact(&mut len_buf).is_err() {
                return false;
            }
            full_data.push(len_buf[0]);

            let msg_len = len_buf[0] as usize;
            let mut chat_data = vec![0u8; msg_len + 32];
            if stream.read_exact(&mut chat_data).is_err() {
                return false;
            }
            full_data.extend_from_slice(&chat_data);
        }

        if let Some(new_archive) = Archive::from_bytes(&full_data) {
            if new_archive.is_valid() {
                let mut current_archive = self.archive.write().unwrap();
                if new_archive.len() > current_archive.len() {
                    *current_archive = new_archive;
                    logger::info(&format!(
                        "Arquivo de chats atualizado com {} mensagens.",
                        current_archive.len()
                    ));
                }
            }
        }
        true
    }
}
