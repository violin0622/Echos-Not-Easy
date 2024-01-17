use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, world!");

    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;

    let mut buf = [0; 32];
    loop {
        stream.write(b"ping").await?;
        let n = stream.read(&mut buf[..]).await?;
        println!("{}", String::from_utf8_lossy(&buf[..n]));
    }
}
