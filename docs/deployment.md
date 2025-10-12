# Deployment Guide

This guide covers deploying NSQ Rust in various environments.

## Table of Contents

- [Production Deployment](#production-deployment)
- [Docker Deployment](#docker-deployment)
- [Kubernetes Deployment](#kubernetes-deployment)
- [Cloud Deployment](#cloud-deployment)
- [High Availability](#high-availability)
- [Load Balancing](#load-balancing)
- [Monitoring](#monitoring)
- [Backup and Recovery](#backup-and-recovery)
- [Security](#security)
- [Performance Tuning](#performance-tuning)
- [Troubleshooting](#troubleshooting)

## Production Deployment

### System Requirements

#### Minimum Requirements

- **CPU**: 2 cores
- **Memory**: 4GB RAM
- **Storage**: 50GB SSD
- **Network**: 1Gbps

#### Recommended Requirements

- **CPU**: 4+ cores
- **Memory**: 8GB+ RAM
- **Storage**: 100GB+ SSD
- **Network**: 10Gbps

### Installation

#### From Source

```bash
# Clone repository
git clone https://github.com/kenelite/nsq-rust.git
cd nsq-rust

# Build release binaries
cargo build --release

# Install binaries
sudo cp target/release/nsqd /usr/local/bin/
sudo cp target/release/nsqlookupd /usr/local/bin/
sudo cp target/release/nsqadmin /usr/local/bin/
```

#### From Pre-built Binaries

```bash
# Download latest release
wget https://github.com/kenelite/nsq-rust/releases/latest/download/nsq-rust-linux-amd64.tar.gz

# Extract
tar -xzf nsq-rust-linux-amd64.tar.gz

# Install
sudo cp nsqd nsqlookupd nsqadmin /usr/local/bin/
```

### Configuration

#### NSQLookupd Configuration

Create `/etc/nsq/nsqlookupd.conf`:

```toml
[tcp]
address = "0.0.0.0:4160"

[http]
address = "0.0.0.0:4161"

[logging]
level = "info"
prefix = "[nsqlookupd] "
verbose = false
```

#### NSQD Configuration

Create `/etc/nsq/nsqd.conf`:

```toml
[tcp]
address = "0.0.0.0:4150"

[http]
address = "0.0.0.0:4151"

[lookupd]
tcp_address = "127.0.0.1:4160"
http_address = "127.0.0.1:4161"

[storage]
data_path = "/var/lib/nsqd"
mem_queue_size = 10000
disk_queue_size = 1000000

[messages]
max_memory_size = 536870912
max_body_size = 5242880
max_rdy_count = 2500

[logging]
level = "info"
prefix = "[nsqd] "
verbose = false
```

#### NSQAdmin Configuration

Create `/etc/nsq/nsqadmin.conf`:

```toml
[http]
address = "0.0.0.0:4171"

[lookupd]
http_address = "127.0.0.1:4161"

[logging]
level = "info"
prefix = "[nsqadmin] "
verbose = false
```

### Systemd Services

#### NSQLookupd Service

Create `/etc/systemd/system/nsqlookupd.service`:

```ini
[Unit]
Description=NSQ Lookup Daemon
After=network.target

[Service]
Type=simple
User=nsq
Group=nsq
ExecStart=/usr/local/bin/nsqlookupd --config=/etc/nsq/nsqlookupd.conf
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

#### NSQD Service

Create `/etc/systemd/system/nsqd.service`:

```ini
[Unit]
Description=NSQ Daemon
After=network.target nsqlookupd.service
Requires=nsqlookupd.service

[Service]
Type=simple
User=nsq
Group=nsq
ExecStart=/usr/local/bin/nsqd --config=/etc/nsq/nsqd.conf
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

#### NSQAdmin Service

Create `/etc/systemd/system/nsqadmin.service`:

```ini
[Unit]
Description=NSQ Admin
After=network.target nsqlookupd.service
Requires=nsqlookupd.service

[Service]
Type=simple
User=nsq
Group=nsq
ExecStart=/usr/local/bin/nsqadmin --config=/etc/nsq/nsqadmin.conf
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

### User and Directory Setup

```bash
# Create nsq user
sudo useradd -r -s /bin/false nsq

# Create directories
sudo mkdir -p /var/lib/nsqd
sudo mkdir -p /var/log/nsq
sudo mkdir -p /etc/nsq

# Set permissions
sudo chown -R nsq:nsq /var/lib/nsqd
sudo chown -R nsq:nsq /var/log/nsq
sudo chown -R nsq:nsq /etc/nsq

# Enable services
sudo systemctl enable nsqlookupd
sudo systemctl enable nsqd
sudo systemctl enable nsqadmin

# Start services
sudo systemctl start nsqlookupd
sudo systemctl start nsqd
sudo systemctl start nsqadmin
```

## Docker Deployment

### Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  nsqlookupd:
    image: nsq-rust:latest
    command: nsqlookupd --tcp-address=0.0.0.0:4160 --http-address=0.0.0.0:4161
    ports:
      - "4160:4160"
      - "4161:4161"
    networks:
      - nsq-network
    restart: unless-stopped

  nsqd:
    image: nsq-rust:latest
    command: nsqd --tcp-address=0.0.0.0:4150 --http-address=0.0.0.0:4151 --lookupd-tcp-address=nsqlookupd:4160 --lookupd-http-address=nsqlookupd:4161
    ports:
      - "4150:4150"
      - "4151:4151"
    volumes:
      - nsqd-data:/var/lib/nsqd
    networks:
      - nsq-network
    depends_on:
      - nsqlookupd
    restart: unless-stopped

  nsqadmin:
    image: nsq-rust:latest
    command: nsqadmin --http-address=0.0.0.0:4171 --lookupd-http-address=nsqlookupd:4161
    ports:
      - "4171:4171"
    networks:
      - nsq-network
    depends_on:
      - nsqlookupd
    restart: unless-stopped

volumes:
  nsqd-data:

networks:
  nsq-network:
    driver: bridge
```

### Dockerfile

Create `Dockerfile`:

```dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/nsqd /usr/local/bin/
COPY --from=builder /app/target/release/nsqlookupd /usr/local/bin/
COPY --from=builder /app/target/release/nsqadmin /usr/local/bin/

RUN useradd -r -s /bin/false nsq
RUN mkdir -p /var/lib/nsqd && chown nsq:nsq /var/lib/nsqd

USER nsq
EXPOSE 4150 4151 4160 4161 4171

CMD ["nsqd"]
```

### Build and Run

```bash
# Build image
docker build -t nsq-rust:latest .

# Run with docker-compose
docker-compose up -d

# Check status
docker-compose ps
```

## Kubernetes Deployment

### Namespace

Create `k8s/namespace.yaml`:

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: nsq
```

### ConfigMaps

Create `k8s/configmap.yaml`:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: nsq-config
  namespace: nsq
data:
  nsqlookupd.conf: |
    [tcp]
    address = "0.0.0.0:4160"
    
    [http]
    address = "0.0.0.0:4161"
    
    [logging]
    level = "info"
    prefix = "[nsqlookupd] "
    verbose = false

  nsqd.conf: |
    [tcp]
    address = "0.0.0.0:4150"
    
    [http]
    address = "0.0.0.0:4151"
    
    [lookupd]
    tcp_address = "nsqlookupd:4160"
    http_address = "nsqlookupd:4161"
    
    [storage]
    data_path = "/var/lib/nsqd"
    mem_queue_size = 10000
    disk_queue_size = 1000000
    
    [messages]
    max_memory_size = 536870912
    max_body_size = 5242880
    max_rdy_count = 2500
    
    [logging]
    level = "info"
    prefix = "[nsqd] "
    verbose = false

  nsqadmin.conf: |
    [http]
    address = "0.0.0.0:4171"
    
    [lookupd]
    http_address = "nsqlookupd:4161"
    
    [logging]
    level = "info"
    prefix = "[nsqadmin] "
    verbose = false
```

### NSQLookupd Deployment

Create `k8s/nsqlookupd.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nsqlookupd
  namespace: nsq
spec:
  replicas: 3
  selector:
    match Willabels:
      app: nsqlookupd
  template:
    metadata:
      labels:
        app: nsqlookupd
    spec:
      containers:
      - name: nsqlookupd
        image: nsq-rust:latest
        command: ["nsqlookupd"]
        args: ["--config=/etc/nsq/nsqlookupd.conf"]
        ports:
        - containerPort: 4160
          name: tcp
        - containerPort: 4161
          name: http
        volumeMounts:
        - name: config
          mountPath: /etc/nsq
        resources:
          requests:
            memory: "256Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /ping
            port: 4161
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ping
            port: 4161
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: nsq-config
---
apiVersion: v1
kind: Service
metadata:
  name: nsqlookupd
  namespace: nsq
spec:
  selector:
    app: nsqlookupd
  ports:
  - name: tcp
    port: 4160
    targetPort: 4160
  - name: http
    port: 4161
    targetPort: 4161
  type: ClusterIP
```

### NSQD Deployment

Create `k8s/nsqd.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nsqd
  namespace: nsq
spec:
  replicas: 3
  selector:
    matchLabels:
      app: nsqd
  template:
    metadata:
      labels:
        app: nsqd
    spec:
      containers:
      - name: nsqd
        image: nsq-rust:latest
        command: ["nsqd"]
        args: ["--config=/etc/nsq/nsqd.conf"]
        ports:
        - containerPort: 4150
          name: tcp
        - containerPort: 4151
          name: http
        volumeMounts:
        - name: config
          mountPath: /etc/nsq
        - name: data
          mountPath: /var/lib/nsqd
        resources:
          requests:
            memory: "512Mi"
            cpu: "200m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /ping
            port: 4151
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ping
            port: 4151
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: nsq-config
      - name: data
        persistentVolumeClaim:
          claimName: nsqd-data
---
apiVersion: v1
kind: Service
metadata:
  name: nsqd
  namespace: nsq
spec:
  selector:
    app: nsqd
  ports:
  - name: tcp
    port: 4150
    targetPort: 4150
  - name: http
    port: 4151
    targetPort: 4151
  type: ClusterIP
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: nsqd-data
  namespace: nsq
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 100Gi
```

### NSQAdmin Deployment

Create `k8s/nsqadmin.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nsqadmin
  namespace: nsq
spec:
  replicas: 2
  selector:
    matchLabels:
      app: nsqadmin
  template:
    metadata:
      labels:
        app: nsqadmin
    spec:
      containers:
      - name: nsqadmin
        image: nsq-rust:latest
        command: ["nsqadmin"]
        args: ["--config=/etc/nsq/nsqadmin.conf"]
        ports:
        - containerPort: 4171
          name: http
        volumeMounts:
        - name: config
          mountPath: /etc/nsq
        resources:
          requests:
            memory: "256Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /ping
            port: 4171
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ping
            port: 4171
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: nsq-config
---
apiVersion: v1
kind: Service
metadata:
  name: nsqadmin
  namespace: nsq
spec:
  selector:
    app: nsqadmin
  ports:
  - name: http
    port: 4171
    targetPort: 4171
  type: LoadBalancer
```

### Ingress

Create `k8s/ingress.yaml`:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: nsq-ingress
  namespace: nsq
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  rules:
  - host: nsq.example.com
    http:
      paths:
      - path: /admin
        pathType: Prefix
        backend:
          service:
            name: nsqadmin
            port:
              number: 4171
      - path: /lookupd
        pathType: Prefix
        backend:
          service:
            name: nsqlookupd
            port:
              number: 4161
```

### Deploy to Kubernetes

```bash
# Apply configurations
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/nsqlookupd.yaml
kubectl apply -f k8s/nsqd.yaml
kubectl apply -f k8s/nsqadmin.yaml
kubectl apply -f k8s/ingress.yaml

# Check status
kubectl get pods -n nsq
kubectl get services -n nsq
kubectl get ingress -n nsq
```

## Cloud Deployment

### AWS Deployment

#### EC2 Instance

```bash
# Launch EC2 instance
aws ec2 run-instances \
    --image-id ami-0c02fb55956c7d316 \
    --instance-type t3.medium \
    --key-name your-key \
    --security-group-ids sg-12345678 \
    --subnet-id subnet-12345678 \
    --user-data file://user-data.sh
```

#### User Data Script

Create `user-data.sh`:

```bash
#!/bin/bash
yum update -y
yum install -y docker
systemctl start docker
systemctl enable docker

# Install NSQ Rust
wget https://github.com/kenelite/nsq-rust/releases/latest/download/nsq-rust-linux-amd64.tar.gz
tar -xzf nsq-rust-linux-amd64.tar.gz
cp nsqd nsqlookupd nsqadmin /usr/local/bin/

# Create configuration
mkdir -p /etc/nsq
cat > /etc/nsq/nsqlookupd.conf << EOF
[tcp]
address = "0.0.0.0:4160"

[http]
address = "0.0.0.0:4161"

[logging]
level = "info"
prefix = "[nsqlookupd] "
verbose = false
EOF

# Start services
nsqlookupd --config=/etc/nsq/nsqlookupd.conf &
nsqd --config=/etc/nsq/nsqd.conf &
nsqadmin --config=/etc/nsq/nsqadmin.conf &
```

#### ECS Task Definition

Create `ecs-task-definition.json`:

```json
{
  "family": "nsq-rust",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "512",
  "memory": "1024",
  "executionRoleArn": "arn:aws:iam::123456789012:role/ecsTaskExecutionRole",
  "containerDefinitions": [
    {
      "name": "nsqlookupd",
      "image": "nsq-rust:latest",
      "command": ["nsqlookupd", "--tcp-address=0.0.0.0:4160", "--http-address=0.0.0.0:4161"],
      "portMappings": [
        {
          "containerPort": 4160,
          "protocol": "tcp"
        },
        {
          "containerPort": 4161,
          "protocol": "tcp"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/nsq-rust",
          "awslogs-region": "us-west-2",
          "awslogs-stream-prefix": "nsqlookupd"
        }
      }
    }
  ]
}
```

### Google Cloud Deployment

#### GKE Cluster

```bash
# Create GKE cluster
gcloud container clusters create nsq-cluster \
    --zone=us-central1-a \
    --num-nodes=3 \
    --machine-type=e2-medium \
    --enable-autoscaling \
    --min-nodes=1 \
    --max-nodes=10

# Get credentials
gcloud container clusters get-credentials nsq-cluster --zone=us-central1-a
```

#### Cloud Run

Create `cloudrun.yaml`:

```yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: nsqadmin
  annotations:
    run.googleapis.com/ingress: all
spec:
  template:
    metadata:
      annotations:
        autoscaling.knative.dev/maxScale: "10"
    spec:
      containers:
      - image: gcr.io/your-project/nsq-rust:latest
        command: ["nsqadmin"]
        args: ["--http-address=0.0.0.0:8080", "--lookupd-http-address=nsqlookupd:4161"]
        ports:
        - containerPort: 8080
        resources:
          limits:
            cpu: "1000m"
            memory: "512Mi"
```

### Azure Deployment

#### AKS Cluster

```bash
# Create resource group
az group create --name nsq-rg --location eastus

# Create AKS cluster
az aks create \
    --resource-group nsq-rg \
    --name nsq-cluster \
    --node-count 3 \
    --enable-addons monitoring \
    --generate-ssh-keys

# Get credentials
az aks get-credentials --resource-group nsq-rg --name nsq-cluster
```

## High Availability

### NSQLookupd HA

Deploy multiple NSQLookupd instances:

```yaml
# nsqlookupd-ha.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nsqlookupd
  namespace: nsq
spec:
  replicas: 3
  selector:
    matchLabels:
      app: nsqlookupd
  template:
    metadata:
      labels:
        app: nsqlookupd
    spec:
      containers:
      - name: nsqlookupd
        image: nsq-rust:latest
        command: ["nsqlookupd"]
        args: ["--tcp-address=0.0.0.0:4160", "--http-address=0.0.0.0:4161"]
        ports:
        - containerPort: 4160
          name: tcp
        - containerPort: 4161
          name: http
        resources:
          requests:
            memory: "256Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /ping
            port: 4161
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ping
            port: 4161
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: nsqlookupd
  namespace: nsq
spec:
  selector:
    app: nsqlookupd
  ports:
  - name: tcp
    port: 4160
    targetPort: 4160
  - name: http
    port: 4161
    targetPort: 4161
  type: ClusterIP
```

### NSQD HA

Deploy multiple NSQD instances:

```yaml
# nsqd-ha.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nsqd
  namespace: nsq
spec:
  replicas: 3
  selector:
    matchLabels:
      app: nsqd
  template:
    metadata:
      labels:
        app: nsqd
    spec:
      containers:
      - name: nsqd
        image: nsq-rust:latest
        command: ["nsqd"]
        args: ["--tcp-address=0.0.0.0:4150", "--http-address=0.0.0.0:4151", "--lookupd-tcp-address=nsqlookupd:4160", "--lookupd-http-address=nsqlookupd:4161"]
        ports:
        - containerPort: 4150
          name: tcp
        - containerPort: 4151
          name: http
        volumeMounts:
        - name: data
          mountPath: /var/lib/nsqd
        resources:
          requests:
            memory: "512Mi"
            cpu: "200m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /ping
            port: 4151
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ping
            port: 4151
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: nsqd-data
---
apiVersion: v1
kind: Service
metadata:
  name: nsqd
  namespace: nsq
spec:
  selector:
    app: nsqd
  ports:
  - name: tcp
    port: 4150
    targetPort: 4150
  - name: http
    port: 4151
    targetPort: 4151
  type: ClusterIP
```

## Load Balancing

### NGINX Load Balancer

Create `nginx.conf`:

```nginx
upstream nsqlookupd {
    server nsqlookupd-1:4161;
    server nsqlookupd-2:4161;
    server nsqlookupd-3:4161;
}

upstream nsqd {
    server nsqd-1:4151;
    server nsqd-2:4151;
    server nsqd-3:4151;
}

upstream nsqadmin {
    server nsqadmin-1:4171;
    server nsqadmin-2:4171;
}

server {
    listen 80;
    server_name nsq.example.com;

    location /lookupd/ {
        proxy_pass http://nsqlookupd/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /nsqd/ {
        proxy_pass http://nsqd/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /admin/ {
        proxy_pass http://nsqadmin/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### HAProxy Load Balancer

Create `haproxy.cfg`:

```haproxy
global
    daemon
    maxconn 4096

defaults
    mode http
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms

frontend nsq_frontend
    bind *:80
    default_backend nsq_backend

backend nsq_backend
    balance roundrobin
    server nsqd1 nsqd-1:4151 check
    server nsqd2 nsqd-2:4151 check
    server nsqd3 nsqd-3:4151 check

frontend nsqlookupd_frontend
    bind *:4161
    default_backend nsqlookupd_backend

backend nsqlookupd_backend
    balance roundrobin
    server nsqlookupd1 nsqlookupd-1:4161 check
    server nsqlookupd2 nsqlookupd-2:4161 check
    server nsqlookupd3 nsqlookupd-3:4161 check

frontend nsqadmin_frontend
    bind *:4171
    default_backend nsqadmin_backend

backend nsqadmin_backend
    balance roundrobin
    server nsqadmin1 nsqadmin-1:4171 check
    server nsqadmin2 nsqadmin-2:4171 check
```

## Monitoring

### Prometheus Configuration

Create `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'nsqd'
    static_configs:
      - targets: ['nsqd-1:4151', 'nsqd-2:4151', 'nsqd-3:4151']
    metrics_path: '/metrics'
    scrape_interval: 5s

  - job_name: 'nsqlookupd'
    static_configs:
      - targets: ['nsqlookupd-1:4161', 'nsqlookupd-2:4161', 'nsqlookupd-3:4161']
    metrics_path: '/metrics'
    scrape_interval: 5s

  - job_name: 'nsqadmin'
    static_configs:
      - targets: ['nsqadmin-1:4171', 'nsqadmin-2:4171']
    metrics_path: '/metrics'
    scrape_interval: 5s
```

### Grafana Dashboard

Create `grafana-dashboard.json`:

```json
{
  "dashboard": {
    "id": null,
    "title": "NSQ Rust Dashboard",
    "tags": ["nsq", "rust"],
    "timezone": "browser",
    "panels": [
      {
        "id": 1,
        "title": "Message Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(nsq_messages_published_total[5m])",
            "legendFormat": "Published"
          },
          {
            "expr": "rate(nsq_messages_consumed_total[5m])",
            "legendFormat": "Consumed"
          }
        ]
      },
      {
        "id": 2,
        "title": "Queue Depth",
        "type": "graph",
        "targets": [
          {
            "expr": "nsq_queue_depth",
            "legendFormat": "{{topic}}.{{channel}}"
          }
        ]
      },
      {
        "id": 3,
        "title": "Client Connections",
        "type": "graph",
        "targets": [
          {
            "expr": "nsq_clients_connected",
            "legendFormat": "{{topic}}.{{channel}}"
          }
        ]
      }
    ]
  }
}
```

### Alerting Rules

Create `alerts.yml`:

```yaml
groups:
- name: nsq.rules
  rules:
  - alert: NSQHighQueueDepth
    expr: nsq_queue_depth > 10000
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "NSQ queue depth is high"
      description: "Queue depth for {{topic}}.{{channel}} is {{$value}}"

  - alert: NSQDown
    expr: up{job=~"nsqd.*"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "NSQD instance is down"
      description: "NSQD instance {{instance}} is down"

  - alert: NSQLookupdDown
    expr: up{job=~"nsqlookupd.*"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "NSQLookupd instance is down"
      description: "NSQLookupd instance {{instance}} is down"
```

## Backup and Recovery

### Data Backup

#### Manual Backup

```bash
# Stop NSQD
sudo systemctl stop nsqd

# Backup data directory
sudo tar -czf nsqd-backup-$(date +%Y%m%d).tar.gz /var/lib/nsqd

# Start NSQD
sudo systemctl start nsqd
```

#### Automated Backup

Create `backup.sh`:

```bash
#!/bin/bash

BACKUP_DIR="/backup/nsq"
DATA_DIR="/var/lib/nsqd"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p $BACKUP_DIR

# Stop NSQD
systemctl stop nsqd

# Backup data
tar -czf $BACKUP_DIR/nsqd-backup-$DATE.tar.gz $DATA_DIR

# Start NSQD
systemctl start nsqd

# Clean old backups (keep last 7 days)
find $BACKUP_DIR -name "nsqd-backup-*.tar.gz" -mtime +7 -delete

echo "Backup completed: $BACKUP_DIR/nsqd-backup-$DATE.tar.gz"
```

#### Cron Job

```bash
# Add to crontab
0 2 * * * /usr/local/bin/backup.sh
```

### Data Recovery

#### Restore from Backup

```bash
# Stop NSQD
sudo systemctl stop nsqd

# Restore data
sudo tar -xzf nsqd-backup-20240101.tar.gz -C /

# Start NSQD
sudo systemctl start nsqd
```

#### Point-in-Time Recovery

```bash
# Stop NSQD
sudo systemctl stop nsqd

# Restore from specific backup
sudo tar -xzf nsqd-backup-20240101_020000.tar.gz -C /

# Start NSQD
sudo systemctl start nsqd
```

## Security

### TLS Configuration

#### Generate Certificates

```bash
# Generate CA
openssl genrsa -out ca-key.pem 4096
openssl req -new -x509 -days 365 -key ca-key.pem -out ca.pem

# Generate server certificate
openssl genrsa -out server-key.pem 4096
openssl req -new -key server-key.pem -out server.csr
openssl x509 -req -days 365 -in server.csr -CA ca.pem -CAkey ca-key.pem -out server.pem
```

#### NSQD TLS Configuration

```toml
[tls]
cert_file = "/etc/nsq/server.pem"
key_file = "/etc/nsq/server-key.pem"
client_auth_policy = "require"
min_version = "1.2"
max_version = "1.3"
```

### Authentication

#### HTTP Authentication

```toml
[http]
auth_user = "admin"
auth_password = "secret"
auth_token = "your-token-here"
```

#### TCP Authentication

```toml
[tcp]
auth_secret = "your-secret-here"
```

### Network Security

#### Firewall Rules

```bash
# Allow NSQD ports
ufw allow 4150/tcp
ufw allow 4151/tcp

# Allow NSQLookupd ports
ufw allow 4160/tcp
ufw allow 4161/tcp

# Allow NSQAdmin ports
ufw allow 4171/tcp

# Deny all other traffic
ufw default deny incoming
ufw default allow outgoing
```

#### VPN Configuration

```bash
# OpenVPN server configuration
port 1194
proto udp
dev tun
ca ca.crt
cert server.crt
key server.key
dh dh2048.pem
server 10.8.0.0 255.255.255.0
push "route 192.168.1.0 255.255.255.0"
client-to-client
duplicate-cn
keepalive 10 120
comp-lzo
persist-key
persist-tun
status openvpn-status.log
verb 3
```

## Performance Tuning

### System Tuning

#### Kernel Parameters

```bash
# Add to /etc/sysctl.conf
net.core.rmem_max = 134217728
net.core.wmem_max = 134217728
net.ipv4.tcp_rmem = 4096 65536 134217728
net.ipv4.tcp_wmem = 4096 65536 134217728
net.core.netdev_max_backlog = 5000
net.ipv4.tcp_congestion_control = bbr
```

#### File Descriptors

```bash
# Add to /etc/security/limits.conf
nsq soft nofile 65536
nsq hard nofile 65536
```

### Application Tuning

#### Memory Configuration

```toml
[messages]
max_memory_size = 1073741824  # 1GB
mem_queue_size = 20000

[storage]
disk_queue_size = 2000000
sync_every = 5000
```

#### Network Configuration

```toml
[messages]
max_rdy_count = 5000
max_output_buffer_size = 131072
max_output_buffer_timeout = "500ms"
max_heartbeat_interval = "30s"
```

### Benchmarking

#### Load Testing

```bash
# Install nsq tools
go install github.com/nsqio/go-nsq@latest

# Publish messages
nsq_pub --topic=test_topic --rate=1000 --count=10000

# Consume messages
nsq_sub --topic=test_topic --channel=test_channel
```

#### Performance Monitoring

```bash
# Monitor system resources
htop
iotop
nethogs

# Monitor NSQ metrics
curl http://localhost:4151/stats
curl http://localhost:4161/nodes
```

## Troubleshooting

### Common Issues

#### High Memory Usage

**Symptoms:**
- High memory consumption
- Slow performance
- Out of memory errors

**Solutions:**
```bash
# Check memory usage
free -h
ps aux --sort=-%mem | head

# Adjust memory settings
# In nsqd.conf
max_memory_size = 536870912  # 512MB
mem_queue_size = 10000
```

#### Connection Issues

**Symptoms:**
- Connection refused errors
- Timeout errors
- Network unreachable

**Solutions:**
```bash
# Check network connectivity
ping nsqlookupd
telnet nsqlookupd 4160
telnet nsqlookupd 4161

# Check firewall
ufw status
iptables -L

# Check DNS resolution
nslookup nsqlookupd
```

#### Performance Issues

**Symptoms:**
- Slow message processing
- High latency
- Low throughput

**Solutions:**
```bash
# Check system resources
top
iostat -x 1
netstat -i

# Check NSQ configuration
nsqd --config=nsqd.conf --validate

# Monitor metrics
curl http://localhost:4151/stats
```

### Log Analysis

#### Log Files

```bash
# NSQD logs
journalctl -u nsqd -f

# NSQLookupd logs
journalctl -u nsqlookupd -f

# NSQAdmin logs
journalctl -u nsqadmin -f
```

#### Log Filtering

```bash
# Filter by level
journalctl -u nsqd --since "1 hour ago" | grep ERROR

# Filter by component
journalctl -u nsqd --since "1 hour ago" | grep "topic"

# Filter by time range
journalctl -u nsqd --since "2024-01-01" --until "2024-01-02"
```

### Debugging

#### Enable Debug Logging

```toml
[logging]
level = "debug"
verbose = true
```

#### Debug Mode

```bash
# Run in debug mode
RUST_LOG=debug nsqd --config=nsqd.conf

# Debug specific components
RUST_LOG=nsqd::topic=debug nsqd --config=nsqd.conf
```

#### Profiling

```bash
# CPU profiling
perf record -p $(pgrep nsqd)
perf report

# Memory profiling
valgrind --tool=massif nsqd --config=nsqd.conf
```

### Recovery Procedures

#### Service Recovery

```bash
# Restart services
sudo systemctl restart nsqlookupd
sudo systemctl restart nsqd
sudo systemctl restart nsqadmin

# Check status
sudo systemctl status nsqlookupd
sudo systemctl status nsqd
sudo systemctl status nsqadmin
```

#### Data Recovery

```bash
# Stop services
sudo systemctl stop nsqd

# Restore from backup
sudo tar -xzf nsqd-backup-20240101.tar.gz -C /

# Start services
sudo systemctl start nsqd
```

#### Cluster Recovery

```bash
# Check cluster health
curl http://localhost:4161/nodes
curl http://localhost:4151/stats

# Restart failed nodes
kubectl delete pod nsqd-1
kubectl delete pod nsqlookupd-1

# Verify recovery
kubectl get pods -n nsq
```

## Additional Resources

- [NSQ Documentation](https://nsq.io/)
- [Rust Documentation](https://doc.rust-lang.org/)
- [Docker Documentation](https://docs.docker.com/)
- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Documentation](https://grafana.com/docs/)
