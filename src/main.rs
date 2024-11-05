#![feature(let_chains)]
mod adb_commands;
mod password;
mod scanning;

use std::env;

#[tokio::main]
async fn main() {
    let hostname = env::var("HOSTNAME").unwrap_or("localhost".to_string());
    let identifier = format!("ADBear@{hostname}");
    let password = crate::password::generate();
    fast_qr::QRBuilder::new(format!("WIFI:T:ADB;S:{identifier};P:{password};;"))
        .build()
        .expect("Failed to print QR code")
        .print();
    if let Ok(info) = scanning::find_pairing_service(&identifier).await {
        let port = info.get_port();
        let ip = info
            .get_addresses_v4()
            .iter()
            .next()
            .copied()
            .unwrap()
            .to_owned();
        crate::adb_commands::pair(ip, port, &password).expect("Failed to pair");
    }

    if let Ok(info) = scanning::find_connection_service().await {
        let port = info.get_port();
        let ip = info
            .get_addresses_v4()
            .iter()
            .next()
            .copied()
            .unwrap()
            .to_owned();
        if let Ok(output) = crate::adb_commands::connect(ip, port)
            && output.status.success()
        {
            if let Ok(output) = crate::adb_commands::get_device_name(ip, port) {
                println!(
                    "Connected to {device_name}",
                    device_name = String::from_utf8_lossy(&output.stdout)
                );
            }
        } else {
            println!("Failed to connect");
        }
    }
}
