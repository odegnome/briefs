use crate::{post::Post, BriefsError};
use sqlite::Connection;
use std::{path::PathBuf, process};
use rand::{thread_rng, Rng};

const DB_NAME: &str = "briefs-dev.db";
pub const POSTS_TABLE: &str = "posts";

pub trait DbInsertString {
    /// A trait for converting the underlying data into Db friendly
    /// syntax. Currently, only implemented for `Post`.
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    fn db_insert_string(&self) -> anyhow::Result<String>;
}

impl DbInsertString for Post {
    fn db_insert_string(&self) -> anyhow::Result<String> {
        // \"\" are needed, otherwise the insertion will fail.
        Ok(format!(
            "{},\"{}\",\"{}\",{},{}",
            self.id()
                .map_err(|_| BriefsError::custom_error("Unable to load post ID".into()))?,
            self.title,
            self.msg,
            self.date,
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

pub fn insert_post(conn: &mut Connection, data: &Post) -> anyhow::Result<()> {
    let value_string = data.db_insert_string()?;
    let statement = format!("INSERT INTO {} VALUES ({})", POSTS_TABLE, value_string);

    conn.execute(statement)?;

    Ok(())
}

pub fn delete_post_by_id(conn: &mut Connection, post_id: usize) -> anyhow::Result<()> {
    let statement = format!("DELETE FROM {} WHERE id={}", POSTS_TABLE, post_id);

    conn.execute(statement)?;

    Ok(())
}

pub fn query_posts(
    conn: &mut Connection,
    posts_limit: Option<u32>,
) -> anyhow::Result<Vec<sqlite::Row>> {
    let statement = format!(
        "SELECT * FROM {} LIMIT {};",
        POSTS_TABLE,
        posts_limit.unwrap_or(20)
    );

    let mut stmt = conn.prepare(statement)?;

    let result: Vec<sqlite::Row> = stmt.iter().filter_map(|val| val.ok()).collect();

    Ok(result)
}

pub fn query_post_by_id(conn: &mut Connection, post_id: usize) -> anyhow::Result<sqlite::Row> {
    let statement = format!("SELECT * FROM {} WHERE id={}", POSTS_TABLE, post_id);

    let mut stmt = conn.prepare(statement)?;

    let mut result: Vec<sqlite::Row> = stmt.iter().filter_map(|val| val.ok()).collect();

    if result.len() == 0 {
        return Err(BriefsError::custom_error("Post not found with the given ID".into()).into());
    } else if result.len() > 1 {
        return Err(BriefsError::custom_error(
            "BROKEN Db: Multiple posts found with the same ID".into(),
        )
        .into());
    }

    Ok(result.remove(0))
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
        return Err(BriefsError::SqliteError {
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
                    create_db(inner_path.join(DB_NAME))?;
                } else {
                    create_db(inner_path)?;
                }
            }
        }
        None => {
            let db_path = std::env::temp_dir().join(DB_NAME);
            create_db(db_path)?;
        }
    }

    Ok(())
}

/// Generates a random db name with four 16-bit fields, such that when generating
/// random numbers, the range of each 16 bit field is 0-65536. Hence,
/// each random db name is `prefix-xxxxx-xxxxx-xxxxx-xxxxx.db`
/// The generated digits are padded with zeroes to ensure standardised
/// length of each field.
pub fn generate_random_db_name() -> String {
    let mut buffer = [0u16; 4];
    thread_rng().fill(&mut buffer);
    let mut result = buffer
        .into_iter()
        .map(|val| format!("{:05}", val.to_be()))
        .collect::<Vec<String>>()
        .join("-");
    result.insert_str(0, "briefs-");
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
    use super::*;
    use sqlite::Value;

    fn generate_random_db_name() -> String {
        let mut buffer = [0u16; 4];
        thread_rng().fill(&mut buffer);
        let mut result = buffer
            .into_iter()
            .map(|val| format!("{:05}", val.to_be()))
            .collect::<Vec<String>>()
            .join("-");
        result.insert_str(0, "briefs-");
        result.push_str(".db");
        result
    }

    #[test]
    fn test_generate_random_db_name() {
        for _ in 0..5 {
            let db_name = generate_random_db_name();
            assert!(db_name.starts_with("briefs-"));
            assert!(db_name.ends_with(".db"));
            assert!(db_name.len() == 33);
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
        let result = setup_tables(&mut conn);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());

        let result = query_table_info(&mut conn, POSTS_TABLE);
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
        let result = setup_tables(&mut conn);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());

        let result = query_table_info(&mut conn, POSTS_TABLE);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());

        let mut conn = sqlite::open(path.clone()).unwrap();
        let post = Post::new(
            0,
            "Post #1".into(),
            "Hello there, this is my first post".into(),
        )
        .unwrap();
        let result = insert_post(&mut conn, &post);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());

        let result = query_posts(&mut conn, None);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());

        //----- Expected values
        let expected_rows = 1;
        let expected_columns = [
            Value::Integer(0),
            Value::String(post.title),
            Value::String(post.msg),
            Value::Integer(
                post.date
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

    #[test]
    fn test_delete_from_table() {
        let db_name = generate_random_db_name();
        let path = std::env::temp_dir().join(db_name.clone());
        setup_db(Some(path.clone())).unwrap();

        assert!(path.exists(), "Db creation failed at expected path");

        let mut conn = sqlite::open(path.clone()).unwrap();
        let result = setup_tables(&mut conn);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());

        let result = query_table_info(&mut conn, POSTS_TABLE);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());

        let mut conn = sqlite::open(path.clone()).unwrap();
        let post = Post::new(
            0,
            "Post #1".into(),
            "Hello there, this is my first post".into(),
        )
        .unwrap();
        let result = insert_post(&mut conn, &post);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());

        let result = query_posts(&mut conn, None);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());

        //----- Expected values
        let expected_rows = 1;
        //-----

        let row_data = result.unwrap();
        let actual_rows = row_data.len();
        assert_eq!(actual_rows, expected_rows);

        println!("{:?}", row_data);

        let result = delete_post_by_id(&mut conn, 0);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());

        let result = query_posts(&mut conn, None);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());

        //----- Expected values
        let expected_rows = 0;
        //-----

        let row_data = result.unwrap();
        let actual_rows = row_data.len();
        assert_eq!(actual_rows, expected_rows);

        std::fs::remove_file(path).expect("Db cleanup failed");
    }
}
