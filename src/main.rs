mod adb_commands;
mod password;
mod scanning;

use mdns_sd::ServiceDaemon;
use std::env;
use std::process::Output;

#[tokio::main]
async fn main() {
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");

    let hostname = env::var("HOSTNAME").unwrap_or("localhost".to_string());
    let identifier = format!("ADBear@{hostname}");
    let password = password::generate();
    fast_qr::QRBuilder::new(format!("WIFI:T:ADB;S:{identifier};P:{password};;"))
        .build()
        .expect("Failed to print QR code")
        .print();

    let info = scanning::find_pairing_service(&mdns, &identifier)
        .await
        .expect("Failed to find pairing service");
    let port = info.get_port();
    let addresses = info.get_addresses_v4();
    let ip = addresses.iter().next().unwrap();
    match adb_commands::pair(ip, port, &password) {
        Ok(Output { status, .. }) if status.success() => {}
        _ => {
            // https://stackoverflow.com/questions/33316006/adb-error-error-protocol-fault-couldnt-read-status-invalid-argument
            println!("Failed to pair, maybe need restart adb server")
        }
    }

    if let Ok(infos) = scanning::find_connection_service(&mdns).await {
        for (name, info) in infos.iter() {
            let port = info.get_port();
            let addresses = info.get_addresses_v4();
            let ip = addresses.iter().next().unwrap();
            let Ok(output) = adb_commands::connect(ip, port) else {
                println!("Failed to connect {name}");
                continue;
            };
            if output.status.success() && !output.stdout.starts_with(b"failed") {
                if let Ok(output) = adb_commands::get_device_name(ip, port) {
                    println!(
                        "Connected to {device_name}",
                        device_name = String::from_utf8_lossy(&output.stdout)
                    );
                }
            }
        }
    }

    mdns.shutdown().unwrap();
}
