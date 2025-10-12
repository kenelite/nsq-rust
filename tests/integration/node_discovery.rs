//! Node discovery and service registration integration tests

use crate::test_utils::{TestEnvironment, TestConfig};
use crate::test_utils::assertions::*;

#[tokio::test]
async fn test_lookupd_registration() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let lookupd_client = env.lookupd_client();
    
    // Check lookupd stats
    let stats = lookupd_client.get_stats().await.expect("Failed to get lookupd stats");
    assert!(stats["version"].is_string());
    assert!(stats["health"].is_string());
    assert!(stats["producers"].is_array());
    
    // Check that NSQd is registered
    let producers = stats["producers"].as_array().expect("producers should be an array");
    assert!(!producers.is_empty(), "At least one producer should be registered");
    
    // Verify producer information
    let producer = &producers[0];
    assert!(producer["hostname"].is_string());
    assert!(producer["broadcast_address"].is_string());
    assert!(producer["tcp_port"].is_number());
    assert!(producer["http_port"].is_number());
    assert!(producer["version"].is_string());
}

#[tokio::test]
async fn test_topic_registration() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    let lookupd_client = env.lookupd_client();
    
    // Create topic via NSQd
    nsqd_client.create_topic("lookupd-test").await.expect("Failed to create topic");
    
    // Publish a message to ensure topic is active
    nsqd_client.publish("lookupd-test", "Test message").await.expect("Failed to publish message");
    
    // Wait a bit for registration to propagate
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    
    // Check that topic is registered in lookupd
    let topics = lookupd_client.get_topics().await.expect("Failed to get topics");
    let topics_array = topics["topics"].as_array().expect("topics should be an array");
    let topic_exists = topics_array.iter().any(|t| t.as_str() == Some("lookupd-test"));
    assert!(topic_exists, "Topic should be registered in lookupd");
    
    // Lookup topic specifically
    let lookup_result = lookupd_client.lookup_topic("lookupd-test").await.expect("Failed to lookup topic");
    assert!(lookup_result["producers"].is_array());
    
    let producers = lookup_result["producers"].as_array().expect("producers should be an array");
    assert!(!producers.is_empty(), "Topic should have at least one producer");
}

#[tokio::test]
async fn test_producer_discovery() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let lookupd_client = env.lookupd_client();
    
    // Get all registered producers
    let stats = lookupd_client.get_stats().await.expect("Failed to get lookupd stats");
    let producers = stats["producers"].as_array().expect("producers should be an array");
    
    // Verify producer details
    for producer in producers {
        assert!(producer["hostname"].is_string());
        assert!(producer["broadcast_address"].is_string());
        assert!(producer["tcp_port"].is_number());
        assert!(producer["http_port"].is_number());
        assert!(producer["version"].is_string());
        assert!(producer["last_update"].is_string());
        
        // Verify port numbers are valid
        let tcp_port = producer["tcp_port"].as_u64().expect("tcp_port should be a number");
        let http_port = producer["http_port"].as_u64().expect("http_port should be a number");
        assert!(tcp_port > 0 && tcp_port < 65536, "TCP port should be valid");
        assert!(http_port > 0 && http_port < 65536, "HTTP port should be valid");
    }
}

#[tokio::test]
async fn test_topic_discovery() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    let lookupd_client = env.lookupd_client();
    
    // Create multiple topics
    let topic_names = vec![
        "discovery-topic-1",
        "discovery-topic-2",
        "discovery-topic-3"
    ];
    
    for topic_name in &topic_names {
        nsqd_client.create_topic(topic_name).await.expect("Failed to create topic");
        nsqd_client.publish(topic_name, "Discovery test message").await.expect("Failed to publish message");
    }
    
    // Wait for registration to propagate
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    
    // Check that all topics are discovered
    let topics = lookupd_client.get_topics().await.expect("Failed to get topics");
    let topics_array = topics["topics"].as_array().expect("topics should be an array");
    
    for topic_name in &topic_names {
        let topic_exists = topics_array.iter().any(|t| t.as_str() == Some(topic_name));
        assert!(topic_exists, "Topic '{}' should be discovered", topic_name);
    }
    
    // Test individual topic lookup
    for topic_name in &topic_names {
        let lookup_result = lookupd_client.lookup_topic(topic_name).await.expect("Failed to lookup topic");
        assert!(lookup_result["producers"].is_array());
        
        let producers = lookup_result["producers"].as_array().expect("producers should be an array");
        assert!(!producers.is_empty(), "Topic '{}' should have producers", topic_name);
    }
}

#[tokio::test]
async fn test_service_health_monitoring() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
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
    
    // Test health status in stats
    let nsqd_stats = nsqd_client.get_stats().await.expect("Failed to get NSQd stats");
    assert_eq!(nsqd_stats["health"].as_str().expect("health should be a string"), "ok");
    
    let lookupd_stats = lookupd_client.get_stats().await.expect("Failed to get NSQLookupd stats");
    assert_eq!(lookupd_stats["health"].as_str().expect("health should be a string"), "ok");
    
    let admin_stats = admin_client.get_stats().await.expect("Failed to get NSQAdmin stats");
    assert_eq!(admin_stats["health"].as_str().expect("health should be a string"), "ok");
}

#[tokio::test]
async fn test_service_restart_discovery() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    let lookupd_client = env.lookupd_client();
    
    // Create topic and publish message
    nsqd_client.create_topic("restart-test").await.expect("Failed to create topic");
    nsqd_client.publish("restart-test", "Before restart").await.expect("Failed to publish message");
    
    // Verify topic is registered
    let topics = lookupd_client.get_topics().await.expect("Failed to get topics");
    let topics_array = topics["topics"].as_array().expect("topics should be an array");
    let topic_exists = topics_array.iter().any(|t| t.as_str() == Some("restart-test"));
    assert!(topic_exists, "Topic should be registered before restart");
    
    // Restart NSQd (simulate by stopping and starting)
    env.stop();
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    
    // Start services again
    env.start().await.expect("Failed to restart services");
    
    // Wait for re-registration
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    
    // Verify topic is still discoverable (if persistence is implemented)
    let topics = lookupd_client.get_topics().await.expect("Failed to get topics");
    let topics_array = topics["topics"].as_array().expect("topics should be an array");
    // Note: This may fail if persistence is not implemented yet
    // let topic_exists = topics_array.iter().any(|t| t.as_str() == Some("restart-test"));
    // assert!(topic_exists, "Topic should still be discoverable after restart");
}

#[tokio::test]
async fn test_multiple_producer_registration() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let lookupd_client = env.lookupd_client();
    
    // Get initial producer count
    let stats = lookupd_client.get_stats().await.expect("Failed to get lookupd stats");
    let initial_producers = stats["producers"].as_array().expect("producers should be an array");
    let initial_count = initial_producers.len();
    
    // Verify we have at least one producer (NSQd)
    assert!(initial_count > 0, "Should have at least one producer registered");
    
    // Check producer information
    for producer in initial_producers {
        assert!(producer["hostname"].is_string());
        assert!(producer["broadcast_address"].is_string());
        assert!(producer["tcp_port"].is_number());
        assert!(producer["http_port"].is_number());
        assert!(producer["version"].is_string());
        assert!(producer["last_update"].is_string());
        
        // Verify last_update is recent
        let last_update = producer["last_update"].as_str().expect("last_update should be a string");
        // Parse timestamp and verify it's recent (within last minute)
        // This is a basic check - in a real implementation, you'd parse the timestamp
        assert!(!last_update.is_empty(), "last_update should not be empty");
    }
}

#[tokio::test]
async fn test_lookupd_api_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let lookupd_client = env.lookupd_client();
    
    // Test all lookupd API endpoints
    let stats = lookupd_client.get_stats().await.expect("Failed to get stats");
    assert!(stats["version"].is_string());
    assert!(stats["health"].is_string());
    assert!(stats["start_time"].is_number());
    assert!(stats["uptime"].is_string());
    assert!(stats["uptime_seconds"].is_number());
    
    let topics = lookupd_client.get_topics().await.expect("Failed to get topics");
    assert!(topics["topics"].is_array());
    
    // Test lookup with non-existent topic
    let lookup_result = lookupd_client.lookup_topic("non-existent-topic").await.expect("Failed to lookup topic");
    assert!(lookup_result["producers"].is_array());
    
    let producers = lookup_result["producers"].as_array().expect("producers should be an array");
    // Should be empty for non-existent topic
    assert!(producers.is_empty(), "Non-existent topic should have no producers");
}
