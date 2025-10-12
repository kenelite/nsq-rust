//! Message handling and management

use std::sync::Arc;
use std::time::{Duration, Instant};
use bytes::Bytes;
use uuid::Uuid;
use parking_lot::RwLock;
use crossbeam_channel::{Receiver, Sender};
use nsq_protocol::{Message, MessageStats};
use nsq_common::{Metrics, Result, NsqError};

/// In-flight message tracking
#[derive(Debug, Clone)]
pub struct InFlightMessage {
    pub message: Message,
    pub client_id: Uuid,
    pub start_time: Instant,
    pub timeout: Duration,
    pub requeue_count: u16,
}

impl InFlightMessage {
    /// Create a new in-flight message
    pub fn new(message: Message, client_id: Uuid, timeout: Duration) -> Self {
        Self {
            message,
            client_id,
            start_time: Instant::now(),
            timeout,
            requeue_count: 0,
        }
    }
    
    /// Check if the message has timed out
    pub fn is_timed_out(&self) -> bool {
        self.start_time.elapsed() > self.timeout
    }
    
    /// Get time remaining until timeout
    pub fn time_remaining(&self) -> Duration {
        self.timeout.saturating_sub(self.start_time.elapsed())
    }
}

/// Message queue for a channel
pub struct MessageQueue {
    /// Memory queue for fast access
    memory_queue: Arc<RwLock<Vec<Message>>>,
    /// Disk queue for persistence
    disk_queue: Option<nsq_common::DiskQueue>,
    /// Maximum memory queue size
    max_memory_size: usize,
    /// Channel for sending messages to consumers
    sender: Sender<Message>,
    /// Channel for receiving messages from producers
    receiver: Receiver<Message>,
    /// In-flight messages
    in_flight: Arc<RwLock<std::collections::HashMap<Uuid, InFlightMessage>>>,
    /// Deferred messages
    deferred: Arc<RwLock<std::collections::HashMap<Uuid, (Message, Instant)>>>,
    /// Metrics
    metrics: Metrics,
    /// Queue statistics
    stats: Arc<RwLock<MessageStats>>,
}

impl MessageQueue {
    /// Create a new message queue
    pub fn new(
        max_memory_size: usize,
        disk_queue: Option<nsq_common::DiskQueue>,
        metrics: Metrics,
    ) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        
        Self {
            memory_queue: Arc::new(RwLock::new(Vec::new())),
            disk_queue,
            max_memory_size,
            sender,
            receiver,
            in_flight: Arc::new(RwLock::new(std::collections::HashMap::new())),
            deferred: Arc::new(RwLock::new(std::collections::HashMap::new())),
            metrics,
            stats: Arc::new(RwLock::new(MessageStats {
                total_messages: 0,
                total_bytes: 0,
                messages_in_flight: 0,
                messages_deferred: 0,
                messages_requeued: 0,
                messages_timed_out: 0,
            })),
        }
    }
    
    /// Put a message into the queue
    pub fn put(&self, message: Message) -> Result<()> {
        let message_size = message.size();
        
        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.total_messages += 1;
            stats.total_bytes += message_size as u64;
        }
        
        // Try memory queue first
        {
            let mut memory_queue = self.memory_queue.write();
            if memory_queue.len() < self.max_memory_size {
                memory_queue.push(message);
                self.metrics.incr("messages.memory", 1);
                return Ok(());
            }
        }
        
        // Fall back to disk queue
        if let Some(ref disk_queue) = self.disk_queue {
            disk_queue.put(&message.body)?;
            self.metrics.incr("messages.disk", 1);
        } else {
            return Err(NsqError::Queue("Memory queue full and no disk queue available".to_string()));
        }
        
        Ok(())
    }
    
    /// Get a message from the queue
    pub fn get(&self) -> Result<Option<Message>> {
        // Try memory queue first
        {
            let mut memory_queue = self.memory_queue.write();
            if let Some(message) = memory_queue.pop() {
                self.metrics.incr("messages.memory.dequeued", 1);
                return Ok(Some(message));
            }
        }
        
        // Try disk queue
        if let Some(ref disk_queue) = self.disk_queue {
            if let Some(data) = disk_queue.get()? {
                let message = Message::from_bytes(Bytes::from(data))?;
                self.metrics.incr("messages.disk.dequeued", 1);
                return Ok(Some(message));
            }
        }
        
        Ok(None)
    }
    
    /// Mark a message as in-flight
    pub fn mark_in_flight(&self, message: Message, client_id: Uuid, timeout: Duration) -> Result<()> {
        let in_flight_msg = InFlightMessage::new(message, client_id, timeout);
        let message_id = in_flight_msg.message.id;
        
        self.in_flight.write().insert(message_id, in_flight_msg);
        
        {
            let mut stats = self.stats.write();
            stats.messages_in_flight += 1;
        }
        
        self.metrics.incr("messages.in_flight", 1);
        Ok(())
    }
    
    /// Finish a message (acknowledge)
    pub fn finish(&self, message_id: Uuid) -> Result<()> {
        if self.in_flight.write().remove(&message_id).is_some() {
            {
                let mut stats = self.stats.write();
                stats.messages_in_flight = stats.messages_in_flight.saturating_sub(1);
            }
            
            self.metrics.incr("messages.finished", 1);
            Ok(())
        } else {
            Err(NsqError::Queue("Message not found in flight".to_string()))
        }
    }
    
    /// Requeue a message
    pub fn requeue(&self, message_id: Uuid, _timeout: Duration) -> Result<()> {
        if let Some(mut in_flight_msg) = self.in_flight.write().remove(&message_id) {
            in_flight_msg.requeue_count += 1;
            in_flight_msg.start_time = Instant::now();
            
            // Put back in queue
            self.put(in_flight_msg.message)?;
            
            {
                let mut stats = self.stats.write();
                stats.messages_in_flight = stats.messages_in_flight.saturating_sub(1);
                stats.messages_requeued += 1;
            }
            
            self.metrics.incr("messages.requeued", 1);
            Ok(())
        } else {
            Err(NsqError::Queue("Message not found in flight".to_string()))
        }
    }
    
    /// Defer a message
    pub fn defer(&self, message_id: Uuid, delay: Duration) -> Result<()> {
        if let Some(in_flight_msg) = self.in_flight.write().remove(&message_id) {
            let defer_time = Instant::now() + delay;
            self.deferred.write().insert(message_id, (in_flight_msg.message, defer_time));
            
            {
                let mut stats = self.stats.write();
                stats.messages_in_flight = stats.messages_in_flight.saturating_sub(1);
                stats.messages_deferred += 1;
            }
            
            self.metrics.incr("messages.deferred", 1);
            Ok(())
        } else {
            Err(NsqError::Queue("Message not found in flight".to_string()))
        }
    }
    
    /// Process deferred messages
    pub fn process_deferred(&self) -> Result<Vec<Message>> {
        let now = Instant::now();
        let mut ready_messages = Vec::new();
        let mut deferred = self.deferred.write();
        
        let ready_ids: Vec<Uuid> = deferred
            .iter()
            .filter(|(_, (_, defer_time))| *defer_time <= now)
            .map(|(id, _)| *id)
            .collect();
        
        for id in ready_ids {
            if let Some((message, _)) = deferred.remove(&id) {
                ready_messages.push(message);
                
                {
                    let mut stats = self.stats.write();
                    stats.messages_deferred = stats.messages_deferred.saturating_sub(1);
                }
                
                self.metrics.incr("messages.deferred.processed", 1);
            }
        }
        
        Ok(ready_messages)
    }
    
    /// Clean up timed out messages
    pub fn cleanup_timeouts(&self) -> Result<Vec<Message>> {
        let mut timed_out = Vec::new();
        let mut in_flight = self.in_flight.write();
        
        let timed_out_ids: Vec<Uuid> = in_flight
            .iter()
            .filter(|(_, msg)| msg.is_timed_out())
            .map(|(id, _)| *id)
            .collect();
        
        for id in timed_out_ids {
            if let Some(in_flight_msg) = in_flight.remove(&id) {
                timed_out.push(in_flight_msg.message);
                
                {
                    let mut stats = self.stats.write();
                    stats.messages_in_flight = stats.messages_in_flight.saturating_sub(1);
                    stats.messages_timed_out += 1;
                }
                
                self.metrics.incr("messages.timed_out", 1);
            }
        }
        
        Ok(timed_out)
    }
    
    /// Get queue statistics
    pub fn stats(&self) -> MessageStats {
        self.stats.read().clone()
    }
    
    /// Get queue depth
    pub fn depth(&self) -> usize {
        self.memory_queue.read().len()
    }
    
    /// Get in-flight count
    pub fn in_flight_count(&self) -> usize {
        self.in_flight.read().len()
    }
    
    /// Get deferred count
    pub fn deferred_count(&self) -> usize {
        self.deferred.read().len()
    }
    
    /// Get sender for consumers
    pub fn sender(&self) -> Sender<Message> {
        self.sender.clone()
    }
    
    /// Get receiver for producers
    pub fn receiver(&self) -> Receiver<Message> {
        self.receiver.clone()
    }
}
