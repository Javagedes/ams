use std::collections::HashMap;

use clap::Parser;

#[derive(clap::Parser, Debug)]
struct Args {
    port: u32,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let ams = ams_core::Ams::bind(format!("127.0.0.1:{}", args.port))
        .await
        .unwrap();

    let mut map = HashMap::new();

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        if input.trim() == "exit" {
            ams.shutdown().await;
            break;
        }

        if input.starts_with("connect") {
            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() != 2 {
                println!("Usage: connect <port>");
                continue;
            }
            let port: u32 = match parts[1].parse() {
                Ok(p) => p,
                Err(_) => {
                    println!("Invalid port number");
                    continue;
                }
            };
            let addr = format!("127.0.0.1:{port}");
            if let Some(new_conn) = ams.connect(addr.clone()).await {
                map.insert(port, new_conn);
                println!("Connected to {addr}");
            }
        } else if input.starts_with("send") {
            let parts: Vec<&str> = input.trim().splitn(3, ' ').collect();
            if parts.len() != 3 {
                println!("Usage: send <port> <message>");
                continue;
            }
            let port: u32 = match parts[1].parse() {
                Ok(p) => p,
                Err(_) => {
                    println!("Invalid port number");
                    continue;
                }
            };
            let message = parts[2];
            if let Some(&conn) = map.get(&port) {
                ams.send_message(ams_core::api::Message {
                    payload: message.to_string(),
                    sender: conn,
                    receiver: conn,
                })
                .await;
            } else {
                println!("No connection found for port {port}");
            }
        } else {
            println!("Unknown command");
        }
    }
}
