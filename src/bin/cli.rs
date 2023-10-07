use catchup::{CatchupResult, Command};
use clap::{Parser, Subcommand};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use tokio::{io::AsyncWriteExt, net::TcpStream};

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    #[arg(short, long)]
    /// Path to config file; defaults to ~/.local/catchup/config.toml
    config: Option<PathBuf>,

    #[arg(short, long)]
    /// IP address of the catchup server
    ip: Option<IpAddr>,
    #[arg(short, long)]
    /// PORT of the catchup server
    port: Option<u16>,

    #[arg(short, long)]
    /// The socket address of the catchup server. For ex, localhost:8080
    socket_addr: Option<SocketAddr>,

    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Subcommand, Clone, Debug)]
pub enum CliCommand {
    /// Creates a new post with the given `title` and `body`
    New {
        #[arg(short, long)]
        title: String,
        #[arg(short, long)]
        body: String,
    },

    /// Catchup with the latest posts
    Catchup {},
}

async fn new_post(mut stream: TcpStream, title: String, body: String) -> CatchupResult<()> {
    let request = Command::Create { title, msg: body };
    stream.writable().await.unwrap();
    let bytes = stream
        .write(&serde_json::to_vec(&request).unwrap().as_slice())
        .await
        .unwrap();
    println!("Written {bytes} bytes");

    let mut kb_buffer = [0u8; 1024];
    stream.readable().await.unwrap();
    match stream.try_read(&mut kb_buffer) {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            let response = String::from_utf8(kb_buffer[..bytes].to_vec()).unwrap();
            println!("{}", response);
        }
        Err(e) => eprintln!("Error reading from stream: {:?}", e),
    };
    Ok(())
}

async fn catchup(mut stream: TcpStream) -> CatchupResult<()> {
    let request = Command::Catchup {};
    let bytes = stream
        .write(&serde_json::to_vec(&request).unwrap().as_slice())
        .await
        .unwrap();
    println!("Written {bytes} bytes");

    let mut kb_buffer = [0u8; 1024];
    stream.readable().await.unwrap();
    match stream.try_read(&mut kb_buffer) {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            let response = String::from_utf8(kb_buffer[..bytes].to_vec()).unwrap();
            println!("{}", response);
        }
        Err(e) => eprintln!("Error reading from stream: {:?}", e),
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let stream = TcpStream::connect(cli.socket_addr.unwrap()).await.unwrap();

    if let Some(cmd) = cli.command {
        println!("{:?}", cmd);
        match cmd {
            CliCommand::New { title, body } => new_post(stream, title, body).await.unwrap(),
            CliCommand::Catchup {} => catchup(stream).await.unwrap(),
        }
    }
}
