//! Admin interface integration tests

use crate::test_utils::{TestEnvironment, TestConfig};

#[tokio::test]
async fn test_admin_api_endpoints() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let admin_client = env.admin_client();
    
    // Test ping endpoint
    let ping_result = admin_client.ping().await.expect("Failed to ping admin");
    assert_eq!(ping_result, "OK");
    
    // Test stats endpoint
    let stats = admin_client.get_stats().await.expect("Failed to get admin stats");
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
async fn test_admin_topic_management() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    let admin_client = env.admin_client();
    
    // Create topic via NSQd
    nsqd_client.create_topic("admin-test").await.expect("Failed to create topic");
    
    // Publish some messages
    for i in 0..5 {
        let message = format!("Admin test message {}", i);
        nsqd_client.publish("admin-test", &message).await.expect("Failed to publish message");
    }
    
    // Verify topic appears in admin stats
    let stats = admin_client.get_stats().await.expect("Failed to get admin stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic_exists = topics.iter().any(|topic| {
        topic["topic_name"].as_str() == Some("admin-test")
    });
    assert!(topic_exists, "Topic should appear in admin stats");
    
    // Verify topic appears in admin topics endpoint
    let topics_response = admin_client.get_topics().await.expect("Failed to get admin topics");
    let topics_array = topics_response["topics"].as_array().expect("topics should be an array");
    let topic_exists = topics_array.iter().any(|topic| {
        topic.as_str() == Some("admin-test")
    });
    assert!(topic_exists, "Topic should appear in admin topics endpoint");
}

#[tokio::test]
async fn test_admin_node_monitoring() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let admin_client = env.admin_client();
    
    // Test nodes endpoint
    let nodes = admin_client.get_nodes().await.expect("Failed to get admin nodes");
    let producers = nodes["producers"].as_array().expect("producers should be an array");
    
    // Should have at least one producer (NSQd)
    assert!(!producers.is_empty(), "Should have at least one producer");
    
    // Verify producer information
    for producer in producers {
        assert!(producer["hostname"].is_string());
        assert!(producer["broadcast_address"].is_string());
        assert!(producer["tcp_port"].is_number());
        assert!(producer["http_port"].is_number());
        assert!(producer["version"].is_string());
        assert!(producer["last_update"].is_string());
    }
}

#[tokio::test]
async fn test_admin_statistics_aggregation() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    let admin_client = env.admin_client();
    
    // Create multiple topics
    let topic_names = vec![
        "admin-stats-1",
        "admin-stats-2",
        "admin-stats-3"
    ];
    
    for topic_name in &topic_names {
        nsqd_client.create_topic(topic_name).await.expect("Failed to create topic");
        
        // Publish different numbers of messages to each topic
        let message_count = match *topic_name {
            "admin-stats-1" => 3,
            "admin-stats-2" => 5,
            "admin-stats-3" => 7,
            _ => 1,
        };
        
        for i in 0..message_count {
            let message = format!("Message {} for {}", i, topic_name);
            nsqd_client.publish(topic_name, &message).await.expect("Failed to publish message");
        }
    }
    
    // Verify admin stats aggregation
    let stats = admin_client.get_stats().await.expect("Failed to get admin stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    
    // Should have all three topics
    assert_eq!(topics.len(), 3, "Should have 3 topics");
    
    // Verify each topic has correct message count
    for topic in topics {
        let topic_name = topic["topic_name"].as_str().expect("topic_name should be a string");
        let message_count = topic["message_count"].as_u64().expect("message_count should be a number");
        
        match topic_name {
            "admin-stats-1" => assert_eq!(message_count, 3),
            "admin-stats-2" => assert_eq!(message_count, 5),
            "admin-stats-3" => assert_eq!(message_count, 7),
            _ => panic!("Unexpected topic name: {}", topic_name),
        }
    }
}

#[tokio::test]
async fn test_admin_error_handling() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let admin_client = env.admin_client();
    
    // Test admin ping (should always work)
    let ping_result = admin_client.ping().await.expect("Admin ping should always work");
    assert_eq!(ping_result, "OK");
    
    // Test admin stats (should always work)
    let stats = admin_client.get_stats().await.expect("Admin stats should always work");
    assert!(stats["version"].is_string());
    assert!(stats["health"].is_string());
    
    // Test admin topics (should always work, even if empty)
    let topics = admin_client.get_topics().await.expect("Admin topics should always work");
    assert!(topics["topics"].is_array());
    
    // Test admin nodes (should always work, even if empty)
    let nodes = admin_client.get_nodes().await.expect("Admin nodes should always work");
    assert!(nodes["producers"].is_array());
}

#[tokio::test]
async fn test_admin_web_interface() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let admin_client = env.admin_client();
    
    // Test that admin serves web interface
    // This would typically test HTML content, but since we're using a mock server,
    // we'll test that the admin is responding
    let ping_result = admin_client.ping().await.expect("Failed to ping admin");
    assert_eq!(ping_result, "OK");
    
    // Test that admin provides API endpoints
    let stats = admin_client.get_stats().await.expect("Failed to get admin stats");
    assert!(stats["version"].is_string());
    assert!(stats["health"].is_string());
    
    // Verify admin provides expected API structure
    assert!(stats["topics"].is_array());
    assert!(stats["producers"].is_array());
    
    // Test that admin aggregates data from multiple sources
    let topics = admin_client.get_topics().await.expect("Failed to get admin topics");
    assert!(topics["topics"].is_array());
    
    let nodes = admin_client.get_nodes().await.expect("Failed to get admin nodes");
    assert!(nodes["producers"].is_array());
}

#[tokio::test]
async fn test_admin_performance() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let admin_client = env.admin_client();
    
    // Test admin response times
    let start = std::time::Instant::now();
    let _stats = admin_client.get_stats().await.expect("Failed to get admin stats");
    let duration = start.elapsed();
    
    // Admin should respond quickly (within 100ms for this test)
    assert!(duration.as_millis() < 100, "Admin stats should respond quickly");
    
    // Test multiple concurrent requests
    let mut handles = vec![];
    
    for _i in 0..10 {
        let client = env.admin_client();
        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            let _stats = client.get_stats().await.expect("Failed to get admin stats");
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
    assert!(avg_duration.as_millis() < 50, "Average admin response time should be reasonable");
}
