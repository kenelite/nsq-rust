//! Common error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum NsqError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Queue error: {0}")]
    Queue(String),
    
    #[error("Metrics error: {0}")]
    Metrics(String),
    
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    
    #[error("Channel error: {0}")]
    Channel(#[from] crossbeam_channel::RecvError),
}

impl From<nsq_protocol::ProtocolError> for NsqError {
    fn from(err: nsq_protocol::ProtocolError) -> Self {
        NsqError::Protocol(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, NsqError>;
