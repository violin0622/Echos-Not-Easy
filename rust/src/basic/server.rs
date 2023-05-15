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
        stream.write(&buf[..n])?;
    }
}
