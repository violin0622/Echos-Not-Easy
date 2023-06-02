#![allow(unused_imports)]
use std::{
    io::{Read, Write},
    net::TcpStream,
    result::Result,
    thread::sleep,
    time::Duration,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let mut stream = TcpStream::connect("127.0.0.1:8080")?;

    let mut buf = [0; 32];
    loop {
        sleep(Duration::from_secs(1));
        stream.write(b"ping")?;
        let n = stream.read(&mut buf[..])?;
        println!("{}", String::from_utf8_lossy(&buf[..n]));
    }
}
