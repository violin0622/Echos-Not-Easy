#![allow(unused_imports)]
use std::{
    char,
    io::{self, ErrorKind, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    str,
    time::{self, Duration},
};

// connect_timeout_retry 会记录起始时间，然后尝试 connect, 如果成功则返回；
// 如果失败则检查是否超过了 timeout，如果超过了 timeout 则返回错误，否则会再次重试.
pub fn connect_timeout_retry(addr: &SocketAddr, timeout: Duration) -> io::Result<TcpStream> {
    let mut next_timeout = timeout;
    let begin = time::Instant::now();
    while !next_timeout.is_zero() {
        match TcpStream::connect_timeout(addr, next_timeout) {
            Ok(sock) => return Ok(sock),
            Err(e)
                if e.kind() == ErrorKind::ConnectionRefused
                    || e.kind() == ErrorKind::Interrupted =>
            {
                next_timeout = timeout.saturating_sub(begin.elapsed());
            }
            Err(e) => return Err(e),
        }
    }
    Err(io::Error::new(ErrorKind::TimedOut, "connect timed out"))
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Msg {
    pub num: u8,
    pub data: [u8; 3],
}

impl Msg {
    pub fn new(num: u8) -> Self {
        let mut msg = Self { num, data: [0; 3] };
        msg.data[0] = if num / 100 == 0 { b' ' } else { num / 100 };
        let b2 = num % 100 / 10;
        msg.data[1] = if b2 == 0 && msg.data[0] == b' ' {
            b' '
        } else {
            b2 + b'0'
        };
        msg.data[2] = num % 10 + b'0';

        msg
    }
}

impl TryFrom<[u8; 4]> for Msg {
    type Error = &'static str;
    fn try_from(buf: [u8; 4]) -> Result<Self, Self::Error> {
        let mut n = 0;
        for byte in buf.iter().skip(1) {
            match byte {
                b' ' => (),
                b'0'..=b'9' => n = n * 10 + (byte - b'0'),
                _ => return Err("invalid msg"),
            }
        }
        if n != buf[0] {
            return Err("invalid msg");
        }
        Ok(Self {
            num: buf[0],
            data: [buf[1], buf[2], buf[3]],
        })
    }
}

impl TryFrom<Msg> for [u8; 4] {
    type Error = &'static str;
    fn try_from(msg: Msg) -> Result<Self, Self::Error> {
        Ok([msg.num, msg.data[0], msg.data[1], msg.data[2]])
    }
}
