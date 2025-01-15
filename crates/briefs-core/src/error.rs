use std::error::Error;
use std::fmt::Display;

pub type BriefsResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BriefsError {
    /// An empty title was provided for the post.
    EmptyTitle,
    /// An empty message was provided for the post.
    EmptyPost,
    /// The title length exceeds the maximum length.
    InvalidTitleLength {
        max_size: usize,
        curr_size: usize,
    },
    /// The post length exceeds the maximum length.
    InvalidPostLength {
        max_size: usize,
        curr_size: usize,
    },
    /// The requested/specified index is Out Of Bounds.
    InvalidIndex {
        posts_count: usize,
        given_index: usize,
    },
    /// The requested/specified ID does not exist.
    InvalidId {},
    /// Custom Error type for errors not covered by the above errors.
    CustomError {
        msg: String,
    },
}

impl Error for BriefsError {}

impl Display for BriefsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BriefsError::EmptyTitle => writeln!(f, "Title cannot be empty"),
            BriefsError::EmptyPost => writeln!(f, "Post cannot be empty"),
            BriefsError::InvalidTitleLength {
                max_size,
                curr_size,
            } => writeln!(
                f,
                "Max allowed size of title: {max_size}, current size: {curr_size}"
            ),
            BriefsError::InvalidPostLength {
                max_size,
                curr_size,
            } => writeln!(
                f,
                "Max allowed size of post: {max_size}, current size: {curr_size}"
            ),
            BriefsError::InvalidIndex {
                posts_count,
                given_index,
            } => writeln!(
                f,
                "The index({given_index}) is greater than posts count({posts_count})"
            ),
            BriefsError::CustomError { msg } => writeln!(f, "{:?}", msg),
            BriefsError::InvalidId { } => writeln!(f, "Post does not exist with the given ID" ),
        }
    }
}
