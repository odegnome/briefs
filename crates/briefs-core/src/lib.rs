//! Welcome to documentation of CatchUp. Hopefully, you will find
//! all that you need within these docs. However, if need be, join
//! the discord(link in github) and post your question.
mod error;
pub mod post;
pub mod state;
pub mod stream;
pub mod db;
pub mod config;
pub mod utils;

use std::fmt::Display;

pub use error::{BriefsError, BriefsResult};

pub mod constant {
    pub const MAX_POST_LEN: u16 = 300;
    pub const MAX_POST_TITLE: u16 = 100;
    pub const STREAM_CACHE_SIZE: u16 = 10;
    pub const CONFIG_DIR: &str = ".briefs";
    pub const CONFIG_FILE: &str = "briefs.toml";
    pub const CONFIG_ENV: &str = "BRIEFSCONF";
    pub const DATA_DIR: &str = "data";
    pub const DATA_FILE: &str = "stream";
    pub const PAGINATION_LIMIT: u32 = 40;
    pub const PAGINATION_DEFAULT: u32 = 20;
}

/// Used to send acknowledgements to the connection handler.
pub type Responder<T> = tokio::sync::oneshot::Sender<T>;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Command {
    Catchup { last_fetch_id: u32 },
    Create { title: String, msg: String },
    UpdateMsg { id: u32, msg: String },
    UpdateTitle { id: u32, title: String },
    Delete { id: u32 },
    Get { id: u32 },
    Metadata {},
}

pub struct StreamCommand {
    pub cmd: Command,
    pub resp: Option<Responder<Vec<u8>>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StreamResponse {
    msg: String,
}

impl StreamResponse {
     pub fn new(msg: String) -> Self {
        Self { msg }
    }
}

impl Display for StreamResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

#[allow(dead_code)]
pub mod prelude {
    use crate::post;
    use std::time::SystemTime;

    use crate::BriefsResult;

    /// This is a potential Response type to be used by stream handler
    /// to respond with to the client. This is needed because in current
    /// state, the cli will be unable to deserialize the data, if there
    /// is an error, as it references a different data struct for serde.
    pub struct StreamResponse<D, E> {
        data: D,
        error: E
    }

    pub trait CatchupStream {
        fn insert_post(&mut self, post: post::Post) -> BriefsResult<()>;
        fn delete_post(&mut self, id: usize) -> BriefsResult<()>;
        fn update_post_msg(&mut self, id: usize, new_msg: String) -> BriefsResult<()>;
        fn update_post_title(&mut self, id: usize, new_title: String) -> BriefsResult<()>;
        fn last_updated(&self) -> SystemTime;
        fn metadata(&self) -> ();
    }

    pub trait CatchupPost {
        fn new(id: usize, title: String, msg: String) -> BriefsResult<()>;
        fn update_msg(id: usize, msg: String) -> BriefsResult<()>;
        fn update_title(id: usize, title: String) -> BriefsResult<()>;
        fn verify_title(title: &String) -> BriefsResult<()>;
        fn verify_msg(msg: &String) -> BriefsResult<()>;
    }

    pub trait DataBase {
        fn insert_post(&self, post: &post::Post) -> BriefsResult<()>;
        fn delete_post(&self, id: usize) -> BriefsResult<()>;
        fn modify_post(&self, post: &post::Post) -> BriefsResult<()>;
        /// Used to retrieve the latest `N` Posts. Needed by the refresh_cache
        /// functionality
        ///
        /// # Errors
        ///
        /// This function will return an error if .
        fn get_n_posts(&self, n: usize) -> BriefsResult<Vec<post::Post>>;
    }

    pub trait Cache {
        fn refresh_cache(&mut self) -> BriefsResult<()>;
    }
}
