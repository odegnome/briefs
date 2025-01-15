use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("ERROR: {msg}")]
    SqliteError { msg: String },

    #[error("ERROR: {msg}")]
    CustomError { msg: String },
}

impl ServerError {
    pub fn custom_error(msg: String) -> Self {
        Self::CustomError { msg }
    }
}
