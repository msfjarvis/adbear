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
    if let Ok(info) = scanning::search_for_device(identifier).await {
        for addr in info.get_addresses_v4() {
            println!("Found device at: {addr}");
        }
    }
}
