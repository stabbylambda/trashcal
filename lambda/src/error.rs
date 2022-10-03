use thiserror::Error;

use tracing::error;

#[derive(Error, Debug)]

pub enum Error {
    #[error("Got an ID that made no sense: {0}")]
    IdError(String),
    #[error("The city has no idea what this ID is: {0}")]
    RedirectPage(String),
    #[error("HTTP Error")]
    HttpError(#[from] reqwest::Error),
    #[error("Parse Error")]
    ParseError,
}

impl From<chrono::format::ParseError> for Error {
    fn from(_: chrono::format::ParseError) -> Self {
        Self::ParseError
    }
}

impl From<strum::ParseError> for Error {
    fn from(_: strum::ParseError) -> Self {
        Self::ParseError
    }
}
