use anyhow::anyhow;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::time::Duration;
use tokio::time::timeout;

const MDNS_SCAN_TYPE: &str = "_adb-tls-connect._tcp.local.";
const MDNS_PAIRING_TYPE: &str = "_adb-tls-pairing._tcp.local.";

async fn find_mdns_service<MatchFn>(
    mdns: &ServiceDaemon,
    service_type: &str,
    is_match: MatchFn,
) -> Option<ServiceInfo>
where
    MatchFn: Fn(&ServiceInfo) -> bool,
{
    let receiver = mdns.browse(service_type).expect("Failed to browse");

    while let Ok(event) = receiver.recv_async().await {
        if let ServiceEvent::ServiceResolved(info) = event {
            if is_match(&info) {
                return Some(info);
            }
        }
    }
    None
}

pub async fn find_pairing_service(identifier: &str) -> anyhow::Result<ServiceInfo> {
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    let service_type = format!("{identifier}.{MDNS_PAIRING_TYPE}");

    match timeout(
        Duration::from_secs(30),
        find_mdns_service(&mdns, &service_type, |info| {
            info.get_fullname() == service_type
        }),
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
        find_mdns_service(&mdns, MDNS_SCAN_TYPE, |info| {
            info.get_fullname().ends_with(MDNS_SCAN_TYPE)
        }),
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
