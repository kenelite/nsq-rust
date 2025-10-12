//! Test utilities for integration tests

use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use reqwest::Client;
use serde_json::Value;

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub nsqd_tcp_port: u16,
    pub nsqd_http_port: u16,
    pub lookupd_tcp_port: u16,
    pub lookupd_http_port: u16,
    pub admin_http_port: u16,
    pub data_path: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            nsqd_tcp_port: 4150,
            nsqd_http_port: 4151,
            lookupd_tcp_port: 4160,
            lookupd_http_port: 4161,
            admin_http_port: 4171,
            data_path: "/tmp/nsq-test".to_string(),
        }
    }
}

/// Test environment manager
pub struct TestEnvironment {
    config: TestConfig,
    processes: Vec<Child>,
    client: Client,
}

impl TestEnvironment {
    pub fn new(config: TestConfig) -> Self {
        Self {
            config,
            processes: Vec::new(),
            client: Client::new(),
        }
    }

    /// Start all NSQ services
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Start NSQLookupd
        let lookupd = Command::new("cargo")
            .args(&[
                "run", "--bin", "nsqlookupd", "--",
                "--tcp-address", &format!("0.0.0.0:{}", self.config.lookupd_tcp_port),
                "--http-address", &format!("0.0.0.0:{}", self.config.lookupd_http_port),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.processes.push(lookupd);

        // Wait for lookupd to start
        sleep(Duration::from_secs(2)).await;

        // Start NSQd
        let nsqd = Command::new("cargo")
            .args(&[
                "run", "--bin", "nsqd", "--",
                "--tcp-address", &format!("0.0.0.0:{}", self.config.nsqd_tcp_port),
                "--http-address", &format!("0.0.0.0:{}", self.config.nsqd_http_port),
                "--lookupd-tcp-address", &format!("127.0.0.1:{}", self.config.lookupd_tcp_port),
                "--data-path", &self.config.data_path,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.processes.push(nsqd);

        // Wait for nsqd to start
        sleep(Duration::from_secs(2)).await;

        // Start NSQAdmin
        let admin = Command::new("cargo")
            .args(&[
                "run", "--bin", "nsqadmin", "--",
                "--http-address", &format!("0.0.0.0:{}", self.config.admin_http_port),
                "--nsqd-http-address", &format!("http://127.0.0.1:{}", self.config.nsqd_http_port),
                "--lookupd-http-address", &format!("http://127.0.0.1:{}", self.config.lookupd_http_port),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.processes.push(admin);

        // Wait for admin to start
        sleep(Duration::from_secs(2)).await;

        // Verify services are running
        self.wait_for_services().await?;

        Ok(())
    }

    /// Wait for all services to be ready
    async fn wait_for_services(&self) -> Result<(), Box<dyn std::error::Error>> {
        let timeout = Duration::from_secs(30);
        let start = Instant::now();

        while start.elapsed() < timeout {
            if self.check_service_health().await.is_ok() {
                return Ok(());
            }
            sleep(Duration::from_millis(500)).await;
        }

        Err("Services failed to start within timeout".into())
    }

    /// Check if all services are healthy
    async fn check_service_health(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Check NSQd
        let nsqd_response = self.client
            .get(&format!("http://127.0.0.1:{}/ping", self.config.nsqd_http_port))
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        if !nsqd_response.status().is_success() {
            return Err("NSQd not healthy".into());
        }

        // Check NSQLookupd
        let lookupd_response = self.client
            .get(&format!("http://127.0.0.1:{}/ping", self.config.lookupd_http_port))
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        if !lookupd_response.status().is_success() {
            return Err("NSQLookupd not healthy".into());
        }

        // Check NSQAdmin
        let admin_response = self.client
            .get(&format!("http://127.0.0.1:{}/api/ping", self.config.admin_http_port))
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        if !admin_response.status().is_success() {
            return Err("NSQAdmin not healthy".into());
        }

        Ok(())
    }

    /// Get NSQd HTTP client
    pub fn nsqd_client(&self) -> NSQdClient {
        NSQdClient::new(self.config.nsqd_http_port, self.client.clone())
    }

    /// Get NSQLookupd HTTP client
    pub fn lookupd_client(&self) -> NSQLookupdClient {
        NSQLookupdClient::new(self.config.lookupd_http_port, self.client.clone())
    }

    /// Get NSQAdmin HTTP client
    pub fn admin_client(&self) -> NSQAdminClient {
        NSQAdminClient::new(self.config.admin_http_port, self.client.clone())
    }

    /// Stop all services
    pub fn stop(&mut self) {
        for mut process in self.processes.drain(..) {
            let _ = process.kill();
            let _ = process.wait();
        }
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        self.stop();
    }
}

/// NSQd HTTP client
pub struct NSQdClient {
    port: u16,
    client: Client,
}

impl NSQdClient {
    pub fn new(port: u16, client: Client) -> Self {
        Self { port, client }
    }

    pub async fn ping(&self) -> Result<String, reqwest::Error> {
        let response = self.client
            .get(&format!("http://127.0.0.1:{}/ping", self.port))
            .send()
            .await?;
        response.text().await
    }

    pub async fn get_stats(&self) -> Result<Value, reqwest::Error> {
        let response = self.client
            .get(&format!("http://127.0.0.1:{}/stats", self.port))
            .send()
            .await?;
        response.json().await
    }

    pub async fn publish(&self, topic: &str, message: &str) -> Result<String, reqwest::Error> {
        let response = self.client
            .post(&format!("http://127.0.0.1:{}/pub?topic={}", self.port, topic))
            .body(message.to_string())
            .send()
            .await?;
        response.text().await
    }

    pub async fn create_topic(&self, topic: &str) -> Result<String, reqwest::Error> {
        let response = self.client
            .post(&format!("http://127.0.0.1:{}/topic/create?topic={}", self.port, topic))
            .send()
            .await?;
        response.text().await
    }

    pub async fn delete_topic(&self, topic: &str) -> Result<String, reqwest::Error> {
        let response = self.client
            .post(&format!("http://127.0.0.1:{}/topic/delete?topic={}", self.port, topic))
            .send()
            .await?;
        response.text().await
    }

    pub async fn pause_topic(&self, topic: &str) -> Result<String, reqwest::Error> {
        let response = self.client
            .post(&format!("http://127.0.0.1:{}/topic/pause?topic={}", self.port, topic))
            .send()
            .await?;
        response.text().await
    }

    pub async fn unpause_topic(&self, topic: &str) -> Result<String, reqwest::Error> {
        let response = self.client
            .post(&format!("http://127.0.0.1:{}/topic/unpause?topic={}", self.port, topic))
            .send()
            .await?;
        response.text().await
    }

    pub async fn pause_channel(&self, topic: &str, channel: &str) -> Result<String, reqwest::Error> {
        let response = self.client
            .post(&format!("http://127.0.0.1:{}/channel/pause?topic={}&channel={}", self.port, topic, channel))
            .send()
            .await?;
        response.text().await
    }

    pub async fn unpause_channel(&self, topic: &str, channel: &str) -> Result<String, reqwest::Error> {
        let response = self.client
            .post(&format!("http://127.0.0.1:{}/channel/unpause?topic={}&channel={}", self.port, topic, channel))
            .send()
            .await?;
        response.text().await
    }

    pub async fn delete_channel(&self, topic: &str, channel: &str) -> Result<String, reqwest::Error> {
        let response = self.client
            .post(&format!("http://127.0.0.1:{}/channel/delete?topic={}&channel={}", self.port, topic, channel))
            .send()
            .await?;
        response.text().await
    }
}

/// NSQLookupd HTTP client
pub struct NSQLookupdClient {
    port: u16,
    client: Client,
}

impl NSQLookupdClient {
    pub fn new(port: u16, client: Client) -> Self {
        Self { port, client }
    }

    pub async fn ping(&self) -> Result<String, reqwest::Error> {
        let response = self.client
            .get(&format!("http://127.0.0.1:{}/ping", self.port))
            .send()
            .await?;
        response.text().await
    }

    pub async fn get_stats(&self) -> Result<Value, reqwest::Error> {
        let response = self.client
            .get(&format!("http://127.0.0.1:{}/stats", self.port))
            .send()
            .await?;
        response.json().await
    }

    pub async fn get_topics(&self) -> Result<Value, reqwest::Error> {
        let response = self.client
            .get(&format!("http://127.0.0.1:{}/topics", self.port))
            .send()
            .await?;
        response.json().await
    }

    pub async fn lookup_topic(&self, topic: &str) -> Result<Value, reqwest::Error> {
        let response = self.client
            .get(&format!("http://127.0.0.1:{}/lookup?topic={}", self.port, topic))
            .send()
            .await?;
        response.json().await
    }

    pub async fn create_topic(&self, topic: &str) -> Result<String, reqwest::Error> {
        let response = self.client
            .post(&format!("http://127.0.0.1:{}/topic/create?topic={}", self.port, topic))
            .send()
            .await?;
        response.text().await
    }

    pub async fn delete_topic(&self, topic: &str) -> Result<String, reqwest::Error> {
        let response = self.client
            .post(&format!("http://127.0.0.1:{}/topic/delete?topic={}", self.port, topic))
            .send()
            .await?;
        response.text().await
    }
}

/// NSQAdmin HTTP client
pub struct NSQAdminClient {
    port: u16,
    client: Client,
}

impl NSQAdminClient {
    pub fn new(port: u16, client: Client) -> Self {
        Self { port, client }
    }

    pub async fn ping(&self) -> Result<String, reqwest::Error> {
        let response = self.client
            .get(&format!("http://127.0.0.1:{}/api/ping", self.port))
            .send()
            .await?;
        response.text().await
    }

    pub async fn get_stats(&self) -> Result<Value, reqwest::Error> {
        let response = self.client
            .get(&format!("http://127.0.0.1:{}/api/stats", self.port))
            .send()
            .await?;
        response.json().await
    }

    pub async fn get_topics(&self) -> Result<Value, reqwest::Error> {
        let response = self.client
            .get(&format!("http://127.0.0.1:{}/api/topics", self.port))
            .send()
            .await?;
        response.json().await
    }

    pub async fn get_nodes(&self) -> Result<Value, reqwest::Error> {
        let response = self.client
            .get(&format!("http://127.0.0.1:{}/api/nodes", self.port))
            .send()
            .await?;
        response.json().await
    }
}

/// Test assertions
pub mod assertions {
    use serde_json::Value;

    pub fn assert_topic_exists(stats: &Value, topic_name: &str) {
        let topics = stats["topics"].as_array().expect("topics should be an array");
        let topic_exists = topics.iter().any(|topic| {
            topic["topic_name"].as_str() == Some(topic_name)
        });
        assert!(topic_exists, "Topic '{}' should exist", topic_name);
    }

    pub fn assert_channel_exists(stats: &Value, topic_name: &str, channel_name: &str) {
        let topics = stats["topics"].as_array().expect("topics should be an array");
        let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some(topic_name))
            .expect(&format!("Topic '{}' should exist", topic_name));
        
        let channels = topic["channels"].as_array().expect("channels should be an array");
        let channel_exists = channels.iter().any(|channel| {
            channel["channel_name"].as_str() == Some(channel_name)
        });
        assert!(channel_exists, "Channel '{}' should exist in topic '{}'", channel_name, topic_name);
    }

    pub fn assert_message_count(stats: &Value, topic_name: &str, expected_count: u64) {
        let topics = stats["topics"].as_array().expect("topics should be an array");
        let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some(topic_name))
            .expect(&format!("Topic '{}' should exist", topic_name));
        
        let message_count = topic["message_count"].as_u64().expect("message_count should be a number");
        assert_eq!(message_count, expected_count, "Topic '{}' should have {} messages", topic_name, expected_count);
    }

    pub fn assert_topic_paused(stats: &Value, topic_name: &str, paused: bool) {
        let topics = stats["topics"].as_array().expect("topics should be an array");
        let topic = topics.iter().find(|t| t["topic_name"].as_str() == Some(topic_name))
            .expect(&format!("Topic '{}' should exist", topic_name));
        
        let is_paused = topic["paused"].as_bool().expect("paused should be a boolean");
        assert_eq!(is_paused, paused, "Topic '{}' paused state should be {}", topic_name, paused);
    }
}
