use thiserror::Error;

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
    #[error("Enum Parse Error")]
    EnumParseError(#[from] strum::ParseError),
    #[error("DateTime Error")]
    TimeZoneError(#[from] chrono::format::ParseError),
}
