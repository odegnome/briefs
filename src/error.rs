use std::error::Error;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum StreamError {
    EmptyTitle,
    EmptyPost,
    InvalidTitleLength {
        max_size: usize,
        curr_size: usize,
    },
    InvalidPostLength {
        max_size: usize,
        curr_size: usize,
    },
    InvalidIndex {
        posts_count: usize,
        given_index: usize,
    },
    CustomError {
        msg: String,
    },
}

impl Error for StreamError {}

impl Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamError::EmptyTitle => writeln!(f, "Title cannot be empty"),
            StreamError::EmptyPost => writeln!(f, "Post cannot be empty"),
            StreamError::InvalidTitleLength {
                max_size,
                curr_size,
            } => writeln!(
                f,
                "Max allowed size of title: {max_size}, current size: {curr_size}"
            ),
            StreamError::InvalidPostLength {
                max_size,
                curr_size,
            } => writeln!(
                f,
                "Max allowed size of post: {max_size}, current size: {curr_size}"
            ),
            StreamError::InvalidIndex {
                posts_count,
                given_index,
            } => writeln!(
                f,
                "The index({given_index}) is greater than posts count({posts_count})"
            ),
            StreamError::CustomError { msg } => writeln!(f, "{:?}", msg),
        }
    }
}
