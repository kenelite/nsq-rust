# NSQLookupd - NSQ Service Discovery Daemon

NSQLookupd 是 NSQ 分布式消息队列系统的服务发现组件，负责管理 nsqd 节点的注册、发现和健康检查。

## 功能特性

### 核心功能
- **服务发现**: 管理 nsqd 节点的注册和发现
- **健康检查**: 监控生产者节点的健康状态
- **主题管理**: 跟踪和管理主题及其通道
- **心跳监控**: 处理生产者的心跳更新
- **墓碑机制**: 支持生产者节点的墓碑标记

### 协议支持
- **TCP 协议**: 支持 nsqd 节点的 TCP 注册
- **HTTP API**: 提供完整的 REST API 接口
- **命令支持**: PING, REGISTER, UNREGISTER, IDENTIFY, VERSION, QUIT

### HTTP API 端点

#### 基础端点
- `GET /ping` - 健康检查
- `GET /info` - 服务器信息
- `GET /stats` - 详细统计信息
- `GET /health` - 健康状态检查

#### 发现端点
- `GET /lookup?topic=<topic>` - 查找主题的生产者
- `GET /lookup?topic=<topic>&channel=<channel>` - 查找主题和通道的生产者
- `GET /topics` - 列出所有主题
- `GET /channels?topic=<topic>` - 列出主题的所有通道
- `GET /nodes` - 列出所有节点

#### 管理端点
- `POST /topic/create?topic=<topic>` - 创建主题
- `POST /topic/delete?topic=<topic>` - 删除主题
- `POST /channel/create?topic=<topic>&channel=<channel>` - 创建通道
- `POST /channel/delete?topic=<topic>&channel=<channel>` - 删除通道
- `POST /tombstone_topic_producer?topic=<topic>&node=<node>` - 标记生产者为墓碑

#### API 端点
- `GET /api/topics` - 获取所有主题的详细信息
- `GET /api/nodes` - 获取所有节点的详细信息
- `GET /api/topics/:topic` - 获取特定主题的详细信息

#### 调试端点
- `GET /debug/pprof/` - 调试信息（占位符）

## 配置选项

### 命令行参数
```bash
--tcp-address=0.0.0.0:4160          # TCP 监听地址
--http-address=0.0.0.1:4161         # HTTP 监听地址
--inactive-producer-timeout=300000  # 非活跃生产者超时（毫秒）
--tombstone-lifetime=45000          # 墓碑生命周期（毫秒）
--log-level=info                    # 日志级别
--log-format=text                   # 日志格式
--statsd-address=<address>          # StatsD 地址
--statsd-prefix=nsqlookupd          # StatsD 前缀
```

### 环境变量
所有配置选项也可以通过环境变量设置，前缀为 `NSQ_`。

## 使用示例

### 启动服务
```bash
nsqlookupd --tcp-address=0.0.0.0:4160 --http-address=0.0.0.0:4161
```

### TCP 协议示例
```bash
# 连接到 nsqlookupd
telnet localhost 4160

# 注册生产者
REGISTER test-topic test-channel

# 发送心跳
IDENTIFY

# 查询版本
VERSION

# 断开连接
QUIT
```

### HTTP API 示例
```bash
# 获取统计信息
curl http://localhost:4161/stats

# 查找主题的生产者
curl "http://localhost:4161/lookup?topic=test-topic"

# 创建主题
curl -X POST "http://localhost:4161/topic/create?topic=test-topic"
```

## 架构设计

### 核心组件
- **RegistrationDB**: 注册数据库，管理生产者和主题信息
- **Producer**: 生产者信息结构，包含健康状态和墓碑信息
- **NsqlookupdServer**: 主服务器，处理 TCP 和 HTTP 请求

### 数据管理
- **生产者映射**: 按 ID 快速查找生产者
- **主题映射**: 按主题组织生产者
- **通道管理**: 跟踪每个主题的通道
- **墓碑管理**: 处理生产者节点的墓碑状态

### 后台任务
- **清理任务**: 定期清理过期的生产者和墓碑
- **心跳更新**: 处理生产者的心跳更新
- **健康检查**: 监控生产者节点的健康状态

## 测试

运行基本功能测试：
```bash
cargo test --package nsqlookupd
```

测试覆盖：
- 生产者注册和注销
- 心跳更新机制
- 墓碑功能
- 通道管理
- ID 生成和地址解析

## 性能特性

- **并发处理**: 使用 Tokio 异步运行时
- **内存效率**: 使用 Arc<RwLock<>> 进行高效的数据共享
- **定期清理**: 自动清理过期数据
- **健康监控**: 实时监控生产者状态

## 兼容性

与标准 NSQ 协议完全兼容，支持：
- nsqd 节点注册
- 客户端服务发现
- 管理工具集成
- 监控系统集成
