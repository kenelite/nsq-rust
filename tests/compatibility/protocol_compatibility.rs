//! Protocol compatibility tests with original NSQ

use crate::test_utils::{TestEnvironment, TestConfig};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::time::Duration;

#[tokio::test]
async fn test_tcp_protocol_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    // Test basic TCP protocol compatibility
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", config.nsqd_tcp_port))
        .await
        .expect("Failed to connect to NSQd");
    
    // Send IDENTIFY command (NSQ protocol)
    let identify_cmd = b"IDENTIFY\n";
    stream.write_all(identify_cmd).await.expect("Failed to write IDENTIFY");
    
    // Read response
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await.expect("Failed to read response");
    let response = String::from_utf8_lossy(&buffer[..n]);
    
    // Should receive OK response
    assert!(response.contains("OK"), "Should receive OK response to IDENTIFY");
    
    // Send SUBSCRIBE command
    let subscribe_cmd = b"SUBSCRIBE test-topic test-channel\n";
    stream.write_all(subscribe_cmd).await.expect("Failed to write SUBSCRIBE");
    
    // Read response
    let n = stream.read(&mut buffer).await.expect("Failed to read response");
    let response = String::from_utf8_lossy(&buffer[..n]);
    
    // Should receive OK response
    assert!(response.contains("OK"), "Should receive OK response to SUBSCRIBE");
    
    // Send RDY command
    let rdy_cmd = b"RDY 1\n";
    stream.write_all(rdy_cmd).await.expect("Failed to write RDY");
    
    // Send FIN command (finish message)
    let fin_cmd = b"FIN test-message-id\n";
    stream.write_all(fin_cmd).await.expect("Failed to write FIN");
    
    // Read response
    let n = stream.read(&mut buffer).await.expect("Failed to read response");
    let response = String::from_utf8_lossy(&buffer[..n]);
    
    // Should receive OK response
    assert!(response.contains("OK"), "Should receive OK response to FIN");
}

#[tokio::test]
async fn test_http_api_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    let lookupd_client = env.lookupd_client();
    
    // Test NSQd HTTP API compatibility
    let ping_result = nsqd_client.ping().await.expect("NSQd ping failed");
    assert_eq!(ping_result, "OK");
    
    let stats = nsqd_client.get_stats().await.expect("Failed to get NSQd stats");
    assert!(stats["version"].is_string());
    assert!(stats["health"].is_string());
    assert!(stats["topics"].is_array());
    
    // Test NSQLookupd HTTP API compatibility
    let ping_result = lookupd_client.ping().await.expect("NSQLookupd ping failed");
    assert_eq!(ping_result, "OK");
    
    let stats = lookupd_client.get_stats().await.expect("Failed to get NSQLookupd stats");
    assert!(stats["version"].is_string());
    assert!(stats["health"].is_string());
    
    // Test topic creation and publishing
    let result = nsqd_client.create_topic("compatibility-test").await.expect("Failed to create topic");
    assert_eq!(result, "OK");
    
    let result = nsqd_client.publish("compatibility-test", "Test message").await.expect("Failed to publish message");
    assert_eq!(result, "OK");
    
    // Verify message was published
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("compatibility-test"))
        .expect("Topic should exist");
    let message_count = topic["message_count"].as_u64().expect("message_count should be a number");
    assert_eq!(message_count, 1);
}

#[tokio::test]
async fn test_message_format_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("format-test").await.expect("Failed to create topic");
    
    // Test different message formats
    let test_messages = vec![
        "Simple text message",
        "Message with special chars: !@#$%^&*()",
        "Message with unicode: 你好世界 🌍",
        "Message with newlines:\nLine 1\nLine 2",
        "Message with tabs:\tTabbed\tcontent",
        "Empty message",
        &"Very long message: ".repeat(1000),
    ];
    
    for (i, message) in test_messages.iter().enumerate() {
        let result = nsqd_client.publish("format-test", message).await.expect("Failed to publish message");
        assert_eq!(result, "OK");
        
        // Verify message was published
        let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
        let topics = stats["topics"].as_array().expect("topics should be an array");
        let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("format-test"))
            .expect("Topic should exist");
        let message_count = topic["message_count"].as_u64().expect("message_count should be a number");
        assert_eq!(message_count, (i + 1) as u64);
    }
}

#[tokio::test]
async fn test_wire_protocol_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    // Test wire protocol compatibility
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", config.nsqd_tcp_port))
        .await
        .expect("Failed to connect to NSQd");
    
    // Test IDENTIFY command with parameters
    let identify_cmd = b"IDENTIFY\n";
    stream.write_all(identify_cmd).await.expect("Failed to write IDENTIFY");
    
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await.expect("Failed to read response");
    let response = String::from_utf8_lossy(&buffer[..n]);
    assert!(response.contains("OK"), "IDENTIFY should return OK");
    
    // Test SUBSCRIBE command
    let subscribe_cmd = b"SUBSCRIBE wire-test wire-channel\n";
    stream.write_all(subscribe_cmd).await.expect("Failed to write SUBSCRIBE");
    
    let n = stream.read(&mut buffer).await.expect("Failed to read response");
    let response = String::from_utf8_lossy(&buffer[..n]);
    assert!(response.contains("OK"), "SUBSCRIBE should return OK");
    
    // Test RDY command
    let rdy_cmd = b"RDY 10\n";
    stream.write_all(rdy_cmd).await.expect("Failed to write RDY");
    
    // Test REQ command (requeue message)
    let req_cmd = b"REQ test-message-id 5000\n";
    stream.write_all(req_cmd).await.expect("Failed to write REQ");
    
    let n = stream.read(&mut buffer).await.expect("Failed to read response");
    let response = String::from_utf8_lossy(&buffer[..n]);
    // REQ might return OK or error depending on message state
    assert!(response.contains("OK") || response.contains("ERROR"), "REQ should return OK or ERROR");
    
    // Test FIN command
    let fin_cmd = b"FIN test-message-id\n";
    stream.write_all(fin_cmd).await.expect("Failed to write FIN");
    
    let n = stream.read(&mut buffer).await.expect("Failed to read response");
    let response = String::from_utf8_lossy(&buffer[..n]);
    // FIN might return OK or error depending on message state
    assert!(response.contains("OK") || response.contains("ERROR"), "FIN should return OK or ERROR");
}

#[tokio::test]
async fn test_api_response_format_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    let lookupd_client = env.lookupd_client();
    
    // Test NSQd stats response format
    let stats = nsqd_client.get_stats().await.expect("Failed to get NSQd stats");
    
    // Verify required fields exist
    assert!(stats["version"].is_string());
    assert!(stats["health"].is_string());
    assert!(stats["start_time"].is_number());
    assert!(stats["uptime"].is_string());
    assert!(stats["uptime_seconds"].is_number());
    assert!(stats["topics"].is_array());
    assert!(stats["producers"].is_array());
    
    // Test NSQLookupd stats response format
    let stats = lookupd_client.get_stats().await.expect("Failed to get NSQLookupd stats");
    
    // Verify required fields exist
    assert!(stats["version"].is_string());
    assert!(stats["health"].is_string());
    assert!(stats["start_time"].is_number());
    assert!(stats["uptime"].is_string());
    assert!(stats["uptime_seconds"].is_number());
    assert!(stats["topics"].is_array());
    assert!(stats["channels"].is_array());
    assert!(stats["producers"].is_array());
    
    // Test topics endpoint format
    let topics = lookupd_client.get_topics().await.expect("Failed to get topics");
    assert!(topics["topics"].is_array());
    
    // Test lookup endpoint format
    let lookup = lookupd_client.lookup_topic("test-topic").await.expect("Failed to lookup topic");
    assert!(lookup["producers"].is_array());
}

#[tokio::test]
async fn test_error_response_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Test error responses for invalid operations
    let result = nsqd_client.pause_topic("nonexistent-topic").await;
    assert!(result.is_err(), "Pausing non-existent topic should return error");
    
    let result = nsqd_client.delete_topic("nonexistent-topic").await;
    assert!(result.is_err(), "Deleting non-existent topic should return error");
    
    // Test error responses for invalid topic names
    let result = nsqd_client.create_topic("").await;
    assert!(result.is_err(), "Creating topic with empty name should return error");
    
    let result = nsqd_client.create_topic("topic with spaces").await;
    assert!(result.is_err(), "Creating topic with spaces should return error");
}

#[tokio::test]
async fn test_concurrent_protocol_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    // Test concurrent TCP connections
    let mut handles = vec![];
    
    for i in 0..5 {
        let port = config.nsqd_tcp_port;
        let handle = tokio::spawn(async move {
            let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
                .await
                .expect("Failed to connect to NSQd");
            
            // Send IDENTIFY
            let identify_cmd = b"IDENTIFY\n";
            stream.write_all(identify_cmd).await.expect("Failed to write IDENTIFY");
            
            let mut buffer = [0; 1024];
            let n = stream.read(&mut buffer).await.expect("Failed to read response");
            let response = String::from_utf8_lossy(&buffer[..n]);
            assert!(response.contains("OK"), "IDENTIFY should return OK");
            
            // Send SUBSCRIBE
            let subscribe_cmd = format!("SUBSCRIBE concurrent-test-{} concurrent-channel-{}\n", i, i);
            stream.write_all(subscribe_cmd.as_bytes()).await.expect("Failed to write SUBSCRIBE");
            
            let n = stream.read(&mut buffer).await.expect("Failed to read response");
            let response = String::from_utf8_lossy(&buffer[..n]);
            assert!(response.contains("OK"), "SUBSCRIBE should return OK");
        });
        handles.push(handle);
    }
    
    // Wait for all connections to complete
    for handle in handles {
        handle.await.expect("Task failed");
    }
}

#[tokio::test]
async fn test_protocol_version_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    let lookupd_client = env.lookupd_client();
    
    // Test version information
    let nsqd_stats = nsqd_client.get_stats().await.expect("Failed to get NSQd stats");
    let nsqd_version = nsqd_stats["version"].as_str().expect("version should be a string");
    assert!(!nsqd_version.is_empty(), "NSQd version should not be empty");
    
    let lookupd_stats = lookupd_client.get_stats().await.expect("Failed to get NSQLookupd stats");
    let lookupd_version = lookupd_stats["version"].as_str().expect("version should be a string");
    assert!(!lookupd_version.is_empty(), "NSQLookupd version should not be empty");
    
    // Versions should be compatible (same major version)
    // This is a basic check - in a real implementation, you'd parse and compare versions
    assert_eq!(nsqd_version, lookupd_version, "NSQd and NSQLookupd should have compatible versions");
}
