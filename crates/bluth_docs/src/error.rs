#[derive(Debug)]
pub enum Error {
    Lambda(lambda_http::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Lambda(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {}
