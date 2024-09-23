#![allow(dead_code)]
use anyhow::anyhow;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use tokio::time::timeout;

const MDNS_SCAN_TYPE: &str = "_adb-tls-connect._tcp.local.";
const MDNS_PAIRING_TYPE: &str = "_adb-tls-pairing._tcp.local.";

pub async fn search_for_device(identifier: String) -> anyhow::Result<ServiceInfo> {
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    let service_name = format!("{identifier}.{MDNS_PAIRING_TYPE}");

    match timeout(
        std::time::Duration::from_secs(30),
        poll_device(&mdns, service_name),
    )
    .await
    {
        Ok(Some(info)) => {
            mdns.shutdown().unwrap();
            Ok(info)
        }
        Ok(None) => {
            mdns.shutdown().unwrap();
            Err(anyhow!("Device not found"))
        }
        Err(_) => {
            mdns.shutdown().unwrap();
            Err(anyhow!("Timeout"))
        }
    }
}

async fn poll_device(mdns: &ServiceDaemon, service_name: String) -> Option<ServiceInfo> {
    let service_type = MDNS_PAIRING_TYPE;
    let receiver = mdns.browse(service_type).expect("Failed to browse");

    while let Ok(event) = receiver.recv_async().await {
        match event {
            ServiceEvent::ServiceResolved(info) => {
                println!("Resolved a new service: {}", info.get_fullname());
                if info.get_fullname() == service_name {
                    return Some(info);
                }
            }
            other_event => {
                println!("Received other event: {:?}", &other_event);
            }
        }
    }
    None
}
