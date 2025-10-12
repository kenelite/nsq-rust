//! Disk queue implementation for message persistence

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
// use memmap2::MmapMut;
use parking_lot::RwLock;
use crate::errors::{NsqError, Result};
use crate::validation::validate_message_size;

/// Disk queue for persisting messages
#[derive(Debug)]
pub struct DiskQueue {
    path: PathBuf,
    max_file_size: usize,
    max_msg_size: usize,
    sync_timeout: std::time::Duration,
    
    // Current file handles
    read_file: Arc<RwLock<Option<File>>>,
    write_file: Arc<RwLock<Option<File>>>,
    
    // File positions
    read_pos: Arc<RwLock<u64>>,
    write_pos: Arc<RwLock<u64>>,
    
    // File numbers
    read_file_num: Arc<RwLock<u64>>,
    write_file_num: Arc<RwLock<u64>>,
    
    // Queue metadata
    depth: Arc<RwLock<u64>>,
    sync_count: Arc<RwLock<u64>>,
}

impl DiskQueue {
    /// Create a new disk queue
    pub fn new<P: AsRef<Path>>(
        path: P,
        max_file_size: usize,
        max_msg_size: usize,
        sync_timeout: std::time::Duration,
    ) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        // Create directory if it doesn't exist
        std::fs::create_dir_all(&path)
            .map_err(|e| NsqError::Io(e))?;
        
        let queue = Self {
            path,
            max_file_size,
            max_msg_size,
            sync_timeout,
            read_file: Arc::new(RwLock::new(None)),
            write_file: Arc::new(RwLock::new(None)),
            read_pos: Arc::new(RwLock::new(0)),
            write_pos: Arc::new(RwLock::new(0)),
            read_file_num: Arc::new(RwLock::new(0)),
            write_file_num: Arc::new(RwLock::new(0)),
            depth: Arc::new(RwLock::new(0)),
            sync_count: Arc::new(RwLock::new(0)),
        };
        
        // Initialize queue from existing files
        queue.initialize()?;
        
        Ok(queue)
    }
    
    /// Initialize queue from existing files
    fn initialize(&self) -> Result<()> {
        // Find the highest numbered file
        let mut max_file_num = 0u64;
        
        if let Ok(entries) = std::fs::read_dir(&self.path) {
            for entry in entries {
                let entry = entry.map_err(|e| NsqError::Io(e))?;
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();
                
                if file_name.starts_with("nsq.") && file_name.ends_with(".dat") {
                    if let Some(num_str) = file_name.strip_prefix("nsq.").and_then(|s| s.strip_suffix(".dat")) {
                        if let Ok(num) = num_str.parse::<u64>() {
                            max_file_num = max_file_num.max(num);
                        }
                    }
                }
            }
        }
        
        *self.write_file_num.write() = max_file_num;
        *self.read_file_num.write() = max_file_num;
        
        // Open the write file
        self.open_write_file()?;
        
        // Calculate current depth
        self.calculate_depth()?;
        
        Ok(())
    }
    
    /// Open the write file
    fn open_write_file(&self) -> Result<()> {
        let file_num = *self.write_file_num.read();
        let file_path = self.path.join(format!("nsq.{}.dat", file_num));
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .map_err(|e| NsqError::Io(e))?;
        
        // Get current file size
        let metadata = file.metadata().map_err(|e| NsqError::Io(e))?;
        *self.write_pos.write() = metadata.len();
        
        *self.write_file.write() = Some(file);
        
        Ok(())
    }
    
    /// Open the read file
    fn open_read_file(&self) -> Result<()> {
        let file_num = *self.read_file_num.read();
        let file_path = self.path.join(format!("nsq.{}.dat", file_num));
        
        if !file_path.exists() {
            return Ok(());
        }
        
        let file = OpenOptions::new()
            .read(true)
            .open(&file_path)
            .map_err(|e| NsqError::Io(e))?;
        
        *self.read_file.write() = Some(file);
        
        Ok(())
    }
    
    /// Calculate current queue depth
    fn calculate_depth(&self) -> Result<()> {
        let mut depth = 0u64;
        
        // Count messages in all files
        if let Ok(entries) = std::fs::read_dir(&self.path) {
            for entry in entries {
                let entry = entry.map_err(|e| NsqError::Io(e))?;
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();
                
                if file_name.starts_with("nsq.") && file_name.ends_with(".dat") {
                    if let Ok(file) = OpenOptions::new().read(true).open(entry.path()) {
                        depth += self.count_messages_in_file(file)?;
                    }
                }
            }
        }
        
        *self.depth.write() = depth;
        Ok(())
    }
    
    /// Count messages in a file
    fn count_messages_in_file(&self, mut file: File) -> Result<u64> {
        let mut count = 0u64;
        let mut _pos = 0u64;
        
        loop {
            // Read message size
            let mut size_buf = [0u8; 4];
            match file.read_exact(&mut size_buf) {
                Ok(_) => {
                    let size = u32::from_be_bytes(size_buf) as u64;
                    _pos += 4;
                    
                    // Skip message data
                    if let Err(_) = file.seek(SeekFrom::Current(size as i64)) {
                        break;
                    }
                    _pos += size;
                    count += 1;
                }
                Err(_) => break,
            }
        }
        
        Ok(count)
    }
    
    /// Put a message into the queue
    pub fn put(&self, data: &[u8]) -> Result<()> {
        validate_message_size(data, self.max_msg_size)?;
        
        let mut write_file = self.write_file.write();
        let file = write_file.as_mut()
            .ok_or_else(|| NsqError::Queue("Write file not open".to_string()))?;
        
        // Check if we need to rotate the file
        let current_pos = *self.write_pos.read();
        if current_pos + 4 + data.len() as u64 > self.max_file_size as u64 {
            self.rotate_write_file()?;
        }
        
        // Write message size and data
        let size = data.len() as u32;
        file.write_all(&size.to_be_bytes())
            .map_err(|e| NsqError::Io(e))?;
        file.write_all(data)
            .map_err(|e| NsqError::Io(e))?;
        file.flush().map_err(|e| NsqError::Io(e))?;
        
        // Update positions
        *self.write_pos.write() += 4 + data.len() as u64;
        *self.depth.write() += 1;
        
        Ok(())
    }
    
    /// Get a message from the queue
    pub fn get(&self) -> Result<Option<Vec<u8>>> {
        // Open read file if needed
        if self.read_file.read().is_none() {
            self.open_read_file()?;
        }
        
        let mut read_file = self.read_file.write();
        let file = read_file.as_mut()
            .ok_or_else(|| NsqError::Queue("Read file not open".to_string()))?;
        
        // Read message size
        let mut size_buf = [0u8; 4];
        match file.read_exact(&mut size_buf) {
            Ok(_) => {
                let size = u32::from_be_bytes(size_buf) as usize;
                
                // Read message data
                let mut data = vec![0u8; size];
                file.read_exact(&mut data)
                    .map_err(|e| NsqError::Io(e))?;
                
                // Update positions
                *self.read_pos.write() += 4 + size as u64;
                *self.depth.write() = (*self.depth.read()).saturating_sub(1);
                
                Ok(Some(data))
            }
            Err(_) => {
                // End of file, try next file
                self.rotate_read_file()?;
                Ok(None)
            }
        }
    }
    
    /// Rotate to the next write file
    fn rotate_write_file(&self) -> Result<()> {
        // Close current write file
        *self.write_file.write() = None;
        
        // Increment file number
        *self.write_file_num.write() += 1;
        
        // Reset write position
        *self.write_pos.write() = 0;
        
        // Open new write file
        self.open_write_file()?;
        
        Ok(())
    }
    
    /// Rotate to the next read file
    fn rotate_read_file(&self) -> Result<()> {
        // Close current read file
        *self.read_file.write() = None;
        
        // Increment file number
        *self.read_file_num.write() += 1;
        
        // Reset read position
        *self.read_pos.write() = 0;
        
        // Open new read file
        self.open_read_file()?;
        
        Ok(())
    }
    
    /// Get current queue depth
    pub fn depth(&self) -> u64 {
        *self.depth.read()
    }
    
    /// Sync the queue to disk
    pub fn sync(&self) -> Result<()> {
        if let Some(ref file) = *self.write_file.read() {
            file.sync_all().map_err(|e| NsqError::Io(e))?;
        }
        
        *self.sync_count.write() += 1;
        Ok(())
    }
    
    /// Get sync count
    pub fn sync_count(&self) -> u64 {
        *self.sync_count.read()
    }
    
    /// Delete old files
    pub fn delete_old_files(&self, max_files: usize) -> Result<()> {
        let mut file_nums = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(&self.path) {
            for entry in entries {
                let entry = entry.map_err(|e| NsqError::Io(e))?;
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();
                
                if file_name.starts_with("nsq.") && file_name.ends_with(".dat") {
                    if let Some(num_str) = file_name.strip_prefix("nsq.").and_then(|s| s.strip_suffix(".dat")) {
                        if let Ok(num) = num_str.parse::<u64>() {
                            file_nums.push((num, entry.path()));
                        }
                    }
                }
            }
        }
        
        // Sort by file number
        file_nums.sort_by_key(|(num, _)| *num);
        
        // Delete old files
        if file_nums.len() > max_files {
            for (_, path) in file_nums.clone().into_iter().take(file_nums.len() - max_files) {
                if let Err(e) = std::fs::remove_file(&path) {
                    tracing::warn!("Failed to delete old file {:?}: {}", path, e);
                }
            }
        }
        
        Ok(())
    }
}
