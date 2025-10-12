# Development Guide

This guide covers development practices, testing, and contribution guidelines for NSQ Rust.

## Table of Contents

- [Development Environment](#development-environment)
- [Project Structure](#project-structure)
- [Building](#building)
- [Testing](#testing)
- [Code Style](#code-style)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [Release Process](#release-process)
- [Debugging](#debugging)
- [Performance Profiling](#performance-profiling)
- [Benchmarking](#benchmarking)

## Development Environment

### Prerequisites

- **Rust**: 1.75+ (stable)
- **Cargo**: Latest version
- **Git**: Latest version
- **Docker**: Latest version (optional)
- **Node.js**: 18+ (for UI development)

### Setup

#### Clone Repository

```bash
git clone https://github.com/kenelite/nsq-rust.git
cd nsq-rust
```

#### Install Rust

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install required components
rustup component add rustfmt clippy
```

#### Install Development Tools

```bash
# Install cargo tools
cargo install cargo-watch
cargo install cargo-expand
cargo install cargo-audit
cargo install cargo-outdated
cargo install cargo-tree
cargo install cargo-deny

# Install testing tools
cargo install cargo-nextest
cargo install cargo-tarpaulin
cargo install cargo-criterion
```

#### Install UI Dependencies

```bash
# Install Node.js dependencies
cd nsqadmin-ui
npm install
cd ..
```

### IDE Setup

#### VS Code

Install recommended extensions:

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "vadimcn.vscode-lldb",
    "ms-vscode.vscode-json",
    "bradlc.vscode-tailwindcss",
    "esbenp.prettier-vscode"
  ]
}
```

#### IntelliJ IDEA

Install Rust plugin and configure:

1. Install Rust plugin
2. Configure Rust toolchain
3. Enable Rust analyzer
4. Configure code formatting

## Project Structure

```
nsq-rust/
├── Cargo.toml                 # Workspace configuration
├── README.md                  # Project documentation
├── LICENSE                     # License file
├── .gitignore                 # Git ignore rules
├── .github/                    # GitHub workflows
│   ├── ci.yml                 # Continuous integration
│   ├── release.yml            # Release workflow
│   └── dependabot.yml         # Dependency updates
├── docs/                       # Documentation
│   ├── installation.md        # Installation guide
│   ├── configuration.md       # Configuration reference
│   ├── api-reference.md       # API documentation
│   ├── deployment.md          # Deployment guide
│   ├── architecture.md        # Architecture guide
│   └── development.md         # Development guide
├── nsq-protocol/              # Protocol library
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── wire.rs            # Wire protocol
│   │   ├── commands.rs        # Command definitions
│   │   ├── messages.rs        # Message format
│   │   ├── compression.rs     # Compression support
│   │   └── error.rs           # Protocol errors
│   └── tests/
├── nsq-common/                # Common utilities
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── config.rs          # Configuration
│   │   ├── logging.rs         # Logging utilities
│   │   ├── metrics.rs         # Metrics collection
│   │   ├── disk_queue.rs      # Disk queue implementation
│   │   └── error.rs           # Common errors
│   └── tests/
├── nsqd/                      # NSQD daemon
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs            # Entry point
│   │   ├── lib.rs
│   │   ├── server.rs          # Server implementation
│   │   ├── topic.rs           # Topic management
│   │   ├── channel.rs         # Channel management
│   │   ├── client.rs          # Client handling
│   │   ├── message.rs         # Message handling
│   │   ├── stats.rs           # Statistics
│   │   └── error.rs           # NSQD errors
│   └── tests/
├── nsqlookupd/                # NSQLookupd daemon
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs            # Entry point
│   │   ├── lib.rs
│   │   ├── server.rs          # Server implementation
│   │   ├── registry.rs        # Service registry
│   │   ├── discovery.rs       # Service discovery
│   │   └── error.rs           # NSQLookupd errors
│   └── tests/
├── nsqadmin/                  # NSQAdmin daemon
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs            # Entry point
│   │   ├── lib.rs
│   │   ├── server.rs          # Server implementation
│   │   ├── api.rs             # API endpoints
│   │   ├── ui.rs              # UI serving
│   │   └── error.rs           # NSQAdmin errors
│   └── tests/
├── nsqadmin-ui/               # Modern web UI
│   ├── package.json
│   ├── vite.config.ts
│   ├── tailwind.config.js
│   ├── tsconfig.json
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── components/
│   │   ├── stores/
│   │   ├── utils/
│   │   └── types/
│   └── dist/                  # Built UI files
├── tools/                     # CLI utilities
│   ├── nsq_to_file/
│   ├── to_nsq/
│   ├── nsq_tail/
│   ├── nsq_stat/
│   ├── nsq_to_http/
│   └── nsq_to_nsq/
├── tests/                     # Integration tests
│   ├── Cargo.toml
│   ├── integration/
│   │   ├── mod.rs
│   │   ├── test_utils.rs
│   │   ├── basic_functionality.rs
│   │   ├── message_flow.rs
│   │   ├── topic_channel_management.rs
│   │   ├── node_discovery.rs
│   │   ├── admin_interface.rs
│   │   ├── performance.rs
│   │   └── error_handling.rs
│   └── compatibility/
│       ├── mod.rs
│       ├── protocol_compatibility.rs
│       ├── api_compatibility.rs
│       ├── wire_protocol.rs
│       └── message_format.rs
└── examples/                   # Example code
    ├── publisher.rs
    ├── consumer.rs
    └── admin.rs
```

## Building

### Basic Build

```bash
# Build all components
cargo build

# Build release version
cargo build --release

# Build specific component
cargo build -p nsqd
cargo build -p nsqlookupd
cargo build -p nsqadmin
```

### Build Options

```bash
# Build with features
cargo build --features tls
cargo build --features metrics
cargo build --features all

# Build for specific target
cargo build --target x86_64-unknown-linux-gnu
cargo build --target aarch64-unknown-linux-gnu

# Build with optimizations
cargo build --release --target x86_64-unknown-linux-gnu
```

### Cross-Compilation

```bash
# Install cross-compilation targets
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu

# Build for Linux
cargo build --release --target x86_64-unknown-linux-gnu

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu

# Build for macOS
cargo build --release --target x86_64-apple-darwin
```

### Docker Build

```bash
# Build Docker image
docker build -t nsq-rust:latest .

# Build multi-arch image
docker buildx build --platform linux/amd64,linux/arm64 -t nsq-rust:latest .
```

## Testing

### Unit Tests

```bash
# Run all unit tests
cargo test

# Run tests for specific component
cargo test -p nsqd
cargo test -p nsqlookupd
cargo test -p nsqadmin

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel
cargo test --jobs 4
```

### Integration Tests

```bash
# Run integration tests
cargo test --test integration

# Run compatibility tests
cargo test --test compatibility

# Run specific integration test
cargo test --test integration basic_functionality

# Run tests with coverage
cargo tarpaulin --out html
```

### Test Configuration

Create `tests/test_config.toml`:

```toml
[test]
timeout = "30s"
retries = 3
parallel = true

[nsqd]
tcp_port = 4150
http_port = 4151
data_path = "/tmp/nsq-test"

[nsqlookupd]
tcp_port = 4160
http_port = 4161

[nsqadmin]
http_port = 4171
```

### Test Utilities

```rust
// tests/test_utils.rs
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub struct TestEnvironment {
    pub nsqd: Option<Child>,
    pub nsqlookupd: Option<Child>,
    pub config: TestConfig,
}

impl TestEnvironment {
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Start NSQLookupd
        let nsqlookupd = Command::new("nsqlookupd")
            .arg("--tcp-address=127.0.0.1:4160")
            .arg("--http-address=127.0.0.1:4161")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        
        self.nsqlookupd = Some(nsqlookupd);
        
        // Wait for NSQLookupd to start
        sleep(Duration::from_secs(2)).await;
        
        // Start NSQD
        let nsqd = Command::new("nsqd")
            .arg("--tcp-address=127.0.0.1:4150")
            .arg("--http-address=127.0.0.1:4151")
            .arg("if lookupd-tcp-address=127.0.0.1:4160")
            .arg("--lookupd-http-address=127.0.0.1:4161")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        
        self.nsqd = Some(nsqd);
        
        // Wait for NSQD to start
        sleep(Duration::from_secs(2)).await;
        
        Ok(())
    }
    
    pub fn cleanup(&mut self) {
        if let Some(mut nsqd) = self.nsqd.take() {
            let _ = nsqd.kill();
        }
        if let Some(mut nsqlookupd) = self.nsqlookupd.take() {
            let _ = nsqlookupd.kill();
        }
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        self.cleanup();
    }
}
```

### Mock Testing

```rust
// tests/mocks.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MockNsqdServer {
    topics: Arc<Mutex<HashMap<String, MockTopic>>>,
    messages: Arc<Mutex<Vec<MockMessage>>>,
}

pub struct MockTopic {
    name: String,
    channels: HashMap<String, MockChannel>,
}

pub struct MockChannel {
    name: String,
    messages: Vec<MockMessage>,
}

pub struct MockMessage {
    id: String,
    body: Vec<u8>,
    timestamp: u64,
}

impl MockNsqdServer {
    pub fn new() -> Self {
        Self {
            topics: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn publish(&self, topic: &str, message: &[u8]) -> Result<(), String> {
        let mut topics = self.topics.lock().unwrap();
        let mut messages = self.messages.lock().unwrap();
        
        let mock_message = MockMessage {
            id: uuid::Uuid::new_v4().to_string(),
            body: message.to_vec(),
            timestamp: current_timestamp(),
        };
        
        messages.push(mock_message);
        
        if let Some(topic) = topics.get_mut第一段话：t) {
            // Add to topic
        } else {
            // Create new topic
            let new_topic = MockTopic {
                name: topic.to_string(),
                channels: HashMap::new(),
            };
            topics.insert(topic.to_string(), new_topic);
        }
        
        Ok(())
    }
}
```

## Code Style

### Rustfmt Configuration

Create `rustfmt.toml`:

```toml
# Rustfmt configuration
edition = "2021"
max_width = 100
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
format_code_in_doc_comments = true
wrap_comments = true
comment_width = 80
```

### Clippy Configuration

Create `clippy.toml`:

```toml
# Clippy configuration
avoid-breaking-exported-api = false
msrv = "1.75"
```

### Code Style Guidelines

#### Naming Conventions

```rust
// Use snake_case for variables and functions
let message_count = 100;
fn process_message() {}

// Use PascalCase for types and traits
struct MessageHandler;
trait MessageProcessor;

// Use SCREAMING_SNAKE_CASE for constants
const MAX_MESSAGE_SIZE: usize = 1024;

// Use descriptive names
let client_connection_timeout = Duration::from_secs(30);
let max_retry_attempts = 3;
```

#### Error Handling

```rust
// Use Result for fallible operations
fn parse_message(data: &[u8]) -> Result<Message, ParseError> {
    // Implementation
}

// Use ? operator for error propagation
fn process_data(data: &[u8]) -> Result<(), ProcessingError> {
    let message = parse_message(data)?;
    let result = validate_message(&message)?;
    Ok(())
}

// Use custom error types
#[derive(Debug, thiserror::Error)]
pub enum ProcessingError {
    #[error("Invalid message format")]
    InvalidFormat,
    #[error("Message too large: {size} bytes")]
    TooLarge { size: usize },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

#### Documentation

```rust
/// Processes a message and returns the result.
///
/// # Arguments
///
/// * `message` - The message to process
/// * `options` - Processing options
///
/// # Returns
///
/// Returns `Ok(ProcessedMessage)` if successful, or `Err(ProcessingError)` if failed.
///
/// # Examples
///
/// ```rust
/// let message = Message::new(b"Hello, World!");
/// let options = ProcessingOptions::default();
/// let result = process_message(message, options)?;
/// ```
pub fn process_message(
    message: Message,
    options: ProcessingOptions,
) -> Result<ProcessedMessage, ProcessingError> {
    // Implementation
}
```

#### Async/Await

```rust
// Use async/await for asynchronous operations
pub async fn handle_client(mut client: Client) -> Result<(), ClientError> {
    while let Some(command) = client.recv_command().await {
        match command {
            Command::Publish(req) => {
                client.publish(req).await?;
            }
            Command::Subscribe(req) => {
                client.subscribe(req).await?;
            }
            _ => {}
        }
    }
    Ok(())
}

// Use tokio::spawn for concurrent tasks
pub async fn start_server() -> Result<(), ServerError> {
    let listener = TcpListener::bind("127.0.0.1:4150").await?;
    
    loop {
        let (stream, addr) = listener.accept().await?;
        let client = Client::new(stream, addr);
        
        tokio::spawn(async move {
            if let Err(e) = handle_client(client).await {
                eprintln!("Client error: {}", e);
            }
        });
    }
}
```

## Documentation

### Code Documentation

#### Module Documentation

```rust
//! NSQ Protocol implementation
//!
//! This module provides the core protocol implementation for NSQ, including
//! wire protocol, message format, and command handling.
//!
//! # Examples
//!
//! ```rust
//! use nsq_protocol::{Client, Message};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = Client::connect("127.0.0.1:4150").await?;
//!     client.publish("test_topic", b"Hello, World!").await?;
//!     Ok(())
//! }
//! ```

pub mod wire;
pub mod commands;
pub mod messages;
pub mod compression;
pub mod error;
```

#### Function Documentation

```rust
/// Publishes a message to the specified topic.
///
/// This function sends a message to the NSQD server for delivery to subscribers.
/// The message will be queued and delivered to all channels subscribed to the topic.
///
/// # Arguments
///
/// * `topic` - The topic name to publish to
/// * `message` - The message content to publish
///
/// # Returns
///
/// Returns `Ok(())` if the message was successfully published, or `Err(PublishError)` if failed.
///
/// # Errors
///
/// This function will return an error if:
/// - The topic name is invalid
/// - The message is too large
/// - The connection to NSQD is lost
/// - The server returns an error
///
/// # Examples
///
/// ```rust
/// use nsq_protocol::Client;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut client = Client::connect("127.0.0.1:4150").await?;
///     
///     // Publish a simple message
///     client.publish("my_topic", b"Hello, World!").await?;
///     
///     // Publish a JSON message
///     let json = serde_json::json!({"message": "Hello, World!"});
///     client.publish("my_topic", json.to_string().as_bytes()).await?;
///     
///     Ok(())
/// }
/// ```
pub async fn publish(&mut self, topic: &str, message: &[u8]) -> Result<(), PublishError> {
    // Implementation
}
```

### API Documentation

#### Generate Documentation

```bash
# Generate documentation
cargo doc --no-deps

# Generate documentation with private items
cargo doc --no-deps --document-private-items

# Generate documentation for specific component
cargo doc -p nsqd --no-deps

# Open documentation in browser
cargo doc --open
```

#### Documentation Comments

```rust
/// Configuration for NSQD server.
///
/// This struct contains all configuration options for the NSQD server,
/// including network settings, storage options, and performance tuning.
///
/// # Examples
///
/// ```rust
/// use nsqd::config::NsqdConfig;
///
/// let config = NsqdConfig::builder()
///     .tcp_address("127.0.0.1:4150")
///     .http_address("127.0.0.1:4151")
///     .data_path("/var/lib/nsqd")
///     .build()?;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NsqdConfig {
    /// TCP address to listen on
    pub tcp_address: String,
    
    /// HTTP address to listen on
    pub http_address: String,
    
    /// Data directory path
    pub data_path: PathBuf,
    
    /// Maximum memory size for messages
    pub max_memory_size: usize,
    
    /// Maximum message body size
    pub max_body_size: usize,
}
```

### README Documentation

#### Component README

Create `nsqd/README.md`:

```markdown
# NSQD

NSQD is the daemon that receives, queues, and delivers messages to clients.

## Features

- High-performance message queuing
- Topic and channel management
- Client connection handling
- HTTP and TCP APIs
- TLS support
- Metrics and monitoring

## Usage

### Basic Usage

```bash
# Start NSQD
nsqd --tcp-address=127.0.0.1:4150 --http-address=127.0.0.1:4151

# Publish a message
curl -X POST "http://127.0.0.1:4151/pub?topic=test_topic" -d "Hello, World!"

# Get statistics
curl "http://127.0.0.1:4151/stats"
```

### Configuration

See [Configuration Guide](../docs/configuration.md) for detailed configuration options.

### API Reference

See [API Reference](../docs/api-reference.md) for complete API documentation.

## Development

### Building

```bash
cargo build -p nsqd
```

### Testing

```bash
cargo test -p nsqd
```

### Running

```bash
cargo run -p nsqd -- --tcp-address=127.0.0.1:4150 --http-address=127.0.0.1:4151
```
```

## Contributing

### Contribution Guidelines

1. **Fork the repository**
2. **Create a feature branch**
3. **Make your changes**
4. **Add tests**
5. **Update documentation**
6. **Submit a pull request**

### Pull Request Process

#### Before Submitting

```bash
# Run tests
cargo test

# Run clippy
cargo clippy -- -D warnings

# Run rustfmt
cargo fmt

# Run security audit
cargo audit

# Check for outdated dependencies
cargo outdated
```

#### Pull Request Template

```markdown
## Description

Brief description of changes.

## Type of Change

- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing

- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist

- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No breaking changes (or documented)
```

### Code Review Process

1. **Automated checks** (CI/CD)
2. **Code review** by maintainers
3. **Testing** in staging environment
4. **Approval** and merge

### Issue Guidelines

#### Bug Reports

```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. See error

**Expected behavior**
What you expected to happen.

**Environment:**
- OS: [e.g. Ubuntu 20.04]
- Rust version: [e.g. 1.75.0]
- NSQ Rust version: [e.g. 1.3.0]

**Additional context**
Add any other context about the problem here.
```

#### Feature Requests

```markdown
**Is your feature request related to a problem?**
A clear description of what the problem is.

**Describe the solution you'd like**
A clear description of what you want to happen.

**Describe alternatives you've considered**
A clear description of any alternative solutions.

**Additional context**
Add any other context or screenshots about the feature request.
```

## Release Process

### Versioning

NSQ Rust follows [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Steps

#### 1. Prepare Release

```bash
# Update version in Cargo.toml
cargo set-version 1.3.1

# Update CHANGELOG.md
# Update documentation
# Run tests
cargo test
```

#### 2. Create Release

```bash
# Create git tag
git tag -a v1.3.1 -m "Release v1.3.1"

# Push tag
git push origin v1.3.1

# Create GitHub release
gh release create v1.3.1 --title "Release v1.3.1" --notes "Release notes"
```

#### 3. Build Artifacts

```bash
# Build for multiple platforms
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-apple-darwin

# Create release archives
tar -czf nsq-rust-linux-amd64.tar.gz target/x86_64-unknown-linux-gnu/release/nsqd target/x86_64-unknown-linux-gnu/release/nsqlookupd target/x86_64-unknown-linux-gnu/release/nsqadmin
```

### Automated Releases

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Build
        run: cargo build --release
        
      - name: Test
        run: cargo test
        
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            target/release/nsqd
            target/release/nsqlookupd
            target/release/nsqadmin
```

## Debugging

### Debug Build

```bash
# Build with debug symbols
cargo build

# Run with debug logging
RUST_LOG=debug cargo run -p nsqd

# Run with specific module logging
RUST_LOG=nsqd::topic=debug cargo run -p nsqd
```

### Debugging Tools

#### LLDB

```bash
# Install LLDB
# macOS: xcode-select --install
# Ubuntu: sudo apt-get install lldb

# Debug with LLDB
lldb target/debug/nsqd
(lldb) run --tcp-address=127.0.0.1:4150
```

#### GDB

```bash
# Install GDB
# Ubuntu: sudo apt-get install gdb

# Debug with GDB
gdb target/debug/nsqd
(gdb) run --tcp-address=127.0.0.1:4150
```

#### VS Code Debugging

Create `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug NSQD",
      "cargo": {
        "args": ["build", "-p", "nsqd"],
        "filter": {
          "name": "nsqd"
        }
      },
      "args": ["--tcp-address=127.0.0.1:4150"],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

### Logging

#### Structured Logging

```rust
use tracing::{info, warn, error, debug, span, Level};

// Create spans for context
let span = span!(Level::INFO, "process_message", message_id = %message.id);
let _enter = span.enter();

info!(message_size = message.body.len(), "Processing message");
debug!("Message content: {}", String::from_utf8_lossy(&message.body));

if message.body.len() > MAX_SIZE {
    warn!(max_size = MAX_SIZE, "Message exceeds maximum size");
}
```

#### Log Analysis

```bash
# Filter logs by level
journalctl -u nsqd --since "1 hour ago" | grep ERROR

# Filter logs by component
journalctl -u nsqd --since "1 hour ago" | grep "topic"

# Follow logs in real-time
journalctl -u nsqd -f
```

## Performance Profiling

### CPU Profiling

#### Perf

```bash
# Install perf
# Ubuntu: sudo apt-get install linux-tools-common

# Profile CPU usage
perf record -p $(pgrep nsqd)
perf report

# Profile with call graphs
perf record -g -p $(pgrep nsqd)
perf report -g
```

#### Flamegraph

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
sudo flamegraph -p $(pgrep nsqd)
```

### Memory Profiling

#### Valgrind

```bash
# Install valgrind
# Ubuntu: sudo apt-get install valgrind

# Profile memory usage
valgrind --tool=massif target/debug/nsqd

# Profile memory leaks
valgrind --tool=memcheck target/debug/nsqd
```

#### Heaptrack

```bash
# Install heaptrack
# Ubuntu: sudo apt-get install heaptrack

# Profile memory usage
heaptrack target/debug/nsqd
```

### Network Profiling

#### Netstat

```bash
# Monitor network connections
netstat -tulpn | grep nsqd

# Monitor network statistics
netstat -i
```

#### Tcpdump

```bash
# Capture network traffic
sudo tcpdump -i lo -w nsqd.pcap port 4150

# Analyze captured traffic
tcpdump -r nsqd.pcap
```

## Benchmarking

### Criterion Benchmarks

#### Setup

```rust
// benches/message_processing.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nsq_protocol::{Message, Topic};

fn benchmark_message_processing(c: &mut Criterion) {
    let mut topic = Topic::new("test_topic");
    
    c.bench_function("process_message", |b| {
        b.iter(|| {
            let message = Message::new(black_box(b"Hello, World!"));
            topic.process_message(black_box(message))
        })
    });
}

criterion_group!(benches, benchmark_message_processing);
criterion_main!(benches);
```

#### Running Benchmarks

```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench message_processing

# Run benchmarks with output
cargo bench -- --nocapture
```

### Load Testing

#### Custom Load Test

```rust
// tests/load_test.rs
use tokio::time::{Duration, Instant};
use nsq_protocol::Client;

#[tokio::test]
async fn load_test_publish() {
    let mut client = Client::connect("127.0.0.1:4150").await.unwrap();
    
    let start_time = Instant::now();
    let message_count = 10000;
    
    for i in 0..message_count {
        let message = format!("Message {}", i);
        client.publish("load_test", message.as_bytes()).await.unwrap();
    }
    
    let duration = start_time.elapsed();
    let throughput = message_count as f64 / duration.as_secs_f64();
    
    println!("Published {} messages in {:?}", message_count, duration);
    println!("Throughput: {:.2} messages/second", throughput);
    
    assert!(throughput > 1000.0, "Throughput too low: {}", throughput);
}
```

#### External Load Testing

```bash
# Install nsq tools
go install github.com/nsqio/go-nsq@latest

# Publish messages
nsq_pub --topic=load_test --rate=1000 --count=10000

# Consume messages
nsq_sub --topic=load_test --channel=load_test
```

### Performance Monitoring

#### Metrics Collection

```rust
// src/metrics.rs
use std::time::Instant;

pub struct PerformanceMetrics {
    start_time: Instant,
    message_count: u64,
    total_bytes: u64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            message_count: 0,
            total_bytes: 0,
        }
    }
    
    pub fn record_message(&mut self, size: usize) {
        self.message_count += 1;
        self.total_bytes += size as u64;
    }
    
    pub fn throughput(&self) -> f64 {
        let duration = self.start_time.elapsed().as_secs_f64();
        self.message_count as f64 / duration
    }
    
    pub fn bandwidth(&self) -> f64 {
        let duration = self.start_time.elapsed().as_secs_f64();
        self.total_bytes as f64 / duration
    }
}
```

#### Continuous Benchmarking

Create `.github/workflows/benchmark.yml`:

```yaml
name: Benchmark

on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM
  push:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run benchmarks
        run: cargo bench -- --save-baseline main
        
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: target/criterion/
```

## Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Axum Guide](https://docs.rs/axum/latest/axum/)
- [Tracing Documentation](https://tracing.rs/)
- [Criterion Documentation](https://docs.rs/criterion/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
