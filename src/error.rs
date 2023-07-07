use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum StreamError {
    CustomError { msg: String },
}

impl Error for StreamError {}

impl Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamError::CustomError { msg } => writeln!(f, "{:?}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StreamError;

    #[test]
    fn print_error() {
        // executing `cargo test --lib -- --show-output` will show correctly
        let myerr = StreamError::CustomError {
            msg: String::from("This is my error"),
        };
        println!("{:?}", myerr);
        println!("{}", myerr);
    }
}
