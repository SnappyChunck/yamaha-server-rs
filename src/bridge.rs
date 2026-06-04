use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, watch};
use tokio_tungstenite::{accept_async, tungstenite::Message as WsMessage};

use crate::protocol::{Config, ConfigCommand, GetCommand, StateMessage, Message as AppMessage};

#[derive(Clone)]
pub struct ClientEvent {
    pub sender_id: usize,
    pub msg: AppMessage,
}

fn parse_message(text: &str) -> Option<AppMessage> {
    match serde_json::from_str::<AppMessage>(text) {
        Ok(msg) => Some(msg),
        Err(_) => {
            eprintln!("Failed to parse message: {text}");
            None
        }
    }
}

pub async fn handle_connection(
    stream: TcpStream,
    sync_tx: broadcast::Sender<ClientEvent>,
    rcp_tx: mpsc::Sender<String>,
    config_tx: watch::Sender<Config>,
    client_id: usize,
) {
    let Ok(ws_stream) = accept_async(stream).await else {
        eprintln!("WebSocket handshake failed");
        return;
    };

    println!("Client {client_id} connected");
    let (mut ws_write, mut ws_read) = ws_stream.split();
    let mut sync_rx = sync_tx.subscribe();

    loop {
        tokio::select! {
            msg_opt = ws_read.next() => {
                match msg_opt {
                    Some(Ok(WsMessage::Text(text))) => {
                        if let Some(msg) = parse_message(&text) {
                            println!("<= Client {client_id}: {text}");
                            match &msg {
                                AppMessage::Config(ConfigCommand::SetHost { host }) => {
                                    println!("Config: host set to {host}");
                                    config_tx.send_modify(|c| c.host = Some(host.clone()));
                                    let _ = sync_tx.send(ClientEvent { sender_id: client_id, msg });
                                }
                                AppMessage::Config(ConfigCommand::SetDeviceMode { mode }) => {
                                    println!("Config: mode set to {mode:?}");
                                    config_tx.send_modify(|c| c.mode = *mode);
                                    let _ = sync_tx.send(ClientEvent { sender_id: client_id, msg });
                                }
                                AppMessage::Get(GetCommand::Config) => {
                                    let current_config = config_tx.borrow().clone();
                                    let reply = AppMessage::State(StateMessage::ConfigState {
                                        host: current_config.host,
                                        mode: current_config.mode,
                                    });
                                    let json = serde_json::to_string(&reply).unwrap();
                                    let _ = ws_write.send(WsMessage::text(json)).await;
                                }
                                AppMessage::Get(GetCommand::Scan) => {
                                    let tx = sync_tx.clone();
                                    
                                    tokio::task::spawn_blocking(move || {
                                        let devices = crate::discovery::scan_for_devices(5);
                                        let msg = AppMessage::State(StateMessage::DeviceList { devices });
                                        
                                        let _ = tx.send(ClientEvent { sender_id: usize::MAX, msg });
                                    });
                                }
                                AppMessage::Get(get_cmd) => {
                                    let mode = config_tx.borrow().mode;
                                    if let Some(rcp) = crate::protocol::build_rpc_get(get_cmd, mode) {
                                        let _ = rcp_tx.send(rcp).await;
                                    }
                                }
                                AppMessage::Command(cmd) => {
                                    let mode = config_tx.borrow().mode;
                                    let rcp = crate::protocol::build_rpc_command(cmd, mode);
                                    let _ = rcp_tx.send(rcp).await;

                                    let _ = sync_tx.send(ClientEvent { sender_id: client_id, msg: msg.clone() });
                                }
                                _ => {
                                    println!("Unsupported type");
                                }
                            }
                        }
                    }
                    Some(Ok(WsMessage::Close(_))) | None => {
                        println!("Client {client_id} disconnected");
                        break;
                    }
                    _ => {}
                }
            }

            sync_res = sync_rx.recv() => {
                match sync_res {
                    Ok(event) => {
                        if event.sender_id == client_id {
                            continue;
                        }
                        let json = serde_json::to_string(&event.msg)
                            .unwrap_or_else(|_| "{}".to_string());
                        if ws_write.send(WsMessage::text(json)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        eprintln!("Client {client_id} lagged, dropped {n} frame(s)");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
}