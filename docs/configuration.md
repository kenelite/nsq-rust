# Configuration Reference

This guide covers all configuration options for NSQ Rust components.

## Table of Contents

- [NSQD Configuration](#nsqd-configuration)
- [NSQLookupd Configuration](#nsqlookupd-configuration)
- [NSQAdmin Configuration](#nsqadmin-configuration)
- [Environment Variables](#environment-variables)
- [Configuration Files](#configuration-files)
- [TLS Configuration](#tls-configuration)
- [Logging Configuration](#logging-configuration)
- [Metrics Configuration](#metrics-configuration)

## NSQD Configuration

### Command Line Options

#### Network Configuration

```bash
--tcp-address=127.0.0.1:4150          # TCP address to listen on
--http-address=127.0.0.1:4151        # HTTP address to listen on
--https-address=127.0.0.1:4152       # HTTPS address to listen on
--broadcast-address=127.0.0.1         # Address to broadcast to lookupd
--broadcast-tcp-port=4150             # TCP port to broadcast
--broadcast-http-port=4151            # HTTP port to broadcast
```

#### Lookupd Configuration

```bash
--lookupd-tcp-address=127.0.0.1:4160  # Lookupd TCP address
--lookupd-http-address=127.0.0.1:4161 # Lookupd HTTP address
```

#### Message Configuration

```bash
--max-memory-size=268435456           # Maximum memory for messages (256MB)
--max-body-size=5242880               # Maximum body size (5MB)
--max-rdy-count=2500                  # Maximum ready count
--max-output-buffer-size=65536        # Maximum output buffer size (64KB)
--max-output-buffer-timeout=1s        # Maximum output buffer timeout
--max-heartbeat-interval=60s          # Maximum heartbeat interval
--max-msg-timeout=15m                 # Maximum message timeout
--max-msg-size=1048576                # Maximum message size (1MB)
--max-req-timeout=1h                  # Maximum request timeout
```

#### Compression Configuration

```bash
--max-deflate-level=6                 # Maximum deflate compression level
--max-snappy-level=6                  # Maximum snappy compression level
```

#### Storage Configuration

```bash
--data-path=/var/lib/nsqd             # Data directory path
--mem-queue-size=10000               # Memory queue size
--disk-queue-size=1000000            # Disk queue size
--sync-timeout=2s                    # Sync timeout
--sync-every=2500                    # Sync every N messages
```

#### Performance Configuration

```bash
--worker-pool-size=4                 # Worker pool size
--max-concurrent-publishers=1000     # Maximum concurrent publishers
--max-concurrent-subscribers=1000    # Maximum concurrent subscribers
```

#### Metrics Configuration

```bash
--statsd-address=127.0.0.1:8125     # StatsD address
--statsd-prefix=nsq.%s               # StatsD prefix
--statsd-interval=60s               # StatsD interval
--statsd-mem-stats=true             # Include memory stats
```

#### Logging Configuration

```bash
--log-level=info                     # Log level (debug, info, warn, error)
--log-prefix="[nsqd] "              # Log prefix
--verbose=false                      # Verbose logging
```

### Configuration File

Create a configuration file `nsqd.conf`:

```toml
# Network configuration
tcp_address = "127.0.0.1:4150"
http_address = "127.0.0.1:4151"
https_address = "127.0.0.1:4152"
broadcast_address = "127.0.0.1"
broadcast_tcp_port = 4150
broadcast_http_port = 4151

# Lookupd configuration
lookupd_tcp_address = "127.0.0.1:4160"
lookupd_http_address = "127.0.0.1:4161"

# Message configuration
max_memory_size = 268435456
max_body_size = 5242880
max_rdy_count = 2500
max_output_buffer_size = 65536
max_output_buffer_timeout = "1s"
max_heartbeat_interval = "60s"
max_msg_timeout = "15m"
max_msg_size = 1048576
max_req_timeout = "1h"

# Compression configuration
max_deflate_level = 6
max_snappy_level = 6

# Storage configuration
data_path = "/var/lib/nsqd"
mem_queue_size = 10000
disk_queue_size = 1000000
sync_timeout = "2s"
sync_every = 2500

# Performance configuration
worker_pool_size = 4
max_concurrent_publishers = 1000
max_concurrent_subscribers = 1000

# Metrics configuration
statsd_address = "127.0.0.1:8125"
statsd_prefix = "nsq.%s"
statsd_interval = "60s"
statsd_mem_stats = true

# Logging configuration
log_level = "info"
log_prefix = "[nsqd] "
verbose = false
```

## NSQLookupd Configuration

### Command Line Options

#### Network Configuration

```bash
--tcp-address=127.0.0.1:4160         # TCP address to listen on
--http-address=127.0.0.1:4161       # HTTP address to listen on
--broadcast-address=127.0.0.1        # Address to broadcast
--broadcast-tcp-port=4160            # TCP port to broadcast
--broadcast-http-port=4161            # HTTP port to broadcast
```

#### Performance Configuration

```bash
--worker-pool-size=4                 # Worker pool size
--max-concurrent-connections=1000   # Maximum concurrent connections
```

#### Logging Configuration

```bash
--log-level=info                     # Log level (debug, info, warn, error)
--log-prefix="[nsqlookupd] "        # Log prefix
--verbose=false                      # Verbose logging
```

### Configuration File

Create a configuration file `nsqlookupd.conf`:

```toml
# Network configuration
tcp_address = "127.0.0.1:4160"
http_address = "127.0.0.1:4161"
broadcast_address = "127.0.0.1"
broadcast_tcp_port = 4160
broadcast_http_port = 4161

# Performance configuration
worker_pool_size = 4
max_concurrent_connections = 1000

# Logging configuration
log_level = "info"
log_prefix = "[nsqlookupd] "
verbose = false
```

## NSQAdmin Configuration

### Command Line Options

#### Network Configuration

```bash
--http-address=127.0.0.1:4171        # HTTP address to listen on
--https-address=127.0.0.1:4172       # HTTPS address to listen on
```

#### Lookupd Configuration

```bash
--lookupd-http-address=127.0.0.1:4161 # Lookupd HTTP address
```

#### Performance Configuration

```bash
--worker-pool-size=4                 # Worker pool size
--max-concurrent-connections=1000   # Maximum concurrent connections
```

#### Logging Configuration

```bash
--log-level=info                     # Log level (debug, info, warn, error)
--log-prefix="[nsqadmin] "          # Log prefix
--verbose=false                      # Verbose logging
```

### Configuration File

Create a configuration file `nsqadmin.conf`:

```toml
# Network configuration
http_address = "127.0.0.1:4171"
https_address = "127.0.0.1:4172"

# Lookupd configuration
lookupd_http_address = "127.0.0.1:4161"

# Performance configuration
worker_pool_size = 4
max_concurrent_connections = 1000

# Logging configuration
log_level = "info"
log_prefix = "[nsqadmin] "
verbose = false
```

## Environment Variables

All configuration options can be set using environment variables:

```bash
# NSQD environment variables
export NSQD_TCP_ADDRESS=127.0.0.1:4150
export NSQD_HTTP_ADDRESS=127.0.0.1:4151
export NSQD_DATA_PATH=/var/lib/nsqd
export NSQD_MAX_MEMORY_SIZE=268435456
export NSQD_LOG_LEVEL=info

# NSQLookupd environment variables
export NSQLOOKUPD_TCP_ADDRESS=127.0.0.1:4160
export NSQLOOKUPD_HTTP_ADDRESS=127.0.0.1:4161
export NSQLOOKUPD_LOG_LEVEL=info

# NSQAdmin environment variables
export NSQADMIN_HTTP_ADDRESS=127.0.0.1:4171
export NSQADMIN_LOOKUPD_HTTP_ADDRESS=127.0.0.1:4161
export NSQADMIN_LOG_LEVEL=info
```

## Configuration Files

### File Formats

NSQ Rust supports multiple configuration file formats:

- **TOML** (recommended): `nsqd.toml`, `nsqlookupd.toml`, `nsqadmin.toml`
- **YAML**: `nsqd.yaml`, `nsqlookupd.yaml`, `nsqadmin.yaml`
- **JSON**: `nsqd.json`, `nsqlookupd.json`, `nsqadmin.json`

### Configuration Precedence

Configuration is loaded in the following order (later overrides earlier):

1. Default values
2. Configuration file
3. Environment variables
4. Command line arguments

### Example Configuration Files

#### TOML Format

```toml
# nsqd.toml
[tcp]
address = "127.0.0.1:4150"

[http]
address = "127.0.0.1:4151"

[lookupd]
tcp_address = "127.0.0.1:4160"
http_address = "127.0.0.1:4161"

[messages]
max_memory_size = 268435456
max_body_size = 5242880
max_rdy_count = 2500

[storage]
data_path = "/var/lib/nsqd"
mem_queue_size = 10000
disk_queue_size = 1000000

[metrics]
statsd_address = "127.0.0.1:8125"
statsd_prefix = "nsq.%s"
statsd_interval = "60s"

[logging]
level = "info"
prefix = "[nsqd] "
verbose = false
```

#### YAML Format

```yaml
# nsqd.yaml
tcp:
  address: "127.0.0.1:4150"

http:
  address: "127.0.0.1:4151"

lookupd:
  tcp_address: "127.0.0.1:4160"
  http_address: "127.0.0.1:4161"

messages:
  max_memory_size: 268435456
  max_body_size: 5242880
  max_rdy_count: 2500

storage:
  data_path: "/var/lib/nsqd"
  mem_queue_size: 10000
  disk_queue_size: 1000000

metrics:
  statsd_address: "127.0.0.1:8125"
  statsd_prefix: "nsq.%s"
  statsd_interval: "60s"

logging:
  level: "info"
  prefix: "[nsqd] "
  verbose: false
```

#### JSON Format

```json
{
  "tcp": {
    "address": "127.0.0.1:4150"
  },
  "http": {
    "address": "127.0.0.1:4151"
  },
  "lookupd": {
    "tcp_address": "127.0.0.1:4160",
    "http_address": "127.0.0.1:4161"
  },
  "messages": {
    "max_memory_size": 268435456,
    "max_body_size": 5242880,
    "max_rdy_count": 2500
  },
  "storage": {
    "data_path": "/var/lib/nsqd",
    "mem_queue_size": 10000,
    "disk_queue_size": 1000000
  },
  "metrics": {
    "statsd_address": "127.0.0.1:8125",
    "statsd_prefix": "nsq.%s",
    "statsd_interval": "60s"
  },
  "logging": {
    "level": "info",
    "prefix": "[nsqd] ",
    "verbose": false
  }
}
```

## TLS Configuration

### Certificate Files

```bash
# Generate self-signed certificate
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

# Or use Let's Encrypt
certbot certonly --standalone -d your-domain.com
```

### TLS Configuration Options

```bash
# NSQD TLS configuration
--tls-cert=/path/to/cert.pem          # TLS certificate file
--tls-key=/path/to/key.pem            # TLS private key file
--tls-client-auth-policy=require      # TLS client auth policy
--tls-min-version=1.2                 # Minimum TLS version
--tls-max-version=1.3                 # Maximum TLS version
--tls-cipher-suites=TLS_AES_256_GCM_SHA384 # TLS cipher suites
```

### Configuration File TLS Section

```toml
[tls]
cert_file = "/path/to/cert.pem"
key_file = "/path/to/key.pem"
client_auth_policy = "require"
min_version = "1.2"
max_version = "1.3"
cipher_suites = ["TLS_AES_256_GCM_SHA384"]
```

## Logging Configuration

### Log Levels

- **`debug`**: Detailed debug information
- **`info`**: General information (default)
- **`warn`**: Warning messages
- **`error`**: Error messages only

### Log Format

```bash
# Structured logging (JSON)
--log-format=json

# Human-readable format
--log-format=text
```

### Log Rotation

```bash
# Log rotation configuration
--log-max-size=100MB                  # Maximum log file size
--log-max-backups=5                  # Maximum number of backup files
--log-max-age=30                     # Maximum age of log files (days)
--log-compress=true                  # Compress rotated log files
```

## Metrics Configuration

### StatsD Integration

```bash
# StatsD configuration
--statsd-address=127.0.0.1:8125       # StatsD server address
--statsd-prefix=nsq.%s               # StatsD metric prefix
--statsd-interval=60s                # StatsD reporting interval
--statsd-mem-stats=true              # Include memory statistics
--statsd-cpu-stats=true              # Include CPU statistics
```

### Prometheus Metrics

```bash
# Prometheus metrics endpoint
--prometheus-address=127.0.0.1:9090   # Prometheus metrics address
--prometheus-path=/metrics            # Prometheus metrics path
```

### Custom Metrics

```rust
// Custom metrics example
use nsq_common::Metrics;

let metrics = Metrics::new(&config)?;
metrics.counter("custom.metric", 1);
metrics.gauge("custom.gauge", 42.0);
metrics.histogram("custom.histogram", 1.5);
```

## Performance Tuning

### Memory Configuration

```bash
# Memory optimization
--max-memory-size=536870912           # 512MB
--mem-queue-size=20000               # Larger memory queue
--max-output-buffer-size=131072      # 128KB output buffer
```

### Network Configuration

```bash
# Network optimization
--max-rdy-count=5000                 # Higher ready count
--max-heartbeat-interval=30s         # Shorter heartbeat interval
--max-output-buffer-timeout=500ms    # Shorter buffer timeout
```

### Storage Configuration

```bash
# Storage optimization
--disk-queue-size=2000000           # Larger disk queue
--sync-every=5000                   # More frequent sync
--sync-timeout=1s                   # Shorter sync timeout
```

## Security Configuration

### Authentication

```bash
# Client authentication
--auth-http-address=127.0.0.1:4181  # Auth HTTP address
--auth-http-timeout=2s              # Auth HTTP timeout
--auth-http-allow-headers=Authorization # Auth HTTP headers
```

### Access Control

```bash
# Access control
--allow-topics-from-cidr=192.168.0.0/16 # Allowed topic CIDR
--deny-topics-from-cidr=10.0.0.0/8   # Denied topic CIDR
```

## Monitoring Configuration

### Health Checks

```bash
# Health check configuration
--health-check-interval=30s          # Health check interval
--health-check-timeout=5s            # Health check timeout
--health-check-retries=3             # Health check retries
```

### Alerting

```bash
# Alerting configuration
--alert-http-address=127.0.0.1:4191  # Alert HTTP address
--alert-http-timeout=5s              # Alert HTTP timeout
--alert-http-method=POST             # Alert HTTP method
```

## Troubleshooting

### Configuration Validation

```bash
# Validate configuration
nsqd --config=nsqd.conf --validate

# Check configuration
nsqd --config=nsqd.conf --check
```

### Debug Mode

```bash
# Enable debug mode
nsqd --log-level=debug --verbose=true

# Debug specific components
RUST_LOG=nsqd=debug nsqlookupd
```

### Configuration Issues

**Error**: `Configuration file not found`
**Solution**: Check file path and permissions

```bash
ls -la nsqd.conf
chmod 644 nsqd.conf
```

**Error**: `Invalid configuration value`
**Solution**: Check configuration syntax

```bash
# Validate TOML
toml validate nsqd.conf

# Validate YAML
yamllint nsqd.yaml

# Validate JSON
jq . nsqd.json
```

## Best Practices

### Configuration Management

1. **Use configuration files** for complex setups
2. **Use environment variables** for sensitive data
3. **Use command line arguments** for overrides
4. **Validate configurations** before deployment
5. **Document configuration changes**

### Security

1. **Use TLS** in production environments
2. **Restrict network access** with firewalls
3. **Use authentication** for sensitive topics
4. **Rotate certificates** regularly
5. **Monitor access logs**

### Performance

1. **Tune memory settings** based on workload
2. **Optimize network settings** for latency
3. **Configure appropriate timeouts**
4. **Monitor resource usage**
5. **Scale horizontally** when needed

## Additional Resources

- [NSQ Configuration Guide](https://nsq.io/overview/quick_start.html)
- [Rust Configuration Best Practices](https://doc.rust-lang.org/book/ch12-05-working-with-environment-variables.html)
- [TOML Configuration Format](https://toml.io/)
- [YAML Configuration Format](https://yaml.org/)
- [JSON Configuration Format](https://www.json.org/)
