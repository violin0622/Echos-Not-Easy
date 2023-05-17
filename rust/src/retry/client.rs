#![allow(unused_imports)]
use std::{
    fmt,
    io::{self, Error, ErrorKind, Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    thread::sleep,
    time::Duration,
};

use echos_not_easy::{connect_timeout_retry, Msg};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let mut correlation_id: u8 = 0;
    let mut buf: [u8; 4] = [0; 4];
    let sock_addr = "localhost:8080"
        .to_socket_addrs()?
        .into_iter()
        .next()
        .unwrap();
    'outer: loop {
        let mut sock = connect_timeout_retry(
            &sock_addr,
            // &SocketAddr::new("localhost".to_socket_addrs()?, 8080),
            std::time::Duration::from_secs(5),
        )
        .expect("Cannot connect to server");

        // 对于 EAGAIN/EWOULDBLOCK/EINTR , 直接重试；
        // 对于 ECONNREST/EPIPE ，重建连接并重试。
        loop {
            sleep(Duration::from_secs(1));
            let msg: [u8; 4] = Msg::new(correlation_id).try_into().unwrap();
            match sock.write_all(&msg[..]) {
                Ok(()) => println!("req: {correlation_id}"),
                Err(e) if e.kind() == ErrorKind::WouldBlock => continue,
                Err(e)
                    if e.kind() == ErrorKind::BrokenPipe
                        || e.kind() == ErrorKind::ConnectionReset =>
                {
                    continue 'outer
                }
                Err(e) => return Err(Box::new(e)),
            }

            match sock.read_exact(&mut buf[..]) {
                Ok(()) => {
                    println!("rep: {correlation_id}");
                    correlation_id += 1;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => continue,
                Err(e)
                    if e.kind() == ErrorKind::ConnectionReset
                        || e.kind() == ErrorKind::UnexpectedEof =>
                {
                    println!("rcv rep: {e}");
                    continue 'outer;
                }
                Err(e) => return Err(Box::new(e)),
            }
        }
    }
}
