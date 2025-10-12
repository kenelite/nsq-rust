# Docker Deployment Guide

This guide covers Docker deployment for NSQ Rust components.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Docker Images](#docker-images)
- [Docker Compose](#docker-compose)
- [Configuration](#configuration)
- [Development](#development)
- [Production](#production)
- [Testing](#testing)
- [Troubleshooting](#troubleshooting)
- [Best Practices](#best-practices)

## Overview

NSQ Rust provides comprehensive Docker support with:

- **Multi-stage builds** for optimized images
- **Multiple compose files** for different environments
- **Health checks** for container monitoring
- **Volume management** for data persistence
- **Network isolation** for security
- **Resource limits** for production deployments

## Quick Start

### Prerequisites

- Docker 20.10+
- Docker Compose 2.0+
- 4GB RAM minimum
- 10GB disk space

### Basic Deployment

```bash
# Clone repository
git clone https://github.com/kenelite/nsq-rust.git
cd nsq-rust

# Build and run
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f
```

### Access Services

- **NSQAdmin**: http://localhost:4171
- **NSQD**: http://localhost:4151
- **NSQLookupd**: http://localhost:4161

## Docker Images

### Available Images

| Image | Description | Ports |
|-------|-------------|-------|
| `nsq-rust-nsqd` | NSQD daemon | 4150 (TCP), 4151 (HTTP) |
| `nsq-rust-nsqlookupd` | NSQLookupd daemon | 4160 (TCP), 4161 (HTTP) |
| `nsq-rust-nsqadmin` | NSQAdmin web interface | 4171 (HTTP) |
| `nsq-rust-test` | Test runner | - |

### Building Images

#### Build All Images

```bash
# Build all images
./scripts/docker-build.sh

# Build with custom tag
./scripts/docker-build.sh --tag v1.3.0

# Build and push to registry
./scripts/docker-build.sh --push --tag v1.3.0
```

#### Build Specific Images

```bash
# Build NSQD only
./scripts/docker-build.sh --nsqd

# Build NSQLookupd only
./scripts/docker-build.sh --nsqlookupd

# Build NSQAdmin only
./scripts/docker-build.sh --nsqadmin

# Build test image only
./scripts/docker-build.sh --test
```

#### Manual Build

```bash
# Build NSQD
docker build -f Dockerfile.nsqd -t nsq-rust-nsqd:latest .

# Build NSQLookupd
docker build -f Dockerfile.nsqlookupd -t nsq-rust-nsqlookupd:latest .

# Build NSQAdmin
docker build -f Dockerfile.nsqadmin -t nsq-rust-nsqadmin:latest .

# Build test image
docker build -f Dockerfile.test -t nsq-rust-test:latest .
```

### Image Details

#### NSQD Image

```dockerfile
FROM rust:1.75-slim as builder
# ... build steps ...
FROM debian:bookworm-slim
# ... runtime setup ...
EXPOSE 4150 4151
CMD ["nsqd"]
```

**Features:**
- Multi-stage build for smaller image size
- Non-root user for security
- Health checks included
- Configurable via environment variables

#### NSQLookupd Image

```dockerfile
FROM rust:1.75-slim as builder
# ... build steps ...
FROM debian:bookworm-slim
# ... runtime setup ...
EXPOSE 4160 4161
CMD ["nsqlookupd"]
```

**Features:**
- Lightweight runtime image
- Service discovery capabilities
- Health monitoring
- Cluster support

#### NSQAdmin Image

```dockerfile
FROM rust:1.75-slim as builder
# ... build steps ...
FROM debian:bookworm-slim
# ... runtime setup ...
EXPOSE 4171
CMD ["nsqadmin"]
```

**Features:**
- Modern web UI included
- Real-time statistics
- Topic and channel management
- Responsive design

## Docker Compose

### Compose Files

| File | Purpose | Environment |
|------|---------|-------------|
| `docker-compose.yml` | Basic setup | Development |
| `docker-compose.dev.yml` | Development | Local development |
| `docker-compose.prod.yml` | Production | Production deployment |
| `docker-compose.test.yml` | Testing | CI/CD testing |

### Basic Compose

```yaml
version: '3.8'

services:
  nsqlookupd:
    build:
      context: .
      dockerfile: Dockerfile.nsqlookupd
    ports:
      - "4160:4160"
      - "4161:4161"
    networks:
      - nsq-network

  nsqd:
    build:
      context: .
      dockerfile: Dockerfile.nsqd
    ports:
      - "4150:4150"
      - "4151:4151"
    depends_on:
      - nsqlookupd
    networks:
      - nsq-network

  nsqadmin:
    build:
      context: .
      dockerfile: Dockerfile.nsqadmin
    ports:
      - "4171:4171"
    depends_on:
      - nsqlookupd
    networks:
      - nsq-network

networks:
  nsq-network:
    driver: bridge
```

### Development Compose

```yaml
version: '3.8'

services:
  nsqlookupd:
    build:
      context: .
      dockerfile: Dockerfile.nsqlookupd
    image: nsq-rust-nsqlookupd:dev
    environment:
      - RUST_LOG=debug
    volumes:
      - nsqlookupd-data:/var/lib/nsqlookupd

  nsqd:
    build:
      context: .
      dockerfile: Dockerfile.nsqd
    image: nsq-rust-nsqd:dev
    environment:
      - RUST_LOG=debug
    volumes:
      - nsqd-data:/var/lib/nsqd
      - ./config:/etc/nsq:ro

  nsqadmin:
    build:
      context: .
      dockerfile: Dockerfile.nsqadmin
    image: nsq-rust-nsqadmin:dev
    environment:
      - RUST_LOG=debug

volumes:
  nsqlookupd-data:
  nsqd-data:
```

### Production Compose

```yaml
version: '3.8'

services:
  nsqlookupd-1:
    build:
      context: .
      dockerfile: Dockerfile.nsqlookupd
    image: nsq-rust-nsqlookupd:latest
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:4161/ping"]
      interval: 30s
      timeout: 3s
      retries: 3
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: '0.5'

  nsqd-1:
    build:
      context: .
      dockerfile: Dockerfile.nsqd
    image: nsq-rust-nsqd:latest
    restart: unless-stopped
    depends_on:
      - nsqlookupd-1
    deploy:
      resources:
        limits:
          memory: 2G
          cpus: '2.0'
```

### Running Compose

#### Using Scripts

```bash
# Development environment
./scripts/docker-run.sh --env dev --action up

# Production environment
./scripts/docker-run.sh --env prod --action up --build

# Test environment
./scripts/docker-run.sh --env test --action up

# View logs
./scripts/docker-run.sh --env dev --action logs

# Stop services
./scripts/docker-run.sh --env dev --action down
```

#### Manual Commands

```bash
# Start services
docker-compose up -d

# Start with build
docker-compose up -d --build

# Start specific services
docker-compose up -d nsqlookupd nsqd

# View logs
docker-compose logs -f

# Stop services
docker-compose down

# Stop and remove volumes
docker-compose down -v
```

## Configuration

### Environment Variables

#### NSQD Configuration

```bash
# Network configuration
NSQD_TCP_ADDRESS=0.0.0.0:4150
NSQD_HTTP_ADDRESS=0.0.0.0:4151

# Lookupd configuration
NSQD_LOOKUPD_TCP_ADDRESS=nsqlookupd:4160
NSQD_LOOKUPD_HTTP_ADDRESS=nsqlookupd:4161

# Storage configuration
NSQD_DATA_PATH=/var/lib/nsqd
NSQD_MEM_QUEUE_SIZE=10000
NSQD_DISK_QUEUE_SIZE=1000000

# Message configuration
NSQD_MAX_MEMORY_SIZE=268435456
NSQD_MAX_BODY_SIZE=5242880
NSQD_MAX_RDY_COUNT=2500

# Logging configuration
RUST_LOG=info
```

#### NSQLookupd Configuration

```bash
# Network configuration
NSQLOOKUPD_TCP_ADDRESS=0.0.0.0:4160
NSQLOOKUPD_HTTP_ADDRESS=0.0.0.0:4161

# Performance configuration
NSQLOOKUPD_WORKER_POOL_SIZE=4
NSQLOOKUPD_MAX_CONCURRENT_CONNECTIONS=1000

# Logging configuration
RUST_LOG=info
```

#### NSQAdmin Configuration

```bash
# Network configuration
NSQADMIN_HTTP_ADDRESS=0.0.0.0:4171

# Lookupd configuration
NSQADMIN_LOOKUPD_HTTP_ADDRESS=nsqlookupd:4161

# Logging configuration
RUST_LOG=info
```

### Configuration Files

#### Mounting Config Files

```yaml
services:
  nsqd:
    volumes:
      - ./config/nsqd.conf:/etc/nsq/nsqd.conf:ro
    command: ["nsqd", "--config=/etc/nsq/nsqd.conf"]
```

#### Using Config Maps

```yaml
services:
  nsqd:
    volumes:
      - nsqd-config:/etc/nsq:ro
    command: ["nsqd", "--config=/etc/nsq/nsqd.conf"]

volumes:
  nsqd-config:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: ./config
```

## Development

### Development Setup

```bash
# Start development environment
./scripts/docker-run.sh --env dev --action up

# Build UI
cd nsqadmin-ui
npm install
npm run build
cd ..

# Rebuild containers
docker-compose -f docker-compose.dev.yml up -d --build
```

### Development Features

- **Hot reloading** for UI development
- **Debug logging** enabled
- **Volume mounts** for configuration
- **Port forwarding** for local access
- **Development tools** included

### Debugging

```bash
# View container logs
docker-compose logs -f nsqd

# Execute shell in container
docker-compose exec nsqd /bin/bash

# Inspect container
docker inspect nsq-nsqd-1

# Monitor resource usage
docker stats
```

## Production

### Production Deployment

```bash
# Deploy production environment
./scripts/docker-run.sh --env prod --action up --build

# Scale NSQD instances
docker-compose -f docker-compose.prod.yml up -d --scale nsqd=5

# Update services
docker-compose -f docker-compose.prod.yml pull
docker-compose -f docker-compose.prod.yml up -d
```

### Production Features

- **High availability** with multiple instances
- **Resource limits** and reservations
- **Health checks** and monitoring
- **Persistent volumes** for data
- **Network isolation** for security
- **Restart policies** for reliability

### Scaling

```yaml
services:
  nsqd:
    deploy:
      replicas: 3
      resources:
        limits:
          memory: 2G
          cpus: '2.0'
        reservations:
          memory: 1G
          cpus: '1.0'
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
```

### Monitoring

```yaml
services:
  nsqd:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:4151/ping"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 5s
    labels:
      - "prometheus.scrape=true"
      - "prometheus.port=4151"
      - "prometheus.path=/metrics"
```

## Testing

### Test Environment

```bash
# Start test environment
./scripts/docker-run.sh --env test --action up

# Run integration tests
docker-compose -f docker-compose.test.yml exec test-runner cargo test

# Run specific tests
docker-compose -f docker-compose.test.yml exec test-runner cargo test basic_functionality
```

### Test Features

- **Isolated environment** for testing
- **Test runner** container
- **Automated test execution**
- **Test data** management
- **CI/CD integration** ready

### Test Configuration

```yaml
services:
  test-runner:
    build:
      context: .
      dockerfile: Dockerfile.test
    depends_on:
      - nsqlookupd
      - nsqd
      - nsqadmin
    environment:
      - RUST_LOG=debug
      - NSQD_HTTP_ADDRESS=nsqd:4151
      - NSQLOOKUPD_HTTP_ADDRESS=nsqlookupd:4161
    volumes:
      - ./tests:/app/tests:ro
    command: ["cargo", "test", "--test", "integration"]
```

## Troubleshooting

### Common Issues

#### Container Won't Start

```bash
# Check container logs
docker-compose logs nsqd

# Check container status
docker-compose ps

# Inspect container
docker inspect nsq-nsqd-1
```

#### Port Conflicts

```bash
# Check port usage
netstat -tulpn | grep :4150

# Change ports in compose file
ports:
  - "4152:4150"  # Use different host port
```

#### Volume Issues

```bash
# List volumes
docker volume ls

# Inspect volume
docker volume inspect nsq-rust_nsqd-data

# Remove volume
docker volume rm nsq-rust_nsqd-data
```

#### Network Issues

```bash
# List networks
docker network ls

# Inspect network
docker network inspect nsq-rust_nsq-network

# Create custom network
docker network create nsq-custom
```

### Debug Commands

```bash
# View all logs
docker-compose logs -f

# View specific service logs
docker-compose logs -f nsqd

# Execute command in container
docker-compose exec nsqd /bin/bash

# Monitor resource usage
docker stats

# Check container health
docker-compose ps
```

### Performance Issues

#### Memory Usage

```bash
# Monitor memory usage
docker stats --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}"

# Set memory limits
deploy:
  resources:
    limits:
      memory: 2G
    reservations:
      memory: 1G
```

#### CPU Usage

```bash
# Monitor CPU usage
docker stats --format "table {{.Container}}\t{{.CPUPerc}}"

# Set CPU limits
deploy:
  resources:
    limits:
      cpus: '2.0'
    reservations:
      cpus: '1.0'
```

## Best Practices

### Security

#### Use Non-Root User

```dockerfile
# Create non-root user
RUN useradd -r -s /bin/false nsq

# Switch to non-root user
USER nsq
```

#### Limit Container Capabilities

```yaml
services:
  nsqd:
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    cap_add:
      - NET_BIND_SERVICE
```

#### Use Secrets for Sensitive Data

```yaml
services:
  nsqd:
    secrets:
      - tls_cert
      - tls_key
    environment:
      - TLS_CERT_FILE=/run/secrets/tls_cert
      - TLS_KEY_FILE=/run/secrets/tls_key

secrets:
  tls_cert:
    file: ./secrets/cert.pem
  tls_key:
    file: ./secrets/key.pem
```

### Performance

#### Optimize Image Size

```dockerfile
# Use multi-stage builds
FROM rust:1.75-slim as builder
# ... build steps ...

FROM debian:bookworm-slim
# ... runtime setup ...

# Remove build dependencies
RUN apt-get remove -y build-essential && apt-get autoremove -y
```

#### Use .dockerignore

```dockerignore
# Git
.git
.gitignore

# Documentation
docs/
README.md

# IDE
.vscode/
.idea/

# OS
.DS_Store
Thumbs.db

# Rust
target/
Cargo.lock
```

#### Cache Dependencies

```dockerfile
# Copy dependency files first
COPY Cargo.toml Cargo.lock ./

# Build dependencies
RUN cargo build --release --dependencies-only

# Copy source code
COPY src/ ./src/

# Build application
RUN cargo build --release
```

### Monitoring

#### Health Checks

```yaml
services:
  nsqd:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:4151/ping"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 5s
```

#### Logging

```yaml
services:
  nsqd:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

#### Metrics

```yaml
services:
  nsqd:
    labels:
      - "prometheus.scrape=true"
      - "prometheus.port=4151"
      - "prometheus.path=/metrics"
```

### Backup and Recovery

#### Volume Backup

```bash
# Backup volume
docker run --rm -v nsq-rust_nsqd-data:/data -v $(pwd):/backup alpine tar czf /backup/nsqd-data.tar.gz -C /data .

# Restore volume
docker run --rm -v nsq-rust_nsqd-data:/data -v $(pwd):/backup alpine tar xzf /backup/nsqd-data.tar.gz -C /data
```

#### Configuration Backup

```bash
# Backup configuration
docker-compose config > docker-compose.backup.yml

# Backup environment
env > .env.backup
```

## Additional Resources

- [Docker Documentation](https://docs.docker.com/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [NSQ Rust Documentation](../README.md)
- [Configuration Guide](configuration.md)
- [Deployment Guide](deployment.md)
