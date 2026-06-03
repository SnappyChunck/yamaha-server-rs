use clap::Parser;

use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc, watch};

use bridge::{ClientEvent, handle_connection};
use protocol::{Config, DeviceMode};

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

    let (sync_tx, _)    = broadcast::channel::<ClientEvent>(64);
    let (rcp_tx, mut rcp_rx) = mpsc::channel::<String>(100);
    let (config_tx, mut config_rx) = watch::channel(Config {
        host: None,
        mode: DeviceMode::Tf1,
    });

    tokio::spawn(async move {
        loop {
            if config_rx.borrow().host.is_none() {
                println!("Yamaha task: waiting for a host...");
                if config_rx.changed().await.is_err() { break; }
                continue;
            }

            let config = config_rx.borrow().clone();
            println!("Yamaha stub: would connect to {:?} as {mode:?}",
                config.host, mode = config.mode);

            loop {
                tokio::select! {
                    Some(rcp) = rcp_rx.recv() => {
                        println!("=> Yamaha stub: {}", rcp.trim());
                    }
                    Ok(()) = config_rx.changed() => {
                        println!("Config changed, reconnecting...");
                        break;
                    }
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

    /*match cli.yamaha_host {
        None => discovery::scan_and_print(),
        Some(host) => {
            let device = discovery::find(&host);

            println!("Connected to {} {}", device.hostname, device.port);

            let target = format!("{}:{}", device.hostname, device.port);

            match TcpStream::connect(&target).await {
                Ok(_) => {
                    println!("TCP connected to {}", target);
                    tokio::spawn(bridge::handle_connection());
                },
                Err(e) => eprintln!("TCP failed: {}", e),
            }
        }
    }*/
}
