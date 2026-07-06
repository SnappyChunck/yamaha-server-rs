use clap::Parser;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, watch};

use bridge::{ClientEvent, handle_connection};
use protocol::{Config, DeviceMode};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::time::Duration;

mod discovery;
mod protocol;
mod bridge;

#[derive(Parser)]
#[command(name = "yamaha-bridge")]
struct Cli {
    #[arg(long, default_value = "127.0.0.1:8080")]
    address: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let (sync_tx, _) = broadcast::channel::<ClientEvent>(64);
    let (rcp_tx, mut rcp_rx) = mpsc::channel::<String>(100);
    let (config_tx, mut config_rx) = watch::channel(Config {
        host: None,
        mode: DeviceMode::Tf1,
    });

    let sync_tx_yamaha = sync_tx.clone();

    tokio::spawn(async move {
        loop {
            let host = loop {
                let current_host = config_rx.borrow().host.clone();
                if let Some(h) = current_host {
                    break h;
                }
                if config_rx.changed().await.is_err() { return; }
            };

            println!("Yamaha task: Resolving mDNS for '{}'...", host);
            
            let host_clone = host.clone();
            let device = match tokio::task::spawn_blocking(move || discovery::find(&host_clone)).await {
                Ok(d) => d,
                Err(_) => continue,
            };

            let target = format!("{}:{}", device.hostname, device.port);
            println!("Yamaha task: Connecting to TCP {}...", target);

            match TcpStream::connect(&target).await {
                Ok(yamaha_stream) => {
                    println!("Successfully connected to Yamaha at {}!", target);
                    let (yamaha_read, mut yamaha_write) = yamaha_stream.into_split();
                    let mut read_lines = BufReader::new(yamaha_read).lines();

                    loop {
                        tokio::select! {
                            Ok(()) = config_rx.changed() => {
                                let new_host = config_rx.borrow().host.clone();
                                if new_host != Some(host.clone()) {
                                    println!("Config changed, reconnecting Yamaha...");
                                    break; 
                                }
                            }
                            
                            Some(rcp) = rcp_rx.recv() => {
                                if let Err(e) = yamaha_write.write_all(rcp.as_bytes()).await {
                                    eprintln!("Failed to write to Yamaha: {e}");
                                    break;
                                }
                            }

                            res = read_lines.next_line() => {
                                match res {
                                    Ok(Some(line)) => {
                                        if line.starts_with("ERROR") {
                                            eprintln!("Yamaha Error: {}", line);
                                        } else if let Some(state_msg) = crate::protocol::parse_yamaha_response(&line) {
                                            let msg = crate::protocol::Message::State(state_msg);
                                            let _ = sync_tx_yamaha.send(ClientEvent { sender_id: 999, msg });
                                        } else {
                                            println!("Yamaha: {}", line);
                                        }
                                    }
                                    _ => {
                                        eprintln!("Yamaha disconnected.");
                                        break; 
                                    }
                                }
                            }
                        }
                    }
                    eprintln!("Yamaha connection lost. Waiting 5s before reconnecting...");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
                Err(e) => {
                    eprintln!("Connection to Yamaha failed: {e}. Retrying in 5s...");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });

    let listener = TcpListener::bind(&cli.address).await.expect("Failed to bind");
    println!("WebSocket server listening on ws://{}", &cli.address);

    let mut next_client_id: usize = 0;
    while let Ok((stream, _)) = listener.accept().await {
        let client_id = next_client_id;
        next_client_id += 1;
        tokio::spawn(handle_connection(
            stream,
            sync_tx.clone(),
            rcp_tx.clone(),
            config_tx.clone(),
            client_id,
        ));
    }
}
