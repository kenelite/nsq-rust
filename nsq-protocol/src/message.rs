//! NSQ Message implementation

use bytes::{Buf, BufMut, Bytes, BytesMut};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::errors::{ProtocolError, Result};

/// NSQ Message structure
#[derive(Debug, Clone)]
pub struct Message {
    /// Unique message ID
    pub id: Uuid,
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
    /// Number of delivery attempts
    pub attempts: u16,
    /// Message body
    pub body: Bytes,
}

impl Message {
    /// Create a new message
    pub fn new(body: Bytes) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            attempts: 0,
            body,
        }
    }
    
    /// Create a message with specific ID and timestamp
    pub fn with_metadata(id: Uuid, timestamp: DateTime<Utc>, attempts: u16, body: Bytes) -> Self {
        Self {
            id,
            timestamp,
            attempts,
            body,
        }
    }
    
    /// Serialize message to bytes for wire protocol
    pub fn to_bytes(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(16 + 8 + 2 + self.body.len());
        
        // Message ID (16 bytes)
        buf.put_slice(self.id.as_bytes());
        
        // Timestamp (8 bytes, nanoseconds since epoch)
        let timestamp_ns = self.timestamp.timestamp_nanos_opt().unwrap_or(0) as u64;
        buf.put_u64(timestamp_ns);
        
        // Attempts (2 bytes)
        buf.put_u16(self.attempts);
        
        // Body
        buf.put_slice(&self.body);
        
        buf.freeze()
    }
    
    /// Deserialize message from bytes
    pub fn from_bytes(mut data: Bytes) -> Result<Self> {
        if data.len() < 26 {
            return Err(ProtocolError::InvalidMessage("Message too short".to_string()));
        }
        
        // Message ID (16 bytes)
        let id_bytes = data.split_to(16);
        let id = Uuid::from_slice(&id_bytes)
            .map_err(|e| ProtocolError::InvalidMessage(format!("Invalid UUID: {}", e)))?;
        
        // Timestamp (8 bytes)
        let timestamp_ns = data.get_u64();
        let timestamp = DateTime::from_timestamp_nanos(timestamp_ns as i64);
        
        // Attempts (2 bytes)
        let attempts = data.get_u16();
        
        // Body (remaining bytes)
        let body = data;
        
        Ok(Self {
            id,
            timestamp,
            attempts,
            body,
        })
    }
    
    /// Get message size in bytes
    pub fn size(&self) -> usize {
        16 + 8 + 2 + self.body.len()
    }
}

/// Message statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStats {
    pub total_messages: u64,
    pub total_bytes: u64,
    pub messages_in_flight: u64,
    pub messages_deferred: u64,
    pub messages_requeued: u64,
    pub messages_timed_out: u64,
}
