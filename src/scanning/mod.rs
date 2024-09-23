use anyhow::anyhow;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::time::Duration;
use tokio::time::timeout;

const MDNS_SCAN_TYPE: &str = "_adb-tls-connect._tcp.local.";
const MDNS_PAIRING_TYPE: &str = "_adb-tls-pairing._tcp.local.";

pub async fn find_pairing_service(identifier: &str) -> anyhow::Result<ServiceInfo> {
    async fn inner(
        mdns: &ServiceDaemon,
        service_type: &str,
        identifier: &str,
    ) -> Option<ServiceInfo> {
        let receiver = mdns.browse(service_type).expect("Failed to browse");
        let service_name = format!("{identifier}.{MDNS_PAIRING_TYPE}");

        while let Ok(event) = receiver.recv_async().await {
            if let ServiceEvent::ServiceResolved(info) = event {
                if info.get_fullname() == service_name {
                    return Some(info);
                }
            }
        }
        None
    }
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");

    match timeout(
        Duration::from_secs(30),
        inner(&mdns, MDNS_PAIRING_TYPE, identifier),
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

pub async fn find_connection_service() -> anyhow::Result<ServiceInfo> {
    async fn inner(mdns: &ServiceDaemon, service_type: &str) -> Option<ServiceInfo> {
        let receiver = mdns.browse(service_type).expect("Failed to browse");

        while let Ok(event) = receiver.recv_async().await {
            if let ServiceEvent::ServiceResolved(info) = event {
                if info.get_fullname().ends_with(service_type) {
                    return Some(info);
                }
            }
        }
        None
    }

    let mdns = ServiceDaemon::new().expect("Failed to create daemon");

    match timeout(Duration::from_secs(30), inner(&mdns, MDNS_SCAN_TYPE)).await {
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
