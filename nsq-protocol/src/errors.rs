//! Protocol error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid frame size: {0}")]
    InvalidFrameSize(usize),
    
    #[error("Invalid frame type: {0}")]
    InvalidFrameType(u8),
    
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),
    
    #[error("Compression error: {0}")]
    Compression(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

pub type Result<T> = std::result::Result<T, ProtocolError>;
