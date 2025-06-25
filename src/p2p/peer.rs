use std::collections::HashSet;
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use tokio::time::interval;
use std::io::{self, BufRead};
use std::sync::Arc;

const SOCKET_BIND_ADDRESS: &str = "[::]";
const SOCKET_BIND_PORT: u16 = 51511;
const VERIFICATION_CODE_SIZE: usize = 16;
const MD5_HASH_SIZE: usize = 16;

#[derive(Debug, Clone)]
pub struct Chat {
    pub message: String,
    pub verification_code: [u8; VERIFICATION_CODE_SIZE],
    pub md5_hash: [u8; MD5_HASH_SIZE],
}

#[derive(Debug, Clone)]
pub struct PeerNode {
    pub peers: HashSet<Ipv4Addr>,
    pub archive: Vec<Chat>,
}

fn peer_request() {

} 

fn peer_list() {

}

fn archive_request() {

}

fn archive_response() {

}

fn send_message_for_peer() {
    println!("Sending message to peer...");
}

fn handle_incoming(_socket: TcpStream, addr: SocketAddr) {
    println!("New connection from {:?}", addr);
}

pub async fn request_archive(peer_node: Arc<Mutex<PeerNode>>, mut stream: TcpStream) {
    tokio::spawn(async move {

        let buf = vec![0x1];
        stream.write_all(&buf).await.unwrap_or_else(|e| {
            eprintln!("Failed to send data: {}", e);
            std::process::exit(1);
        });

        let mut size_to_read = vec![0; 5];
        stream
            .read_exact(&mut size_to_read)
            .await
            .unwrap_or_else(|e| {
                eprintln!("Failed to read data: {}", e);
                std::process::exit(1);
            });

        // Interpret bytes 1..5 as a big-endian u32 for the size
        let size = u32::from_be_bytes([
            size_to_read[1],
            size_to_read[2],
            size_to_read[3],
            size_to_read[4],
        ]);

        let mut buf = vec![0; size as usize];
        let bytes_read = stream.read(&mut buf).await.unwrap_or_else(|e| {
            eprintln!("Failed to read data: {}", e);
            std::process::exit(1);
        });

        println!("Received response 0x2: {:?}", &buf[..bytes_read]);

        // let buf = vec![0x3];
        // stream.write_all(&buf).unwrap_or_else(|e| {
        //     eprintln!("Failed to send data: {}", e);
        //     std::process::exit(1);
        // });

        // let mut buff = vec![0; 1024];
        // let bytes_read = stream.read(&mut buff).unwrap_or_else(|e| {
        //     eprintln!("Failed to read data: {}", e);
        //     std::process::exit(1);
        // });

        // println!("Received response 0x4: {:?}", &buff[..bytes_read]);
        // println!("Message {}", String::from_utf8_lossy(&buff[..bytes_read]));
        });
}

pub async fn handle_new_connection() {
    tokio::spawn(async move {
        let addr = format!("{}:{}", SOCKET_BIND_ADDRESS, SOCKET_BIND_PORT);

        let listener = TcpListener::bind(addr).await.unwrap_or_else(|err| {
            eprintln!("Failed to bind to {}:{} {}. Is the server already running?", SOCKET_BIND_ADDRESS, SOCKET_BIND_PORT, err);
            std::process::exit(1);
        });

        loop {
            println!("Accepted connection from");
            let (socket, addr) = listener.accept().await.unwrap();
            handle_incoming(socket, addr);
        }
    });
}

pub async fn send_periodic_message() {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(5));

        loop {
            interval.tick().await;
            send_message_for_peer();
        }
    });
}

fn send_message(message: &str) { 
    println!("Mensagem digitada: {}", message);
}

pub fn handle_input(peer_node: Arc<Mutex<PeerNode>>) {
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    loop {
        let mut input = String::new();
        if handle.read_line(&mut input).is_ok() {
            let message = input.trim();
            send_message(message);
            println!("{:?}", peer_node);
        }
    }
}
