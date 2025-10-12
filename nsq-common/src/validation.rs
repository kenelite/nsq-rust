//! Validation utilities

use regex::Regex;
use crate::errors::{NsqError, Result};

lazy_static::lazy_static! {
    static ref TOPIC_CHANNEL_NAME_REGEX: Regex = Regex::new(r"^[\.a-zA-Z0-9_-]+$").unwrap();
}

/// Validate topic or channel name
pub fn validate_topic_channel_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(NsqError::Validation("Name cannot be empty".to_string()));
    }
    
    if name.len() > 64 {
        return Err(NsqError::Validation("Name too long (max 64 characters)".to_string()));
    }
    
    if !TOPIC_CHANNEL_NAME_REGEX.is_match(name) {
        return Err(NsqError::Validation(
            "Name contains invalid characters. Only letters, numbers, dots, underscores, and hyphens are allowed".to_string()
        ));
    }
    
    Ok(())
}

/// Validate message body size
pub fn validate_message_size(body: &[u8], max_size: usize) -> Result<()> {
    if body.len() > max_size {
        return Err(NsqError::Validation(
            format!("Message too large: {} bytes (max: {} bytes)", body.len(), max_size)
        ));
    }
    
    Ok(())
}

/// Validate timeout value
pub fn validate_timeout(timeout: u64, max_timeout: u64) -> Result<()> {
    if timeout > max_timeout {
        return Err(NsqError::Validation(
            format!("Timeout too large: {}ms (max: {}ms)", timeout, max_timeout)
        ));
    }
    
    Ok(())
}

/// Validate address format
pub fn validate_address(addr: &str) -> Result<()> {
    if addr.is_empty() {
        return Err(NsqError::Validation("Address cannot be empty".to_string()));
    }
    
    // Check if it's a valid socket address or unix socket path
    if addr.contains(':') {
        // TCP address
        let parts: Vec<&str> = addr.split(':').collect();
        if parts.len() != 2 {
            return Err(NsqError::Validation("Invalid TCP address format".to_string()));
        }
        
        if parts[1].parse::<u16>().is_err() {
            return Err(NsqError::Validation("Invalid port number".to_string()));
        }
    } else if addr.starts_with('/') {
        // Unix socket path
        if addr.len() > 108 {
            return Err(NsqError::Validation("Unix socket path too long".to_string()));
        }
    } else {
        return Err(NsqError::Validation("Invalid address format".to_string()));
    }
    
    Ok(())
}
