use mdns_sd::{ServiceDaemon, ServiceEvent};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const SERVICE_TYPE: &str = "_ypa-scp._tcp.local.";
//const SERVICE_TYPE: &str = "_wled._tcp.local.";
//const SERVICE_TYPE: &str = "_airplay._tcp.local."; 

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveredDevice {
    pub hostname: String,
    pub port: u16,
}

pub struct YamahaDevice {
    pub hostname: String,
    pub port: u16,
}

pub fn scan_for_devices(timeout_secs: u64) -> Vec<DiscoveredDevice> {
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    let receiver = mdns.browse(SERVICE_TYPE).expect("Failed to browse");
    
    let mut devices: Vec<DiscoveredDevice> = Vec::new();
    let start = std::time::Instant::now();

    while start.elapsed() < Duration::from_secs(timeout_secs) {
        if let Ok(event) = receiver.recv_timeout(Duration::from_millis(500))
            && let ServiceEvent::ServiceResolved(info) = event 
        {
            let hostname = info.get_hostname().trim_end_matches('.').to_string();
            
            if !devices.iter().any(|d| d.hostname == hostname) {
                devices.push(DiscoveredDevice {
                    hostname,
                    port: info.get_port(),
                });
            }
        }
    }
    
    let _ = mdns.shutdown();
    devices
}

pub fn find(hostname: &str) -> YamahaDevice {
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    let receiver = mdns.browse(SERVICE_TYPE).expect("Failed to browse");

    while let Ok(event) = receiver.recv() {
        if let ServiceEvent::ServiceResolved(info) = event {
            let found = info.get_hostname();

            if found == hostname {
                println!("Found! {} {}", found, info.get_port());
                let _ = mdns.shutdown();
                return YamahaDevice {
                    hostname: found.to_string(),
                    port: info.get_port(),
                };
            }
        }
    }

    panic!("mDNS channel closed unexpectedly.");
}