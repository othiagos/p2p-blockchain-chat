mod constants;
mod core;
mod logger;
mod network;

use constants::TCP_PORT;
use network::P2PNode;
use std::env;
use std::io::{self, BufRead, Write};
use std::net::Ipv4Addr;

fn main() {
    let args: Vec<String> = env::args().collect();
    let initial_peer = args.get(1).cloned();

    logger::set_log_level(logger::LogLevel::Debug);

    logger::info("Iniciando Chat P2P com Blockchain...");
    if let Some(peer) = &initial_peer {
        logger::info(&format!("Tentando conectar ao peer inicial: {}", peer));
    } else {
        logger::info("Nenhum peer inicial especificado. Aguardando conexões...");
    }

    let node = P2PNode::new();
    node.start_listener();

    if let Some(peer_addr) = initial_peer {
        node.connect_to_peer(&peer_addr);
    }

    user_input_loop(&node);
}

fn user_input_loop(node: &P2PNode) {
    print_help();
    let stdin = io::stdin();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if stdin.lock().read_line(&mut input).is_err() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let command = parts[0];
        let args = &parts[1..];

        match command {
            "chat" | "c" => handle_chat(node, args),
            "history" | "h" => handle_history(node),
            "peers" | "p" => handle_peers(node),
            "status" | "s" => handle_status(node),
            "help" | "?" => print_help(),
            "quit" | "q" => break,
            _ => {
                println!(
                    "Comando desconhecido: '{}'. Digite 'help' para ver a lista de comandos.",
                    command
                );
            }
        }
    }
}

fn handle_chat(node: &P2PNode, args: &[&str]) {
    if args.is_empty() {
        eprintln!("Uso: chat <mensagem>");
        return;
    }
    let message = args.join(" ");
    let mut archive = node.archive.write().unwrap();
    if archive.add_message(message) {
        drop(archive);
        node.publish_archive();
    }
}

fn handle_history(node: &P2PNode) {
    let archive = node.archive.read().unwrap();
    if archive.chats.is_empty() {
        println!("O histórico de chats está vazio.");
    } else {
        println!("--- Histórico de Chats ({} mensagens) ---", archive.len());
        for (i, chat) in archive.chats.iter().enumerate() {
            println!("[{}]: {}", i, chat.message);
        }
        println!("-------------------------------------------");
    }
}

fn handle_peers(node: &P2PNode) {
    let peers = node.peers.lock().unwrap();
    let ips = peers.get_ips();
    if ips.is_empty() {
        println!("Nenhum peer conectado.");
    } else {
        println!("--- Peers Conhecidos ({}) ---", ips.len());
        for ip_u32 in ips {
            println!("- {}", Ipv4Addr::from(ip_u32));
        }
        println!("-----------------------------");
    }
}

fn handle_status(node: &P2PNode) {
    let peers_count = node.peers.lock().unwrap().get_ips().len();
    let archive_len = node.archive.read().unwrap().len();
    println!("--- Status do Nó ---");
    println!("Porta TCP: {}", TCP_PORT);
    println!("Peers conhecidos: {}", peers_count);
    println!("Mensagens no arquivo: {}", archive_len);
    println!("--------------------");
}

fn print_help() {
    println!("\nComandos disponíveis:");
    println!("  chat <mensagem> - Minera e envia uma nova mensagem");
    println!("  history         - Lista todo o histórico de chats");
    println!("  peers           - Mostra os peers conectados e conhecidos");
    println!("  status          - Exibe o status geral do nó");
    println!("  help            - Mostra esta ajuda");
    println!("  quit            - Sai do programa\n");
}
