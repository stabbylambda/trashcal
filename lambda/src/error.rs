use std::fmt;

use tracing::error;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    IdError(String),
    RedirectPage(String),
    HttpError,
    ParseError,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::RedirectPage(id) => write!(f, "The city has no idea what this ID: {id}"),
            Error::IdError(id) => write!(f, "Got an ID that made no sense: {id}"),
            Error::HttpError => write!(f, "HTTP Error"),
            Error::ParseError => write!(f, "Parse Error"),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        error!("got an http error {err}");
        Self::HttpError
    }
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
