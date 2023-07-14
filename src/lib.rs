mod error;
pub mod post;
pub mod stream;

pub use error::StreamError;

pub mod constant {
    pub const MAX_POST_LEN: u16 = 300;
    pub const MAX_POST_TITLE: u16 = 100;
}
