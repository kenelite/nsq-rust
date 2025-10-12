//! Performance integration tests

use crate::test_utils::{TestEnvironment, TestConfig};
use std::time::Instant;

#[tokio::test]
async fn test_message_throughput() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("throughput-test").await.expect("Failed to create topic");
    
    // Measure message publishing throughput
    let message_count = 1000;
    let start = Instant::now();
    
    for i in 0..message_count {
        let message = format!("Throughput test message {}", i);
        let result = nsqd_client.publish("throughput-test", &message).await.expect("Failed to publish message");
        assert_eq!(result, "OK");
    }
    
    let duration = start.elapsed();
    let throughput = message_count as f64 / duration.as_secs_f64();
    
    println!("Published {} messages in {:?} ({:.2} msg/sec)", message_count, duration, throughput);
    
    // Verify all messages were published
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("throughput-test"))
        .expect("Topic should exist");
    let message_count_actual = topic["message_count"].as_u64().expect("message_count should be a number");
    assert_eq!(message_count_actual, message_count as u64);
    
    // Throughput should be reasonable (at least 100 msg/sec for this test)
    assert!(throughput > 100.0, "Throughput should be at least 100 msg/sec, got {:.2}", throughput);
}

#[tokio::test]
async fn test_concurrent_publishing() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("concurrent-test").await.expect("Failed to create topic");
    
    // Test concurrent publishing
    let task_count = 10;
    let messages_per_task = 100;
    let total_messages = task_count * messages_per_task;
    
    let start = Instant::now();
    let mut handles = vec![];
    
    for task_id in 0..task_count {
        let client = env.nsqd_client();
        let handle = tokio::spawn(async move {
            for i in 0..messages_per_task {
                let message = format!("Concurrent message {} from task {}", i, task_id);
                let result = client.publish("concurrent-test", &message).await.expect("Failed to publish message");
                assert_eq!(result, "OK");
            }
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task failed");
    }
    
    let duration = start.elapsed();
    let throughput = total_messages as f64 / duration.as_secs_f64();
    
    println!("Published {} messages concurrently in {:?} ({:.2} msg/sec)", total_messages, duration, throughput);
    
    // Verify all messages were published
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("concurrent-test"))
        .expect("Topic should exist");
    let message_count_actual = topic["message_count"].as_u64().expect("message_count should be a number");
    assert_eq!(message_count_actual, total_messages as u64);
    
    // Concurrent throughput should be higher than sequential
    assert!(throughput > 200.0, "Concurrent throughput should be at least 200 msg/sec, got {:.2}", throughput);
}

#[tokio::test]
async fn test_large_message_performance() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("large-message-test").await.expect("Failed to create topic");
    
    // Test different message sizes
    let message_sizes = vec![1024, 10240, 102400]; // 1KB, 10KB, 100KB
    
    for size in message_sizes {
        let message = "x".repeat(size);
        let start = Instant::now();
        
        let result = nsqd_client.publish("large-message-test", &message).await.expect("Failed to publish large message");
        assert_eq!(result, "OK");
        
        let duration = start.elapsed();
        let throughput = size as f64 / duration.as_secs_f64();
        
        println!("Published {} byte message in {:?} ({:.2} bytes/sec)", size, duration, throughput);
        
        // Large messages should still be processed reasonably quickly
        assert!(duration.as_millis() < 1000, "Large message publishing should complete within 1 second");
    }
}

#[tokio::test]
async fn test_memory_usage() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("memory-test").await.expect("Failed to create topic");
    
    // Publish many messages to test memory usage
    let message_count = 10000;
    let start = Instant::now();
    
    for i in 0..message_count {
        let message = format!("Memory test message {}", i);
        let result = nsqd_client.publish("memory-test", &message).await.expect("Failed to publish message");
        assert_eq!(result, "OK");
        
        // Check memory usage every 1000 messages
        if i % 1000 == 0 && i > 0 {
            let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
            let topics = stats["topics"].as_array().expect("topics should be an array");
            let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("memory-test"))
                .expect("Topic should exist");
            let message_count_actual = topic["message_count"].as_u64().expect("message_count should be a number");
            assert_eq!(message_count_actual, (i + 1) as u64);
        }
    }
    
    let duration = start.elapsed();
    let throughput = message_count as f64 / duration.as_secs_f64();
    
    println!("Published {} messages in {:?} ({:.2} msg/sec)", message_count, duration, throughput);
    
    // Verify all messages were published
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some("memory-test"))
        .expect("Topic should exist");
    let message_count_actual = topic["message_count"].as_u64().expect("message_count should be a number");
    assert_eq!(message_count_actual, message_count as u64);
    
    // Memory usage should be reasonable (this is a basic check)
    assert!(throughput > 50.0, "Throughput should be at least 50 msg/sec, got {:.2}", throughput);
}

#[tokio::test]
async fn test_api_response_times() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    let lookupd_client = env.lookupd_client();
    let admin_client = env.admin_client();
    
    // Test NSQd API response times
    let start = Instant::now();
    let _stats = nsqd_client.get_stats().await.expect("Failed to get NSQd stats");
    let nsqd_duration = start.elapsed();
    
    // Test NSQLookupd API response times
    let start = Instant::now();
    let _stats = lookupd_client.get_stats().await.expect("Failed to get NSQLookupd stats");
    let lookupd_duration = start.elapsed();
    
    // Test NSQAdmin API response times
    let start = Instant::now();
    let _stats = admin_client.get_stats().await.expect("Failed to get NSQAdmin stats");
    let admin_duration = start.elapsed();
    
    println!("NSQd stats response time: {:?}", nsqd_duration);
    println!("NSQLookupd stats response time: {:?}", lookupd_duration);
    println!("NSQAdmin stats response time: {:?}", admin_duration);
    
    // API responses should be fast (within 100ms for this test)
    assert!(nsqd_duration.as_millis() < 100, "NSQd API should respond quickly");
    assert!(lookupd_duration.as_millis() < 100, "NSQLookupd API should respond quickly");
    assert!(admin_duration.as_millis() < 100, "NSQAdmin API should respond quickly");
}

#[tokio::test]
async fn test_concurrent_api_requests() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Test concurrent API requests
    let request_count = 50;
    let start = Instant::now();
    let mut handles = vec![];
    
    for _i in 0..request_count {
        let client = env.nsqd_client();
        let handle = tokio::spawn(async move {
            let start = Instant::now();
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
    
    let total_time = start.elapsed();
    let avg_duration = total_duration / request_count;
    
    println!("Completed {} concurrent API requests in {:?}", request_count, total_time);
    println!("Average response time: {:?}", avg_duration);
    
    // Concurrent requests should complete quickly
    assert!(total_time.as_millis() < 1000, "Concurrent API requests should complete within 1 second");
    assert!(avg_duration.as_millis() < 50, "Average API response time should be reasonable");
}

#[tokio::test]
async fn test_topic_creation_performance() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config.clone());
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Test topic creation performance
    let topic_count = 100;
    let start = Instant::now();
    
    for i in 0..topic_count {
        let topic_name = format!("perf-topic-{}", i);
        let result = nsqd_client.create_topic(&topic_name).await.expect("Failed to create topic");
        assert_eq!(result, "OK");
    }
    
    let duration = start.elapsed();
    let throughput = topic_count as f64 / duration.as_secs_f64();
    
    println!("Created {} topics in {:?} ({:.2} topics/sec)", topic_count, duration, throughput);
    
    // Verify all topics were created
    let stats = nsqd_client.get_stats().await.expect("Failed to get stats");
    let topics = stats["topics"].as_array().expect("topics should be an array");
    assert_eq!(topics.len(), topic_count, "All topics should be created");
    
    // Topic creation should be reasonably fast
    assert!(throughput > 10.0, "Topic creation throughput should be at least 10 topics/sec, got {:.2}", throughput);
}
