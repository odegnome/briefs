use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum StreamError {
    CustomError { msg: String },
}

impl Error for StreamError {}

impl Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: Something happened")
    }
}
