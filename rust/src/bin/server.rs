#![allow(unused_imports)]
use std::{
    io::{self, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    str, thread,
    time::Duration,
};

use echos_not_easy::Msg;

fn main() {
    println!("Hello, world!");

    let listener = TcpListener::bind("localhost:8080").expect("Cannot bind to port");
    loop {
        let (sock, addr) = listener.accept().expect("Cannot accept connection");
        println!("new client: {addr:?}");
        if let Err(e) = handle_echo(sock) {
            println!("client {addr:?} error: {e}");
        }
    }
}

// handle_echo 读取客户端发送的数据，睡眠一秒后，将其原样返回。
fn handle_echo(mut sock: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf: [u8; 4] = [0; 4];
    loop {
        let req: Msg;
        match sock.read_exact(&mut buf[..]) {
            Ok(()) => {
                req = Msg::try_from(buf)?;
                println!("req: {}", req.num);
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => continue,
            Err(e) => {
                println!("rcv req: {e}");
                return Err(Box::new(e));
            }
        }

        let rep: [u8; 4] = req.try_into()?;
        match sock.write_all(&rep[..]) {
            Ok(()) => println!("rep: {}", rep[0]),
            Err(e) if e.kind() == ErrorKind::WouldBlock => continue,
            Err(e) => {
                println!("snd rep: {e}");
                return Err(Box::new(e));
            }
        }
    }
}
