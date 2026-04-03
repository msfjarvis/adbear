use anyhow::anyhow;
use if_addrs::get_if_addrs;
use mdns_sd::{ResolvedService, ServiceDaemon, ServiceEvent};
use std::collections::HashSet;
use std::net::Ipv4Addr;
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

/// Given a set of candidate IPv4 addresses (from mDNS), pick the one that
/// shares a subnet with one of our local network interfaces. Falls back to
/// an arbitrary address when no match is found.
pub fn pick_best_ipv4(candidates: HashSet<Ipv4Addr>) -> Option<Ipv4Addr> {
    let interfaces: Vec<(Ipv4Addr, Ipv4Addr)> = get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|iface| {
            if let if_addrs::IfAddr::V4(v4) = iface.addr {
                Some((v4.ip, v4.netmask))
            } else {
                None
            }
        })
        .collect();
    pick_best_ipv4_with_interfaces(candidates, &interfaces)
}

fn pick_best_ipv4_with_interfaces(
    candidates: HashSet<Ipv4Addr>,
    interfaces: &[(Ipv4Addr, Ipv4Addr)],
) -> Option<Ipv4Addr> {
    let same_subnet = |local: Ipv4Addr, mask: Ipv4Addr, remote: Ipv4Addr| -> bool {
        let local_octets = u32::from(local);
        let mask_octets = u32::from(mask);
        let remote_octets = u32::from(remote);
        (local_octets & mask_octets) == (remote_octets & mask_octets)
    };

    // Prefer any candidate that shares a subnet with a local interface.
    for candidate in &candidates {
        for &(local_ip, netmask) in interfaces {
            if same_subnet(local_ip, netmask, *candidate) {
                return Some(*candidate);
            }
        }
    }

    // Fallback: return any candidate.
    candidates.into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(addrs: &[&str]) -> HashSet<Ipv4Addr> {
        addrs.iter().map(|s| s.parse().unwrap()).collect()
    }

    /// When only one candidate is present, it must be returned regardless.
    #[test]
    fn single_candidate_returned() {
        let result = pick_best_ipv4(set(&["192.168.1.50"]));
        assert_eq!(result, Some("192.168.1.50".parse().unwrap()));
    }

    /// When the set is empty, None is returned.
    #[test]
    fn empty_returns_none() {
        assert_eq!(pick_best_ipv4(set(&[])), None);
    }

    /// Prefer a LAN address over a Tailscale address when both are present.
    /// We simulate "local interfaces" by passing them explicitly to the inner
    /// helper so the test is deterministic and doesn't depend on the host machine.
    #[test]
    fn prefers_lan_over_tailscale() {
        // Simulated local interface: 192.168.1.10/24  (mask 255.255.255.0)
        let local_ip: Ipv4Addr = "192.168.1.10".parse().unwrap();
        let netmask: Ipv4Addr = "255.255.255.0".parse().unwrap();

        let candidates = set(&["10.0.0.37", "192.168.1.50"]);
        let result = pick_best_ipv4_with_interfaces(candidates, &[(local_ip, netmask)]);
        assert_eq!(result, Some("192.168.1.50".parse().unwrap()));
    }

    /// When no candidate shares a subnet, fall back to any candidate (non-None).
    #[test]
    fn fallback_when_no_subnet_match() {
        // Local interface on a completely different subnet
        let local_ip: Ipv4Addr = "172.16.0.1".parse().unwrap();
        let netmask: Ipv4Addr = "255.255.0.0".parse().unwrap();

        let candidates = set(&["10.0.0.37", "192.168.1.50"]);
        let result = pick_best_ipv4_with_interfaces(candidates, &[(local_ip, netmask)]);
        assert!(result.is_some());
    }

    /// When multiple candidates match, any one of the matching ones is returned.
    #[test]
    fn multiple_matching_candidates_returns_one_of_them() {
        let local_ip: Ipv4Addr = "192.168.1.10".parse().unwrap();
        let netmask: Ipv4Addr = "255.255.255.0".parse().unwrap();

        let candidates = set(&["192.168.1.50", "192.168.1.51"]);
        let result = pick_best_ipv4_with_interfaces(candidates.clone(), &[(local_ip, netmask)]);
        assert!(result.is_some());
        assert!(candidates.contains(&result.unwrap()));
    }
}
