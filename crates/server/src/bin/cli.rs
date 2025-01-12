#![allow(dead_code)]
use briefs_core::BriefsError;
use briefs_core::{state::CatchUpResponse, BriefsResult, Command};
use clap::{ArgAction, Parser, Subcommand};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::{net::IpAddr, path::PathBuf};
use tokio::io::AsyncReadExt;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::pki_types::pem::PemObject;
use tokio_rustls::rustls::pki_types::{CertificateDer, ServerName};
use tokio_rustls::{rustls, TlsConnector};

const BUFFER_SIZE: usize = 10240;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    /// Path to config file; defaults to ~/.briefs/config.toml
    config: Option<PathBuf>,

    #[arg(short, long)]
    /// The socket address of the briefs server. For ex, localhost:8080
    socket_addr: Option<SocketAddr>,

    #[arg(long)]
    cafile: Option<PathBuf>,

    #[arg(short, long, action = ArgAction::SetTrue)]
    /// Select if the output should be json
    json: bool,

    #[command(subcommand)]
    command: BriefsCommand,
}

#[derive(Subcommand, Clone, Debug)]
pub enum BriefsCommand {
    /// Creates a new post with the given `title` and `message`
    NewPost {
        #[arg(short, long)]
        title: Option<String>,
        #[arg(short, long)]
        msg: Option<String>,
    },

    /// briefs with the latest posts
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

async fn new_post(
    mut stream: TlsStream<TcpStream>,
    title: Option<String>,
    msg: Option<String>,
) -> BriefsResult<()> {
    let inner_title = title.unwrap_or_else(|| {
        print!("Enter post title: ");
        std::io::stdout().flush().unwrap();
        let mut buf = String::new();
        std::io::stdin()
            .read_line(&mut buf)
            .expect("Unable to read post title.");
        buf
    });

    let inner_msg = msg.unwrap_or_else(|| {
        println!("Enter post msg(Press Ctrl-d on new line to end): ");
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .expect("Unable to read post msg.");
        buf.replace("\n", " ").trim().into()
    });

    let request = Command::Create {
        title: inner_title,
        msg: inner_msg,
    };
    let _bytes = stream
        .write_all(&serde_json::to_vec(&request).unwrap().as_slice())
        .await
        .unwrap();
    // println!("Written {bytes} bytes");
    //
    // stream.flush().await.unwrap();
    // println!("Flushed data");
    stream.shutdown().await.unwrap();
    println!("Completed shutdown");

    // let mut kb_buffer = [0u8; BUFFER_SIZE];
    let mut kb_buffer = Vec::with_capacity(BUFFER_SIZE);
    // stream.readable().await.unwrap();
    match stream.read_to_end(&mut kb_buffer).await {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            let response = String::from_utf8(kb_buffer[..bytes].to_vec()).unwrap();
            println!("{}", response);
        }
        Err(e) => eprintln!("Error reading from stream: {:?}", e),
    };

    Ok(())
}

async fn briefs(
    mut stream: TlsStream<TcpStream>,
    starting_index: usize,
    json: bool,
) -> BriefsResult<()> {
    let request = Command::Catchup {
        last_fetch_id: starting_index,
    };
    stream
        .write_all(&serde_json::to_vec(&request).unwrap().as_slice())
        .await
        .unwrap();
    stream.shutdown().await.unwrap();
    // println!("Written {bytes} bytes");

    // let mut kb_buffer = [0u8; BUFFER_SIZE];
    let mut kb_buffer = Vec::with_capacity(BUFFER_SIZE);
    // stream.readable().await.unwrap();
    match stream.read_to_end(&mut kb_buffer).await {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            // let response = String::from_utf8(kb_buffer[..bytes].to_vec()).map_err(|_| {
            //     BriefsError::CustomError {
            //         msg: "Unable to decode UTF-8".into(),
            //     }
            // })?;
            let response = serde_json::from_slice::<crate::CatchUpResponse>(&kb_buffer[..bytes])?;
            // let response = serde_json::from_str::<crate::CatchUpResponse>(&response)?;
            if !json {
                println!("caught_up: {}", response.caught_up);
                for post in response.posts.into_iter() {
                    println!("{}", post);
                }
            } else {
                println!("{:#?}", response);
            }
        }
        Err(e) => eprintln!("Error reading from stream: {:?}", e),
    }
    Ok(())
}

async fn get_post(mut stream: TlsStream<TcpStream>, id: usize) -> BriefsResult<()> {
    let request = Command::Get { id };
    stream
        .write_all(&serde_json::to_vec(&request).unwrap().as_slice())
        .await
        .unwrap();
    // println!("Written {bytes} bytes");
    stream.shutdown().await.unwrap();

    // let mut kb_buffer = [0u8; BUFFER_SIZE];
    let mut kb_buffer = Vec::with_capacity(BUFFER_SIZE);
    // stream.readable().await.unwrap();
    match stream.read_to_end(&mut kb_buffer).await {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            // let response = String::from_utf8(kb_buffer[..bytes].to_vec()).map_err(|_| {
            //     BriefsError::CustomError {
            //         msg: "Unable to decode UTF-8".into(),
            //     }
            // })?;
            let response = serde_json::from_slice::<briefs_core::post::Post>(&kb_buffer[..bytes])?;
            println!("{:#?}", response);
        }
        Err(e) => eprintln!("Error reading from stream: {:?}", e),
    }
    Ok(())
}

async fn remove_post(mut stream: TlsStream<TcpStream>, id: usize) -> BriefsResult<()> {
    let request = Command::Delete { id };
    stream
        .write(&serde_json::to_vec(&request).unwrap().as_slice())
        .await
        .unwrap();
    stream.shutdown().await.unwrap();
    // println!("Written {bytes} bytes");

    // let mut kb_buffer = [0u8; BUFFER_SIZE];
    let mut kb_buffer = Vec::with_capacity(BUFFER_SIZE);
    // stream.readable().await.unwrap();
    match stream.read_to_end(&mut kb_buffer).await {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            // let response = String::from_utf8(kb_buffer[..bytes].to_vec()).map_err(|_| {
            //     BriefsError::CustomError {
            //         msg: "Unable to decode UTF-8".into(),
            //     }
            // })?;
            let response = serde_json::from_slice::<String>(&kb_buffer[..bytes]);
            println!("{:?}", response);
        }
        Err(e) => eprintln!("Error reading from stream: {:?}", e),
    }
    Ok(())
}

async fn update_msg(mut stream: TlsStream<TcpStream>, id: usize, msg: String) -> BriefsResult<()> {
    let request = Command::UpdateMsg { id, msg };
    stream
        .write(&serde_json::to_vec(&request).unwrap().as_slice())
        .await
        .unwrap();
    stream.shutdown().await.unwrap();
    // println!("Written {bytes} bytes");

    // let mut kb_buffer = [0u8; BUFFER_SIZE];
    let mut kb_buffer = Vec::with_capacity(BUFFER_SIZE);
    // stream.readable().await.unwrap();
    match stream.read_to_end(&mut kb_buffer).await {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            // let response = String::from_utf8(kb_buffer[..bytes].to_vec()).map_err(|_| {
            //     BriefsError::CustomError {
            //         msg: "Unable to decode UTF-8".into(),
            //     }
            // })?;
            let response = serde_json::from_slice::<String>(&kb_buffer[..bytes]);
            println!("{:?}", response);
        }
        Err(e) => eprintln!("Error reading from stream: {:?}", e),
    }
    Ok(())
}

async fn update_title(
    mut stream: TlsStream<TcpStream>,
    id: usize,
    title: String,
) -> BriefsResult<()> {
    let request = Command::UpdateTitle { id, title };
    stream
        .write(&serde_json::to_vec(&request).unwrap().as_slice())
        .await
        .unwrap();
    stream.shutdown().await.unwrap();
    // println!("Written {bytes} bytes");

    // let mut kb_buffer = [0u8; BUFFER_SIZE];
    let mut kb_buffer = Vec::with_capacity(BUFFER_SIZE);
    // stream.readable().await.unwrap();
    match stream.read_to_end(&mut kb_buffer).await {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            // let response = String::from_utf8(kb_buffer[..bytes].to_vec()).map_err(|_| {
            //     BriefsError::CustomError {
            //         msg: "Unable to decode UTF-8".into(),
            //     }
            // })?;
            let response = serde_json::from_slice::<String>(&kb_buffer[..bytes]);
            println!("{:?}", response);
        }
        Err(e) => eprintln!("Error reading from stream: {:?}", e),
    }
    Ok(())
}

async fn stream_metadata(mut stream: TlsStream<TcpStream>) -> BriefsResult<()> {
    let request = Command::Metadata {};
    stream
        .write(&serde_json::to_vec(&request).unwrap().as_slice())
        .await
        .unwrap();
    stream.shutdown().await.unwrap();
    // println!("Written {bytes} bytes");

    // let mut kb_buffer = [0u8; BUFFER_SIZE];
    let mut kb_buffer = Vec::with_capacity(BUFFER_SIZE);
    // stream.readable().await.unwrap();
    match stream.read_to_end(&mut kb_buffer).await {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            // let response = String::from_utf8(kb_buffer[..bytes].to_vec()).map_err(|_| {
            //     BriefsError::CustomError {
            //         msg: "Unable to decode UTF-8".into(),
            //     }
            // })?;
            let response =
                serde_json::from_slice::<briefs_core::state::StreamMetadata>(&kb_buffer[..bytes])?;
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
    // !-------
    // Shouldn't be hardcoded
    // -------!
    let domain = ServerName::try_from("brief.com").unwrap();

    let mut root_cert_store = rustls::RootCertStore::empty();
    if let Some(cafile) = &cli.cafile {
        for cert in CertificateDer::pem_file_iter(cafile).unwrap() {
            root_cert_store.add(cert.unwrap()).unwrap();
        }
    } else {
        root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    // !-------
    // Probably don't need Arc here
    // -------!
    let connector = TlsConnector::from(Arc::new(config));
    let stream = TcpStream::connect(socket).await.unwrap();

    let stream = connector.connect(domain, stream).await.unwrap();

    match cli.command {
        BriefsCommand::NewPost { title, msg } => new_post(stream, title, msg).await.unwrap(),
        BriefsCommand::Catchup { idx } => {
            let result = briefs(stream, idx.unwrap_or_default(), cli.json).await;
            if result.is_err() {
                eprintln!("ERROR: {}", result.unwrap_err());
            }
        }
        BriefsCommand::GetPost { id } => {
            let result = get_post(stream, id).await;
            if result.is_err() {
                eprintln!("ERROR: {}", result.unwrap_err());
            }
        }
        BriefsCommand::DeletePost { id } => {
            let result = remove_post(stream, id).await;
            if result.is_err() {
                eprintln!("ERROR: {}", result.unwrap_err());
            }
        }
        BriefsCommand::UpdateMsg { id, msg } => {
            let result = update_msg(stream, id, msg).await;
            if result.is_err() {
                eprintln!("ERROR: {}", result.unwrap_err());
            }
        }
        BriefsCommand::UpdateTitle { id, title } => {
            let result = update_title(stream, id, title).await;
            if result.is_err() {
                eprintln!("ERROR: {}", result.unwrap_err());
            }
        }
        BriefsCommand::StreamMetadata {} => {
            let result = stream_metadata(stream).await;
            if result.is_err() {
                eprintln!("ERROR: {}", result.unwrap_err());
            }
        }
    }
}
