//! Welcome to documentation of CatchUp. Hopefully, you will find
//! all that you need within these docs. However, if need be, join
//! the discord(link in github) and post your question.
mod error;
pub mod post;
pub mod state;
pub mod stream;

pub use error::{CatchupResult, StreamError};

pub mod constant {
    pub const MAX_POST_LEN: u16 = 300;
    pub const MAX_POST_TITLE: u16 = 100;
}

/// Used to send acknowledgements to the connection handler.
pub type Responder<T> = tokio::sync::oneshot::Sender<T>;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Command {
    Catchup { last_fetch_id: usize },
    Create { title: String, msg: String },
    Read { index: usize },
    Update,
    Delete { index: usize },
    Get,
    Subscribe {},
}

pub struct StreamCommand {
    pub cmd: Command,
    pub resp: Option<Responder<String>>,
}

pub mod prelude {
    use crate::post;
    use std::time::SystemTime;

    use crate::CatchupResult;

    pub trait CatchupStream {
        fn add_post(&mut self, post: post::Post) -> CatchupResult<()>;
        fn remove_post(&mut self, id: usize) -> CatchupResult<()>;
        fn update_post_msg(&mut self, id: usize, new_msg: String) -> CatchupResult<()>;
        fn last_updated(&self) -> SystemTime;
        fn size(&self) -> usize;
        fn date_of_inception(&self) -> SystemTime;
    }

    pub trait CatchupPost {
        fn new<T>(id: usize, title: String, msg: String) -> CatchupResult<T>;
        fn update_msg(id: usize, msg: String) -> CatchupResult<()>;
        fn update_title(id: usize, title: String) -> CatchupResult<()>;
        fn verify_title(title: &String) -> CatchupResult<()>;
        fn verify_msg(msg: &String) -> CatchupResult<()>;
    }

    pub trait DataBase {
        fn insert_post(&self, post: &post::Post);
        fn delete_post(&self, post: &post::Post);
        fn modify_post(&self, post: &post::Post);
    }
}
