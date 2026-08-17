use std::fmt;

#[derive(Debug)]
pub enum Error {
    MissingDatastarHeader,
    Body(String),
    Json(String),
    Decode(String),
    MissingDatastarQuery,
    MissingSignal(&'static str),
    Serialize(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingDatastarHeader => write!(f, "Missing Datastar-Request header"),
            Error::Body(err) => write!(f, "Failed to read body: {err}"),
            Error::Json(err) => write!(f, "Invalid JSON: {err}"),
            Error::Decode(err) => write!(f, "Invalid URL encoding: {err}"),
            Error::MissingDatastarQuery => write!(f, "Missing datastar query parameter"),
            Error::MissingSignal(signal) => write!(f, "Missing signal: {signal}"),
            Error::Serialize(err) => write!(f, "Failed to serialize signal: {err}"),
        }
    }
}

impl std::error::Error for Error {}
