//! Basic tests for nsqlookupd functionality

use nsqlookupd::server::{NsqlookupdServer, Producer, RegistrationDB};
use nsq_common::NsqlookupdConfig;

#[tokio::test]
async fn test_producer_registration() {
    let config = NsqlookupdConfig::default();
    let server = NsqlookupdServer::new(config).expect("Failed to create server");
    
    let producer = Producer::new(
        "127.0.0.1:12345".to_string(),
        "test-host".to_string(),
        "127.0.0.1".to_string(),
        4150,
        4151,
        "1.0.0".to_string(),
    );
    
    server.db.register_producer("test-topic".to_string(), producer.clone());
    
    let producers = server.db.get_producers("test-topic");
    assert_eq!(producers.len(), 1);
    assert_eq!(producers[0].hostname, "test-host");
    assert_eq!(producers[0].tcp_port, 4150);
}

#[tokio::test]
async fn test_producer_heartbeat() {
    let mut producer = Producer::new(
        "127.0.0.1:12345".to_string(),
        "test-host".to_string(),
        "127.0.0.1".to_string(),
        4150,
        4151,
        "1.0.0".to_string(),
    );
    
    let initial_update = producer.last_update;
    
    // Wait a bit to ensure time difference
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    
    producer.update_heartbeat();
    
    assert!(producer.last_update > initial_update);
}

#[tokio::test]
async fn test_producer_tombstone() {
    let mut producer = Producer::new(
        "127.0.0.1:12345".to_string(),
        "test-host".to_string(),
        "127.0.0.1".to_string(),
        4150,
        4151,
        "1.0.0".to_string(),
    );
    
    assert!(!producer.tombstoned);
    assert!(producer.tombstoned_at.is_none());
    
    producer.tombstone();
    
    assert!(producer.tombstoned);
    assert!(producer.tombstoned_at.is_some());
}

#[tokio::test]
async fn test_channel_management() {
    let db = RegistrationDB::new();
    
    // Add channels
    db.add_channel("test-topic", "channel1");
    db.add_channel("test-topic", "channel2");
    db.add_channel("test-topic", "channel1"); // Duplicate should be ignored
    
    let channels = db.get_channels("test-topic");
    assert_eq!(channels.len(), 2);
    assert!(channels.contains(&"channel1".to_string()));
    assert!(channels.contains(&"channel2".to_string()));
    
    // Remove channel
    db.remove_channel("test-topic", "channel1");
    let channels = db.get_channels("test-topic");
    assert_eq!(channels.len(), 1);
    assert_eq!(channels[0], "channel2");
}

#[tokio::test]
async fn test_producer_id_generation() {
    let producer = Producer::new(
        "127.0.0.1:12345".to_string(),
        "test-host".to_string(),
        "127.0.0.1".to_string(),
        4150,
        4151,
        "1.0.0".to_string(),
    );
    
    assert_eq!(producer.get_id(), "127.0.0.1:4150");
    assert_eq!(producer.get_http_url(), "http://127.0.0.1:4151");
    assert_eq!(producer.get_tcp_address(), "127.0.0.1:4150");
}
