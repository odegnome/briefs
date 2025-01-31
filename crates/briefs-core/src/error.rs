use thiserror::Error;

pub type BriefsResult<T> = anyhow::Result<T>;

#[derive(Error, Debug)]
pub enum BriefsError {
    /// An empty title was provided for the post.
    #[error("Title cannot be empty")]
    EmptyTitle,
    /// An empty message was provided for the post.
    #[error("Post cannot be empty")]
    EmptyPost,
    /// The title length exceeds the maximum length.
    #[error("Max allowed size of title: {max_size}, current size: {curr_size}")]
    InvalidTitleLength {
        max_size: usize,
        curr_size: usize,
    },
    /// The post length exceeds the maximum length.
    #[error("Max allowed size of post: {max_size}, current size: {curr_size}")]
    InvalidPostLength {
        max_size: usize,
        curr_size: usize,
    },
    /// The requested/specified index is Out Of Bounds.
    #[error("The index({given_index}) is greater than posts count({posts_count})")]
    InvalidIndex {
        posts_count: usize,
        given_index: usize,
    },
    /// The requested/specified ID does not exist.
    #[error("Post does not exist with the given ID")]
    InvalidId {},
    /// An error occured in a sqlite operation. This is just
    /// a wrapper around the error message.
    #[error("ERROR: {msg}")]
    SqliteError { msg: String },
    /// Parsing of sqlite::Value into required type failed.
    #[error("ERROR: Unable to parse input sqlite `Value` into required type")]
    SqliteValueParseError,
    /// Custom Error type for errors not covered by the above errors.
    #[error("{msg}")]
    CustomError {
        msg: String,
    },
}

impl BriefsError {
    pub fn custom_error(msg: String) -> Self {
        Self::CustomError { msg }
    }
}
