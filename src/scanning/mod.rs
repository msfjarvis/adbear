use anyhow::anyhow;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::{sleep, timeout};

const MDNS_SCAN_TYPE: &str = "_adb-tls-connect._tcp.local.";
const MDNS_PAIRING_TYPE: &str = "_adb-tls-pairing._tcp.local.";

async fn find_mdns_service(
    mdns: &ServiceDaemon,
    service_type: &str,
    is_match: impl Fn(&ServiceInfo) -> bool,
) -> Option<ServiceInfo> {
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

pub async fn find_pairing_service(
    mdns: &ServiceDaemon,
    identifier: &str,
) -> anyhow::Result<ServiceInfo> {
    let service_type = MDNS_PAIRING_TYPE.to_string();

    match timeout(
        Duration::from_secs(30),
        find_mdns_service(&mdns, MDNS_PAIRING_TYPE, |info| {
            info.get_fullname() == format!("{}.{service_type}", identifier)
        }),
    )
    .await
    {
        Ok(Some(info)) => Ok(info),
        Ok(None) => Err(anyhow!("Device not found")),
        Err(_) => Err(anyhow!("Timeout")),
    }
}

pub async fn find_connection_service(
    mdns: &ServiceDaemon,
) -> anyhow::Result<HashMap<String, ServiceInfo>> {
    let map = HashMap::<String, ServiceInfo>::new();

    let map = Arc::new(Mutex::new(map));
    let map_clone = map.clone();

    let task = async move {
        let receiver = mdns.browse(MDNS_SCAN_TYPE).expect("Failed to browse");
        while let Ok(event) = receiver.recv_async().await {
            if let ServiceEvent::ServiceResolved(info) = event {
                if info.get_fullname().ends_with(MDNS_SCAN_TYPE) {
                    println!(
                        "Found service: {} at port {}",
                        info.get_fullname(),
                        info.get_port()
                    );
                    let mut map = map_clone.lock().unwrap();
                    map.insert(info.get_fullname().to_string(), info);
                }
            }
        }
    };

    tokio::select!(
        _ = sleep(Duration::from_secs(3)) => {}
        _ = task => {}
    );

    let lock = map.lock().unwrap();
    Ok(lock.clone())
}
