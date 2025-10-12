//! NSQ Command implementation
//! 
//! Commands are sent over the wire protocol to control NSQ behavior

use bytes::{Buf, BufMut, Bytes, BytesMut};
// use serde::{Deserialize, Serialize};
use crate::errors::{ProtocolError, Result};

/// NSQ Commands
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    // Producer commands
    Pub { topic: String, body: Bytes },
    Mpub { topic: String, bodies: Vec<Bytes> },
    Dpub { topic: String, delay: u64, body: Bytes },
    
    // Consumer commands
    Sub { topic: String, channel: String },
    Rdy { count: u32 },
    Fin { message_id: Bytes },
    Req { message_id: Bytes, timeout: u64 },
    Touch { message_id: Bytes },
    
    // Control commands
    Identify { data: serde_json::Value },
    Auth { secret: String },
    Nop,
    Close,
}

impl Command {
    /// Serialize command to bytes
    pub fn to_bytes(&self) -> Result<Bytes> {
        let mut buf = BytesMut::new();
        
        match self {
            Command::Pub { topic, body } => {
                buf.put_slice(b"PUB ");
                buf.put_slice(topic.as_bytes());
                buf.put_slice(b"\n");
                buf.put_u32(body.len() as u32);
                buf.put_slice(body);
            }
            
            Command::Mpub { topic, bodies } => {
                buf.put_slice(b"MPUB ");
                buf.put_slice(topic.as_bytes());
                buf.put_slice(b"\n");
                buf.put_u32(bodies.len() as u32);
                for body in bodies {
                    buf.put_u32(body.len() as u32);
                    buf.put_slice(body);
                }
            }
            
            Command::Dpub { topic, delay, body } => {
                buf.put_slice(b"DPUB ");
                buf.put_slice(topic.as_bytes());
                buf.put_slice(b"\n");
                buf.put_u64(*delay);
                buf.put_u32(body.len() as u32);
                buf.put_slice(body);
            }
            
            Command::Sub { topic, channel } => {
                buf.put_slice(b"SUB ");
                buf.put_slice(topic.as_bytes());
                buf.put_slice(b" ");
                buf.put_slice(channel.as_bytes());
                buf.put_slice(b"\n");
            }
            
            Command::Rdy { count } => {
                buf.put_slice(b"RDY ");
                buf.put_slice(count.to_string().as_bytes());
                buf.put_slice(b"\n");
            }
            
            Command::Fin { message_id } => {
                buf.put_slice(b"FIN ");
                buf.put_slice(message_id);
                buf.put_slice(b"\n");
            }
            
            Command::Req { message_id, timeout } => {
                buf.put_slice(b"REQ ");
                buf.put_slice(message_id);
                buf.put_slice(b" ");
                buf.put_slice(timeout.to_string().as_bytes());
                buf.put_slice(b"\n");
            }
            
            Command::Touch { message_id } => {
                buf.put_slice(b"TOUCH ");
                buf.put_slice(message_id);
                buf.put_slice(b"\n");
            }
            
            Command::Identify { data } => {
                buf.put_slice(b"IDENTIFY\n");
                let json = serde_json::to_vec(data)
                    .map_err(|e| ProtocolError::Serialization(e.to_string()))?;
                buf.put_u32(json.len() as u32);
                buf.put_slice(&json);
            }
            
            Command::Auth { secret } => {
                buf.put_slice(b"AUTH\n");
                buf.put_u32(secret.len() as u32);
                buf.put_slice(secret.as_bytes());
            }
            
            Command::Nop => {
                buf.put_slice(b"NOP\n");
            }
            
            Command::Close => {
                buf.put_slice(b"CLS\n");
            }
        }
        
        Ok(buf.freeze())
    }
    
    /// Deserialize command from bytes
    pub fn from_bytes(mut data: Bytes) -> Result<Self> {
        // Find the end of the command line
        let line_end = data.windows(1).position(|w| w[0] == b'\n')
            .ok_or_else(|| ProtocolError::InvalidCommand("Missing newline".to_string()))?;
        
        let command_line = data.split_to(line_end + 1);
        let command_str = std::str::from_utf8(&command_line[..line_end])
            .map_err(|e| ProtocolError::InvalidCommand(e.to_string()))?;
        
        let parts: Vec<&str> = command_str.split_whitespace().collect();
        
        match parts[0] {
            "PUB" => {
                if parts.len() != 2 {
                    return Err(ProtocolError::InvalidCommand("Invalid PUB command".to_string()));
                }
                let topic = parts[1].to_string();
                let body_len = data.get_u32() as usize;
                let body = data.split_to(body_len);
                Ok(Command::Pub { topic, body })
            }
            
            "MPUB" => {
                if parts.len() != 2 {
                    return Err(ProtocolError::InvalidCommand("Invalid MPUB command".to_string()));
                }
                let topic = parts[1].to_string();
                let count = data.get_u32() as usize;
                let mut bodies = Vec::with_capacity(count);
                for _ in 0..count {
                    let body_len = data.get_u32() as usize;
                    bodies.push(data.split_to(body_len));
                }
                Ok(Command::Mpub { topic, bodies })
            }
            
            "DPUB" => {
                if parts.len() != 2 {
                    return Err(ProtocolError::InvalidCommand("Invalid DPUB command".to_string()));
                }
                let topic = parts[1].to_string();
                let delay = data.get_u64();
                let body_len = data.get_u32() as usize;
                let body = data.split_to(body_len);
                Ok(Command::Dpub { topic, delay, body })
            }
            
            "SUB" => {
                if parts.len() != 3 {
                    return Err(ProtocolError::InvalidCommand("Invalid SUB command".to_string()));
                }
                let topic = parts[1].to_string();
                let channel = parts[2].to_string();
                Ok(Command::Sub { topic, channel })
            }
            
            "RDY" => {
                if parts.len() != 2 {
                    return Err(ProtocolError::InvalidCommand("Invalid RDY command".to_string()));
                }
                let count = parts[1].parse::<u32>()
                    .map_err(|e| ProtocolError::InvalidCommand(e.to_string()))?;
                Ok(Command::Rdy { count })
            }
            
            "FIN" => {
                if parts.len() != 2 {
                    return Err(ProtocolError::InvalidCommand("Invalid FIN command".to_string()));
                }
                let message_id = Bytes::from(parts[1].as_bytes().to_vec());
                Ok(Command::Fin { message_id })
            }
            
            "REQ" => {
                if parts.len() != 3 {
                    return Err(ProtocolError::InvalidCommand("Invalid REQ command".to_string()));
                }
                let message_id = Bytes::from(parts[1].as_bytes().to_vec());
                let timeout = parts[2].parse::<u64>()
                    .map_err(|e| ProtocolError::InvalidCommand(e.to_string()))?;
                Ok(Command::Req { message_id, timeout })
            }
            
            "TOUCH" => {
                if parts.len() != 2 {
                    return Err(ProtocolError::InvalidCommand("Invalid TOUCH command".to_string()));
                }
                let message_id = Bytes::from(parts[1].as_bytes().to_vec());
                Ok(Command::Touch { message_id })
            }
            
            "IDENTIFY" => {
                let data_len = data.get_u32() as usize;
                let data_bytes = data.split_to(data_len);
                let data = serde_json::from_slice(&data_bytes)
                    .map_err(|e| ProtocolError::Serialization(e.to_string()))?;
                Ok(Command::Identify { data })
            }
            
            "AUTH" => {
                let secret_len = data.get_u32() as usize;
                let secret_bytes = data.split_to(secret_len);
                let secret = String::from_utf8(secret_bytes.to_vec())
                    .map_err(|e| ProtocolError::InvalidCommand(e.to_string()))?;
                Ok(Command::Auth { secret })
            }
            
            "NOP" => Ok(Command::Nop),
            "CLS" => Ok(Command::Close),
            
            _ => Err(ProtocolError::InvalidCommand(format!("Unknown command: {}", parts[0]))),
        }
    }
    
    /// Get command name
    pub fn name(&self) -> &'static str {
        match self {
            Command::Pub { .. } => "PUB",
            Command::Mpub { .. } => "MPUB",
            Command::Dpub { .. } => "DPUB",
            Command::Sub { .. } => "SUB",
            Command::Rdy { .. } => "RDY",
            Command::Fin { .. } => "FIN",
            Command::Req { .. } => "REQ",
            Command::Touch { .. } => "TOUCH",
            Command::Identify { .. } => "IDENTIFY",
            Command::Auth { .. } => "AUTH",
            Command::Nop => "NOP",
            Command::Close => "CLS",
        }
    }
}
