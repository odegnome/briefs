use catchup_core::StreamError;
use catchup_core::{state::CatchUpResponse, CatchupResult, Command};
use clap::{Parser, Subcommand};
use std::net::{Ipv4Addr, SocketAddr};
use std::{net::IpAddr, path::PathBuf};
use tokio::{io::AsyncWriteExt, net::TcpStream};

const BUFFER_SIZE: usize = 10240;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    /// Path to config file; defaults to ~/.catchup/config.toml
    config: Option<PathBuf>,

    #[arg(short, long)]
    /// The socket address of the catchup server. For ex, localhost:8080
    socket_addr: Option<SocketAddr>,

    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand, Clone, Debug)]
pub enum CliCommand {
    /// Creates a new post with the given `title` and `message`
    NewPost {
        #[arg(short, long)]
        title: String,
        #[arg(short, long)]
        msg: String,
    },

    /// Catchup with the latest posts
    Catchup {
        idx: Option<usize>,
    },

    GetPost {
        id: usize,
    },

    DeletePost {
        id: usize,
    },

    UpdateMsg {
        id: usize,
        msg: String,
    },

    UpdateTitle {
        id: usize,
        title: String,
    },

    StreamMetadata {},
}

async fn new_post(mut stream: TcpStream, title: String, msg: String) -> CatchupResult<()> {
    let request = Command::Create { title, msg };
    stream.writable().await.unwrap();
    let bytes = stream
        .write(&serde_json::to_vec(&request).unwrap().as_slice())
        .await
        .unwrap();
    println!("Written {bytes} bytes");

    let mut kb_buffer = [0u8; BUFFER_SIZE];
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

async fn catchup(mut stream: TcpStream, starting_index: usize) -> CatchupResult<()> {
    let request = Command::Catchup {
        last_fetch_id: starting_index,
    };
    let bytes = stream
        .write(&serde_json::to_vec(&request).unwrap().as_slice())
        .await
        .unwrap();
    println!("Written {bytes} bytes");

    let mut kb_buffer = [0u8; BUFFER_SIZE];
    stream.readable().await.unwrap();
    match stream.try_read(&mut kb_buffer) {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            let response = String::from_utf8(kb_buffer[..bytes].to_vec()).map_err(|_| {
                StreamError::CustomError {
                    msg: "Unable to decode UTF-8".into(),
                }
            })?;
            let response = serde_json::from_str::<CatchUpResponse>(&response)?;
            println!("{:#?}", response);
        }
        Err(e) => eprintln!("Error reading from stream: {:?}", e),
    }
    Ok(())
}

async fn get_post(mut stream: TcpStream, id: usize) -> CatchupResult<()> {
    let request = Command::Get { id };
    let bytes = stream
        .write(&serde_json::to_vec(&request).unwrap().as_slice())
        .await
        .unwrap();
    println!("Written {bytes} bytes");

    let mut kb_buffer = [0u8; BUFFER_SIZE];
    stream.readable().await.unwrap();
    match stream.try_read(&mut kb_buffer) {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            let response = String::from_utf8(kb_buffer[..bytes].to_vec()).map_err(|_| {
                StreamError::CustomError {
                    msg: "Unable to decode UTF-8".into(),
                }
            })?;
            let response = serde_json::from_str::<catchup_core::post::Post>(&response)?;
            println!("{:#?}", response);
        }
        Err(e) => eprintln!("Error reading from stream: {:?}", e),
    }
    Ok(())
}

async fn remove_post(mut stream: TcpStream, id: usize) -> CatchupResult<()> {
    let request = Command::Delete { id };
    let bytes = stream
        .write(&serde_json::to_vec(&request).unwrap().as_slice())
        .await
        .unwrap();
    println!("Written {bytes} bytes");

    let mut kb_buffer = [0u8; BUFFER_SIZE];
    stream.readable().await.unwrap();
    match stream.try_read(&mut kb_buffer) {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            let response = String::from_utf8(kb_buffer[..bytes].to_vec()).map_err(|_| {
                StreamError::CustomError {
                    msg: "Unable to decode UTF-8".into(),
                }
            })?;
            println!("{:?}", response);
        }
        Err(e) => eprintln!("Error reading from stream: {:?}", e),
    }
    Ok(())
}

async fn update_msg(mut stream: TcpStream, id: usize, msg: String) -> CatchupResult<()> {
    let request = Command::UpdateMsg { id, msg };
    let bytes = stream
        .write(&serde_json::to_vec(&request).unwrap().as_slice())
        .await
        .unwrap();
    println!("Written {bytes} bytes");

    let mut kb_buffer = [0u8; BUFFER_SIZE];
    stream.readable().await.unwrap();
    match stream.try_read(&mut kb_buffer) {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            let response = String::from_utf8(kb_buffer[..bytes].to_vec()).map_err(|_| {
                StreamError::CustomError {
                    msg: "Unable to decode UTF-8".into(),
                }
            })?;
            println!("{:?}", response);
        }
        Err(e) => eprintln!("Error reading from stream: {:?}", e),
    }
    Ok(())
}

async fn update_title(mut stream: TcpStream, id: usize, title: String) -> CatchupResult<()> {
    let request = Command::UpdateTitle { id, title };
    let bytes = stream
        .write(&serde_json::to_vec(&request).unwrap().as_slice())
        .await
        .unwrap();
    println!("Written {bytes} bytes");

    let mut kb_buffer = [0u8; BUFFER_SIZE];
    stream.readable().await.unwrap();
    match stream.try_read(&mut kb_buffer) {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            let response = String::from_utf8(kb_buffer[..bytes].to_vec()).map_err(|_| {
                StreamError::CustomError {
                    msg: "Unable to decode UTF-8".into(),
                }
            })?;
            println!("{:?}", response);
        }
        Err(e) => eprintln!("Error reading from stream: {:?}", e),
    }
    Ok(())
}

async fn stream_metadata(mut stream: TcpStream) -> CatchupResult<()> {
    let request = Command::Metadata {};
    let bytes = stream
        .write(&serde_json::to_vec(&request).unwrap().as_slice())
        .await
        .unwrap();
    println!("Written {bytes} bytes");

    let mut kb_buffer = [0u8; BUFFER_SIZE];
    stream.readable().await.unwrap();
    match stream.try_read(&mut kb_buffer) {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            let response = String::from_utf8(kb_buffer[..bytes].to_vec()).map_err(|_| {
                StreamError::CustomError {
                    msg: "Unable to decode UTF-8".into(),
                }
            })?;
            let response = serde_json::from_str::<catchup_core::state::StreamMetadata>(&response)?;
            println!("{:#?}", response);
        }
        Err(e) => eprintln!("Error reading from stream: {:?}", e),
    }
    Ok(())
}

fn validate_socket(cli: &Cli) -> Result<SocketAddr, ()> {
    if let Some(socket_addr) = cli.socket_addr {
        return Ok(socket_addr);
    } else {
        return Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080));
    }
    //if let Some(config_file) = cli.config.to_owned() {
    //let config_file_exists = config_file.exists();
    //let is_file_path = config_file.is_file();

    //if !is_file_path || !config_file_exists {
    //eprintln!("Error: Path not for a file or doesn't exist");
    //return Err(());
    //}

    //unimplemented!();
    //}

    //Err(())
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let socket = validate_socket(&cli).unwrap();
    let stream = TcpStream::connect(socket).await.unwrap();

    match cli.command {
        CliCommand::NewPost { title, msg } => new_post(stream, title, msg).await.unwrap(),
        CliCommand::Catchup { idx } => {
            let result = catchup(stream, idx.unwrap_or_default()).await;
            if result.is_err() {
                eprintln!("ERROR: {}", result.unwrap_err());
            }
        }
        CliCommand::GetPost { id } => {
            let result = get_post(stream, id).await;
            if result.is_err() {
                eprintln!("ERROR: {}", result.unwrap_err());
            }
        }
        CliCommand::DeletePost { id } => {
            let result = remove_post(stream, id).await;
            if result.is_err() {
                eprintln!("ERROR: {}", result.unwrap_err());
            }
        }
        CliCommand::UpdateMsg { id, msg } => {
            let result = update_msg(stream, id, msg).await;
            if result.is_err() {
                eprintln!("ERROR: {}", result.unwrap_err());
            }
        }
        CliCommand::UpdateTitle { id, title } => {
            let result = update_title(stream, id, title).await;
            if result.is_err() {
                eprintln!("ERROR: {}", result.unwrap_err());
            }
        }
        CliCommand::StreamMetadata {} => {
            let result = stream_metadata(stream).await;
            if result.is_err() {
                eprintln!("ERROR: {}", result.unwrap_err());
            }
        }
    }
}
