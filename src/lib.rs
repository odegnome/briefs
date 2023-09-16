mod error;
pub mod post;
pub mod stream;

pub use error::{StreamError, CatchupResult};

pub mod constant {
    pub const MAX_POST_LEN: u16 = 300;
    pub const MAX_POST_TITLE: u16 = 100;
}

mod prelude {
    use crate::post;
    use std::time::SystemTime;

    use crate::CatchupResult;

    pub trait Stream {
        fn add_post(&mut self, post: post::Post) -> CatchupResult<()>;
        fn remove_post(&mut self, id: usize) -> CatchupResult<()>;
        fn update_post_msg(&mut self, id: usize, new_msg: String) -> CatchupResult<()>;
        fn last_updated(&self) -> SystemTime;
        fn size(&self) -> usize;
        fn date_of_inception(&self) -> SystemTime;
    }

    pub trait Post {
        fn new<T>(id: usize, title: String, msg: String) -> CatchupResult<T>;
        fn update_msg(id: usize, msg: String) -> CatchupResult<()>;
        fn update_title(id: usize, title: String) -> CatchupResult<()>;
        fn verify_title(title: &String) -> CatchupResult<()>;
        fn verify_msg(msg: &String) -> CatchupResult<()>;
    }
}
