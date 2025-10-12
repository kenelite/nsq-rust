//! NSQ Protocol Codec for Tokio
//! 
//! Implements the tokio-util codec traits for NSQ protocol

use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};
use crate::{Frame, Command, Message, ProtocolError, Result};

/// NSQ Protocol Decoder
pub struct NsqDecoder {
    max_frame_size: usize,
}

impl NsqDecoder {
    /// Create a new decoder with default max frame size
    pub fn new() -> Self {
        Self {
            max_frame_size: 5 * 1024 * 1024, // 5MB default
        }
    }
    
    /// Create a new decoder with custom max frame size
    pub fn with_max_frame_size(max_frame_size: usize) -> Self {
        Self { max_frame_size }
    }
}

impl Default for NsqDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for NsqDecoder {
    type Item = Frame;
    type Error = ProtocolError;
    
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        if src.len() < 5 {
            return Ok(None);
        }
        
        // Read frame size (4 bytes)
        let frame_size = u32::from_be_bytes([src[0], src[1], src[2], src[3]]) as usize;
        
        if frame_size > self.max_frame_size {
            return Err(ProtocolError::InvalidFrameSize(frame_size));
        }
        
        if src.len() < 5 + frame_size {
            return Ok(None);
        }
        
        // Split the frame data
        let frame_data = src.split_to(5 + frame_size);
        let frame = Frame::from_bytes(frame_data.freeze())?;
        
        Ok(Some(frame))
    }
}

/// NSQ Protocol Encoder
pub struct NsqEncoder;

impl Encoder<Frame> for NsqEncoder {
    type Error = ProtocolError;
    
    fn encode(&mut self, item: Frame, dst: &mut BytesMut) -> Result<()> {
        let frame_bytes = item.to_bytes();
        dst.extend_from_slice(&frame_bytes);
        Ok(())
    }
}

/// Command Encoder
pub struct CommandEncoder;

impl Encoder<Command> for CommandEncoder {
    type Error = ProtocolError;
    
    fn encode(&mut self, item: Command, dst: &mut BytesMut) -> Result<()> {
        let command_bytes = item.to_bytes()?;
        dst.extend_from_slice(&command_bytes);
        Ok(())
    }
}

/// Message Encoder
pub struct MessageEncoder;

impl Encoder<Message> for MessageEncoder {
    type Error = ProtocolError;
    
    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<()> {
        let message_bytes = item.to_bytes();
        dst.extend_from_slice(&message_bytes);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FrameType, Message};
    use bytes::Bytes;
    
    #[test]
    fn test_frame_codec() {
        let original_frame = Frame::new(FrameType::Message, Bytes::from("test message"));
        let encoded = original_frame.to_bytes();
        
        let mut decoder = NsqDecoder::new();
        let mut src = BytesMut::from(&encoded[..]);
        let decoded = decoder.decode(&mut src).unwrap().unwrap();
        
        assert_eq!(decoded.frame_type, FrameType::Message);
        assert_eq!(decoded.body, Bytes::from("test message"));
    }
    
    #[test]
    fn test_message_codec() {
        let original_message = Message::new(Bytes::from("test body"));
        let encoded = original_message.to_bytes();
        
        let decoded = Message::from_bytes(encoded).unwrap();
        
        assert_eq!(decoded.body, Bytes::from("test body"));
        assert_eq!(decoded.attempts, 0);
    }
}
