# P2P Blockchain Chat

A peer-to-peer (P2P) chat with an immutable message history based on a simplified blockchain, developed in Rust.

---

## Table of Contents

- [Description](#description)
- [Protocol](#protocol)
- [Requirements](#requirements)
- [Build](#build)
- [Run](#run)
- [Available Commands](#available-commands)

---

## Description

This project implements a decentralized chat system where each message is recorded in a blockchain-like structure, ensuring integrity and an immutable history. Nodes connect directly via TCP, forming a P2P network.

---

## Protocol

### 1. Peer Discovery (P2P)

- Initial connection to a known peer
- Exchange of `PeerRequest` (`0x1`) and `PeerList` (`0x2`) messages
- Connection to all newly discovered peers
- Periodic sending of `PeerRequest` every 5 seconds

### 2. Chat History (simple blockchain)

- Uses `ArchiveRequest` (`0x3`) and `ArchiveResponse` (`0x4`) messages
- Each chat message includes:
  - Length (1 byte)
  - ASCII text
  - Verifier code (16 bytes)
  - MD5 hash (16 bytes)
- History validation:
  - Last hash starts with two zero bytes
  - Hash matches the sequence of the last 20 chats (excluding the final hash)
  - Recursive validation of the previous history

### 3. Sending Messages

- Verifier code must be mined until the hash conditions are met
- A new chat history is created with the mined message
- The new history is broadcast to all peers via `ArchiveResponse`

### 4. Notifications (optional)

- `NotificationMessage` (`0x5`) is used to report errors or inconsistencies

### Message Table

| Type                  | Code  | Description                                                        |
|-----------------------|-------|--------------------------------------------------------------------|
| `PeerRequest`         | `0x1` | Requests the peer list                                             |
| `PeerList`            | `0x2` | Returns a list of IPs (4 bytes per peer)                          |
| `ArchiveRequest`      | `0x3` | Requests the chat history                                          |
| `ArchiveResponse`     | `0x4` | Sends the full validated chat history                              |
| `NotificationMessage` | `0x5` | Reports errors or unexpected situations (optional message)         |

---

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (stable version)
- Cargo (Rust’s package manager)

Install Rust quickly:

```sh
curl https://sh.rustup.rs -sSf | sh
```

Check the installation:

```sh
rustc --version
```

---

## Build

Debug mode (development):

```sh
cargo build
```

Release mode (optimized):

```sh
cargo build --release
```

---

## Run

Start a node waiting for connections:

```sh
cargo run
```

Or connect to an existing peer:

```sh
cargo run -- <PEER_IP>
```

Example:

```sh
cargo run -- 127.0.0.1
```

---

## Available Commands

In the chat prompt, use:

- `chat <message>` — Mines and sends a new message to the network
- `history` — Lists the full chat history
- `peers` — Shows connected and known peers
- `status` — Displays the current node status
- `addpeer <ip>` — Manually connects to a new peer
- `filechat <file>` — Sends messages from a text file (one per line)
- `help` — Lists all available commands
- `quit` — Exits the program

---
