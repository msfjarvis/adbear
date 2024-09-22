mod password;

use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hostname = env::var("HOSTNAME").unwrap_or("localhost".to_string());
    let password = crate::password::generate_password();
    qr2term::print_qr(format!("WIFI:T:ADB;S:ADBear@{hostname};P:{password};;"))
        .map_err(|e| e.into())
}
