use std::{
    io,
    net::Ipv4Addr,
    process::{Command, Output, Stdio},
};

pub fn pair(ip: &Ipv4Addr, port: u16, password: &str) -> io::Result<Output> {
    Command::new("adb")
        .arg("pair")
        .arg(format!("{ip}:{port}"))
        .arg(password)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
}

pub fn connect(ip: &Ipv4Addr, port: u16) -> io::Result<Output> {
    Command::new("adb")
        .arg("connect")
        .arg(format!("{ip}:{port}"))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
}

pub fn get_device_name(ip: &Ipv4Addr, port: u16) -> io::Result<Output> {
    Command::new("adb")
        .arg("-s")
        .arg(format!("{ip}:{port}"))
        .arg("shell")
        .arg("getprop ro.product.model")
        .output()
}
