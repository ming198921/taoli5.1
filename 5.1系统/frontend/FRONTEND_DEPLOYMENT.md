# 5.1套利系统前端部署指南

## 📋 系统概览

**前端技术栈**: React 18 + TypeScript + Ant Design + Vite  
**端口配置**: 3003 (外网可访问)  
**API网关**: localhost:3000  
**接口总数**: 387个API接口  
**微服务数**: 7个微服务  

## 🏗️ 架构图

```
┌─────────────────────────────────────────────────┐
│                前端应用                          │
│            React + TypeScript                   │
│              Port: 3003                         │
│           (外网可访问)                           │
└─────────────────┬───────────────────────────────┘
                  │ HTTP/WebSocket
┌─────────────────▼───────────────────────────────┐
│              统一API网关                         │
│            Rust + Axum                         │
│              Port: 3000                        │
└─────────────────┬───────────────────────────────┘
                  │ 智能路由
    ┌─────────────┼─────────────┐
    │             │             │
┌───▼───┐ ┌───────▼───┐ ┌───────▼───┐
│日志服务│ │清洗服务   │ │策略服务   │
│4001   │ │4002      │ │4003      │
│45 API │ │52 API    │ │38 API    │
└───────┘ └───────────┘ └───────────┘
┌───────┐ ┌───────────┐ ┌───────────┐
│性能服务│ │交易服务   │ │AI模型服务 │
│4004   │ │4005      │ │4006      │
│67 API │ │41 API    │ │48 API    │
└───────┘ └───────────┘ └───────────┘
          ┌───────────┐
          │配置服务   │
          │4007      │
          │96 API    │
          └───────────┘
```

## 🚀 快速部署

### 1. 检查环境

```bash
# 检查Node.js版本 (需要18+)
node -v

# 检查npm版本
npm -v

# 检查统一网关是否运行
curl http://localhost:3000/health
```

### 2. 安装依赖

```bash
cd /home/ubuntu/arbitrage-frontend-v5.1
npm install
```

### 3. 启动前端

```bash
# 使用启动脚本 (推荐)
./start-frontend.sh

# 或者直接启动
npm run dev
```

### 4. 访问地址

- **本地访问**: http://localhost:3003
- **外网访问**: http://YOUR_SERVER_IP:3003

## 📁 项目结构

```
arbitrage-frontend-v5.1/
├── src/
│   ├── api/                    # API客户端
│   │   └── apiClient.ts       # 统一API客户端和WebSocket管理
│   ├── services/              # 服务层 (387个API接口)
│   │   ├── loggingService.ts  # 日志服务 (45个API)
│   │   ├── cleaningService.ts # 清洗服务 (52个API)
│   │   ├── strategyService.ts # 策略服务 (38个API)
│   │   ├── performanceService.ts # 性能服务 (67个API)
│   │   ├── tradingService.ts  # 交易服务 (41个API)
│   │   ├── aiModelService.ts  # AI模型服务 (48个API)
│   │   ├── configService.ts   # 配置服务 (96个API)
│   │   ├── systemControlService.ts # 系统控制服务
│   │   └── index.ts          # 统一服务导出
│   ├── pages/                 # 页面组件
│   │   ├── Dashboard.tsx      # 主仪表板 (387个API概览)
│   │   ├── LoggingModule.tsx  # 日志管理页面
│   │   ├── CleaningModule.tsx # 清洗管理页面
│   │   ├── StrategyModule.tsx # 策略管理页面
│   │   ├── PerformanceModule.tsx # 性能管理页面
│   │   ├── TradingModule.tsx  # 交易管理页面
│   │   ├── AIModelModule.tsx  # AI模型管理页面
│   │   ├── ConfigModule.tsx   # 配置管理页面
│   │   └── SystemControl.tsx  # 系统控制页面
│   ├── App.tsx               # 主应用组件
│   └── main.tsx              # 入口文件
├── .env                      # 环境配置
├── vite.config.ts            # Vite配置
├── package.json              # 依赖配置
└── start-frontend.sh         # 启动脚本
```

## 🔧 配置说明

### 环境变量 (.env)

```bash
# API网关配置
VITE_API_BASE_URL=http://localhost:3000
VITE_WS_BASE_URL=ws://localhost:3000

# 系统信息
VITE_SYSTEM_NAME=5.1套利系统
VITE_SYSTEM_VERSION=v5.1.0
VITE_TOTAL_APIS=387
VITE_MICROSERVICES_COUNT=7
```

### Vite配置 (vite.config.ts)

```typescript
export default defineConfig({
  server: {
    port: 3003,
    host: '0.0.0.0', // 允许外网访问
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true
      },
      '/ws': {
        target: 'ws://localhost:3000',
        ws: true,
        changeOrigin: true
      }
    }
  }
})
```

## 📊 API接口分布

| 服务名称 | 端口 | API数量 | 功能描述 |
|----------|------|---------|----------|
| logging-service | 4001 | 45个 | 日志收集、分析、实时流 |
| cleaning-service | 4002 | 52个 | 数据清洗、质量控制 |
| strategy-service | 4003 | 38个 | 策略管理、热重载 |
| performance-service | 4004 | 67个 | 性能优化、资源监控 |
| trading-service | 4005 | 41个 | 订单管理、风险控制 |
| ai-model-service | 4006 | 48个 | AI模型管理、训练推理 |
| config-service | 4007 | 96个 | 配置管理、版本控制 |
| **总计** | - | **387个** | **完整套利系统** |

## 🌐 前端页面功能

### 1. 主仪表板 (`/dashboard`)
- 387个API接口状态总览
- 7个微服务健康监控
- 实时性能指标展示
- 系统状态可视化

### 2. 日志管理 (`/logging`)
- 45个日志API接口管理
- 实时日志流 (WebSocket)
- 日志级别配置
- 日志分析和搜索

### 3. 清洗管理 (`/cleaning`)
- 52个清洗API接口管理
- 清洗规则配置
- 交易所数据配置
- 数据质量监控

### 4. 策略管理 (`/strategy`)
- 38个策略API接口管理
- 策略生命周期控制
- 实时监控和调试
- 热重载功能

### 5. 性能管理 (`/performance`)
- 67个性能API接口管理
- CPU/内存/网络/磁盘监控
- 性能优化工具
- 基准测试

### 6. 交易管理 (`/trading`)
- 41个交易API接口管理
- 订单和仓位监控
- 资金管理
- 风险控制

### 7. AI模型管理 (`/ai-model`)
- 48个AI模型API接口管理
- 模型训练和部署
- 推理服务监控
- 特征工程工具

### 8. 配置管理 (`/config`)
- 96个配置API接口管理
- 版本控制
- 热重载管理
- 环境配置

### 9. 系统控制 (`/system`)
- 系统启停控制
- 服务管理
- 备份恢复
- 系统诊断

## 🔗 API调用示例

### 基础API调用

```typescript
import { serviceManager } from '../services';

// 获取所有策略
const strategies = await serviceManager.strategy.listStrategies();

// 启动策略
await serviceManager.strategy.startStrategy('triangular-v4');

// 获取实时日志
const logs = await serviceManager.logging.getRealtimeLogStream();

// 获取系统状态
const status = await serviceManager.systemControl.getSystemStatus();
```

### WebSocket连接

```typescript
import { loggingService } from '../services';

// 连接实时日志流
const ws = loggingService.connectRealtimeLogs(
  (logData) => {
    console.log('New log:', logData);
  },
  (error) => {
    console.error('WebSocket error:', error);
  }
);
```

## 🛡️ 生产部署

### 1. 构建生产版本

```bash
npm run build
```

### 2. 使用Nginx部署

```nginx
server {
    listen 3003;
    server_name YOUR_DOMAIN;
    
    root /path/to/arbitrage-frontend-v5.1/dist;
    index index.html;
    
    location / {
        try_files $uri $uri/ /index.html;
    }
    
    location /api {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
    
    location /ws {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

### 3. 使用PM2部署

```bash
# 安装PM2
npm install -g pm2

# 启动前端服务
pm2 start npm --name "arbitrage-frontend" -- run preview

# 设置开机自启
pm2 startup
pm2 save
```

## 🔍 故障排除

### 1. 端口占用问题

```bash
# 检查端口占用
lsof -i :3003

# 清理端口
pkill -f "vite.*3003"
```

### 2. API连接问题

```bash
# 检查统一网关状态
curl http://localhost:3000/health

# 检查微服务状态
curl http://localhost:4001/health  # 日志服务
curl http://localhost:4002/health  # 清洗服务
# ... 其他服务
```

### 3. WebSocket连接问题

```bash
# 检查WebSocket连接
wscat -c ws://localhost:3000/ws/logs/realtime
```

## 📈 性能监控

前端集成了完整的性能监控：

- **API响应时间监控**: 所有387个API接口的响应时间
- **WebSocket连接状态**: 实时连接状态监控
- **前端性能指标**: 页面加载时间、内存使用等
- **用户操作追踪**: 用户行为分析

## 🎯 总结

5.1套利系统前端已完成：

✅ **387个API接口完整对接**  
✅ **7个微服务统一管理**  
✅ **实时WebSocket连接**  
✅ **响应式UI设计**  
✅ **生产级部署方案**  
✅ **外网访问支持**  
✅ **完整错误处理**  
✅ **性能监控集成**  

前端现在可以100%控制后端所有运行，实现真实的生产级套利系统管理！ 

## 📋 系统概览

**前端技术栈**: React 18 + TypeScript + Ant Design + Vite  
**端口配置**: 3003 (外网可访问)  
**API网关**: localhost:3000  
**接口总数**: 387个API接口  
**微服务数**: 7个微服务  

## 🏗️ 架构图

```
┌─────────────────────────────────────────────────┐
│                前端应用                          │
│            React + TypeScript                   │
│              Port: 3003                         │
│           (外网可访问)                           │
└─────────────────┬───────────────────────────────┘
                  │ HTTP/WebSocket
┌─────────────────▼───────────────────────────────┐
│              统一API网关                         │
│            Rust + Axum                         │
│              Port: 3000                        │
└─────────────────┬───────────────────────────────┘
                  │ 智能路由
    ┌─────────────┼─────────────┐
    │             │             │
┌───▼───┐ ┌───────▼───┐ ┌───────▼───┐
│日志服务│ │清洗服务   │ │策略服务   │
│4001   │ │4002      │ │4003      │
│45 API │ │52 API    │ │38 API    │
└───────┘ └───────────┘ └───────────┘
┌───────┐ ┌───────────┐ ┌───────────┐
│性能服务│ │交易服务   │ │AI模型服务 │
│4004   │ │4005      │ │4006      │
│67 API │ │41 API    │ │48 API    │
└───────┘ └───────────┘ └───────────┘
          ┌───────────┐
          │配置服务   │
          │4007      │
          │96 API    │
          └───────────┘
```

## 🚀 快速部署

### 1. 检查环境

```bash
# 检查Node.js版本 (需要18+)
node -v

# 检查npm版本
npm -v

# 检查统一网关是否运行
curl http://localhost:3000/health
```

### 2. 安装依赖

```bash
cd /home/ubuntu/arbitrage-frontend-v5.1
npm install
```

### 3. 启动前端

```bash
# 使用启动脚本 (推荐)
./start-frontend.sh

# 或者直接启动
npm run dev
```

### 4. 访问地址

- **本地访问**: http://localhost:3003
- **外网访问**: http://YOUR_SERVER_IP:3003

## 📁 项目结构

```
arbitrage-frontend-v5.1/
├── src/
│   ├── api/                    # API客户端
│   │   └── apiClient.ts       # 统一API客户端和WebSocket管理
│   ├── services/              # 服务层 (387个API接口)
│   │   ├── loggingService.ts  # 日志服务 (45个API)
│   │   ├── cleaningService.ts # 清洗服务 (52个API)
│   │   ├── strategyService.ts # 策略服务 (38个API)
│   │   ├── performanceService.ts # 性能服务 (67个API)
│   │   ├── tradingService.ts  # 交易服务 (41个API)
│   │   ├── aiModelService.ts  # AI模型服务 (48个API)
│   │   ├── configService.ts   # 配置服务 (96个API)
│   │   ├── systemControlService.ts # 系统控制服务
│   │   └── index.ts          # 统一服务导出
│   ├── pages/                 # 页面组件
│   │   ├── Dashboard.tsx      # 主仪表板 (387个API概览)
│   │   ├── LoggingModule.tsx  # 日志管理页面
│   │   ├── CleaningModule.tsx # 清洗管理页面
│   │   ├── StrategyModule.tsx # 策略管理页面
│   │   ├── PerformanceModule.tsx # 性能管理页面
│   │   ├── TradingModule.tsx  # 交易管理页面
│   │   ├── AIModelModule.tsx  # AI模型管理页面
│   │   ├── ConfigModule.tsx   # 配置管理页面
│   │   └── SystemControl.tsx  # 系统控制页面
│   ├── App.tsx               # 主应用组件
│   └── main.tsx              # 入口文件
├── .env                      # 环境配置
├── vite.config.ts            # Vite配置
├── package.json              # 依赖配置
└── start-frontend.sh         # 启动脚本
```

## 🔧 配置说明

### 环境变量 (.env)

```bash
# API网关配置
VITE_API_BASE_URL=http://localhost:3000
VITE_WS_BASE_URL=ws://localhost:3000

# 系统信息
VITE_SYSTEM_NAME=5.1套利系统
VITE_SYSTEM_VERSION=v5.1.0
VITE_TOTAL_APIS=387
VITE_MICROSERVICES_COUNT=7
```

### Vite配置 (vite.config.ts)

```typescript
export default defineConfig({
  server: {
    port: 3003,
    host: '0.0.0.0', // 允许外网访问
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true
      },
      '/ws': {
        target: 'ws://localhost:3000',
        ws: true,
        changeOrigin: true
      }
    }
  }
})
```

## 📊 API接口分布

| 服务名称 | 端口 | API数量 | 功能描述 |
|----------|------|---------|----------|
| logging-service | 4001 | 45个 | 日志收集、分析、实时流 |
| cleaning-service | 4002 | 52个 | 数据清洗、质量控制 |
| strategy-service | 4003 | 38个 | 策略管理、热重载 |
| performance-service | 4004 | 67个 | 性能优化、资源监控 |
| trading-service | 4005 | 41个 | 订单管理、风险控制 |
| ai-model-service | 4006 | 48个 | AI模型管理、训练推理 |
| config-service | 4007 | 96个 | 配置管理、版本控制 |
| **总计** | - | **387个** | **完整套利系统** |

## 🌐 前端页面功能

### 1. 主仪表板 (`/dashboard`)
- 387个API接口状态总览
- 7个微服务健康监控
- 实时性能指标展示
- 系统状态可视化

### 2. 日志管理 (`/logging`)
- 45个日志API接口管理
- 实时日志流 (WebSocket)
- 日志级别配置
- 日志分析和搜索

### 3. 清洗管理 (`/cleaning`)
- 52个清洗API接口管理
- 清洗规则配置
- 交易所数据配置
- 数据质量监控

### 4. 策略管理 (`/strategy`)
- 38个策略API接口管理
- 策略生命周期控制
- 实时监控和调试
- 热重载功能

### 5. 性能管理 (`/performance`)
- 67个性能API接口管理
- CPU/内存/网络/磁盘监控
- 性能优化工具
- 基准测试

### 6. 交易管理 (`/trading`)
- 41个交易API接口管理
- 订单和仓位监控
- 资金管理
- 风险控制

### 7. AI模型管理 (`/ai-model`)
- 48个AI模型API接口管理
- 模型训练和部署
- 推理服务监控
- 特征工程工具

### 8. 配置管理 (`/config`)
- 96个配置API接口管理
- 版本控制
- 热重载管理
- 环境配置

### 9. 系统控制 (`/system`)
- 系统启停控制
- 服务管理
- 备份恢复
- 系统诊断

## 🔗 API调用示例

### 基础API调用

```typescript
import { serviceManager } from '../services';

// 获取所有策略
const strategies = await serviceManager.strategy.listStrategies();

// 启动策略
await serviceManager.strategy.startStrategy('triangular-v4');

// 获取实时日志
const logs = await serviceManager.logging.getRealtimeLogStream();

// 获取系统状态
const status = await serviceManager.systemControl.getSystemStatus();
```

### WebSocket连接

```typescript
import { loggingService } from '../services';

// 连接实时日志流
const ws = loggingService.connectRealtimeLogs(
  (logData) => {
    console.log('New log:', logData);
  },
  (error) => {
    console.error('WebSocket error:', error);
  }
);
```

## 🛡️ 生产部署

### 1. 构建生产版本

```bash
npm run build
```

### 2. 使用Nginx部署

```nginx
server {
    listen 3003;
    server_name YOUR_DOMAIN;
    
    root /path/to/arbitrage-frontend-v5.1/dist;
    index index.html;
    
    location / {
        try_files $uri $uri/ /index.html;
    }
    
    location /api {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
    
    location /ws {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

### 3. 使用PM2部署

```bash
# 安装PM2
npm install -g pm2

# 启动前端服务
pm2 start npm --name "arbitrage-frontend" -- run preview

# 设置开机自启
pm2 startup
pm2 save
```

## 🔍 故障排除

### 1. 端口占用问题

```bash
# 检查端口占用
lsof -i :3003

# 清理端口
pkill -f "vite.*3003"
```

### 2. API连接问题

```bash
# 检查统一网关状态
curl http://localhost:3000/health

# 检查微服务状态
curl http://localhost:4001/health  # 日志服务
curl http://localhost:4002/health  # 清洗服务
# ... 其他服务
```

### 3. WebSocket连接问题

```bash
# 检查WebSocket连接
wscat -c ws://localhost:3000/ws/logs/realtime
```

## 📈 性能监控

前端集成了完整的性能监控：

- **API响应时间监控**: 所有387个API接口的响应时间
- **WebSocket连接状态**: 实时连接状态监控
- **前端性能指标**: 页面加载时间、内存使用等
- **用户操作追踪**: 用户行为分析

## 🎯 总结

5.1套利系统前端已完成：

✅ **387个API接口完整对接**  
✅ **7个微服务统一管理**  
✅ **实时WebSocket连接**  
✅ **响应式UI设计**  
✅ **生产级部署方案**  
✅ **外网访问支持**  
✅ **完整错误处理**  
✅ **性能监控集成**  

前端现在可以100%控制后端所有运行，实现真实的生产级套利系统管理！ 