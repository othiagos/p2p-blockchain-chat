mod p2p;

use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

const EXPECTED_ARGUMENTS: usize = 1;
const SOCKET_BIND_PORT: u16 = 51511;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let peer_node = Arc::new(Mutex::new(p2p::peer::PeerNode {
        peers: HashSet::new(),
        archive: Vec::new(),
    }));

    if args.len() >= EXPECTED_ARGUMENTS {
        let server_address: &str = args.get(1).unwrap().as_ref();
        let stream = TcpStream::connect((server_address, SOCKET_BIND_PORT))
            .await
            .unwrap_or_else(|e| {
                eprintln!(
                    "Failed to connect to {}: {}. Is the server running?",
                    server_address, e
                );
                std::process::exit(1);
            });

        p2p::peer::request_archive(peer_node.clone(), stream).await;
    }

    p2p::peer::handle_new_connection().await;
    p2p::peer::send_periodic_message().await;
    p2p::peer::handle_input(peer_node);
}
