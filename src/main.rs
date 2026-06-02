use protocol::DeviceMode;

use clap::Parser;
use tokio::net::{TcpStream};

mod discovery;
mod protocol;
mod bridge;

#[derive(Parser)]
#[command(name = "yamaha-bridge")]
struct Cli {
    #[arg(long)]
    yamaha_host: Option<String>,

    #[arg(short, long, value_enum, default_value_t = DeviceMode::Tio)]
    mode: DeviceMode,

    #[arg(long, default_value = "127.0.0.1:8080")]
    address: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.yamaha_host {
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
    }
}
