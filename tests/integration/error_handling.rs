//! Error handling integration tests

use crate::test_utils::{TestEnvironment, TestConfig};

#[tokio::test]
async fn test_invalid_topic_names() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Test invalid topic names
    let invalid_names = vec![
        "",  // Empty name
        "topic with spaces",
        "topic\nwith\nnewlines",
        "topic\twith\ttabs",
        "topic#with#special#chars",
        "topic@with@symbols",
        "topic/with/slashes",
        "topic\\with\\backslashes",
        "topic:with:colons",
        "topic;with;semicolons",
        "topic,with,commas",
        "topic.with.dots",
        "topic?with?question?marks",
        "topic!with!exclamation!marks",
        "topic(with)parentheses",
        "topic[with]brackets",
        "topic{with}braces",
        "topic<with>angle>brackets",
        "topic|with|pipes",
        "topic=with=equals",
        "topic+with+plus",
        "topic*with*asterisks",
        "topic^with^carets",
        "topic%with%percent",
        "topic$with$dollars",
        "topic&with&ampersands",
        "topic~with~tildes",
        "topic`with`backticks",
    ];
    
    for topic_name in &invalid_names {
        let result = nsqd_client.create_topic(topic_name).await;
        
        // Invalid topic names should either fail or be sanitized
        match result {
            Ok(response) => {
                // If creation succeeded, verify the topic name was sanitized
                assert_eq!(response, "OK");
                let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
                let topics = stats["topics"].as_array().expect("topics should be an array");
                let topic_exists = topics.iter().any(|topic| {
                    topic["topic_name"].as_str() == Some(topic_name)
                });
                // Topic should either not exist or be sanitized
                assert!(!topic_exists, "Invalid topic name '{}' should be rejected or sanitized", topic_name);
            }
            Err(_) => {
                // Topic creation failed as expected
            }
        }
    }
}

#[tokio::test]
async fn test_invalid_channel_names() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create a valid topic first
    nsqd_client.create_topic("channel-test").await.expect("Failed to create topic");
    
    // Test invalid channel names
    let invalid_names = vec![
        "",  // Empty name
        "channel with spaces",
        "channel\nwith\nnewlines",
        "channel\twith\ttabs",
        "channel#with#special#chars",
        "channel@with@symbols",
        "channel/with/slashes",
        "channel\\with\\backslashes",
        "channel:with:colons",
        "channel;with;semicolons",
        "channel,with,commas",
        "channel.with.dots",
        "channel?with?question?marks",
        "channel!with!exclamation!marks",
        "channel(with)parentheses",
        "channel[with]brackets",
        "channel{with}braces",
        "channel<with>angle>brackets",
        "channel|with|pipes",
        "channel=with=equals",
        "channel+with+plus",
        "channel*with*asterisks",
        "channel^with^carets",
        "channel%with%percent",
        "channel$with$dollars",
        "channel&with&ampersands",
        "channel~with~tildes",
        "channel`with`backticks",
    ];
    
    for channel_name in &invalid_names {
        // Try to pause a channel with invalid name
        let result = nsqd_client.pause_channel("channel-test", channel_name).await;
        
        // Invalid channel names should fail
        assert!(result.is_err(), "Invalid channel name '{}' should be rejected", channel_name);
    }
}

#[tokio::test]
async fn test_nonexistent_topic_operations() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Test operations on non-existent topic
    let nonexistent_topic = "nonexistent-topic";
    
    // Pause non-existent topic
    let result = nsqd_client.pause_topic(nonexistent_topic).await;
    assert!(result.is_err(), "Pausing non-existent topic should fail");
    
    // Unpause non-existent topic
    let result = nsqd_client.unpause_topic(nonexistent_topic).await;
    assert!(result.is_err(), "Unpausing non-existent topic should fail");
    
    // Delete non-existent topic
    let result = nsqd_client.delete_topic(nonexistent_topic).await;
    assert!(result.is_err(), "Deleting non-existent topic should fail");
}

#[tokio::test]
async fn test_nonexistent_channel_operations() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create a topic first
    nsqd_client.create_topic("channel-ops-test").await.expect("Failed to create topic");
    
    // Test operations on non-existent channel
    let nonexistent_channel = "nonexistent-channel";
    
    // Pause non-existent channel
    let result = nsqd_client.pause_channel("channel-ops-test", nonexistent_channel).await;
    assert!(result.is_err(), "Pausing non-existent channel should fail");
    
    // Unpause non-existent channel
    let result = nsqd_client.unpause_channel("channel-ops-test", nonexistent_channel).await;
    assert!(result.is_err(), "Unpausing non-existent channel should fail");
    
    // Delete non-existent channel
    let result = nsqd_client.delete_channel("channel-ops-test", nonexistent_channel).await;
    assert!(result.is_err(), "Deleting non-existent channel should fail");
}

#[tokio::test]
async fn test_large_message_handling() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("large-message-test").await.expect("Failed to create topic");
    
    // Test very large messages
    let large_sizes = vec![
        1024 * 1024,      // 1MB
        10 * 1024 * 1024, // 10MB
        100 * 1024 * 1024, // 100MB
    ];
    
    for size in large_sizes {
        let large_message = "x".repeat(size);
        let result = nsqd_client.publish("large-message-test", &large_message).await;
        
        // Large messages should either succeed or fail gracefully
        match result {
            Ok(response) => {
                assert_eq!(response, "OK");
                println!("Successfully published {} byte message", size);
            }
            Err(e) => {
                println!("Failed to publish {} byte message: {}", size, e);
                // Large message rejection is acceptable
            }
        }
    }
}

#[tokio::test]
async fn test_malformed_requests() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let client = reqwest::Client::new();
    
    // Test malformed HTTP requests
    let malformed_requests = vec![
        // Invalid URLs
        format!("http://127.0.0.1:{}/invalid-endpoint", config.nsqd_http_port),
        format!("http://127.0.0.1:{}/pub", config.nsqd_http_port), // Missing topic parameter
        format!("http://127.0.0.1:{}/topic/pause", config.nsqd_http_port), // Missing topic parameter
        
        // Invalid methods
        format!("http://127.0.0.1:{}/stats", config.nsqd_http_port), // GET instead of POST for some endpoints
    ];
    
    for url in &malformed_requests {
        let result = client.get(url).send().await;
        
        // Malformed requests should either fail or return appropriate error codes
        match result {
            Ok(response) => {
                let status = response.status();
                // Should return 400 (Bad Request) or 404 (Not Found) or 405 (Method Not Allowed)
                assert!(status.is_client_error(), "Malformed request should return client error, got {}", status);
            }
            Err(_) => {
                // Request failed as expected
            }
        }
    }
}

#[tokio::test]
async fn test_concurrent_error_conditions() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("concurrent-error-test").await.expect("Failed to create topic");
    
    // Test concurrent operations that might cause errors
    let mut handles = vec![];
    
    // Concurrent pause/unpause operations
    for i in 0..10 {
        let client = env.nsqd_client();
        let handle = tokio::spawn(async move {
            if i % 2 == 0 {
                let result = client.pause_topic("concurrent-error-test").await;
                // Should succeed or fail gracefully
                match result {
                    Ok(response) => assert_eq!(response, "OK"),
                    Err(_) => {} // Acceptable
                }
            } else {
                let result = client.unpause_topic("concurrent-error-test").await;
                // Should succeed or fail gracefully
                match result {
                    Ok(response) => assert_eq!(response, "OK"),
                    Err(_) => {} // Acceptable
                }
            }
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for handle in handles {
        handle.await.expect("Task failed");
    }
    
    // Verify topic is in a consistent state
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("concurrent-error-test"))
        .expect("Topic should exist");
    
    // Topic should have a valid paused state
    assert!(topic["paused"].is_boolean(), "Topic should have a valid paused state");
}

#[tokio::test]
async fn test_service_unavailability() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Test operations when service is running
    let result = nsqd_client.ping().await.expect("Service should be available");
    assert_eq!(result, "OK");
    
    // Stop services
    env.stop();
    
    // Test operations when service is unavailable
    let result = nsqd_client.ping().await;
    assert!(result.is_err(), "Service should be unavailable after stopping");
    
    // Test other operations when service is unavailable
    let result = nsqd_client.get_stats().await;
    assert!(result.is_err(), "Stats should fail when service is unavailable");
    
    let result = nsqd_client.create_topic("test").await;
    assert!(result.is_err(), "Topic creation should fail when service is unavailable");
}

#[tokio::test]
async fn test_resource_exhaustion() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Test creating many topics to potentially exhaust resources
    let topic_count = 1000;
    let mut successful_creations = 0;
    
    for i in 0..topic_count {
        let topic_name = format!("resource-test-{}", i);
        let result = nsqd_client.create_topic(&topic_name).await;
        
        match result {
            Ok(response) => {
                assert_eq!(response, "OK");
                successful_creations += 1;
            }
            Err(_) => {
                // Resource exhaustion is acceptable
                break;
            }
        }
    }
    
    println!("Successfully created {} topics before resource exhaustion", successful_creations);
    
    // Should have created at least some topics
    assert!(successful_creations > 0, "Should have created at least some topics");
    
    // Verify created topics exist
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    assert_eq!(topics.len(), successful_creations, "All created topics should exist");
}
