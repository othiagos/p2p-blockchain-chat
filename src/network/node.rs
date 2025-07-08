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

            logger::info(&format!("Escutando por conexões na porta {TCP_PORT}"));

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let node_clone = Arc::clone(&node_arc);

                        thread::spawn(move || {
                            node_clone.handle_peer_connection(stream);
                        });
                    }
                    Err(e) => logger::warn(&format!("Falha ao aceitar conexão: {e}")),
                }
            }
        });
    }

    pub fn clone_state(&self) -> Self {
        P2PNode {
            peers: Arc::clone(&self.peers),
            archive: Arc::clone(&self.archive),
        }
    }

    fn handle_peer_connection(&self, mut stream: TcpStream) {
        let peer_addr = match stream.peer_addr() {
            Ok(addr) => addr,
            Err(_) => {
                logger::warn("Conexão terminada antes de identificar o IP do peer");
                return;
            }
        };

        let peer_ip_u32 = if let IpAddr::V4(ipv4) = peer_addr.ip() {
            u32::from(ipv4)
        } else {
            logger::warn(&format!(
                "Conexão de peer IPv6 não suportada: {peer_addr}"
            ));
            return;
        };

        self.peers.lock().unwrap().add_peer(peer_ip_u32);

        logger::debug(&format!("Novo peer conectado: {peer_addr}"));
        let node_clone = self.clone_state();
        let stream_clone = stream.try_clone().expect("Falha ao clonar stream TCP");

        thread::spawn(move || {
            node_clone.peer_requester_thread(stream_clone);
        });

        loop {
            let mut msg_type_buf = [0u8; 1];
            match stream.read_exact(&mut msg_type_buf) {
                Ok(_) => {
                    if !MessageType::is_valid_message(msg_type_buf[0]) {
                        continue;
                    }

                    let msg_type = MessageType::from(msg_type_buf[0]);
                    if !self.handle_message(msg_type, &mut stream) {
                        break;
                    }
                }
                Err(_) => {
                    logger::info(&format!("Peer {peer_addr} desconectado."));
                    break;
                }
            }
        }

        self.peers.lock().unwrap().remove_peer(peer_ip_u32);
    }

    fn peer_requester_thread(&self, mut stream: TcpStream) {
        loop {
            thread::sleep(Duration::from_secs(5));

            logger::debug("Enviando pedido de lista de peers");
            if stream.write_all(&[MessageType::PeerRequest as u8]).is_err() {
                logger::warn("Falha ao enviar pedido de lista de peers.");
                break;
            }

            logger::debug("Enviando pedido de arquivo de chats");
            if stream
                .write_all(&[MessageType::ArchiveRequest as u8])
                .is_err()
            {
                logger::warn("Falha ao enviar pedido de arquivo de chats.");
                break;
            }

            if !self.handle_archive_request(&mut stream) {
                logger::warn("Falha ao propagar arquivo de chats para o peer.");
                break;
            }
        }
    }

    pub fn handle_message(&self, msg_type: MessageType, stream: &mut TcpStream) -> bool {
        match msg_type {
            MessageType::PeerRequest => self.handle_peer_request(stream),
            MessageType::PeerResponse => self.handle_peer_response(stream),
            MessageType::ArchiveRequest => self.handle_archive_request(stream),
            MessageType::ArchiveResponse => self.handle_archive_response(stream),
            MessageType::NotificationMessage => self.handle_notification_message(stream),
        }
    }

    fn handle_peer_request(&self, stream: &mut TcpStream) -> bool {
        logger::debug("Enviando lista de peers");
        let peer_list = self.peers.lock().unwrap();
        let response = peer_list.to_bytes();

        stream.write_all(&response).is_ok()
    }

    fn handle_peer_response(&self, stream: &mut TcpStream) -> bool {
        logger::debug("Recebendo lista de peers");

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

    fn handle_archive_request(&self, stream: &mut TcpStream) -> bool {
        logger::debug("Enviando arquivo de chats");
        let arch = self.archive.read().unwrap();

        if arch.len() > 0 {
            let response = arch.to_bytes();
            stream.write_all(&response).is_ok()
        } else {
            true
        }
    }

    pub fn connect_to_peer(&self, peer_addr: &str) {
        let addr = format!("{peer_addr}:{TCP_PORT}");
        let node_clone = Arc::new(self.clone_state());
        let peer_addr = peer_addr.to_string();

        thread::spawn(move || match TcpStream::connect(&addr) {
            Ok(stream) => {
                logger::info(&format!("Conectado com sucesso ao peer: {peer_addr}"));
                let node_clone_inner = Arc::clone(&node_clone);
                node_clone_inner.handle_peer_connection(stream);
            }
            Err(e) => logger::warn(&format!("Falha ao conectar ao peer {peer_addr}: {e}")),
        });
    }

    fn handle_archive_response(&self, stream: &mut TcpStream) -> bool {
        logger::debug("Recebendo arquivo de chats");
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

    fn handle_notification_message(&self, stream: &mut TcpStream) -> bool {
        let mut len_buf = [0u8; 1];

        if stream.read_exact(&mut len_buf).is_err() {
            logger::warn("Falha ao ler tamanho da mensagem de notificação.");
            return false;
        }

        let msg_len = len_buf[0] as usize;

        let mut msg_buf = vec![0u8; msg_len];
        if stream.read_exact(&mut msg_buf).is_err() {
            logger::warn("Falha ao ler mensagem de notificação.");
            return false;
        }

        match String::from_utf8(msg_buf) {
            Ok(msg) => {
                logger::debug(&format!("Notificação recebida: {msg}"));
                true
            }
            Err(_) => {
                logger::warn("Mensagem de notificação recebida não está em ASCII válido.");
                false
            }
        }
    }
}
