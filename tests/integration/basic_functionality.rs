//! Basic functionality integration tests

use crate::test_utils::{TestEnvironment, TestConfig};
use crate::test_utils::assertions::*;

#[tokio::test]
async fn test_service_startup() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    
    // Start all services
    env.start().await.expect("Failed to start services");
    
    // Verify all services are running
    let nsqd_client = env.nsqd_client();
    let lookupd_client = env.lookupd_client();
    let admin_client = env.admin_client();
    
    // Test ping endpoints
    let nsqd_ping = nsqd_client.ping().await.expect("NSQd ping failed");
    assert_eq!(nsqd_ping, "OK");
    
    let lookupd_ping = lookupd_client.ping().await.expect("NSQLookupd ping failed");
    assert_eq!(lookupd_ping, "OK");
    
    let admin_ping = admin_client.ping().await.expect("NSQAdmin ping failed");
    assert_eq!(admin_ping, "OK");
}

#[tokio::test]
async fn test_stats_endpoints() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    let lookupd_client = env.lookupd_client();
    let admin_client = env.admin_client();
    
    // Test NSQd stats
    let nsqd_stats = nsqd_client.get_stats().await.expect("Failed to get NSQd stats");
    assert!(nsqd_stats["version"].is_string());
    assert!(nsqd_stats["health"].is_string());
    assert!(nsqd_stats["topics"].is_array());
    
    // Test NSQLookupd stats
    let lookupd_stats = lookupd_client.get_stats().await.expect("Failed to get NSQLookupd stats");
    assert!(lookupd_stats["version"].is_string());
    assert!(lookupd_stats["health"].is_string());
    
    // Test NSQAdmin stats
    let admin_stats = admin_client.get_stats().await.expect("Failed to get NSQAdmin stats");
    assert!(admin_stats["version"].is_string());
    assert!(admin_stats["health"].is_string());
}

#[tokio::test]
async fn test_topic_creation() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    let lookupd_client = env.lookupd_client();
    
    // Create topic via NSQd
    let result = nsqd_client.create_topic("test-topic").await.expect("Failed to create topic");
    assert_eq!(result, "OK");
    
    // Verify topic exists in NSQd stats
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    assert_topic_exists(&stats, "test-topic");
    
    // Verify topic exists in NSQLookupd
    let topics = lookupd_client.get_topics().await.expect("Failed to get topics");
    let topics_array = topics["topics"].as_array().expect("topics should be an array");
    assert!(topics_array.iter().any(|t| t.as_str() == Some("test-topic")));
}

#[tokio::test]
async fn test_message_publishing() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic first
    nsqd_client.create_topic("test-messages").await.expect("Failed to create topic");
    
    // Publish a message
    let result = nsqd_client.publish("test-messages", "Hello, NSQ!").await.expect("Failed to publish message");
    assert_eq!(result, "OK");
    
    // Verify message count increased
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    assert_message_count(&stats, "test-messages", 1);
    
    // Publish multiple messages
    for i in 0..5 {
        let message = format!("Message {}", i);
        let result = nsqd_client.publish("test-messages", &message).await.expect("Failed to publish message");
        assert_eq!(result, "OK");
    }
    
    // Verify total message count
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    assert_message_count(&stats, "test-messages", 6);
}

#[tokio::test]
async fn test_topic_pause_unpause() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("test-pause").await.expect("Failed to create topic");
    
    // Publish some messages
    for i in 0..3 {
        let message = format!("Message {}", i);
        nsqd_client.publish("test-pause", &message).await.expect("Failed to publish message");
    }
    
    // Verify topic is not paused
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    assert_topic_paused(&stats, "test-pause", false);
    
    // Pause topic
    let result = nsqd_client.pause_topic("test-pause").await.expect("Failed to pause topic");
    assert_eq!(result, "OK");
    
    // Verify topic is paused
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    assert_topic_paused(&stats, "test-pause", true);
    
    // Unpause topic
    let result = nsqd_client.unpause_topic("test-pause").await.expect("Failed to unpause topic");
    assert_eq!(result, "OK");
    
    // Verify topic is not paused
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    assert_topic_paused(&stats, "test-pause", false);
}

#[tokio::test]
async fn test_topic_deletion() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    let lookupd_client = env.lookupd_client();
    
    // Create topic
    nsqd_client.create_topic("test-delete").await.expect("Failed to create topic");
    
    // Verify topic exists
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    assert_topic_exists(&stats, "test-delete");
    
    // Delete topic
    let result = nsqd_client.delete_topic("test-delete").await.expect("Failed to delete topic");
    assert_eq!(result, "OK");
    
    // Verify topic no longer exists
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic_exists = topics.iter().any(|topic| {
        topic["topic_name"].as_str() == Some("test-delete")
    });
    assert!(!topic_exists, "Topic 'test-delete' should not exist after deletion");
}

#[tokio::test]
async fn test_admin_interface() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let admin_client = env.admin_client();
    
    // Test admin stats endpoint
    let stats = admin_client.get_stats().await.expect("Failed to get admin stats");
    assert!(stats["version"].is_string());
    assert!(stats["health"].is_string());
    assert!(stats["topics"].is_array());
    
    // Test admin topics endpoint
    let topics = admin_client.get_topics().await.expect("Failed to get admin topics");
    assert!(topics["topics"].is_array());
    
    // Test admin nodes endpoint
    let nodes = admin_client.get_nodes().await.expect("Failed to get admin nodes");
    assert!(nodes["producers"].is_array());
}

#[tokio::test]
async fn test_error_handling() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Test publishing to non-existent topic
    let result = nsqd_client.publish("non-existent-topic", "test").await;
    // This should either succeed (topic auto-created) or fail gracefully
    match result {
        Ok(response) => {
            // Topic was auto-created
            assert_eq!(response, "OK");
        }
        Err(_) => {
            // Publishing failed as expected
        }
    }
    
    // Test pausing non-existent topic
    let result = nsqd_client.pause_topic("non-existent-topic").await;
    assert!(result.is_err(), "Pausing non-existent topic should fail");
    
    // Test deleting non-existent topic
    let result = nsqd_client.delete_topic("non-existent-topic").await;
    assert!(result.is_err(), "Deleting non-existent topic should fail");
}
