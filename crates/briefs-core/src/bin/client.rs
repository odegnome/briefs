use briefs_core::Command;
use serde_json;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let server_addr = "192.168.1.16:8080";
    let mut stream = TcpStream::connect(server_addr).await.unwrap();
    println!("Connected with '{server_addr}'");

    let request = Command::Create {
        title: String::from("First Post"),
        msg: String::from(
            "My/Our first post. This is a demo post to test the proper \
            functioning of my/our new stream. Follow to keep yourself updated \
            with the latest updates.",
        ),
    };

    let bytes = stream
        .write(&serde_json::to_vec(&request).unwrap()[..])
        .await
        .unwrap();
    println!("Written {bytes} bytes");

    let mut kb_buffer = [0u8; 10240];
    stream.readable().await.unwrap();
    match stream.try_read(&mut kb_buffer) {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            let response = String::from_utf8(kb_buffer[..bytes].to_vec()).unwrap();
            println!("{}", response);
        }
        Err(e) => eprintln!("Error reading into buffer: {:?}", e),
    }
}
