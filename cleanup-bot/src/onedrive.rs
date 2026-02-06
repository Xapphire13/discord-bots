mod auth;
mod client;

pub use auth::TokenStore;
pub use client::OneDriveClient;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum OneDriveError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Token storage error: {0}")]
    TokenStorage(String),

    #[error("Upload failed: {0}")]
    Upload(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
