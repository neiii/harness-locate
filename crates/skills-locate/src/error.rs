use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(String),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("ZIP error: {0}")]
    Zip(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid source: {0}")]
    InvalidSource(String),

    #[error("Invalid UTF-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}
