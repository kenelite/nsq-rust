# CORS 配置说明

## 问题描述

在开发模式下运行 nsqadmin-ui 时，前端（通常运行在 `http://localhost:3000`）试图访问后端服务（如 `http://127.0.0.1:4161`）时，会遇到 CORS（跨域资源共享）错误：

```
Access to XMLHttpRequest at 'http://127.0.0.1:4161/ping' from origin 
'http://localhost:3000' has been blocked by CORS policy: 
No 'Access-Control-Allow-Origin' header is present on the requested resource.
```

## 解决方案

为所有 NSQ 服务（nsqadmin、nsqlookupd、nsqd）添加了 CORS 支持，允许前端在开发模式下访问这些服务。

## 实现细节

### 1. 依赖更新

为以下 Cargo.toml 文件添加了 `tower-http` 依赖：

```toml
tower-http = { version = "0.5.1", features = ["cors"] }
```

- `nsqadmin/Cargo.toml`
- `nsqlookupd/Cargo.toml`
- `nsqd/Cargo.toml`

### 2. 代码更新

#### NSQAdmin (`nsqadmin/src/server.rs`)

```rust
use tower_http::{
    services::ServeDir,
    cors::{CorsLayer, Any},
};

fn create_router(self) -> Router {
    let server = Arc::new(self);
    
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    Router::new()
        // ... routes ...
        .layer(cors)
        .with_state(server)
}
```

#### NSQLookupd (`nsqlookupd/src/server.rs`)

```rust
use tower_http::cors::{CorsLayer, Any};

fn create_router(&self) -> Router {
    let server = Arc::new(self.clone());
    
    // Configure CORS to allow frontend access during development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    Router::new()
        // ... routes ...
        .layer(cors)
        .with_state(server)
}
```

#### NSQd (`nsqd/src/server.rs`)

```rust
use tower_http::cors::{CorsLayer, Any};

fn create_http_router(&self) -> Router {
    let server = self.clone();
    
    // Configure CORS to allow frontend access during development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    Router::new()
        // ... routes ...
        .layer(cors)
        .with_state(server)
}
```

## CORS 配置说明

当前配置使用了最宽松的 CORS 策略，适用于开发环境：

```rust
let cors = CorsLayer::new()
    .allow_origin(Any)      // 允许任何来源
    .allow_methods(Any)     // 允许任何 HTTP 方法
    .allow_headers(Any);    // 允许任何请求头
```

### 生产环境建议

在生产环境中，应该使用更严格的 CORS 策略：

```rust
use tower_http::cors::{CorsLayer, AllowOrigin};

let cors = CorsLayer::new()
    // 只允许特定的源
    .allow_origin([
        "https://your-domain.com".parse::<HeaderValue>().unwrap(),
        "https://admin.your-domain.com".parse::<HeaderValue>().unwrap(),
    ])
    // 只允许特定的方法
    .allow_methods([Method::GET, Method::POST])
    // 只允许特定的头
    .allow_headers([
        header::CONTENT_TYPE,
        header::AUTHORIZATION,
    ])
    // 允许凭证
    .allow_credentials(true)
    // 预检请求缓存时间
    .max_age(Duration::from_secs(3600));
```

## 使用方法

### 开发模式

1. **启动后端服务**：
```bash
# 启动所有服务
make dev

# 或分别启动
cargo run -p nsqlookupd
cargo run -p nsqd -- --lookupd-tcp-address=127.0.0.1:4160
cargo run -p nsqadmin -- --lookupd-http-addresses=127.0.0.1:4161
```

2. **启动前端开发服务器**：
```bash
cd nsqadmin-ui
npm run dev
```

3. **访问前端**：
打开浏览器访问 `http://localhost:3000`（或 Vite 显示的端口）

现在前端可以正常访问后端 API，不会再出现 CORS 错误。

### 生产模式

在生产环境中，有两种推荐的部署方式：

#### 方式 1: 使用构建后的静态文件

```bash
# 构建前端
cd nsqadmin-ui
npm run build

# 启动 nsqadmin（会自动服务于静态文件）
cargo run -p nsqadmin --release
```

访问 `http://your-server:4171`，不存在跨域问题。

#### 方式 2: 使用反向代理

使用 Nginx 作为反向代理：

```nginx
server {
    listen 80;
    server_name admin.example.com;

    # 前端静态文件
    location / {
        root /path/to/nsqadmin-ui/dist;
        try_files $uri $uri/ /index.html;
    }

    # API 代理到 nsqadmin
    location /api/ {
        proxy_pass http://127.0.0.1:4171;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # 直接代理到 nsqlookupd（可选）
    location /lookupd/ {
        proxy_pass http://127.0.0.1:4161/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # 直接代理到 nsqd（可选）
    location /nsqd/ {
        proxy_pass http://127.0.0.1:4151/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

这种方式下，所有请求都来自同一个域名，不存在跨域问题。

## 测试 CORS

### 测试命令

```bash
# 测试 nsqadmin CORS
curl -H "Origin: http://localhost:3000" \
     -H "Access-Control-Request-Method: GET" \
     -X OPTIONS \
     http://127.0.0.1:4171/api/ping -v

# 测试 nsqlookupd CORS
curl -H "Origin: http://localhost:3000" \
     -H "Access-Control-Request-Method: GET" \
     -X OPTIONS \
     http://127.0.0.1:4161/ping -v

# 测试 nsqd CORS
curl -H "Origin: http://localhost:3000" \
     -H "Access-Control-Request-Method: GET" \
     -X OPTIONS \
     http://127.0.0.1:4151/ping -v
```

成功的响应应该包含以下头：
```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: *
Access-Control-Allow-Headers: *
```

## 故障排除

### 问题 1: 仍然出现 CORS 错误

**解决方案**:
1. 确保服务已重新编译并重启
2. 清除浏览器缓存
3. 检查浏览器开发者工具的网络标签，确认响应头

### 问题 2: OPTIONS 请求失败

**解决方案**:
CORS 预检请求（OPTIONS）必须返回正确的头。确保：
1. 服务器正确处理 OPTIONS 请求
2. 返回适当的 CORS 头

### 问题 3: 生产环境 CORS 配置

**解决方案**:
不要在生产环境使用 `Any`，而应该配置具体的允许源：

```rust
.allow_origin([
    "https://your-domain.com".parse().unwrap()
])
```

## 安全建议

1. **开发环境**: 使用宽松的 CORS 配置（如当前实现）
2. **生产环境**: 使用严格的 CORS 配置，只允许信任的域
3. **API 密钥**: 对于敏感操作，添加 API 密钥认证
4. **HTTPS**: 在生产环境中始终使用 HTTPS
5. **反向代理**: 使用 Nginx/Traefik 等反向代理来统一域名

## 相关文件

- `nsqadmin/src/server.rs` - NSQAdmin CORS 配置
- `nsqlookupd/src/server.rs` - NSQLookupd CORS 配置
- `nsqd/src/server.rs` - NSQd CORS 配置
- `nsqadmin/Cargo.toml` - NSQAdmin 依赖
- `nsqlookupd/Cargo.toml` - NSQLookupd 依赖
- `nsqd/Cargo.toml` - NSQd 依赖

## 参考资料

- [MDN: CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS)
- [tower-http CORS 文档](https://docs.rs/tower-http/latest/tower_http/cors/)
- [Axum 中间件指南](https://docs.rs/axum/latest/axum/middleware/)

---

**更新日期**: 2025-11-02
**版本**: 1.3.0

