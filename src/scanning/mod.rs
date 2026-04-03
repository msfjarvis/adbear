use anyhow::anyhow;
use mdns_sd::{ResolvedService, ServiceDaemon, ServiceEvent};
use std::time::Duration;
use tokio::time::timeout;

const MDNS_SCAN_TYPE: &str = "_adb-tls-connect._tcp.local.";
const MDNS_PAIRING_TYPE: &str = "_adb-tls-pairing._tcp.local.";

async fn find_mdns_service(
    mdns: &ServiceDaemon,
    service_type: &str,
    is_match: impl Fn(&ResolvedService) -> bool,
) -> Option<Box<ResolvedService>> {
    let receiver = mdns.browse(service_type).expect("Failed to browse");

    while let Ok(event) = receiver.recv_async().await {
        match event {
            ServiceEvent::ServiceResolved(info) => {
                if is_match(&info) {
                    return Some(info);
                }
            }
            ServiceEvent::SearchStopped(_) => break,
            _ => {}
        }
    }
    None
}

pub async fn find_pairing_service(
    mdns: &ServiceDaemon,
    identifier: &str,
) -> anyhow::Result<Box<ResolvedService>> {
    let expected_fullname = format!("{identifier}.{MDNS_PAIRING_TYPE}");

    match timeout(
        Duration::from_secs(30),
        find_mdns_service(mdns, MDNS_PAIRING_TYPE, |info| {
            info.get_fullname() == expected_fullname
        }),
    )
    .await
    {
        Ok(Some(info)) => Ok(info),
        Ok(None) => Err(anyhow!("Device not found")),
        Err(_) => Err(anyhow!("Timeout waiting for pairing service")),
    }
}

pub async fn find_connection_service(
    mdns: &ServiceDaemon,
    identifier: &str,
) -> anyhow::Result<Box<ResolvedService>> {
    let expected_prefix = format!("{identifier}.");

    match timeout(
        Duration::from_secs(30),
        find_mdns_service(mdns, MDNS_SCAN_TYPE, |info| {
            info.get_fullname().starts_with(&expected_prefix)
        }),
    )
    .await
    {
        Ok(Some(info)) => Ok(info),
        Ok(None) => Err(anyhow!("Device not found")),
        Err(_) => Err(anyhow!("Timeout waiting for connection service")),
    }
}
