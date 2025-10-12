//! Channel management

use std::sync::Arc;
use uuid::Uuid;
use parking_lot::RwLock;
use nsq_protocol::Message;
use nsq_common::{Metrics, Result, validate_topic_channel_name};
use crate::message::MessageQueue;

/// Channel represents a message channel within a topic
pub struct Channel {
    /// Channel name
    pub name: String,
    /// Topic name
    pub topic_name: String,
    /// Message queue for this channel
    message_queue: Arc<MessageQueue>,
    /// Channel statistics
    stats: Arc<RwLock<ChannelStats>>,
    /// Metrics
    metrics: Metrics,
    /// Channel creation time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Whether the channel is paused
    paused: Arc<RwLock<bool>>,
}

/// Channel statistics
#[derive(Debug, Clone)]
pub struct ChannelStats {
    pub message_count: u64,
    pub depth: u64,
    pub backend_depth: u64,
    pub in_flight_count: u64,
    pub deferred_count: u64,
    pub requeue_count: u64,
    pub timeout_count: u64,
    pub client_count: u64,
}

impl Default for ChannelStats {
    fn default() -> Self {
        Self {
            message_count: 0,
            depth: 0,
            backend_depth: 0,
            in_flight_count: 0,
            deferred_count: 0,
            requeue_count: 0,
            timeout_count: 0,
            client_count: 0,
        }
    }
}

impl Channel {
    /// Create a new channel
    pub fn new(
        name: String,
        topic_name: String,
        message_queue: Arc<MessageQueue>,
        metrics: Metrics,
    ) -> Result<Self> {
        validate_topic_channel_name(&name)?;
        
        Ok(Self {
            name,
            topic_name,
            message_queue,
            stats: Arc::new(RwLock::new(ChannelStats::default())),
            metrics,
            created_at: chrono::Utc::now(),
            paused: Arc::new(RwLock::new(false)),
        })
    }
    
    /// Distribute a message from the topic's message queue
    pub fn distribute_message(&self) -> Result<()> {
        if *self.paused.read() {
            return Ok(());
        }
        
        if let Some(message) = self.message_queue.get()? {
            // Put message in channel's queue
            self.message_queue.put(message)?;
            
            {
                let mut stats = self.stats.write();
                stats.message_count += 1;
                stats.depth = self.message_queue.depth() as u64;
            }
            
            self.metrics.incr("messages.distributed", 1);
        }
        
        Ok(())
    }
    
    /// Get a message from the channel queue
    pub fn get_message(&self) -> Result<Option<Message>> {
        if *self.paused.read() {
            return Ok(None);
        }
        
        self.message_queue.get()
    }
    
    /// Mark a message as in-flight
    pub fn mark_in_flight(&self, message: Message, client_id: Uuid, timeout: std::time::Duration) -> Result<()> {
        self.message_queue.mark_in_flight(message, client_id, timeout)?;
        
        {
            let mut stats = self.stats.write();
            stats.in_flight_count += 1;
        }
        
        self.metrics.incr("messages.in_flight", 1);
        Ok(())
    }
    
    /// Finish a message (acknowledge)
    pub fn finish_message(&self, message_id: Uuid) -> Result<()> {
        self.message_queue.finish(message_id)?;
        
        {
            let mut stats = self.stats.write();
            stats.in_flight_count = stats.in_flight_count.saturating_sub(1);
        }
        
        self.metrics.incr("messages.finished", 1);
        Ok(())
    }
    
    /// Requeue a message
    pub fn requeue_message(&self, message_id: Uuid, timeout: std::time::Duration) -> Result<()> {
        self.message_queue.requeue(message_id, timeout)?;
        
        {
            let mut stats = self.stats.write();
            stats.requeue_count += 1;
        }
        
        self.metrics.incr("messages.requeued", 1);
        Ok(())
    }
    
    /// Defer a message
    pub fn defer_message(&self, message_id: Uuid, delay: std::time::Duration) -> Result<()> {
        self.message_queue.defer(message_id, delay)?;
        
        {
            let mut stats = self.stats.write();
            stats.deferred_count += 1;
        }
        
        self.metrics.incr("messages.deferred", 1);
        Ok(())
    }
    
    /// Process deferred messages
    pub fn process_deferred(&self) -> Result<()> {
        let ready_messages = self.message_queue.process_deferred()?;
        
        for message in ready_messages {
            self.message_queue.put(message)?;
        }
        
        Ok(())
    }
    
    /// Clean up timed out messages
    pub fn cleanup_timeouts(&self) -> Result<()> {
        let timed_out_messages = self.message_queue.cleanup_timeouts()?;
        
        {
            let mut stats = self.stats.write();
            stats.timeout_count += timed_out_messages.len() as u64;
        }
        
        self.metrics.incr("messages.timed_out", timed_out_messages.len() as u64);
        
        // Requeue timed out messages
        for message in timed_out_messages {
            if let Err(e) = self.message_queue.put(message) {
                tracing::warn!("Failed to requeue timed out message: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Get channel statistics
    pub fn stats(&self) -> ChannelStats {
        let mut stats = self.stats.read().clone();
        
        // Update real-time stats
        stats.depth = self.message_queue.depth() as u64;
        stats.in_flight_count = self.message_queue.in_flight_count() as u64;
        stats.deferred_count = self.message_queue.deferred_count() as u64;
        
        stats
    }
    
    /// Get message queue depth
    pub fn depth(&self) -> usize {
        self.message_queue.depth()
    }
    
    /// Get in-flight count
    pub fn in_flight_count(&self) -> usize {
        self.message_queue.in_flight_count()
    }
    
    /// Get deferred count
    pub fn deferred_count(&self) -> usize {
        self.message_queue.deferred_count()
    }
    
    /// Pause the channel
    pub fn pause(&self) -> Result<()> {
        *self.paused.write() = true;
        self.metrics.incr("channels.paused", 1);
        Ok(())
    }
    
    /// Unpause the channel
    pub fn unpause(&self) -> Result<()> {
        *self.paused.write() = false;
        self.metrics.incr("channels.unpaused", 1);
        Ok(())
    }
    
    /// Check if channel is paused
    pub fn is_paused(&self) -> bool {
        *self.paused.read()
    }
    
    /// Delete the channel
    pub fn delete(&self) -> Result<()> {
        // Pause the channel first
        self.pause()?;
        
        self.metrics.incr("channels.deleted", 1);
        Ok(())
    }
}
