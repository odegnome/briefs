mod error;
pub mod interfaces;

pub use error::ServerError;

use briefs_core::{Command, StreamCommand, db::setup_db};
use tokio_rustls::server::TlsStream;
use std::path::PathBuf;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{mpsc, oneshot},
};

/// 10Kb buffer
const BUFFER_SIZE: usize = 10240;
pub const POSTS_TABLE: &str = "posts";

pub mod interprocess {
    use super::oneshot;

    pub enum Status {
        Success,
        Failure,
        Undefined,
    }

    pub struct InterProcessStatus {
        // data: Vec<u8> <-- Maybe?
        // error: Vec<u8> <-- Maybe?
        pub status: Status,
        pub code: u32,
        pub message: [u8; 60],
    }

    impl InterProcessStatus {
        pub fn new(status: Status, code: u32, message: [u8; 60]) -> Self {
            Self {
                status,
                code,
                message,
            }
        }
    }

    /// Sending a response over a oneshot channel returns the input value
    /// as the error. So, no point in error handling thus this function.
    pub fn respond_with_string(responder: oneshot::Sender<String>, msg: String) {
        let _ = responder.send(msg);
    }

    pub fn respond_with_bytes(responder: oneshot::Sender<Vec<u8>>, msg: Vec<u8>) {
        let _ = responder.send(msg);
    }
}

pub fn setup_server(db_path: Option<PathBuf>) -> anyhow::Result<()> {
    setup_db(db_path)?;
    Ok(())
}

pub async fn handle_conn_request(
    mut conn: TlsStream<TcpStream>,
    tx: mpsc::Sender<StreamCommand>,
) {
    println!("Succesfully connected with {:?}", conn.get_ref().0.peer_addr());

    // let mut kb_buffer = [0u8; BUFFER_SIZE];
    let mut kb_buffer = Vec::with_capacity(BUFFER_SIZE);

    match conn.read_to_end(&mut kb_buffer).await {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            let cmd = serde_json::from_slice::<Command>(&kb_buffer[..bytes]).unwrap();
            println!("{:?}", cmd);
            let (responder, sender) = oneshot::channel();
            let wrapped_cmd = StreamCommand {
                cmd,
                resp: Some(responder),
            };
            tx.send(wrapped_cmd).await.unwrap();
            let result = sender.await.unwrap();
            // println!("CONN:\n{}", result);
            conn.write_all(result.as_slice()).await.unwrap();
            conn.shutdown().await.unwrap();
        }
        Err(e) => eprintln!("Error reading into buffer: {:?}", e),
    }
}

