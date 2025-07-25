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

    logger::set_log_level(logger::LogLevel::Off);

    logger::info("Iniciando Chat P2P com Blockchain...");
    if let Some(peer) = &initial_peer {
        logger::info(&format!("Tentando conectar ao peer inicial: {peer}"));
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
            "c" | "chat" => handle_chat(node, args),
            "h" | "history" => handle_history(node),
            "p" | "peers" => handle_peers(node),
            "s" | "status" => handle_status(node),
            "a" | "addpeer" => handle_addpeer(node, args),
            "f" | "filechat" => handle_filechat(node, args),
            "?" | "help" => print_help(),
            "q" | "quit" => break,
            _ => {
            println!(
                "Comando desconhecido: '{command}'. Digite 'help' para ver a lista de comandos."
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

    archive.add_message(message);
}

fn handle_history(node: &P2PNode) {
    let archive = node.archive.read().unwrap();
    if archive.chats.is_empty() {
        println!("O histórico de chats está vazio.");
    } else {
        println!("--- Histórico de Chats ({} mensagens) ---", archive.len());
        for (i, chat) in archive.chats.iter().enumerate() {
            let width = archive.len().to_string().len();
            println!("[{:0w$}] {}", i, chat.message, w = width);
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
    println!("Porta TCP: {TCP_PORT}");
    println!("Peers conhecidos: {peers_count}");
    println!("Mensagens no arquivo: {archive_len}");
    println!("--------------------");
}

fn handle_addpeer(node: &P2PNode, args: &[&str]) {
    if let Some(ip) = args.first() {
        node.connect_to_peer(ip);
    } else {
        println!("Uso: addpeer <ip>");
    }
}

fn handle_filechat(node: &P2PNode, args: &[&str]) {
    if args.is_empty() {
        eprintln!("Uso: filechat <caminho_do_arquivo>");
        return;
    }

    let file_path = args[0];
    match std::fs::File::open(file_path) {
        Ok(file) => {
            let reader = io::BufReader::new(file);
            for msg in reader.lines().map_while(Result::ok) {
                handle_chat(node, &[&msg]);
            }
        }
        Err(e) => {
            eprintln!("Erro ao abrir o arquivo '{file_path}': {e}");
        }
    }
}

fn print_help() {
    println!("\nComandos disponíveis:");
    println!("  chat <mensagem>         - Minera e envia uma nova mensagem");
    println!("  history                 - Lista todo o histórico de chats");
    println!("  peers                   - Mostra os peers conectados e conhecidos");
    println!("  status                  - Exibe o status geral do nó");
    println!("  addpeer <ip>            - Adiciona e conecta a um novo peer pelo IP");
    println!("  filechat <arquivo>      - Envia mensagens de um arquivo texto");
    println!("  help                    - Mostra esta ajuda");
    println!("  quit                    - Sai do programa\n");
}
