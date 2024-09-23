mod password;
mod scanning;

use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hostname = env::var("HOSTNAME").unwrap_or("localhost".to_string());
    let password = crate::password::generate_password();
    fast_qr::QRBuilder::new(format!("WIFI:T:ADB;S:ADBear@{hostname};P:{password};;"))
        .build()
        .expect("Failed to print QR code")
        .print();
    crate::scanning::search_for_devices();
    Ok(())
}
