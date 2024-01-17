#[cfg(not(test))]
use tokio::net::{TcpListener, ToSocketAddrs};
#[cfg(test)]
use turmoil::{net::TcpListener, ToSocketAddrs};

use snafu::prelude::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result {
    println!("Hello, world!");
    server("localhost:8080").await.whatever_context("")
}

pub type Result<T = (), E = snafu::Whatever> = std::result::Result<T, E>;

// Handle connections one by one.
pub async fn server<A: ToSocketAddrs>(addr: A) -> Result {
    let mut buf = [0; 32];
    let sock = TcpListener::bind(addr)
        .await
        .whatever_context("bind address")?;
    loop {
        let (mut stream, _) = sock.accept().await.whatever_context("accept connections")?;
        println!("receiving");
        match stream.read(&mut buf[..8]).await {
            Ok(0) => continue,
            Ok(n) if n != 8 => whatever!("w"),
            Ok(_) => {
                let num = u32::from_be_bytes(buf[4..8].try_into().whatever_context("")?);
                stream
                    .write(&[b"pong", &num.to_be_bytes()[..]].concat())
                    .await
                    .whatever_context("write response")?;
                println!("rsp sended!");
            }
            Err(_) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{io, time::SystemTime};

    use crate::server;
    use rand::SeedableRng;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        time::{timeout, Duration},
    };
    use turmoil::{
        net::{TcpListener, TcpStream},
        Builder, Result,
    };

    fn assert_error_kind<T>(res: io::Result<T>, kind: io::ErrorKind) {
        assert_eq!(res.err().map(|e| e.kind()), Some(kind));
    }

    fn new_sim() -> turmoil::Sim<'static> {
        Builder::new()
            .epoch(SystemTime::UNIX_EPOCH)
            .rng(rand::rngs::StdRng::seed_from_u64(10))
            .ip_version(turmoil::IpVersion::V6)
            .build()
    }

    const LOCALHOST: &str = "[::]:80";

    #[test]
    fn connect_twice() -> turmoil::Result {
        let mut sim = new_sim();
        sim.host("server", || async {
            let sock = TcpListener::bind(LOCALHOST).await?;
            loop {
                sock.accept().await?;
            }
        });
        sim.step()?;
        sim.client("client", async {
            TcpStream::connect("server:80").await?;
            println!("c1");
            TcpStream::connect("server:80").await?;
            println!("c2");
            Ok(())
        });
        sim.run()
    }

    #[test]
    fn reboot() -> turmoil::Result {
        let mut sim = new_sim();
        sim.host("s1", || async {
            server(LOCALHOST).await?;
            Ok(())
        });
        sim.client("c1", async {
            TcpStream::connect("s1:80").await?;
            println!("c1 connect succees");
            Ok(())
        });
        sim.run()?;
        sim.crash("s1");
        println!("s1 crash");
        sim.client("c2", async {
            assert!(
                timeout(Duration::from_millis(1), TcpStream::connect("s1:80"))
                    .await
                    .is_err()
            );
            println!("c2 connect should timeout");
            Ok(())
        });
        sim.run()?;
        sim.bounce("s1");
        println!("s1 recover");
        sim.run()?;
        sim.client("c3", async {
            TcpStream::connect("s1:80").await?;
            println!("c3 connect succees");
            Ok(())
        });
        sim.run()
    }

    #[test]
    fn echo() -> Result {
        let mut sim = new_sim();
        sim.host("s1", || async {
            server(LOCALHOST).await?;
            Ok(())
        });
        sim.client("c1", async {
            let mut sock = TcpStream::connect("s1:80").await?;
            let mut buf = [0; 32];
            for i in 0u32..10 {
                sock.write(&[b"ping", &i.to_be_bytes()[..]].concat())
                    .await?;
                if let 0 = sock.read(&mut buf[..4]).await? {
                    break;
                }
                assert_eq!(b"pong", &buf[..4]);
                assert_eq!(i, sock.read_u32().await?);
            }
            Ok(())
        });
        sim.run()
    }

    #[test]
    fn network_partitions_during_connect() -> turmoil::Result {
        let mut sim = new_sim();

        sim.host("server", || async {
            server(LOCALHOST).await?;
            Ok(())
        });

        sim.client("client", async {
            turmoil::partition("client", "server");
            assert_error_kind(
                TcpStream::connect("server:80").await,
                io::ErrorKind::ConnectionRefused,
            );
            turmoil::repair("client", "server");

            turmoil::hold("client", "server");
            assert!(
                timeout(Duration::from_secs(1), TcpStream::connect(("server", 80)))
                    .await
                    .is_err()
            );
            turmoil::repair("client", "server");

            TcpStream::connect("server:80").await?;
            Ok(())
        });

        sim.run()
    }
}
