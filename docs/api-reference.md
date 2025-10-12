# API Reference

This document provides a comprehensive reference for all NSQ Rust APIs.

## Table of Contents

- [NSQD HTTP API](#nsqd-http-api)
- [NSQLookupd HTTP API](#nsqlookupd-http-api)
- [NSQAdmin HTTP API](#nsqadmin-http-api)
- [TCP Protocol](#tcp-protocol)
- [Client Libraries](#client-libraries)
- [Error Codes](#error-codes)
- [Rate Limiting](#rate-limiting)
- [Authentication](#authentication)

## NSQD HTTP API

### Base URL

```
http://localhost:4151
```

### Endpoints

#### Health Check

**GET** `/ping`

Returns `OK` if the server is healthy.

**Response:**
```
200 OK
OK
```

#### Server Information

**GET** `/info`

Returns server information and configuration.

**Response:**
```json
{
  "version": "1.3.0",
  "build": "rust",
  "tcp_port": 4150,
  "http_port": 4151,
  "https_port": 4152,
  "broadcast_address": "127.0.0.1",
  "hostname": "localhost",
  "start_time": 1640995200,
  "uptime": 3600
}
```

#### Statistics

**GET** `/stats`

Returns detailed statistics about topics, channels, and clients.

**Response:**
```json
{
  "version": "1.3.0",
  "health": "OK",
  "start_time": 1640995200,
  "uptime": 3600,
  "topics": [
    {
      "topic_name": "test_topic",
      "message_count": 1000,
      "depth": 100,
      "backend_depth": 0,
      "paused": false,
      "channels": [
        {
          "channel_name": "test_channel",
          "message_count": 500,
          "depth": 50,
          "backend_depth": 0,
          "paused": false,
          "clients": [
            {
              "client_id": "client_123",
              "hostname": "localhost",
              "version": "1.3.0",
              "remote_address": "127.0.0.1:12345",
              "state": "SUBSCRIBED",
              "ready_count": 10,
              "in_flight_count": 5,
              "message_count": 100,
              "finish_count": 95,
              "requeue_count": 0,
              "connect_time": 1640995200,
              "sample_rate": 0.0,
              "deflate": false,
              "snappy": false,
              "user_agent": "nsq-rust/1.3.0",
              "tls": false
            }
          ]
        }
      ]
    }
  ]
}
```

#### Publish Message

**POST** `/pub?topic=<topic>`

Publishes a message to the specified topic.

**Parameters:**
- `topic` (required): Topic name

**Request Body:**
```
Message content
```

**Response:**
```
200 OK
OK
```

**Error Responses:**
```
400 Bad Request
E_BAD_TOPIC
```

#### Publish Multiple Messages

**POST** `/mpub?topic=<topic>`

Publishes multiple messages to the specified topic.

**Parameters:**
- `topic` (required): Topic name

**Request Body:**
```
Message 1
Message 2
Message 3
```

**Response:**
```
200 OK
OK
```

#### Create Topic

**POST** `/topic/create?topic=<topic>`

Creates a new topic.

**Parameters:**
- `topic` (required): Topic name

**Response:**
```
200 OK
OK
```

#### Delete Topic

**POST** `/topic/delete?topic=<topic>`

Deletes a topic and all its channels.

**Parameters:**
- `topic` (required): Topic name

**Response:**
```
200 OK
OK
```

#### Pause Topic

**POST** `/topic/pause?topic=<topic>`

Pauses message delivery to all channels in the topic.

**Parameters:**
- `topic` (required): Topic name

**Response:**
```
200 OK
OK
```

#### Unpause Topic

**POST** `/topic/unpause?topic=<topic>`

Resumes message delivery to all channels in the topic.

**Parameters:**
- `topic` (required): Topic name

**Response:**
```
200 OK
OK
```

#### Create Channel

**POST** `/channel/create?topic=<topic>&channel=<channel>`

Creates a new channel in the specified topic.

**Parameters:**
- `topic` (required): Topic name
- `channel` (required): Channel name

**Response:**
```
200 OK
OK
```

#### Delete Channel

**POST** `/channel/delete?topic=<topic>&channel=<channel>`

Deletes a channel from the specified topic.

**Parameters:**
- `topic` (required): Topic name
- `channel` (required): Channel name

**Response:**
```
200 OK
OK
```

#### Pause Channel

**POST** `/channel/pause?topic=<topic>&channel=<channel>`

Pauses message delivery to the specified channel.

**Parameters:**
- `topic` (required): Topic name
- `channel` (required): Channel name

**Response:**
```
200 OK
OK
```

#### Unpause Channel

**POST** `/channel/unpause?topic=<topic>&channel=<channel>`

Resumes message delivery to the specified channel.

**Parameters:**
- `topic` (required): Topic name
- `channel` (required): Channel name

**Response:**
```
200 OK
OK
```

#### Empty Topic

**POST** `/topic/empty?topic=<topic>`

Empties all messages from the specified topic.

**Parameters:**
- `topic` (required): Topic name

**Response:**
```
200 OK
OK
```

#### Empty Channel

**POST** `/channel/empty?topic=<topic>&channel=<channel>`

Empties all messages from the specified channel.

**Parameters:**
- `topic` (required): Topic name
- `channel` (required): Channel name

**Response:**
```
200 OK
OK
```

## NSQLookupd HTTP API

### Base URL

```
http://localhost:4161
```

### Endpoints

#### Health Check

**GET** `/ping`

Returns `OK` if the server is healthy.

**Response:**
```
200 OK
OK
```

#### Server Information

**GET** `/info`

Returns server information and configuration.

**Response:**
```json
{
  "version": "1.3.0",
  "build": "rust",
  "tcp_port": 4160,
  "http_port": 4161,
  "broadcast_address": "127.0.0.1",
  "hostname": "localhost",
  "start_time": 1640995200,
  "uptime": 3600
}
```

#### Lookup Topic

**GET** `/lookup?topic=<topic>`

Returns all NSQD nodes that have the specified topic.

**Parameters:**
- `topic` (required): Topic name

**Response:**
```json
{
  "channels": [
    "channel1",
    "channel2"
  ],
  "producers": [
    {
      "hostname": "localhost",
      "broadcast_address": "127.0.0.1",
      "tcp_port": 4150,
      "http_port": 4151,
      "version": "1.3.0",
      "tombstoned": false,
      "tombstoned_at": null
    }
  ]
}
```

#### Lookup Channel

**GET** `/lookup?topic=<topic>&channel=<channel>`

Returns all NSQD nodes that have the specified topic and channel.

**Parameters:**
- `topic` (required): Topic name
- `channel` (required): Channel name

**Response:**
```json
{
  "channels": [
    "channel1",
    "channel2"
  ],
  "producers": [
    {
      "hostname": "localhost",
      "broadcast_address": "127.0.0.1",
      "tcp_port": 4150,
      "http_port": 4151,
      "version": "1.3.0",
      "tombstoned": false,
      "tombstoned_at": null
    }
  ]
}
```

#### List Topics

**GET** `/topics`

Returns all topics registered with the lookupd.

**Response:**
```json
{
  "topics": [
    "topic1",
    "topic2",
    "topic3"
  ]
}
```

#### List Channels

**GET** `/channels?topic=<topic>`

Returns all channels for the specified topic.

**Parameters:**
- `topic` (required): Topic name

**Response:**
```json
{
  "channels": [
    "channel1",
    "channel2",
    "channel3"
  ]
}
```

#### List Nodes

**GET** `/nodes`

Returns all NSQD nodes registered with the lookupd.

**Response:**
```json
{
  "producers": [
    {
      "hostname": "localhost",
      "broadcast_address": "127.0.0.1",
      "tcp_port": 4150,
      "http_port": 4151,
      "version": "1.3.0",
      "tombstoned": false,
      "tombstoned_at": null
    }
  ]
}
```

#### Delete Topic

**POST** `/topic/delete?topic=<topic>`

Deletes a topic from the lookupd.

**Parameters:**
- `topic` (required): Topic name

**Response:**
```
200 OK
OK
```

#### Delete Channel

**POST** `/channel/delete?topic=<topic>&channel=<channel>`

Deletes a channel from the lookupd.

**Parameters:**
- `topic` (required): Topic name
- `channel` (required): Channel name

**Response:**
```
200 OK
OK
```

#### Tombstone Producer

**POST** `/tombstone_topic_producer?topic=<topic>&node=<node>`

Tombstones a producer for a specific topic.

**Parameters:**
- `topic` (required): Topic name
- `node` (required): Node address (hostname:port)

**Response:**
```
200 OK
OK
```

## NSQAdmin HTTP API

### Base URL

```
http://localhost:4171
```

### Endpoints

#### Health Check

**GET** `/ping`

Returns `OK` if the server is healthy.

**Response:**
```
200 OK
OK
```

#### Server Information

**GET** `/info`

Returns server information and configuration.

**Response:**
```json
{
  "version": "1.3.0",
  "build": "rust",
  "http_port": 4171,
  "https_port": 4172,
  "hostname": "localhost",
  "start_time": 1640995200,
  "uptime": 3600
}
```

#### Dashboard Statistics

**GET** `/api/stats`

Returns aggregated statistics from all NSQD nodes.

**Response:**
```json
{
  "version": "1.3.0",
  "health": "OK",
  "start_time": 1640995200,
  "uptime": 3600,
  "topics": [
    {
      "topic_name": "test_topic",
      "message_count": 1000,
      "depth": 100,
      "backend_depth": 0,
      "paused": false,
      "channels": [
        {
          "channel_name": "test_channel",
          "message_count": 500,
          "depth": 50,
          "backend_depth": 0,
          "paused": false,
          "clients": [
            {
              "client_id": "client_123",
              "hostname": "localhost",
              "version": "1.3.0",
              "remote_address": "127.0.0.1:12345",
              "state": "SUBSCRIBED",
              "ready_count": 10,
              "in_flight_count": 5,
              "message_count": 100,
              "finish_count": 95,
              "requeue_count": 0,
              "connect_time": 1640995200,
              "sample_rate": 0.0,
              "deflate": false,
              "snappy": false,
              "user_agent": "nsq-rust/1.3.0",
              "tls": false
            }
          ]
        }
      ]
    }
  ],
  "nodes": [
    {
      "hostname": "localhost",
      "broadcast_address": "127.0.0.1",
      "tcp_port": 4150,
      "http_port": 4151,
      "version": "1.3.0",
      "tombstoned": false,
      "tombstoned_at": null
    }
  ]
}
```

#### Topic Management

**GET** `/api/topics`

Returns all topics across all NSQD nodes.

**Response:**
```json
{
  "topics": [
    {
      "topic_name": "test_topic",
      "message_count": 1000,
      "depth": 100,
      "backend_depth": 0,
      "paused": false,
      "channels": [
        {
          "channel_name": "test_channel",
          "message_count": 500,
          "depth": 50,
          "backend_depth": 0,
          "paused": false
        }
      ]
    }
  ]
}
```

#### Channel Management

**GET** `/api/channels?topic=<topic>`

Returns all channels for the specified topic.

**Parameters:**
- `topic` (required): Topic name

**Response:**
```json
{
  "channels": [
    {
      "channel_name": "test_channel",
      "message_count": 500,
      "depth": 50,
      "backend_depth": 0,
      "paused": false,
      "clients": [
        {
          "client_id": "client_123",
          "hostname": "localhost",
          "version": "1.3.0",
          "remote_address": "127.0.0.1:12345",
          "state": "SUBSCRIBED",
          "ready_count": 10,
          "in_flight_count": 5,
          "message_count": 100,
          "finish_count": 95,
          "requeue_count": 0,
          "connect_time": 1640995200,
          "sample_rate": 0.0,
          "deflate": false,
          "snappy": false,
          "user_agent": "nsq-rust/1.3.0",
          "tls": false
        }
      ]
    }
  ]
}
```

#### Node Management

**GET** `/api/nodes`

Returns all NSQD nodes.

**Response:**
```json
{
  "nodes": [
    {
      "hostname": "localhost",
      "broadcast_address": "127.0.0.1",
      "tcp_port": 4150,
      "http_port": 4151,
      "version": "1.3.0",
      "tombstoned": false,
      "tombstoned_at": null
    }
  ]
}
```

## TCP Protocol

### Connection

Connect to NSQD on the configured TCP port (default: 4150).

### Commands

#### IDENTIFY

**Command:** `IDENTIFY\n`

**Body:** JSON configuration

**Response:** JSON response

**Example:**
```
IDENTIFY
{"client_id":"test_client","hostname":"localhost","user_agent":"nsq-rust/1.3.0","feature_negotiation":true}
```

#### SUBSCRIBE

**Command:** `SUBSCRIBE <topic> <channel>\n`

**Example:**
```
SUBSCRIBE test_topic test_channel
```

#### PUBLISH

**Command:** `PUBLISH <topic>\n`

**Body:** Message content

**Example:**
```
PUBLISH test_topic
Hello, World!
```

#### MPUB

**Command:** `MPUB <topic>\n`

**Body:** Number of messages (4 bytes) + Message 1 length (4 bytes) + Message 1 + Message 2 length (4 bytes) + Message 2 + ...

**Example:**
```
MPUB test_topic
0000000200000005Hello0000000005World
```

#### READY

**Command:** `READY <count>\n`

**Example:**
```
READY 10
```

#### FINISH

**Command:** `FINISH <message_id>\n`

**Example:**
```
FINISH 1234567890
```

#### REQUEUE

**Command:** `REQUEUE <message_id> <timeout>\n`

**Example:**
```
REQUEUE 1234567890 5000
```

#### NOP

**Command:** `NOP\n`

#### CLOSE

**Command:** `CLOSE\n`

### Message Format

#### Frame

```
[4 bytes: Frame Type][4 bytes: Frame Size][Frame Data]
```

#### Frame Types

- `0`: Response
- `1`: Error
- `2`: Message

#### Message Frame

```
[8 bytes: Timestamp][2 bytes: Attempts][16 bytes: Message ID][Message Body]
```

### Error Codes

- `E_INVALID`: Invalid command
- `E_BAD_TOPIC`: Invalid topic name
- `E_BAD_CHANNEL`: Invalid channel name
- `E_BAD_MESSAGE`: Invalid message
- `E_PUB_FAILED`: Publish failed
- `E_MPUB_FAILED`: Multi-publish failed
- `E_FIN_FAILED`: Finish failed
- `E_REQ_FAILED`: Requeue failed
- `E_TOUCH_FAILED`: Touch failed
- `E_AUTH_FAILED`: Authentication failed
- `E_UNAUTHORIZED`: Unauthorized access

## Client Libraries

### Rust Client

```rust
use nsq_protocol::{Client, Message, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Connect to NSQD
    let mut client = Client::connect("127.0.0.1:4150").await?;
    
    // Identify
    client.identify("test_client", "localhost", "nsq-rust/1.3.0").await?;
    
    // Subscribe to topic
    client.subscribe("test_topic", "test_channel").await?;
    
    // Set ready count
    client.ready(10).await?;
    
    // Receive messages
    while let Some(message) = client.recv().await? {
        println!("Received: {}", String::from_utf8_lossy(&message.body));
        
        // Finish message
        client.finish(&message.id).await?;
    }
    
    Ok(())
}
```

### Publisher Example

```rust
use nsq_protocol::{Publisher, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Connect to NSQD
    let mut publisher = Publisher::connect("127.0.0.1:4150").await?;
    
    // Identify
    publisher.identify("test_publisher", "localhost", "nsq-rust/1.3.0").await?;
    
    // Publish message
    publisher.publish("test_topic", b"Hello, World!").await?;
    
    // Publish multiple messages
    let messages = vec![
        b"Message 1".to_vec(),
        b"Message 2".to_vec(),
        b"Message 3".to_vec(),
    ];
    publisher.mpub("test_topic", messages).await?;
    
    Ok(())
}
```

## Error Codes

### HTTP Error Codes

- `400 Bad Request`: Invalid request
- `401 Unauthorized`: Authentication required
- `403 Forbidden`: Access denied
- `404 Not Found`: Resource not found
- `405 Method Not Allowed`: HTTP method not allowed
- `500 Internal Server Error`: Server error
- `502 Bad Gateway`: Gateway error
- `503 Service Unavailable`: Service unavailable

### NSQ Error Codes

- `E_INVALID`: Invalid command
- `E_BAD_TOPIC`: Invalid topic name
- `E_BAD_CHANNEL`: Invalid channel name
- `E_BAD_MESSAGE`: Invalid message
- `E_PUB_FAILED`: Publish failed
- `E_MPUB_FAILED`: Multi-publish failed
- `E_FIN_FAILED`: Finish failed
- `E_REQ_FAILED`: Requeue failed
- `E_TOUCH_FAILED`: Touch failed
- `E_AUTH_FAILED`: Authentication failed
- `E_UNAUTHORIZED`: Unauthorized access

## Rate Limiting

### HTTP Rate Limiting

NSQ Rust implements rate limiting for HTTP endpoints:

- **Default limit**: 100 requests per second per IP
- **Burst limit**: 200 requests per second per IP
- **Window**: 1 second

### TCP Rate Limiting

TCP connections are rate limited:

- **Default limit**: 1000 messages per second per connection
- **Burst limit**: 2000 messages per second per connection
- **Window**: 1 second

### Configuration

```bash
# HTTP rate limiting
--http-rate-limit=100                 # Requests per second
--http-rate-burst=200                # Burst limit
--http-rate-window=1s                # Time window

# TCP rate limiting
--tcp-rate-limit=1000                # Messages per second
--tcp-rate-burst=2000                # Burst limit
--tcp-rate-window=1s                  # Time window
```

## Authentication

### HTTP Authentication

NSQ Rust supports HTTP authentication:

```bash
# Basic authentication
--http-auth-user=admin
--http-auth-password=secret

# Token authentication
--http-auth-token=your-token-here
```

### TCP Authentication

TCP connections can be authenticated:

```bash
# TCP authentication
--tcp-auth-secret=your-secret-here
```

### Client Authentication

Clients must authenticate:

```rust
// Basic authentication
let client = Client::connect("127.0.0.1:4150")
    .auth_basic("admin", "secret")
    .await?;

// Token authentication
let client = Client::connect("127.0.0.1:4150")
    .auth_token("your-token-here")
    .await?;
```

## Examples

### Publishing Messages

```bash
# Publish single message
curl -X POST "http://localhost:4151/pub?topic=test_topic" \
     -d "Hello, World!"

# Publish multiple messages
curl -X POST "http://localhost:4151/mpub?topic=test_topic" \
     -d "Message 1
Message 2
Message 3"
```

### Consuming Messages

```rust
use nsq_protocol::{Client, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut client = Client::connect("127.0.0.1:4150").await?;
    client.identify("consumer", "localhost", "nsq-rust/1.3.0").await?;
    client.subscribe("test_topic", "test_channel").await?;
    client.ready(10).await?;
    
    while let Some(message) = client.recv().await? {
        println!("Received: {}", String::from_utf8_lossy(&message.body));
        client.finish(&message.id).await?;
    }
    
    Ok(())
}
```

### Topic Management

```bash
# Create topic
curl -X POST "http://localhost:4151/topic/create?topic=test_topic"

# Create channel
curl -X POST "http://localhost:4151/channel/create?topic=test_topic&channel=test_channel"

# Pause topic
curl -X POST "http://localhost:4151/topic/pause?topic=test_topic"

# Unpause topic
curl -X POST "http://localhost:4151/topic/unpause?topic=test_topic"

# Delete topic
curl -X POST "http://localhost:4151/topic/delete?topic=test_topic"
```

### Statistics

```bash
# Get NSQD statistics
curl "http://localhost:4151/stats"

# Get NSQLookupd statistics
curl "http://localhost:4161/nodes"

# Get NSQAdmin statistics
curl "http://localhost:4171/api/stats"
```

## Additional Resources

- [NSQ Protocol Specification](https://nsq.io/clients/tcp_protocol_spec.html)
- [NSQ HTTP API Documentation](https://nsq.io/overview/quick_start.html)
- [NSQ Client Libraries](https://nsq.io/clients/client_libraries.html)
- [NSQ Best Practices](https://nsq.io/overview/design.html)
