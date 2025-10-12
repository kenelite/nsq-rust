# NSQ Rust Integration Tests

This directory contains comprehensive integration and compatibility tests for the NSQ Rust implementation.

## Test Structure

### Integration Tests (`tests/integration/`)

Integration tests verify the complete NSQ system functionality:

- **`basic_functionality.rs`**: Core NSQ operations (ping, stats, topic creation, message publishing)
- **`message_flow.rs`**: Message publishing, consumption, and TCP protocol handling
- **`topic_channel_management.rs`**: Topic and channel lifecycle management
- **`node_discovery.rs`**: Service discovery and registration
- **`admin_interface.rs`**: NSQAdmin web interface functionality
- **`performance.rs`**: Performance and throughput testing
- **`error_handling.rs`**: Error conditions and edge cases

### Compatibility Tests (`tests/compatibility/`)

Compatibility tests ensure the Rust implementation is compatible with the original NSQ:

- **`protocol_compatibility.rs`**: Wire protocol compatibility
- **`api_compatibility.rs`**: HTTP API compatibility
- **`wire_protocol.rs`**: TCP wire protocol testing
- **`message_format.rs`**: Message format and encoding compatibility

## Test Utilities

### `test_utils.rs`

Provides test infrastructure:

- **`TestEnvironment`**: Manages NSQ service lifecycle
- **`TestConfig`**: Test configuration
- **`NSQdClient`**: HTTP client for NSQd operations
- **`NSQLookupdClient`**: HTTP client for NSQLookupd operations
- **`NSQAdminClient`**: HTTP client for NSQAdmin operations
- **Assertions**: Helper functions for test assertions

## Running Tests

### Prerequisites

- Rust 1.70+ with Cargo
- All NSQ services must be buildable
- Test dependencies installed

### Basic Test Execution

```bash
# Run all integration tests
cargo test --test integration

# Run all compatibility tests
cargo test --test compatibility

# Run specific test file
cargo test --test integration basic_functionality

# Run with output
cargo test --test integration -- --nocapture
```

### Test Categories

#### Basic Functionality Tests
```bash
cargo test --test integration test_service_startup
cargo test --test integration test_stats_endpoints
cargo test --test integration test_topic_creation
cargo test --test integration test_message_publishing
```

#### Message Flow Tests
```bash
cargo test --test integration test_tcp_protocol_basic
cargo test --test integration test_message_ordering
cargo test --test integration test_concurrent_publishing
```

#### Performance Tests
```bash
cargo test --test integration test_message_throughput
cargo test --test integration test_concurrent_publishing
cargo test --test integration test_api_response_times
```

#### Compatibility Tests
```bash
cargo test --test compatibility test_tcp_protocol_compatibility
cargo test --test compatibility test_api_compatibility
cargo test --test compatibility test_wire_protocol_commands
```

## Test Configuration

### Environment Variables

- `NSQ_TEST_TCP_PORT`: NSQd TCP port (default: 4150)
- `NSQ_TEST_HTTP_PORT`: NSQd HTTP port (default: 4151)
- `NSQ_TEST_LOOKUPD_TCP_PORT`: NSQLookupd TCP port (default: 4160)
- `NSQ_TEST_LOOKUPD_HTTP_PORT`: NSQLookupd HTTP port (default: 4161)
- `NSQ_TEST_ADMIN_HTTP_PORT`: NSQAdmin HTTP port (default: 4171)
- `NSQ_TEST_DATA_PATH`: Data directory path (default: /tmp/nsq-test)

### Test Isolation

Each test runs in isolation:
- Fresh service instances
- Clean data directories
- Unique port assignments
- Automatic cleanup

## Test Coverage

### Core Functionality
- ✅ Service startup and health checks
- ✅ HTTP API endpoints
- ✅ TCP protocol handling
- ✅ Topic and channel management
- ✅ Message publishing and consumption
- ✅ Service discovery and registration
- ✅ Admin interface functionality

### Performance Testing
- ✅ Message throughput measurement
- ✅ Concurrent operation testing
- ✅ API response time validation
- ✅ Memory usage monitoring
- ✅ Large message handling

### Error Handling
- ✅ Invalid input validation
- ✅ Service unavailability handling
- ✅ Resource exhaustion scenarios
- ✅ Malformed request handling
- ✅ Concurrent error conditions

### Compatibility
- ✅ Wire protocol compatibility
- ✅ HTTP API compatibility
- ✅ Message format compatibility
- ✅ Response format validation
- ✅ Error response compatibility

## Test Results

### Expected Outcomes

All tests should pass when the NSQ Rust implementation is complete:

- **Integration Tests**: Verify complete system functionality
- **Compatibility Tests**: Ensure compatibility with original NSQ
- **Performance Tests**: Validate performance characteristics
- **Error Tests**: Confirm proper error handling

### Test Failures

Common test failure scenarios:

1. **Service Startup Failures**: Missing dependencies or port conflicts
2. **API Compatibility Issues**: Response format mismatches
3. **Protocol Errors**: Wire protocol incompatibilities
4. **Performance Issues**: Throughput or latency problems
5. **Error Handling**: Incorrect error responses

## Debugging Tests

### Verbose Output
```bash
cargo test --test integration -- --nocapture --test-threads=1
```

### Single Test Execution
```bash
cargo test --test integration test_service_startup -- --exact
```

### Test Logging
```bash
RUST_LOG=debug cargo test --test integration
```

## Continuous Integration

### GitHub Actions
```yaml
name: Integration Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --test integration
      - run: cargo test --test compatibility
```

### Local Development
```bash
# Run tests before committing
cargo test --test integration
cargo test --test compatibility

# Run specific test categories
cargo test --test integration performance
cargo test --test compatibility protocol
```

## Test Maintenance

### Adding New Tests

1. Create test functions in appropriate files
2. Use existing test utilities and assertions
3. Follow naming convention: `test_<functionality>`
4. Add documentation for complex tests

### Updating Tests

1. Update test expectations when APIs change
2. Maintain backward compatibility where possible
3. Update documentation for test changes
4. Verify tests still pass after updates

### Test Dependencies

- `tokio`: Async runtime for tests
- `reqwest`: HTTP client for API testing
- `serde_json`: JSON serialization/deserialization
- `tokio-test`: Testing utilities

## Performance Benchmarks

### Expected Performance

- **Message Throughput**: >1000 msg/sec (single topic)
- **API Response Time**: <100ms (95th percentile)
- **Concurrent Connections**: >100 TCP connections
- **Memory Usage**: <100MB (idle state)

### Benchmark Tests

```bash
# Run performance benchmarks
cargo test --test integration performance --release

# Run specific benchmark
cargo test --test integration test_message_throughput --release
```

## Troubleshooting

### Common Issues

1. **Port Conflicts**: Change test ports in configuration
2. **Permission Errors**: Ensure write access to data directory
3. **Timeout Issues**: Increase test timeouts for slow systems
4. **Memory Issues**: Reduce test concurrency or message sizes

### Debug Commands

```bash
# Check service status
cargo run --bin nsqd -- --help
cargo run --bin nsqlookupd -- --help
cargo run --bin nsqadmin -- --help

# Manual service testing
curl http://localhost:4151/ping
curl http://localhost:4161/ping
curl http://localhost:4171/api/ping
```

## Contributing

### Test Guidelines

1. **Isolation**: Each test should be independent
2. **Cleanup**: Tests should clean up after themselves
3. **Documentation**: Complex tests should be documented
4. **Performance**: Tests should complete within reasonable time
5. **Reliability**: Tests should be deterministic and repeatable

### Code Style

- Use descriptive test names
- Follow Rust naming conventions
- Add comments for complex logic
- Use appropriate error handling
- Prefer async/await over callbacks

## Future Enhancements

### Planned Tests

- **TLS/SSL Testing**: Secure communication testing
- **Cluster Testing**: Multi-node cluster functionality
- **Persistence Testing**: Message persistence and recovery
- **Compression Testing**: Message compression support
- **Metrics Testing**: Metrics collection and reporting

### Test Infrastructure

- **Test Containers**: Docker-based test isolation
- **Load Testing**: High-throughput performance testing
- **Chaos Testing**: Failure scenario testing
- **Long-running Tests**: Stability and memory leak testing
