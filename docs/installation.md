# Installation Guide

This guide covers various methods to install and set up NSQ Rust on different platforms.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation Methods](#installation-methods)
- [Platform-Specific Instructions](#platform-specific-instructions)
- [Verification](#verification)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements

- **Operating System**: Linux, macOS, or Windows
- **Architecture**: x86_64 (AMD64) or ARM64
- **Memory**: Minimum 512MB RAM, recommended 2GB+
- **Disk Space**: Minimum 1GB free space
- **Network**: TCP/UDP ports 4150-4152, 4160-4162, 4171

### Software Dependencies

- **Rust**: 1.70+ with Cargo
- **Node.js**: 18+ (for NSQAdmin UI)
- **Git**: For source installation
- **OpenSSL**: For TLS support (optional)

## Installation Methods

### Method 1: From Source (Recommended)

#### Step 1: Clone Repository

```bash
git clone https://github.com/kenelite/nsq-rust.git
cd nsq-rust
```

#### Step 2: Build Components

```bash
# Build all components in release mode
cargo build --release

# Build specific components
cargo build --release --bin nsqd
cargo build --release --bin nsqlookupd
cargo build --release --bin nsqadmin
```

#### Step 3: Build NSQAdmin UI

```bash
cd nsqadmin-ui

# Install dependencies
npm install

# Build production version
npm run build

# Return to root directory
cd ..
```

#### Step 4: Install Binaries (Optional)

```bash
# Install to system PATH
cargo install --path nsqd
cargo install --path nsqlookupd
cargo install --path nsqadmin

# Or create symlinks
sudo ln -s $(pwd)/target/release/nsqd /usr/local/bin/nsqd
sudo ln -s $(pwd)/target/release/nsqlookupd /usr/local/bin/nsqlookupd
sudo ln -s $(pwd)/target/release/nsqadmin /usr/local/bin/nsqadmin
```

### Method 2: Binary Releases

#### Download Pre-built Binaries

```bash
# Download latest release
wget https://github.com/kenelite/nsq-rust/releases/latest/download/nsq-rust-linux-x86_64.tar.gz

# Extract
tar -xzf nsq-rust-linux-x86_64.tar.gz

# Install
sudo cp nsqd nsqlookupd nsqadmin /usr/local/bin/
```

#### Verify Installation

```bash
nsqd --version
nsqlookupd --version
nsqadmin --version
```

### Method 3: Package Managers

#### Homebrew (macOS)

```bash
# Add tap
brew tap kenelite/nsq-rust

# Install
brew install nsq-rust
```

#### Cargo Install

```bash
# Install individual components
cargo install nsqd
cargo install nsqlookupd
cargo install nsqadmin

# Install CLI tools
cargo install nsq_to_file
cargo install to_nsq
cargo install nsq_tail
cargo install nsq_stat
```

## Platform-Specific Instructions

### Linux

#### Ubuntu/Debian

```bash
# Install dependencies
sudo apt update
sudo apt install -y build-essential curl git

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# Build NSQ Rust
git clone https://github.com/kenelite/nsq-rust.git
cd nsq-rust
cargo build --release
```

#### CentOS/RHEL/Fedora

```bash
# Install dependencies
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y curl git

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Node.js
curl -fsSL https://rpm.nodesource.com/setup_18.x | sudo bash -
sudo dnf install -y nodejs

# Build NSQ Rust
git clone https://github.com/kenelite/nsq-rust.git
cd nsq-rust
cargo build --release
```

#### Arch Linux

```bash
# Install dependencies
sudo pacman -S base-devel curl git rust nodejs npm

# Build NSQ Rust
git clone https://github.com/kenelite/nsq-rust.git
cd nsq-rust
cargo build --release
```

### macOS

#### Using Homebrew

```bash
# Install dependencies
brew install rust nodejs git

# Build NSQ Rust
git clone https://github.com/kenelite/nsq-rust.git
cd nsq-rust
cargo build --release
```

#### Manual Installation

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Node.js
brew install nodejs

# Build NSQ Rust
git clone https://github.com/kenelite/nsq-rust.git
cd nsq-rust
cargo build --release
```

### Windows

#### Using Chocolatey

```powershell
# Install dependencies
choco install rust nodejs git

# Build NSQ Rust
git clone https://github.com/kenelite/nsq-rust.git
cd nsq-rust
cargo build --release
```

#### Manual Installation

1. Install Rust from [rustup.rs](https://rustup.rs/)
2. Install Node.js from [nodejs.org](https://nodejs.org/)
3. Install Git from [git-scm.com](https://git-scm.com/)
4. Open Command Prompt or PowerShell
5. Clone and build:

```cmd
git clone https://github.com/kenelite/nsq-rust.git
cd nsq-rust
cargo build --release
```

## Verification

### Test Installation

#### 1. Start Services

```bash
# Terminal 1: Start NSQLookupd
./target/release/nsqlookupd \
    --tcp-address=127.0.0.1:4160 \
    --http-address=127.0.0.1:4161

# Terminal 2: Start NSQD
./target/release/nsqd \
    --tcp-address=127.0.0.1:4150 \
    --http-address=127.0.0.1:4151 \
    --lookupd-tcp-address=127.0.0.1:4160

# Terminal 3: Start NSQAdmin
./target/release/nsqadmin \
    --lookupd-http-address=127.0.0.1:4161 \
    --http-address=127.0.0.1:4171
```

#### 2. Test Basic Functionality

```bash
# Test NSQLookupd
curl http://127.0.0.1:4161/ping

# Test NSQD
curl http://127.0.0.1:4151/ping

# Test NSQAdmin
curl http://127.0.0.1:4171/ping

# Publish a message
curl -d "Hello NSQ!" http://127.0.0.1:4151/pub?topic=test

# Check statistics
curl http://127.0.0.1:4151/stats
```

#### 3. Test Web Interface

1. Open http://127.0.0.1:4171 in your browser
2. Verify the NSQAdmin interface loads
3. Check that topics and channels are visible

### Performance Test

```bash
# Run performance test
cargo test --test integration test_message_throughput

# Run compatibility test
cargo test --test compatibility
```

## Troubleshooting

### Common Issues

#### Build Errors

**Error**: `error: failed to compile`
**Solution**: Ensure Rust 1.70+ is installed and up to date

```bash
rustup update
cargo clean
cargo build --release
```

**Error**: `error: linking with 'cc' failed`
**Solution**: Install build tools

```bash
# Ubuntu/Debian
sudo apt install build-essential

# CentOS/RHEL/Fedora
sudo dnf groupinstall "Development Tools"

# macOS
xcode-select --install
```

#### Runtime Errors

**Error**: `Address already in use`
**Solution**: Check for existing NSQ processes

```bash
# Check running processes
ps aux | grep nsq

# Kill existing processes
pkill nsqd
pkill nsqlookupd
pkill nsqadmin

# Or use different ports
./target/release/nsqd --tcp-address=127.0.0.1:4152
```

**Error**: `Permission denied`
**Solution**: Check file permissions

```bash
# Make binaries executable
chmod +x target/release/nsqd
chmod +x target/release/nsqlookupd
chmod +x target/release/nsqadmin

# Or run with sudo if needed
sudo ./target/release/nsqd
```

#### Network Issues

**Error**: `Connection refused`
**Solution**: Check firewall settings

```bash
# Ubuntu/Debian
sudo ufw allow 4150:4152
sudo ufw allow 4160:4162
sudo ufw allow 4171

# CentOS/RHEL/Fedora
sudo firewall-cmd --permanent --add-port=4150-4152/tcp
sudo firewall-cmd --permanent --add-port=4160-4162/tcp
sudo firewall-cmd --permanent --add-port=4171/tcp
sudo firewall-cmd --reload
```

### Getting Help

If you encounter issues not covered here:

1. Check the [Troubleshooting Guide](troubleshooting.md)
2. Search [GitHub Issues](https://github.com/kenelite/nsq-rust/issues)
3. Create a new issue with:
   - Operating system and version
   - Rust version (`rustc --version`)
   - Error messages and logs
   - Steps to reproduce

### Next Steps

After successful installation:

1. Read the [Configuration Guide](configuration.md)
2. Follow the [Deployment Guide](deployment.md)
3. Explore the [API Reference](api.md)
4. Check out [Performance Tuning](performance.md)

## Additional Resources

- [NSQ Rust GitHub Repository](https://github.com/kenelite/nsq-rust)
- [NSQ Official Documentation](https://nsq.io/)
- [Rust Documentation](https://doc.rust-lang.org/)
- [Tokio Documentation](https://tokio.rs/)
