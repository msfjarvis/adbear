use anyhow::anyhow;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::time::Duration;
use tokio::time::timeout;

const MDNS_SCAN_TYPE: &str = "_adb-tls-connect._tcp.local.";
const MDNS_PAIRING_TYPE: &str = "_adb-tls-pairing._tcp.local.";

enum ServiceNameMatcher {
    Exact,
    Suffix,
}

async fn find_mdns_service(
    mdns: &ServiceDaemon,
    service_type: &str,
    matcher: ServiceNameMatcher,
) -> Option<ServiceInfo> {
    let receiver = mdns.browse(service_type).expect("Failed to browse");

    while let Ok(event) = receiver.recv_async().await {
        if let ServiceEvent::ServiceResolved(info) = event {
            match matcher {
                ServiceNameMatcher::Exact => {
                    if info.get_fullname() == service_type {
                        return Some(info);
                    }
                }
                ServiceNameMatcher::Suffix => {
                    if info.get_fullname().ends_with(service_type) {
                        return Some(info);
                    }
                }
            }
        }
    }
    None
}

pub async fn find_pairing_service(identifier: &str) -> anyhow::Result<ServiceInfo> {
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");

    match timeout(
        Duration::from_secs(30),
        find_mdns_service(
            &mdns,
            &format!("{identifier}.{MDNS_PAIRING_TYPE}"),
            ServiceNameMatcher::Exact,
        ),
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
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");

    match timeout(
        Duration::from_secs(30),
        find_mdns_service(&mdns, MDNS_SCAN_TYPE, ServiceNameMatcher::Suffix),
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
