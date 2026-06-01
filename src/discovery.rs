use mdns_sd::{ServiceDaemon, ServiceEvent};

//const SERVICE_TYPE: &str = "_ypa-scp._tcp.local.";
const SERVICE_TYPE: &str = "_wled._tcp.local.";

pub struct YamahaDevice {
    pub hostname: String,
    pub port: u16,
}

pub fn scan_and_print() {
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    let receiver = mdns.browse(SERVICE_TYPE).expect("Failed to browse");

    println!("No --yamaha-host given. Scanning for Yamaha SCP devices (Ctrl+C to exit):\n");

    while let Ok(event) = receiver.recv() {
        if let ServiceEvent::ServiceResolved(info) = event {
            let hostname = info.get_hostname();
            println!("{} {}", hostname, info.get_port());
        }
    }
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