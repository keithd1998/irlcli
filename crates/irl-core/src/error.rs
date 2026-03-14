use thiserror::Error;

#[derive(Error, Debug)]
pub enum IrlError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Failed to parse response: {0}")]
    Parse(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("API key missing for '{service}'. Run: irl config set {service}.api_key <YOUR_KEY>")]
    ApiKeyMissing { service: String },

    #[error("API returned error {status}: {message}")]
    ApiError { status: u16, message: String },

    #[error("{0}")]
    Other(String),
}

impl From<std::io::Error> for IrlError {
    fn from(err: std::io::Error) -> Self {
        IrlError::Other(err.to_string())
    }
}

impl From<serde_json::Error> for IrlError {
    fn from(err: serde_json::Error) -> Self {
        IrlError::Parse(err.to_string())
    }
}

impl From<toml::de::Error> for IrlError {
    fn from(err: toml::de::Error) -> Self {
        IrlError::Config(err.to_string())
    }
}
