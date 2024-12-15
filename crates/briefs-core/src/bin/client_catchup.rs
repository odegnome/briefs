use briefs_core::Command;
use serde_json;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let server_addr = "192.168.1.16:8080";
    let mut tcp_stream = TcpStream::connect(server_addr).await.unwrap();
    println!("Connected with '{server_addr}'");

    let request = Command::Catchup { last_fetch_id: 0 };
    let bytes = tcp_stream
        .write(&serde_json::to_vec(&request).unwrap()[..])
        .await
        .unwrap();
    println!("Written {bytes} bytes");

    // 10MB buffer
    let mut kb_buffer = [0u8; 10240];
    tcp_stream.readable().await.unwrap();
    match tcp_stream.try_read(&mut kb_buffer) {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            let response = String::from_utf8(kb_buffer[..bytes].to_vec()).unwrap();
            println!("{}", response);
        }
        Err(e) => eprintln!("Error reading into buffer: {:?}", e),
    }
}
