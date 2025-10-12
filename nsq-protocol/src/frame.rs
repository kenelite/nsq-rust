//! NSQ Frame implementation
//! 
//! NSQ uses a simple frame-based protocol over TCP

use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::errors::{ProtocolError, Result};

/// NSQ Frame types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    Response = 0,
    Error = 1,
    Message = 2,
}

impl TryFrom<u8> for FrameType {
    type Error = ProtocolError;
    
    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(FrameType::Response),
            1 => Ok(FrameType::Error),
            2 => Ok(FrameType::Message),
            _ => Err(ProtocolError::InvalidFrameType(value)),
        }
    }
}

/// NSQ Frame structure
#[derive(Debug, Clone)]
pub struct Frame {
    pub frame_type: FrameType,
    pub body: Bytes,
}

impl Frame {
    /// Create a new frame
    pub fn new(frame_type: FrameType, body: Bytes) -> Self {
        Self { frame_type, body }
    }
    
    /// Serialize frame to bytes
    pub fn to_bytes(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(4 + self.body.len());
        buf.put_u32(self.body.len() as u32);
        buf.put_u8(self.frame_type as u8);
        buf.put_slice(&self.body);
        buf.freeze()
    }
    
    /// Deserialize frame from bytes
    pub fn from_bytes(mut data: Bytes) -> Result<Self> {
        if data.len() < 5 {
            return Err(ProtocolError::InvalidFrameSize(data.len()));
        }
        
        let size = data.get_u32() as usize;
        let frame_type = FrameType::try_from(data.get_u8())?;
        
        if data.len() < size {
            return Err(ProtocolError::InvalidFrameSize(data.len()));
        }
        
        let body = data.split_to(size);
        
        Ok(Self { frame_type, body })
    }
    
    /// Get frame size including header
    pub fn total_size(&self) -> usize {
        4 + 1 + self.body.len()
    }
}
