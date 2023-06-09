#![allow(unused_imports)]
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let sock = TcpListener::bind("127.0.0.1:8080")?;

    let mut buf = [0; 32];
    let (mut stream, _addr) = sock.accept()?;
    loop {
        let n = stream.read(&mut buf[..])?;
        if n == 0 {
            break;
        }
        println!("{}", String::from_utf8_lossy(&buf[..n]));
        stream.write(b"pong")?;
    }
    Ok(())
}
