use std::{
    io,
    net::Ipv4Addr,
    process::{Command, Output, Stdio},
};

pub fn pair(ip: Ipv4Addr, port: u16, password: &str) -> io::Result<Output> {
    Command::new("adb")
        .arg("pair")
        .arg(format!("{ip}:{port}"))
        .arg(password)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
}

pub fn connect(ip: Ipv4Addr, port: u16) -> io::Result<Output> {
    Command::new("adb")
        .arg("connect")
        .arg(format!("{ip}:{port}"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
}

pub fn get_device_name(ip: Ipv4Addr, port: u16) -> io::Result<Output> {
    Command::new("adb")
        .arg("-s")
        .arg(format!("{ip}:{port}"))
        .arg("shell")
        .arg("getprop")
        .arg("ro.product.model")
        .output()
}

#[derive(Debug, PartialEq)]
pub enum ConnectOutcome {
    Connected,
    AlreadyConnected,
    Failed(String),
}

pub fn parse_connect_output(output: &Output) -> ConnectOutcome {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    let combined = combined.trim();

    if combined.starts_with("already connected to") {
        ConnectOutcome::AlreadyConnected
    } else if combined.starts_with("connected to") {
        ConnectOutcome::Connected
    } else {
        ConnectOutcome::Failed(combined.to_string())
    }
}
