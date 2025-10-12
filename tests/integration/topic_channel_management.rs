//! Topic and channel management integration tests

use crate::test_utils::{TestEnvironment, TestConfig};
use crate::test_utils::assertions::*;

#[tokio::test]
async fn test_topic_lifecycle() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    let result = nsqd_client.create_topic("lifecycle-test").await.expect("Failed to create topic");
    assert_eq!(result, "OK");
    
    // Verify topic exists
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    assert_topic_exists(&stats, "lifecycle-test");
    
    // Publish some messages
    for i in 0..5 {
        let message = format!("Lifecycle message {}", i);
        nsqd_client.publish("lifecycle-test", &message).await.expect("Failed to publish message");
    }
    
    // Verify messages
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    assert_message_count(&stats, "lifecycle-test", 5);
    
    // Pause topic
    let result = nsqd_client.pause_topic("lifecycle-test").await.expect("Failed to pause topic");
    assert_eq!(result, "OK");
    
    // Verify topic is paused
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    assert_topic_paused(&stats, "lifecycle-test", true);
    
    // Unpause topic
    let result = nsqd_client.unpause_topic("lifecycle-test").await.expect("Failed to unpause topic");
    assert_eq!(result, "OK");
    
    // Verify topic is not paused
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    assert_topic_paused(&stats, "lifecycle-test", false);
    
    // Delete topic
    let result = nsqd_client.delete_topic("lifecycle-test").await.expect("Failed to delete topic");
    assert_eq!(result, "OK");
    
    // Verify topic no longer exists
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic_exists = topics.iter().any(|topic| {
        topic["topic_name"].as_str() == Some("lifecycle-test")
    });
    assert!(!topic_exists, "Topic should not exist after deletion");
}

#[tokio::test]
async fn test_multiple_topics() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create multiple topics
    let topic_names = vec![
        "topic-1",
        "topic-2", 
        "topic-3",
        "topic-4",
        "topic-5"
    ];
    
    for topic_name in &topic_names {
        let result = nsqd_client.create_topic(topic_name).await.expect("Failed to create topic");
        assert_eq!(result, "OK");
    }
    
    // Verify all topics exist
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    for topic_name in &topic_names {
        assert_topic_exists(&stats, topic_name);
    }
    
    // Publish messages to each topic
    for (i, topic_name) in topic_names.iter().enumerate() {
        let message = format!("Message for {}", topic_name);
        nsqd_client.publish(topic_name, &message).await.expect("Failed to publish message");
        
        // Verify message count for this topic
        let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
        assert_message_count(&stats, topic_name, 1);
    }
}

#[tokio::test]
async fn test_channel_creation() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("channel-creation-test").await.expect("Failed to create topic");
    
    // Publish a message to trigger channel creation
    nsqd_client.publish("channel-creation-test", "Trigger channel creation").await.expect("Failed to publish message");
    
    // Verify channel was created
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("channel-creation-test"))
        .expect("Topic should exist");
    
    let channels = topic["channels"].as_array().expect("channels should be an array");
    assert!(!channels.is_empty(), "At least one channel should be created");
    
    // Check channel properties
    let channel = &channels[0];
    assert!(channel["channel_name"].is_string());
    assert!(channel["depth"].is_number());
    assert!(channel["message_count"].is_number());
    assert!(channel["paused"].is_boolean());
}

#[tokio::test]
async fn test_channel_pause_unpause() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic and publish message to create channel
    nsqd_client.create_topic("channel-pause-test").await.expect("Failed to create topic");
    nsqd_client.publish("channel-pause-test", "Create channel").await.expect("Failed to publish message");
    
    // Get initial stats
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("channel-pause-test"))
        .expect("Topic should exist");
    let channels = topic["channels"].as_array().expect("channels should be an array");
    let channel = &channels[0];
    let channel_name = channel["channel_name"].as_str().expect("Channel name should be a string");
    
    // Verify channel is not paused initially
    assert_eq!(channel["paused"].as_bool().expect("paused should be boolean"), false);
    
    // Pause channel
    let result = nsqd_client.pause_channel("channel-pause-test", channel_name).await.expect("Failed to pause channel");
    assert_eq!(result, "OK");
    
    // Verify channel is paused
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("channel-pause-test"))
        .expect("Topic should exist");
    let channels = topic["channels"].as_array().expect("channels should be an array");
    let channel = &channels[0];
    assert_eq!(channel["paused"].as_bool().expect("paused should be boolean"), true);
    
    // Unpause channel
    let result = nsqd_client.unpause_channel("channel-pause-test", channel_name).await.expect("Failed to unpause channel");
    assert_eq!(result, "OK");
    
    // Verify channel is not paused
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("channel-pause-test"))
        .expect("Topic should exist");
    let channels = topic["channels"].as_array().expect("channels should be an array");
    let channel = &channels[0];
    assert_eq!(channel["paused"].as_bool().expect("paused should be boolean"), false);
}

#[tokio::test]
async fn test_channel_deletion() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic and publish message to create channel
    nsqd_client.create_topic("channel-delete-test").await.expect("Failed to create topic");
    nsqd_client.publish("channel-delete-test", "Create channel").await.expect("Failed to publish message");
    
    // Get channel name
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("channel-delete-test"))
        .expect("Topic should exist");
    let channels = topic["channels"].as_array().expect("channels should be an array");
    let channel = &channels[0];
    let channel_name = channel["channel_name"].as_str().expect("Channel name should be a string");
    
    // Verify channel exists
    assert_channel_exists(&stats, "channel-delete-test", channel_name);
    
    // Delete channel
    let result = nsqd_client.delete_channel("channel-delete-test", channel_name).await.expect("Failed to delete channel");
    assert_eq!(result, "OK");
    
    // Verify channel no longer exists
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("channel-delete-test"))
        .expect("Topic should exist");
    let channels = topic["channels"].as_array().expect("channels should be an array");
    let channel_exists = channels.iter().any(|channel| {
        channel["channel_name"].as_str() == Some(channel_name)
    });
    assert!(!channel_exists, "Channel should not exist after deletion");
}

#[tokio::test]
async fn test_topic_channel_statistics() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("stats-test").await.expect("Failed to create topic");
    
    // Publish multiple messages
    for i in 0..10 {
        let message = format!("Stats message {}", i);
        nsqd_client.publish("stats-test", &message).await.expect("Failed to publish message");
    }
    
    // Check topic statistics
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("stats-test"))
        .expect("Topic should exist");
    
    // Verify topic statistics
    assert_eq!(topic["message_count"].as_u64().expect("message_count should be a number"), 10);
    assert!(topic["depth"].is_number());
    assert!(topic["backend_depth"].is_number());
    assert!(topic["paused"].is_boolean());
    
    // Check channel statistics
    let channels = topic["channels"].as_array().expect("channels should be an array");
    assert!(!channels.is_empty(), "At least one channel should exist");
    
    let channel = &channels[0];
    assert!(channel["channel_name"].is_string());
    assert!(channel["depth"].is_number());
    assert!(channel["backend_depth"].is_number());
    assert!(channel["message_count"].is_number());
    assert!(channel["in_flight_count"].is_number());
    assert!(channel["deferred_count"].is_number());
    assert!(channel["requeue_count"].is_number());
    assert!(channel["timeout_count"].is_number());
    assert!(channel["paused"].is_boolean());
    assert!(channel["clients"].is_array());
}

#[tokio::test]
async fn test_topic_naming_validation() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Test valid topic names
    let valid_names = vec![
        "valid-topic",
        "topic_123",
        "TOPIC",
        "topic-name-123",
        "a",
        "very-long-topic-name-with-many-characters"
    ];
    
    for topic_name in &valid_names {
        let result = nsqd_client.create_topic(topic_name).await.expect("Failed to create topic");
        assert_eq!(result, "OK");
        
        // Verify topic exists
        let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
        assert_topic_exists(&stats, topic_name);
    }
    
    // Test invalid topic names (should fail)
    let invalid_names = vec![
        "",  // Empty name
        "topic with spaces",
        "topic\nwith\nnewlines",
        "topic\twith\ttabs",
        "topic#with#special#chars",
        "topic@with@symbols",
    ];
    
    for topic_name in &invalid_names {
        let result = nsqd_client.create_topic(topic_name).await;
        // These should fail or be rejected
        if result.is_ok() {
            // If creation succeeded, the topic should be invalid or sanitized
            let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
            let topics = stats["topics"].as_array().expect("topics should be an array");
            let topic_exists = topics.iter().any(|topic| {
                topic["topic_name"].as_str() == Some(topic_name)
            });
            // Topic should either not exist or be sanitized
            assert!(!topic_exists || topic_name != topic_name, "Invalid topic name should be rejected or sanitized");
        }
    }
}
