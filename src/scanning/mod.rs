#![allow(dead_code)]
use mdns_sd::{ServiceDaemon, ServiceEvent};

const MDNS_SCAN_TYPE: &str = "_adb-tls-connect._tcp.local.";
const MDNS_PAIRING_TYPE: &str = "_adb-tls-pairing._tcp.local.";

pub fn search_for_devices() {
    // Create a daemon
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");

    // Browse for a service type.
    let service_type = MDNS_PAIRING_TYPE;
    let receiver = mdns.browse(service_type).expect("Failed to browse");

    // Receive the browse events in sync or async. Here is
    // an example of using a thread. Users can call `receiver.recv_async().await`
    // if running in async environment.
    std::thread::spawn(move || {
        while let Ok(event) = receiver.recv() {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    println!("Resolved a new service: {}", info.get_fullname());
                }
                other_event => {
                    println!("Received other event: {:?}", &other_event);
                }
            }
        }
    });

    // Gracefully shutdown the daemon.
    std::thread::sleep(std::time::Duration::from_secs(30));
    mdns.shutdown().unwrap();
}
