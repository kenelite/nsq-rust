//! API compatibility tests with original NSQ

use crate::integration::test_utils::{TestEnvironment, TestConfig};

#[tokio::test]
async fn test_nsqd_http_api_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Test ping endpoint
    let ping_result = nsqd_client.ping().await.expect("NSQd ping failed");
    assert_eq!(ping_result, "OK");
    
    // Test stats endpoint
    let stats = nsqd_client.get_stats().await.expect("Failed to get NSQd stats");
    
    // Verify required fields exist (compatible with original NSQ)
    assert!(stats["version"].is_string());
    assert!(stats["health"].is_string());
    assert!(stats["start_time"].is_number());
    assert!(stats["uptime"].is_string());
    assert!(stats["uptime_seconds"].is_number());
    assert!(stats["topics"].is_array());
    assert!(stats["producers"].is_array());
    
    // Test topic creation
    let result = nsqd_client.create_topic("api-compat-test").await.expect("Failed to create topic");
    assert_eq!(result, "OK");
    
    // Test message publishing
    let result = nsqd_client.publish("api-compat-test", "Test message").await.expect("Failed to publish message");
    assert_eq!(result, "OK");
    
    // Test topic pause/unpause
    let result = nsqd_client.pause_topic("api-compat-test").await.expect("Failed to pause topic");
    assert_eq!(result, "OK");
    
    let result = nsqd_client.unpause_topic("api-compat-test").await.expect("Failed to unpause topic");
    assert_eq!(result, "OK");
    
    // Test topic deletion
    let result = nsqd_client.delete_topic("api-compat-test").await.expect("Failed to delete topic");
    assert_eq!(result, "OK");
}

#[tokio::test]
async fn test_lookupd_http_api_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let lookupd_client = env.lookupd_client();
    
    // Test ping endpoint
    let ping_result = lookupd_client.ping().await.expect("NSQLookupd ping failed");
    assert_eq!(ping_result, "OK");
    
    // Test stats endpoint
    let stats = lookupd_client.get_stats().await.expect("Failed to get NSQLookupd stats");
    
    // Verify required fields exist (compatible with original NSQ)
    assert!(stats["version"].is_string());
    assert!(stats["health"].is_string());
    assert!(stats["start_time"].is_number());
    assert!(stats["uptime"].is_string());
    assert!(stats["uptime_seconds"].is_number());
    assert!(stats["topics"].is_array());
    assert!(stats["channels"].is_array());
    assert!(stats["producers"].is_array());
    
    // Test topics endpoint
    let topics = lookupd_client.get_topics().await.expect("Failed to get topics");
    assert!(topics["topics"].is_array());
    
    // Test lookup endpoint
    let lookup = lookupd_client.lookup_topic("test-topic").await.expect("Failed to lookup topic");
    assert!(lookup["producers"].is_array());
    
    // Test topic creation
    let result = lookupd_client.create_topic("lookupd-compat-test").await.expect("Failed to create topic");
    assert_eq!(result, "OK");
    
    // Test topic deletion
    let result = lookupd_client.delete_topic("lookupd-compat-test").await.expect("Failed to delete topic");
    assert_eq!(result, "OK");
}

#[tokio::test]
async fn test_admin_http_api_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let admin_client = env.admin_client();
    
    // Test ping endpoint
    let ping_result = admin_client.ping().await.expect("NSQAdmin ping failed");
    assert_eq!(ping_result, "OK");
    
    // Test stats endpoint
    let stats = admin_client.get_stats().await.expect("Failed to get NSQAdmin stats");
    
    // Verify required fields exist (compatible with original NSQ)
    assert!(stats["version"].is_string());
    assert!(stats["health"].is_string());
    assert!(stats["topics"].is_array());
    assert!(stats["producers"].is_array());
    
    // Test topics endpoint
    let topics = admin_client.get_topics().await.expect("Failed to get admin topics");
    assert!(topics["topics"].is_array());
    
    // Test nodes endpoint
    let nodes = admin_client.get_nodes().await.expect("Failed to get admin nodes");
    assert!(nodes["producers"].is_array());
}

#[tokio::test]
async fn test_api_response_format_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    let lookupd_client = env.lookupd_client();
    
    // Create topic and publish message
    nsqd_client.create_topic("format-compat-test").await.expect("Failed to create topic");
    nsqd_client.publish("format-compat-test", "Test message").await.expect("Failed to publish message");
    
    // Test NSQd stats format
    let stats = nsqd_client.get_stats().await.expect("Failed to get NSQd stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("format-compat-test"))
        .expect("Topic should exist");
    
    // Verify topic structure (compatible with original NSQ)
    assert!(topic["topic_name"].is_string());
    assert!(topic["channels"].is_array());
    assert!(topic["depth"].is_number());
    assert!(topic["backend_depth"].is_number());
    assert!(topic["message_count"].is_number());
    assert!(topic["paused"].is_boolean());
    
    // Verify channel structure
    let channels = topic["channels"].as_array().expect("channels should be an array");
    if !channels.is_empty() {
        let channel = &channels[0];
        assert!(channel["channel_name"].is_string());
        assert!(channel["depth"].is_number());
        assert!(channel["backend_depth"].is_number());
        assert!(channel["message_count"].is_number());
        assert!(channel["in_flight_count"].is_number());
        assert!(channel["deferred_count"].is_number());
        assert!(channel["requeue_count"].is_number());
        assert!(channel["timeout_count"].is_number());
        assert!(channel["clients"].is_array());
        assert!(channel["paused"].is_boolean());
    }
    
    // Test NSQLookupd stats format
    let stats = lookupd_client.get_stats().await.expect("Failed to get NSQLookupd stats");
    let producers = stats["producers"].as_array().expect("producers should be an array");
    
    // Verify producer structure (compatible with original NSQ)
    if !producers.is_empty() {
        let producer = &producers[0];
        assert!(producer["hostname"].is_string());
        assert!(producer["broadcast_address"].is_string());
        assert!(producer["tcp_port"].is_number());
        assert!(producer["http_port"].is_number());
        assert!(producer["version"].is_string());
        assert!(producer["last_update"].is_string());
    }
}

#[tokio::test]
async fn test_api_error_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    let lookupd_client = env.lookupd_client();
    
    // Test error responses (compatible with original NSQ)
    let result = nsqd_client.pause_topic("nonexistent-topic").await;
    assert!(result.is_err(), "Pausing non-existent topic should return error");
    
    let result = nsqd_client.delete_topic("nonexistent-topic").await;
    assert!(result.is_err(), "Deleting non-existent topic should return error");
    
    // Test lookupd error responses
    let result = lookupd_client.lookup_topic("nonexistent-topic").await;
    assert!(result.is_ok(), "Lookupd should handle non-existent topics gracefully");
    
    let lookup_result = result.expect("Lookup should succeed");
    let producers = lookup_result["producers"].as_array().expect("producers should be an array");
    assert!(producers.is_empty(), "Non-existent topic should have no producers");
}

#[tokio::test]
async fn test_api_method_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let client = reqwest::Client::new();
    
    // Test HTTP method compatibility
    let base_url = format!("http://127.0.0.1:{}", config.nsqd_http_port);
    
    // GET /stats should work
    let response = client.get(&format!("{}/stats", base_url)).send().await.expect("Failed to send request");
    assert!(response.status().is_success(), "GET /stats should succeed");
    
    // GET /ping should work
    let response = client.get(&format!("{}/ping", base_url)).send().await.expect("Failed to send request");
    assert!(response.status().is_success(), "GET /ping should succeed");
    
    // POST /pub should work
    let response = client.post(&format!("{}/pub?topic=test", base_url))
        .body("test message")
        .send()
        .await
        .expect("Failed to send request");
    assert!(response.status().is_success(), "POST /pub should succeed");
    
    // POST /topic/create should work
    let response = client.post(&format!("{}/topic/create?topic=test-topic", base_url))
        .send()
        .await
        .expect("Failed to send request");
    assert!(response.status().is_success(), "POST /topic/create should succeed");
}

#[tokio::test]
async fn test_api_parameter_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", config.nsqd_http_port);
    
    // Test query parameter compatibility
    let response = client.post(&format!("{}/pub?topic=param-test", base_url))
        .body("test message")
        .send()
        .await
        .expect("Failed to send request");
    assert!(response.status().is_success(), "POST /pub with topic parameter should succeed");
    
    // Test topic creation with parameter
    let response = client.post(&format!("{}/topic/create?topic=param-test-topic", base_url))
        .send()
        .await
        .expect("Failed to send request");
    assert!(response.status().is_success(), "POST /topic/create with topic parameter should succeed");
    
    // Test topic pause with parameter
    let response = client.post(&format!("{}/topic/pause?topic=param-test-topic", base_url))
        .send()
        .await
        .expect("Failed to send request");
    assert!(response.status().is_success(), "POST /topic/pause with topic parameter should succeed");
    
    // Test topic unpause with parameter
    let response = client.post(&format!("{}/topic/unpause?topic=param-test-topic", base_url))
        .send()
        .await
        .expect("Failed to send request");
    assert!(response.status().is_success(), "POST /topic/unpause with topic parameter should succeed");
}

#[tokio::test]
async fn test_api_content_type_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", config.nsqd_http_port);
    
    // Test different content types
    let content_types = vec![
        "text/plain",
        "application/json",
        "application/x-www-form-urlencoded",
    ];
    
    for content_type in content_types {
        let response = client.post(&format!("{}/pub?topic=content-type-test", base_url))
            .header("Content-Type", content_type)
            .body("test message")
            .send()
            .await
            .expect("Failed to send request");
        
        // Should succeed regardless of content type
        assert!(response.status().is_success(), "POST /pub with Content-Type {} should succeed", content_type);
    }
}

#[tokio::test]
async fn test_api_timeout_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Test API timeout handling
    let start = std::time::Instant::now();
    let _stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let duration = start.elapsed();
    
    // API should respond within reasonable time
    assert!(duration.as_millis() < 1000, "API should respond within 1 second");
    
    // Test concurrent requests
    let mut handles = vec![];
    
    for _i in 0..10 {
        let client = env.nsqd_client();
        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            let _stats = client.get_stats().await.expect("Failed to get stats");
            start.elapsed()
        });
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let mut total_duration = std::time::Duration::new(0, 0);
    for handle in handles {
        let duration = handle.await.expect("Task failed");
        total_duration += duration;
    }
    
    // Average response time should be reasonable
    let avg_duration = total_duration / 10;
    assert!(avg_duration.as_millis() < 100, "Average API response time should be reasonable");
}
