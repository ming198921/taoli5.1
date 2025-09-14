# 5.1套利系统部署指南

本指南介绍如何在不同环境中部署和控制5.1套利系统。

## 支持的部署环境

### 1. 直接进程控制 (Direct)
- **适用场景**: 开发环境、简单部署
- **特点**: 前端直接控制后端进程启停
- **配置**: `REACT_APP_DEPLOYMENT_TYPE=direct`

### 2. Systemd服务管理 (推荐生产环境)
- **适用场景**: Linux生产服务器、AWS EC2
- **特点**: 系统级服务管理、自动重启、日志管理
- **配置**: `REACT_APP_DEPLOYMENT_TYPE=systemd`

### 3. AWS ECS容器服务
- **适用场景**: AWS云环境、容器化部署
- **特点**: 自动扩缩容、负载均衡、服务发现
- **配置**: `REACT_APP_DEPLOYMENT_TYPE=ecs`

### 4. Kubernetes集群
- **适用场景**: 大规模分布式部署
- **特点**: 高可用、自动故障转移、滚动更新
- **配置**: `REACT_APP_DEPLOYMENT_TYPE=k8s`

## 快速开始

### 方案1: Systemd部署（推荐）

#### 1. 编译项目
```bash
cd /home/ubuntu/5.1xitong/5.1系统
cargo build --release
```

#### 2. 运行部署脚本
```bash
./scripts/deploy-systemd.sh
```

#### 3. 配置前端
```bash
cd /home/ubuntu/arbitrage-frontend-v5.1
cp .env.example .env.local
```

编辑 `.env.local`:
```env
REACT_APP_DEPLOYMENT_TYPE=systemd
REACT_APP_API_URL=http://localhost:8080
```

#### 4. 启动前端
```bash
npm run dev
```

### 方案2: AWS ECS部署

#### 1. 创建Docker镜像
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/arbitrage-system /usr/local/bin/
EXPOSE 8080
CMD ["arbitrage-system"]
```

#### 2. 部署到ECS
- 创建ECR仓库
- 构建并推送镜像
- 创建ECS集群和服务
- 配置负载均衡器

#### 3. 配置前端
```env
REACT_APP_DEPLOYMENT_TYPE=ecs
REACT_APP_ECS_CLUSTER=arbitrage-cluster
REACT_APP_API_URL=https://your-alb-url
```

### 方案3: Kubernetes部署

#### 1. 创建Kubernetes配置
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: arbitrage-system
spec:
  replicas: 3
  selector:
    matchLabels:
      app: arbitrage
  template:
    metadata:
      labels:
        app: arbitrage
    spec:
      containers:
      - name: arbitrage
        image: your-registry/arbitrage-system:latest
        ports:
        - containerPort: 8080
---
apiVersion: v1
kind: Service
metadata:
  name: arbitrage-service
spec:
  selector:
    app: arbitrage
  ports:
    - protocol: TCP
      port: 80
      targetPort: 8080
  type: LoadBalancer
```

#### 2. 部署到集群
```bash
kubectl apply -f k8s-deployment.yaml
```

#### 3. 配置前端
```env
REACT_APP_DEPLOYMENT_TYPE=k8s
REACT_APP_K8S_NAMESPACE=default
REACT_APP_API_URL=https://your-k8s-ingress
```

## 环境变量配置

### 前端环境变量 (.env.local)
```env
# 必需配置
REACT_APP_API_URL=http://localhost:8080
REACT_APP_DEPLOYMENT_TYPE=systemd

# AWS ECS配置
REACT_APP_ECS_CLUSTER=arbitrage-cluster
REACT_APP_AWS_REGION=us-west-2

# Kubernetes配置
REACT_APP_K8S_NAMESPACE=arbitrage
REACT_APP_K8S_CONTEXT=default

# 功能开关
REACT_APP_ENABLE_LOGS=true
REACT_APP_ENABLE_METRICS=true
```

## API端点说明

### 通用控制API
- `POST /api/system/start` - 启动系统
- `POST /api/system/stop` - 停止系统
- `GET /api/system/status` - 系统状态

### Systemd特有API
- `POST /api/control/systemd/start` - 启动systemd服务
- `POST /api/control/systemd/stop` - 停止systemd服务
- `POST /api/control/systemd/restart` - 重启systemd服务
- `GET /api/control/systemd/status` - 服务状态
- `GET /api/control/systemd/logs` - 服务日志

### ECS特有API
- `PUT /api/control/ecs/services` - 扩缩容ECS服务
- `POST /api/control/ecs/restart` - 重启ECS服务
- `GET /api/control/ecs/services/{cluster}/{service}` - 服务状态

### K8s特有API
- `POST /api/control/k8s/scale` - 扩缩容部署
- `POST /api/control/k8s/rollout-restart` - 滚动重启
- `GET /api/control/k8s/deployments/{namespace}/{name}` - 部署状态

## 常用运维命令

### Systemd环境
```bash
# 查看服务状态
sudo systemctl status arbitrage-system

# 启动/停止/重启服务
sudo systemctl start arbitrage-system
sudo systemctl stop arbitrage-system
sudo systemctl restart arbitrage-system

# 查看日志
sudo journalctl -u arbitrage-system -f

# 启用/禁用开机启动
sudo systemctl enable arbitrage-system
sudo systemctl disable arbitrage-system
```

### ECS环境
```bash
# 使用AWS CLI管理
aws ecs describe-services --cluster arbitrage-cluster --services arbitrage-system
aws ecs update-service --cluster arbitrage-cluster --service arbitrage-system --desired-count 2
```

### K8s环境
```bash
# 查看部署状态
kubectl get deployments
kubectl describe deployment arbitrage-system

# 扩缩容
kubectl scale deployment arbitrage-system --replicas=3

# 滚动更新
kubectl rollout restart deployment/arbitrage-system

# 查看日志
kubectl logs -f deployment/arbitrage-system
```

## 故障排查

### 常见问题

1. **服务无法启动**
   - 检查端口是否被占用: `sudo lsof -i :8080`
   - 查看服务日志: `sudo journalctl -u arbitrage-system -n 50`
   - 检查文件权限和路径

2. **前端无法连接后端**
   - 确认API_URL配置正确
   - 检查防火墙设置
   - 验证CORS配置

3. **权限问题**
   - 确保sudo权限配置正确
   - 检查文件所有权: `ls -la /home/ubuntu/5.1xitong/`

### 日志位置

- **Systemd**: `sudo journalctl -u arbitrage-system`
- **直接运行**: 标准输出/错误
- **ECS**: CloudWatch Logs
- **K8s**: `kubectl logs`

## 性能调优

### Systemd服务配置
- 调整资源限制 (`LimitNOFILE`, `LimitNPROC`)
- 配置内存和CPU限制
- 优化重启策略

### 容器配置
- 设置合适的资源请求和限制
- 配置健康检查
- 优化镜像大小

## 安全考虑

- 使用非特权用户运行服务
- 启用systemd安全特性
- 配置网络隔离
- 定期更新依赖项
- 使用HTTPS和TLS加密

## 监控和告警

- 集成Prometheus指标收集
- 配置Grafana仪表板
- 设置告警规则
- 日志聚合和分析