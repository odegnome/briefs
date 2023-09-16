use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8878").await.unwrap();
    println!("Created stream");

    let _result = stream.write(b"Hello World").await.unwrap();
}
