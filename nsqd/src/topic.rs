//! Topic management

use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;
use nsq_protocol::Message;
use nsq_common::{Metrics, Result, NsqError, validate_topic_channel_name};
use crate::channel::Channel;
use crate::message::MessageQueue;

/// Topic represents a message topic
pub struct Topic {
    /// Topic name
    pub name: String,
    /// Channels in this topic
    channels: Arc<RwLock<HashMap<String, Arc<Channel>>>>,
    /// Message queue for this topic
    message_queue: Arc<MessageQueue>,
    /// Topic statistics
    stats: Arc<RwLock<TopicStats>>,
    /// Metrics
    metrics: Metrics,
    /// Topic creation time
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Topic statistics
#[derive(Debug, Clone)]
pub struct TopicStats {
    pub message_count: u64,
    pub channel_count: u64,
    pub depth: u64,
    pub backend_depth: u64,
    pub in_flight_count: u64,
    pub deferred_count: u64,
    pub requeue_count: u64,
    pub timeout_count: u64,
}

impl Default for TopicStats {
    fn default() -> Self {
        Self {
            message_count: 0,
            channel_count: 0,
            depth: 0,
            backend_depth: 0,
            in_flight_count: 0,
            deferred_count: 0,
            requeue_count: 0,
            timeout_count: 0,
        }
    }
}

impl Topic {
    /// Create a new topic
    pub fn new(
        name: String,
        max_memory_size: usize,
        disk_queue: Option<nsq_common::DiskQueue>,
        metrics: Metrics,
    ) -> Result<Self> {
        validate_topic_channel_name(&name)?;
        
        let message_queue = Arc::new(MessageQueue::new(max_memory_size, disk_queue, metrics.clone()));
        
        Ok(Self {
            name,
            channels: Arc::new(RwLock::new(HashMap::new())),
            message_queue,
            stats: Arc::new(RwLock::new(TopicStats::default())),
            metrics,
            created_at: chrono::Utc::now(),
        })
    }
    
    /// Add a channel to this topic
    pub fn add_channel(&self, channel_name: String) -> Result<Arc<Channel>> {
        validate_topic_channel_name(&channel_name)?;
        
        let mut channels = self.channels.write();
        
        if channels.contains_key(&channel_name) {
            return Err(NsqError::Validation("Channel already exists".to_string()));
        }
        
        let channel = Arc::new(Channel::new(
            channel_name.clone(),
            self.name.clone(),
            self.message_queue.clone(),
            self.metrics.clone(),
        )?);
        
        channels.insert(channel_name, channel.clone());
        
        {
            let mut stats = self.stats.write();
            stats.channel_count += 1;
        }
        
        self.metrics.incr("channels.created", 1);
        
        Ok(channel)
    }
    
    /// Get a channel by name
    pub fn get_channel(&self, channel_name: &str) -> Option<Arc<Channel>> {
        self.channels.read().get(channel_name).cloned()
    }
    
    /// Remove a channel
    pub fn remove_channel(&self, channel_name: &str) -> Result<()> {
        let mut channels = self.channels.write();
        
        if channels.remove(channel_name).is_some() {
            {
                let mut stats = self.stats.write();
                stats.channel_count = stats.channel_count.saturating_sub(1);
            }
            
            self.metrics.incr("channels.removed", 1);
            Ok(())
        } else {
            Err(NsqError::Validation("Channel not found".to_string()))
        }
    }
    
    /// Get all channels
    pub fn get_channels(&self) -> Vec<Arc<Channel>> {
        self.channels.read().values().cloned().collect()
    }
    
    /// Publish a message to this topic
    pub fn publish(&self, message: Message) -> Result<()> {
        self.message_queue.put(message)?;
        
        {
            let mut stats = self.stats.write();
            stats.message_count += 1;
            stats.depth = self.message_queue.depth() as u64;
        }
        
        self.metrics.incr("messages.published", 1);
        
        // Distribute message to all channels
        let channels = self.get_channels();
        for channel in channels {
            if let Err(e) = channel.distribute_message() {
                tracing::warn!("Failed to distribute message to channel {}: {}", channel.name, e);
            }
        }
        
        Ok(())
    }
    
    /// Publish multiple messages
    pub fn publish_multiple(&self, messages: Vec<Message>) -> Result<()> {
        for message in messages {
            self.publish(message)?;
        }
        Ok(())
    }
    
    /// Get topic statistics
    pub fn stats(&self) -> TopicStats {
        let mut stats = self.stats.read().clone();
        
        // Update real-time stats
        stats.depth = self.message_queue.depth() as u64;
        stats.in_flight_count = self.message_queue.in_flight_count() as u64;
        stats.deferred_count = self.message_queue.deferred_count() as u64;
        
        // Aggregate channel stats
        let channels = self.get_channels();
        for channel in channels {
            let channel_stats = channel.stats();
            stats.backend_depth += channel_stats.backend_depth;
        }
        
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
    
    /// Process deferred messages
    pub fn process_deferred(&self) -> Result<()> {
        let ready_messages = self.message_queue.process_deferred()?;
        
        for message in ready_messages {
            self.publish(message)?;
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
            if let Err(e) = self.publish(message) {
                tracing::warn!("Failed to requeue timed out message: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Pause the topic
    pub fn pause(&self) -> Result<()> {
        let channels = self.get_channels();
        for channel in channels {
            channel.pause()?;
        }
        
        self.metrics.incr("topics.paused", 1);
        Ok(())
    }
    
    /// Unpause the topic
    pub fn unpause(&self) -> Result<()> {
        let channels = self.get_channels();
        for channel in channels {
            channel.unpause()?;
        }
        
        self.metrics.incr("topics.unpaused", 1);
        Ok(())
    }
    
    /// Check if topic is paused
    pub fn is_paused(&self) -> bool {
        let channels = self.get_channels();
        channels.iter().any(|channel| channel.is_paused())
    }
    
    /// Delete the topic
    pub fn delete(&self) -> Result<()> {
        // Pause all channels first
        self.pause()?;
        
        // Remove all channels
        let channel_names: Vec<String> = self.channels.read().keys().cloned().collect();
        for channel_name in channel_names {
            self.remove_channel(&channel_name)?;
        }
        
        self.metrics.incr("topics.deleted", 1);
        Ok(())
    }
}
