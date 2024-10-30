mod error;
pub mod interfaces;

pub use error::ServerError;

use catchup_core::{Command, StreamCommand};
use rand::{thread_rng, Rng};
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
pub const POSTS_TABLE: &str = "posts";

pub mod database {
    use std::time::SystemTime;

    use catchup_core::post::Post;
    use sqlite::Connection;

    use super::*;

    pub trait DbInsertString {
        fn db_insert_string(&self) -> anyhow::Result<String>;
    }

    impl DbInsertString for Post {
        fn db_insert_string(&self) -> anyhow::Result<String> {
            // \"\" are needed, otherwise the insertion will fail.
            Ok(format!(
                "{},\"{}\",\"{}\",{},{}",
                self.id()
                    .map_err(|_| ServerError::custom_error("Unable to load post ID".into()))?,
                self.title,
                self.msg,
                self.date.duration_since(SystemTime::UNIX_EPOCH)?.as_secs(),
                self.edited
            ))
        }
    }

    pub fn setup_tables(conn: &mut Connection) -> anyhow::Result<()> {
        let statement = format!(
            "
            CREATE TABLE IF NOT EXISTS {POSTS_TABLE} 
            (id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            msg TEXT NOT NULL,
            date INTEGER NOT NULL,
            edited BOOLEAN);
        "
        );

        conn.execute(statement)?;

        Ok(())
    }

    pub fn query_table_info(
        conn: &mut Connection,
        table_name: &str,
    ) -> anyhow::Result<Vec<sqlite::Row>> {
        let statement = format!("PRAGMA table_info({table_name});");

        let mut stmt = conn.prepare(statement)?;

        let result: Vec<sqlite::Row> = stmt.iter().filter_map(|val| val.ok()).collect();

        Ok(result)
    }

    pub fn create_db(path: PathBuf) -> anyhow::Result<()> {
        let _conn = sqlite::open(path.as_path())?;

        Ok(())
    }

    pub fn insert_post(conn: &mut Connection, table_name: &str, data: &Post) -> anyhow::Result<()> {
        let value_string = data.db_insert_string()?;
        let statement = format!("INSERT INTO {} VALUES ({})", table_name, value_string);

        conn.execute(statement)?;

        Ok(())
    }

    pub fn query_posts(
        conn: &mut Connection,
        table_name: &str,
        posts_limit: Option<u32>,
    ) -> anyhow::Result<Vec<sqlite::Row>> {
        let statement = format!(
            "SELECT * FROM {} LIMIT {};",
            table_name,
            posts_limit.unwrap_or(20)
        );

        let mut stmt = conn.prepare(statement)?;

        let result: Vec<sqlite::Row> = stmt.iter().filter_map(|val| val.ok()).collect();

        Ok(result)
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

pub fn generate_random_db_name() -> String {
    let mut buffer = [0u16; 4];
    thread_rng().fill(&mut buffer);
    let mut result = buffer
        .into_iter()
        .map(|val| format!("{:05}", val.to_be()))
        .collect::<Vec<String>>()
        .join("-");
    result.insert_str(0, "catchup-");
    result.push_str(".db");
    result
}

pub fn generate_temp_db() -> PathBuf {
    let random_db_name = generate_random_db_name();
    let temp_dir = std::env::temp_dir().join(random_db_name);
    temp_dir
}

#[cfg(test)]
mod test {
    use std::time::SystemTime;

    use catchup_core::post::Post;
    use sqlite::Value;

    use super::*;

    fn generate_random_db_name() -> String {
        let mut buffer = [0u16; 4];
        thread_rng().fill(&mut buffer);
        let mut result = buffer
            .into_iter()
            .map(|val| format!("{:05}", val.to_be()))
            .collect::<Vec<String>>()
            .join("-");
        result.insert_str(0, "catchup-");
        result.push_str(".db");
        result
    }

    #[test]
    fn test_generate_random_db_name() {
        for _ in 0..5 {
            let db_name = generate_random_db_name();
            assert!(db_name.starts_with("catchup-"));
            assert!(db_name.ends_with(".db"));
            assert!(db_name.len() == 34);
        }
    }

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
        let db_name = generate_random_db_name();
        let path = std::env::temp_dir().join(db_name.clone());
        setup_db(Some(path.clone())).unwrap();

        assert!(path.exists(), "Db creation failed at expected path");

        let mut conn = sqlite::open(path.clone()).unwrap();
        let result = database::setup_tables(&mut conn);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());

        let result = database::query_table_info(&mut conn, POSTS_TABLE);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());

        //----- Expected values
        let expected_rows = 5u8;
        let expected_columns = [
            Value::String("id".into()),
            Value::String("title".into()),
            Value::String("msg".into()),
            Value::String("date".into()),
            Value::String("edited".into()),
        ];
        //-----

        let mut actual_rows = 0u8;
        let mut actual_columns: Vec<Value> = Vec::new();
        for mut row in result.unwrap().into_iter() {
            actual_rows += 1;
            // println!("{:?}", row);
            actual_columns.push(row.take(1));
        }
        assert!(actual_rows == expected_rows, "Number of rows don't match");
        assert_eq!(actual_columns, expected_columns);

        std::fs::remove_file(path).expect("Db cleanup failed");
    }

    #[test]
    fn test_insert_into_table() {
        let db_name = generate_random_db_name();
        let path = std::env::temp_dir().join(db_name.clone());
        setup_db(Some(path.clone())).unwrap();

        assert!(path.exists(), "Db creation failed at expected path");

        let mut conn = sqlite::open(path.clone()).unwrap();
        let result = database::setup_tables(&mut conn);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());

        let result = database::query_table_info(&mut conn, POSTS_TABLE);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());

        let mut conn = sqlite::open(path.clone()).unwrap();
        let post = Post::new(
            0,
            "Post #1".into(),
            "Hello there, this is my first post".into(),
        )
        .unwrap();
        let result = database::insert_post(&mut conn, POSTS_TABLE, &post);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());

        let result = database::query_posts(&mut conn, POSTS_TABLE, None);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());

        //----- Expected values
        let expected_rows = 1;
        let expected_columns = [
            Value::Integer(0),
            Value::String(post.title),
            Value::String(post.msg),
            Value::Integer(
                post.date
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .try_into()
                    .expect("Error: Unable to convert timestamp to sqlite integer"),
            ),
            Value::Integer(0),
        ];
        //-----

        let mut row_data = result.unwrap();
        let actual_rows = row_data.len();
        assert_eq!(actual_rows, expected_rows);

        let actual_columns = [
            row_data[0].take(0),
            row_data[0].take(1),
            row_data[0].take(2),
            row_data[0].take(3),
            row_data[0].take(4),
        ];
        assert_eq!(actual_columns, expected_columns);

        std::fs::remove_file(path).expect("Db cleanup failed");
    }
}
