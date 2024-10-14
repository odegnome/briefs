mod error;

pub use error::ServerError;

use catchup_core::{Command, StreamCommand};
use sqlite;
use std::{net::SocketAddr, path::PathBuf, process};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::{mpsc, oneshot},
};

/// 10Kb buffer
const BUFFER_SIZE: usize = 10240;
const DB_NAME: &str = "catchup-dev.db";

pub mod database {
    use super::*;

    pub fn setup_tables(path: PathBuf) -> anyhow::Result<()> {
        let conn = sqlite::open(path)?;

        let statement = "
            CREATE TABLE IF NOT EXISTS posts
            (id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            body TEXT NOT NULL);
        ";

        conn.execute(statement)?;
        Ok(())
    }

    pub fn query_table_info(path: PathBuf) -> anyhow::Result<()> {
        let conn = sqlite::open(path)?;

        let statement = "
            PRAGMA table_info(posts);
        ";

        let mut stmt = conn.prepare(statement)?;

        for row in stmt.iter() {
            println!("{:?}", row?);
        }
        Ok(())
    }

    pub fn create_db(path: PathBuf) -> anyhow::Result<()> {
        let _conn = sqlite::open(path.as_path())?;

        Ok(())
    }
}

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
                println!("Given path does not exist, creating a new db");

                if !inner_path.to_str().unwrap().ends_with(".db") {
                    database::create_db(inner_path.join(DB_NAME))?;
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

pub fn setup_server() -> anyhow::Result<()> {
    Ok(())
}

pub fn setup_connection_handler() -> anyhow::Result<()> {
    Ok(())
}

pub async fn handle_conn_request(
    mut conn: (TcpStream, SocketAddr),
    tx: mpsc::Sender<StreamCommand>,
) {
    println!("Succesfully connected with {:?}", conn.1);

    conn.0.readable().await.unwrap();

    let mut kb_buffer = [0u8; BUFFER_SIZE];

    match conn.0.try_read(&mut kb_buffer) {
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
            println!("CONN:\n{}", result);
            conn.0.write(result.as_bytes()).await.unwrap();
        }
        Err(e) => eprintln!("Error reading into buffer: {:?}", e),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_setup_db() {
        // Setup Db with path
        let path = std::env::current_dir().unwrap();
        setup_db(Some(path.clone())).unwrap();
        let updated_path = path.join(DB_NAME);

        assert!(updated_path.exists(), "Db creation failed at expected path");

        std::fs::remove_file(updated_path).expect("Db cleanup failed");

        // Setup Db w/o path
        setup_db(None).unwrap();
        let path = std::env::temp_dir().join(DB_NAME);

        assert!(path.exists(), "Db creation failed at expected path");

        std::fs::remove_file(path).expect("Db cleanup failed");
    }

    #[test]
    fn test_setup_tables() {
        setup_db(None).unwrap();
        let path = std::env::temp_dir().join(DB_NAME);

        assert!(path.exists(), "Db creation failed at expected path");

        assert!(database::setup_tables(path.clone()).is_ok());
        assert!(database::query_table_info(path.clone()).is_ok());

        std::fs::remove_file(path).expect("Db cleanup failed");
    }
}
