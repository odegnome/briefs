mod error;
pub mod interfaces;

pub use error::ServerError;

use briefs_core::{Command, StreamCommand};
use rand::{thread_rng, Rng};
use sqlite;
use tokio_rustls::server::TlsStream;
use std::{path::PathBuf, process};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{mpsc, oneshot},
};

/// 10Kb buffer
const BUFFER_SIZE: usize = 10240;
const DB_NAME: &str = "briefs-dev.db";
pub const POSTS_TABLE: &str = "posts";

pub mod interprocess {
    use super::oneshot;

    pub enum Status {
        Success,
        Failure,
        Undefined,
    }

    pub struct InterProcessStatus {
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

// pub mod database {
//     use briefs_core::db::*;
// }

/// path - Can be either a complete file path(with .db suffix) or
///        a directory name which will then be appended with default
///        db name.
pub fn setup_db(path: Option<PathBuf>) -> anyhow::Result<()> {
    // Check if sqlite3 is installed
    let sqlite3_check = process::Command::new("sqlite3")
        .arg("-version")
        .output()
        .expect("sqlite3 not installed");

    if !sqlite3_check.status.success() {
        return Err(ServerError::SqliteError {
            msg: String::from_utf8(sqlite3_check.stderr)
                .expect("Unable to parse sqlite3 error to string"),
        }
        .into());
    };

    println!(
        "Found sqlite3: {}",
        String::from_utf8(sqlite3_check.stdout).expect("Unable to parse sqlite3 stdout")
    );

    // Setup Db
    match path {
        Some(inner_path) => {
            if !inner_path.try_exists()? || inner_path.is_dir() {
                println!("{inner_path:?} does not exist or is a directory; creating a new db");

                if !inner_path.to_str().unwrap().ends_with(".db") {
                    database::create_db(inner_path.join(DB_NAME))?;
                } else {
                    database::create_db(inner_path)?;
                }
            }
        }
        None => {
            let db_path = std::env::temp_dir().join(DB_NAME);
            database::create_db(db_path)?;
        }
    }

    Ok(())
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

