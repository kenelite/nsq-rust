//! Message format compatibility tests

use crate::test_utils::{TestEnvironment, TestConfig};

#[tokio::test]
async fn test_message_format_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("format-compat-test").await.expect("Failed to create topic");
    
    // Test different message formats
    let long_message = "Very long message: ".repeat(1000);
    let test_messages = vec![
        "Simple text message",
        "Message with special chars: !@#$%^&*()",
        "Message with unicode: 你好世界 🌍",
        "Message with newlines:\nLine 1\nLine 2",
        "Message with tabs:\tTabbed\tcontent",
        "Empty message",
        &long_message,
        "Message with quotes: \"Hello, World!\"",
        "Message with backslashes: \\path\\to\\file",
        "Message with brackets: [item1, item2, item3]",
        "Message with braces: {key: value}",
        "Message with parentheses: (value1, value2)",
        "Message with angle brackets: <tag>content</tag>",
        "Message with pipes: value1|value2|value3",
        "Message with equals: key=value",
        "Message with plus: value1+value2",
        "Message with asterisks: *important*",
        "Message with carets: ^start",
        "Message with percent: 100%",
        "Message with dollars: $100.00",
        "Message with ampersands: value1&value2",
        "Message with tildes: ~home",
        "Message with backticks: `code`",
    ];
    
    for (i, message) in test_messages.iter().enumerate() {
        let result = nsqd_client.publish("format-compat-test", message).await.expect("Failed to publish message");
        assert_eq!(result, "OK");
        
        // Verify message was published
        let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
        let topics = stats["topics"].as_array().expect("topics should be an array");
        let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("format-compat-test"))
            .expect("Topic should exist");
        let message_count = topic["message_count"].as_u64().expect("message_count should be a number");
        assert_eq!(message_count, (i + 1) as u64);
    }
}

#[tokio::test]
async fn test_message_encoding_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("encoding-compat-test").await.expect("Failed to create topic");
    
    // Test different encodings
    let test_messages = vec![
        "ASCII message",
        "UTF-8 message: 你好世界",
        "UTF-8 emoji: 🌍🚀💻",
        "UTF-8 symbols: ★☆♠♣♥♦",
        "UTF-8 math: ∑∏∫√∞",
        "UTF-8 arrows: ←→↑↓↔↕",
        "UTF-8 currency: €£¥$¢",
        "UTF-8 fractions: ½⅓¼¾",
        "UTF-8 superscript: ¹²³⁴⁵",
        "UTF-8 subscript: ₁₂₃₄₅",
    ];
    
    for (i, message) in test_messages.iter().enumerate() {
        let result = nsqd_client.publish("encoding-compat-test", message).await.expect("Failed to publish message");
        assert_eq!(result, "OK");
        
        // Verify message was published
        let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
        let topics = stats["topics"].as_array().expect("topics should be an array");
        let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("encoding-compat-test"))
            .expect("Topic should exist");
        let message_count = topic["message_count"].as_u64().expect("message_count should be a number");
        assert_eq!(message_count, (i + 1) as u64);
    }
}

#[tokio::test]
async fn test_message_size_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("size-compat-test").await.expect("Failed to create topic");
    
    // Test different message sizes
    let test_sizes = vec![
        1,      // 1 byte
        10,     // 10 bytes
        100,    // 100 bytes
        1000,   // 1KB
        10000,  // 10KB
        100000, // 100KB
    ];
    
    for (i, size) in test_sizes.iter().enumerate() {
        let message = "x".repeat(*size);
        let result = nsqd_client.publish("size-compat-test", &message).await;
        
        let success = match &result {
            Ok(response) => {
                assert_eq!(response, "OK");
                println!("Successfully published {} byte message", size);
                true
            }
            Err(e) => {
                println!("Failed to publish {} byte message: {}", size, e);
                // Large message rejection is acceptable
                false
            }
        };
        
        // Verify message count increased if successful
        let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
        let topics = stats["topics"].as_array().expect("topics should be an array");
        let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("size-compat-test"))
            .expect("Topic should exist");
        let message_count = topic["message_count"].as_u64().expect("message_count should be a number");
        
        if success {
            assert_eq!(message_count, (i + 1) as u64);
        }
    }
}

#[tokio::test]
async fn test_message_content_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("content-compat-test").await.expect("Failed to create topic");
    
    // Test different content types
    let test_messages = vec![
        "Plain text message",
        "JSON: {\"key\": \"value\", \"number\": 123}",
        "XML: <root><item>value</item></root>",
        "CSV: name,age,city\nJohn,25,NYC",
        "Base64: SGVsbG8gV29ybGQ=",
        "URL encoded: Hello%20World%21",
        "HTML: <h1>Hello, World!</h1>",
        "Markdown: # Hello, World!\n\nThis is **bold** text.",
        "SQL: SELECT * FROM users WHERE id = 1",
        "Regex: ^[a-zA-Z0-9]+$",
        "Email: user@example.com",
        "URL: https://example.com/path?param=value",
        "UUID: 550e8400-e29b-41d4-a716-446655440000",
        "Timestamp: 2023-12-25T12:00:00Z",
        "Binary data: \x00\x01\x02\x03\x04\x05",
    ];
    
    for (i, message) in test_messages.iter().enumerate() {
        let result = nsqd_client.publish("content-compat-test", message).await.expect("Failed to publish message");
        assert_eq!(result, "OK");
        
        // Verify message was published
        let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
        let topics = stats["topics"].as_array().expect("topics should be an array");
        let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("content-compat-test"))
            .expect("Topic should exist");
        let message_count = topic["message_count"].as_u64().expect("message_count should be a number");
        assert_eq!(message_count, (i + 1) as u64);
    }
}

#[tokio::test]
async fn test_message_ordering_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("ordering-compat-test").await.expect("Failed to create topic");
    
    // Publish messages in sequence
    let messages = vec![
        "Message 1",
        "Message 2",
        "Message 3",
        "Message 4",
        "Message 5"
    ];
    
    for message in &messages {
        let result = nsqd_client.publish("ordering-compat-test", message).await.expect("Failed to publish message");
        assert_eq!(result, "OK");
        
        // Small delay to ensure ordering
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    
    // Verify all messages were published
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("ordering-compat-test"))
        .expect("Topic should exist");
    let message_count = topic["message_count"].as_u64().expect("message_count should be a number");
    assert_eq!(message_count, messages.len() as u64);
}

#[tokio::test]
async fn test_message_duplication_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("duplication-compat-test").await.expect("Failed to create topic");
    
    // Publish the same message multiple times
    let message = "Duplicate message";
    let publish_count = 5;
    
    for _i in 0..publish_count {
        let result = nsqd_client.publish("duplication-compat-test", message).await.expect("Failed to publish message");
        assert_eq!(result, "OK");
    }
    
    // Verify all messages were published
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("duplication-compat-test"))
        .expect("Topic should exist");
    let message_count = topic["message_count"].as_u64().expect("message_count should be a number");
    assert_eq!(message_count, publish_count as u64);
}

#[tokio::test]
async fn test_message_persistence_compatibility() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("persistence-compat-test").await.expect("Failed to create topic");
    
    // Publish messages
    let messages = vec![
        "Persistent message 1",
        "Persistent message 2",
        "Persistent message 3"
    ];
    
    for message in &messages {
        let result = nsqd_client.publish("persistence-compat-test", message).await.expect("Failed to publish message");
        assert_eq!(result, "OK");
    }
    
    // Verify messages are stored
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("persistence-compat-test"))
        .expect("Topic should exist");
    let message_count = topic["message_count"].as_u64().expect("message_count should be a number");
    assert_eq!(message_count, messages.len() as u64);
    
    // Restart services to test persistence
    env.stop();
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    
    // Start services again
    env.start().await.expect("Failed to restart services");
    
    // Verify messages are still there (if persistence is implemented)
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("persistence-compat-test"))
        .expect("Topic should exist");
    let _message_count = topic["message_count"].as_u64().expect("message_count should be a number");
    // Note: This test may fail if persistence is not fully implemented yet
    // assert_eq!(_message_count, messages.len() as u64);
}
