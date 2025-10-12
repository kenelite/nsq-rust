//! Compression support for NSQ messages

use bytes::Bytes;
use crate::errors::{ProtocolError, Result};

/// Compression types supported by NSQ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    None,
    Deflate,
    Snappy,
}

impl CompressionType {
    /// Get compression type from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "none" | "" => Ok(CompressionType::None),
            "deflate" => Ok(CompressionType::Deflate),
            "snappy" => Ok(CompressionType::Snappy),
            _ => Err(ProtocolError::Compression(format!("Unknown compression type: {}", s))),
        }
    }
    
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            CompressionType::None => "none",
            CompressionType::Deflate => "deflate",
            CompressionType::Snappy => "snappy",
        }
    }
}

/// Compress data using the specified compression type
pub fn compress(data: &[u8], compression: CompressionType) -> Result<Bytes> {
    match compression {
        CompressionType::None => Ok(Bytes::copy_from_slice(data)),
        
        CompressionType::Deflate => {
            use flate2::write::DeflateEncoder;
            use flate2::Compression;
            use std::io::Write;
            
            let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(data)
                .map_err(|e| ProtocolError::Compression(e.to_string()))?;
            let compressed = encoder.finish()
                .map_err(|e| ProtocolError::Compression(e.to_string()))?;
            
            Ok(Bytes::from(compressed))
        }
        
        CompressionType::Snappy => {
            let compressed = snap::raw::Encoder::new()
                .compress_vec(data)
                .map_err(|e| ProtocolError::Compression(e.to_string()))?;
            
            Ok(Bytes::from(compressed))
        }
    }
}

/// Decompress data using the specified compression type
pub fn decompress(data: &[u8], compression: CompressionType) -> Result<Bytes> {
    match compression {
        CompressionType::None => Ok(Bytes::copy_from_slice(data)),
        
        CompressionType::Deflate => {
            use flate2::read::DeflateDecoder;
            use std::io::Read;
            
            let mut decoder = DeflateDecoder::new(data);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)
                .map_err(|e| ProtocolError::Compression(e.to_string()))?;
            
            Ok(Bytes::from(decompressed))
        }
        
        CompressionType::Snappy => {
            let decompressed = snap::raw::Decoder::new()
                .decompress_vec(data)
                .map_err(|e| ProtocolError::Compression(e.to_string()))?;
            
            Ok(Bytes::from(decompressed))
        }
    }
}

/// Detect compression type from data
pub fn detect_compression(data: &[u8]) -> CompressionType {
    if data.len() < 4 {
        return CompressionType::None;
    }
    
    // Check for deflate magic bytes
    if data.len() >= 2 && data[0] == 0x78 && (data[1] == 0x01 || data[1] == 0x5e || data[1] == 0x9c || data[1] == 0xda) {
        return CompressionType::Deflate;
    }
    
    // Check for snappy magic bytes
    if data.len() >= 4 && data[0] == b's' && data[1] == b'N' && data[2] == b'a' && data[3] == b'P' {
        return CompressionType::Snappy;
    }
    
    CompressionType::None
}
