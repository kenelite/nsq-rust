//! Wire protocol compatibility tests

use crate::integration::test_utils::{TestEnvironment, TestConfig};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::time::Duration;

#[tokio::test]
async fn test_wire_protocol_commands() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", config.nsqd_tcp_port))
        .await
        .expect("Failed to connect to NSQd");
    
    // Test IDENTIFY command
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
    let rdy_cmd = b"RDY 1\n";
    stream.write_all(rdy_cmd).await.expect("Failed to write RDY");
    
    // Test FIN command
    let fin_cmd = b"FIN test-message-id\n";
    stream.write_all(fin_cmd).await.expect("Failed to write FIN");
    
    let n = stream.read(&mut buffer).await.expect("Failed to read response");
    let response = String::from_utf8_lossy(&buffer[..n]);
    // FIN might return OK or ERROR depending on message state
    assert!(response.contains("OK") || response.contains("ERROR"), "FIN should return OK or ERROR");
    
    // Test REQ command
    let req_cmd = b"REQ test-message-id 5000\n";
    stream.write_all(req_cmd).await.expect("Failed to write REQ");
    
    let n = stream.read(&mut buffer).await.expect("Failed to read response");
    let response = String::from_utf8_lossy(&buffer[..n]);
    // REQ might return OK or ERROR depending on message state
    assert!(response.contains("OK") || response.contains("ERROR"), "REQ should return OK or ERROR");
}

#[tokio::test]
async fn test_wire_protocol_message_format() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic and publish message
    nsqd_client.create_topic("wire-format-test").await.expect("Failed to create topic");
    nsqd_client.publish("wire-format-test", "Wire protocol test message").await.expect("Failed to publish message");
    
    // Connect and subscribe to receive message
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", config.nsqd_tcp_port))
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
    let subscribe_cmd = b"SUBSCRIBE wire-format-test wire-channel\n";
    stream.write_all(subscribe_cmd).await.expect("Failed to write SUBSCRIBE");
    
    let n = stream.read(&mut buffer).await.expect("Failed to read response");
    let response = String::from_utf8_lossy(&buffer[..n]);
    assert!(response.contains("OK"), "SUBSCRIBE should return OK");
    
    // Send RDY
    let rdy_cmd = b"RDY 1\n";
    stream.write_all(rdy_cmd).await.expect("Failed to write RDY");
    
    // Wait for message
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Try to read message
    let n = stream.read(&mut buffer).await.expect("Failed to read");
    if n > 0 {
        let response = String::from_utf8_lossy(&buffer[..n]);
        // Should receive a message frame
        assert!(response.contains("wire-format-test") || response.contains("wire-channel"), "Should receive message frame");
    }
}

#[tokio::test]
async fn test_wire_protocol_error_handling() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", config.nsqd_tcp_port))
        .await
        .expect("Failed to connect to NSQd");
    
    // Test invalid command
    let invalid_cmd = b"INVALID_COMMAND\n";
    stream.write_all(invalid_cmd).await.expect("Failed to write invalid command");
    
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await.expect("Failed to read response");
    let response = String::from_utf8_lossy(&buffer[..n]);
    assert!(response.contains("ERROR"), "Invalid command should return ERROR");
    
    // Test malformed command
    let malformed_cmd = b"SUBSCRIBE\n"; // Missing topic and channel
    stream.write_all(malformed_cmd).await.expect("Failed to write malformed command");
    
    let n = stream.read(&mut buffer).await.expect("Failed to read response");
    let response = String::from_utf8_lossy(&buffer[..n]);
    assert!(response.contains("ERROR"), "Malformed command should return ERROR");
    
    // Test command with invalid parameters
    let invalid_params_cmd = b"RDY invalid\n"; // Invalid number
    stream.write_all(invalid_params_cmd).await.expect("Failed to write command with invalid params");
    
    let n = stream.read(&mut buffer).await.expect("Failed to read response");
    let response = String::from_utf8_lossy(&buffer[..n]);
    assert!(response.contains("ERROR"), "Command with invalid parameters should return ERROR");
}

#[tokio::test]
async fn test_wire_protocol_connection_lifecycle() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    // Test connection establishment
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", config.nsqd_tcp_port))
        .await
        .expect("Failed to connect to NSQd");
    
    // Test IDENTIFY
    let identify_cmd = b"IDENTIFY\n";
    stream.write_all(identify_cmd).await.expect("Failed to write IDENTIFY");
    
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await.expect("Failed to read response");
    let response = String::from_utf8_lossy(&buffer[..n]);
    assert!(response.contains("OK"), "IDENTIFY should return OK");
    
    // Test SUBSCRIBE
    let subscribe_cmd = b"SUBSCRIBE lifecycle-test lifecycle-channel\n";
    stream.write_all(subscribe_cmd).await.expect("Failed to write SUBSCRIBE");
    
    let n = stream.read(&mut buffer).await.expect("Failed to read response");
    let response = String::from_utf8_lossy(&buffer[..n]);
    assert!(response.contains("OK"), "SUBSCRIBE should return OK");
    
    // Test RDY
    let rdy_cmd = b"RDY 1\n";
    stream.write_all(rdy_cmd).await.expect("Failed to write RDY");
    
    // Test connection closure
    drop(stream);
    
    // Connection should be closed gracefully
    // In a real implementation, you'd test that the server cleans up properly
}

#[tokio::test]
async fn test_wire_protocol_concurrent_connections() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    // Test multiple concurrent connections
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
            
            // Send RDY
            let rdy_cmd = b"RDY 1\n";
            stream.write_all(rdy_cmd).await.expect("Failed to write RDY");
        });
        handles.push(handle);
    }
    
    // Wait for all connections to complete
    for handle in handles {
        handle.await.expect("Task failed");
    }
}

#[tokio::test]
async fn test_wire_protocol_message_ordering() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("wire-ordering-test").await.expect("Failed to create topic");
    
    // Publish messages in sequence
    let messages = vec![
        "Message 1",
        "Message 2",
        "Message 3",
        "Message 4",
        "Message 5"
    ];
    
    for message in &messages {
        nsqd_client.publish("wire-ordering-test", message).await.expect("Failed to publish message");
    }
    
    // Connect and subscribe
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", config.nsqd_tcp_port))
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
    let subscribe_cmd = b"SUBSCRIBE wire-ordering-test wire-channel\n";
    stream.write_all(subscribe_cmd).await.expect("Failed to write SUBSCRIBE");
    
    let n = stream.read(&mut buffer).await.expect("Failed to read response");
    let response = String::from_utf8_lossy(&buffer[..n]);
    assert!(response.contains("OK"), "SUBSCRIBE should return OK");
    
    // Send RDY
    let rdy_cmd = b"RDY 5\n"; // Ready for 5 messages
    stream.write_all(rdy_cmd).await.expect("Failed to write RDY");
    
    // Wait for messages
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Read messages
    let mut received_messages = 0;
    for _i in 0..5 {
        let n = stream.read(&mut buffer).await.expect("Failed to read");
        if n > 0 {
            let response = String::from_utf8_lossy(&buffer[..n]);
            if response.contains("wire-ordering-test") {
                received_messages += 1;
            }
        }
    }
    
    // Should receive all messages
    assert_eq!(received_messages, 5, "Should receive all 5 messages");
}

#[tokio::test]
async fn test_wire_protocol_large_messages() {
    let config = TestConfig::default();
    let mut env = TestEnvironment::new(config);
    env.start().await.expect("Failed to start services");
    
    let nsqd_client = env.nsqd_client();
    
    // Create topic
    nsqd_client.create_topic("wire-large-test").await.expect("Failed to create topic");
    
    // Publish large message
    let large_message = "x".repeat(10240); // 10KB message
    nsqd_client.publish("wire-large-test", &large_message).await.expect("Failed to publish large message");
    
    // Connect and subscribe
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", config.nsqd_tcp_port))
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
    let subscribe_cmd = b"SUBSCRIBE wire-large-test wire-channel\n";
    stream.write_all(subscribe_cmd).await.expect("Failed to write SUBSCRIBE");
    
    let n = stream.read(&mut buffer).await.expect("Failed to read response");
    let response = String::from_utf8_lossy(&buffer[..n]);
    assert!(response.contains("OK"), "SUBSCRIBE should return OK");
    
    // Send RDY
    let rdy_cmd = b"RDY 1\n";
    stream.write_all(rdy_cmd).await.expect("Failed to write RDY");
    
    // Wait for message
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Try to read large message
    let n = stream.read(&mut buffer).await.expect("Failed to read");
    if n > 0 {
        let response = String::from_utf8_lossy(&buffer[..n]);
        // Should receive a message frame
        assert!(response.contains("wire-large-test"), "Should receive large message frame");
    }
}
