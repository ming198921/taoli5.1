# 5.1套利系统前端SDK

完整的TypeScript SDK，提供对5.1套利系统的全面控制能力，实现前端100%控制后端功能。

## 功能特点

- **完整API覆盖**: 覆盖系统所有API端点和功能
- **实时数据支持**: WebSocket连接，实时获取市场数据、警报和系统状态
- **认证与权限**: 完整的用户认证、角色管理和权限控制
- **生产级特性**: 自动重连、错误重试、请求缓存、性能优化
- **类型安全**: 完整的TypeScript类型定义
- **易于使用**: 简洁的API设计和丰富的工具方法

## 安装

```bash
npm install @arbitrage-system/frontend-sdk
```

## 快速开始

### 基础使用

```typescript
import { ArbitrageSystemSDK } from '@arbitrage-system/frontend-sdk';

// 创建SDK实例
const sdk = new ArbitrageSystemSDK({
  baseUrl: 'http://localhost:8080',
  wsUrl: 'ws://localhost:8080/ws',
  enableLogging: true,
});

// 初始化SDK
await sdk.initialize();

// 用户登录
const user = await sdk.login({
  username: 'admin',
  password: 'password123',
});

console.log('登录成功:', user);
```

### 系统控制

```typescript
// 获取系统状态
const status = await sdk.system.getSystemStatus();
console.log('系统状态:', status);

// 启动/停止系统
await sdk.system.startSystem();
await sdk.system.stopSystem();

// 执行健康检查
const health = await sdk.healthCheck();
console.log('健康状态:', health);
```

### 市场数据访问

```typescript
// 获取市场数据
const marketData = await sdk.qingxi.getMarketData();
console.log('市场数据:', marketData);

// 获取订单簿
const orderBook = await sdk.qingxi.getOrderBook('BTC/USDT', 'binance');
console.log('订单簿:', orderBook);

// 获取套利机会
const opportunities = await sdk.qingxi.getArbitrageOpportunities({
  page: 1,
  limit: 10,
  minProfitPercent: 0.1,
});
console.log('套利机会:', opportunities);
```

### 仪表板数据

```typescript
// 获取仪表板统计
const stats = await sdk.dashboard.getDashboardStats();
console.log('系统统计:', stats);

// 获取Sankey图数据
const sankeyData = await sdk.dashboard.getSankeyData({
  timeRange: {
    startTime: '2024-01-01T00:00:00Z',
    endTime: '2024-01-31T23:59:59Z',
  },
});

// 获取利润曲线
const profitCurve = await sdk.dashboard.getProfitCurve({
  granularity: 'day',
  timeRange: {
    startTime: '2024-01-01T00:00:00Z',
    endTime: '2024-01-31T23:59:59Z',
  },
});
```

### 监控和警报

```typescript
// 获取系统健康检查
const healthChecks = await sdk.monitoring.getHealthChecks();
console.log('健康检查:', healthChecks);

// 获取警报列表
const alerts = await sdk.monitoring.getAlerts({
  page: 1,
  limit: 20,
  type: 'error',
});

// 创建警报规则
const rule = await sdk.monitoring.createAlertRule({
  name: 'CPU使用率过高',
  description: 'CPU使用率超过80%时触发警报',
  metric: 'cpu_usage_percent',
  operator: 'gt',
  threshold: 80,
  severity: 'warning',
  cooldown_minutes: 5,
});
```

### 实时数据订阅

```typescript
// 连接WebSocket
await sdk.connectWebSocket();

// 订阅市场数据更新
const marketDataSub = sdk.subscribeMarketData((data) => {
  console.log('市场数据更新:', data);
});

// 订阅套利机会
const opportunitiesSub = sdk.subscribeArbitrageOpportunities((opportunity) => {
  console.log('新的套利机会:', opportunity);
});

// 订阅系统警报
const alertsSub = sdk.subscribeAlerts((alert) => {
  console.log('系统警报:', alert);
  
  // 自动确认警报
  if (alert.type === 'info') {
    sdk.monitoring.acknowledgeAlert(alert.id, '自动确认');
  }
});

// 取消订阅
marketDataSub.unsubscribe();
opportunitiesSub.unsubscribe();
alertsSub.unsubscribe();
```

### 用户和权限管理

```typescript
// 检查用户权限
const hasAdminAccess = await sdk.hasPermission('admin.system.control');
const isAdmin = await sdk.isAdmin();

// 管理用户（需要管理员权限）
if (isAdmin) {
  // 创建新用户
  const newUser = await sdk.auth.createUser({
    username: 'trader1',
    email: 'trader1@example.com',
    password: 'password123',
    role: 'trader',
  });

  // 获取用户列表
  const users = await sdk.auth.getUsers({
    page: 1,
    limit: 10,
  });

  // 禁用用户
  await sdk.auth.setUserStatus(newUser.id, false);
}
```

### 批量操作

```typescript
// 使用批量操作助手
const batch = sdk.batch();

// 添加多个操作
const marketDataPromise = batch.add(() => sdk.qingxi.getMarketData());
const statsPromise = batch.add(() => sdk.dashboard.getDashboardStats());
const alertsPromise = batch.add(() => sdk.monitoring.getAlerts());

// 并行执行所有操作
const results = await batch.executeParallel();
console.log('成功的操作:', results.successful);
console.log('失败的操作:', results.failed);
```

### 错误处理

```typescript
import { ArbitrageSDKError } from '@arbitrage-system/frontend-sdk';

try {
  await sdk.qingxi.getMarketData();
} catch (error) {
  if (error instanceof ArbitrageSDKError) {
    console.error('SDK错误:', error.code, error.message);
    console.error('详细信息:', error.details);
  } else {
    console.error('未知错误:', error);
  }
  
  // 使用工具方法格式化错误
  const formattedError = ArbitrageSystemSDK.formatError(error);
  console.error('格式化错误:', formattedError);
}
```

### 重试和容错

```typescript
// 使用SDK提供的重试工具
const data = await ArbitrageSystemSDK.retry(
  () => sdk.qingxi.getMarketData(),
  3,  // 最大重试次数
  1000  // 重试延迟(ms)
);

// HTTP客户端自动重试
const opportunities = await sdk.httpClient.requestWithRetry(
  () => sdk.qingxi.getArbitrageOpportunities(),
  5,    // 最大重试次数
  2000  // 重试延迟(ms)
);
```

## API参考

### SDK配置

```typescript
interface SDKConfig {
  baseUrl: string;           // API服务器地址
  wsUrl?: string;           // WebSocket服务器地址
  apiKey?: string;          // API密钥
  timeout?: number;         // 请求超时时间(ms)
  retryAttempts?: number;   // 重试次数
  retryDelay?: number;      // 重试延迟(ms)
  enableLogging?: boolean;  // 启用日志
}
```

### 主要服务

#### AuthService - 认证服务
- 用户登录/登出
- 权限管理
- 用户管理（管理员功能）
- 会话管理

#### QingxiService - 数据服务
- 市场数据获取
- 订单簿数据
- 收集器状态管理
- 套利机会获取和执行

#### DashboardService - 仪表板服务
- 统计数据
- Sankey图数据
- 利润曲线
- 性能分析
- 自定义报告

#### MonitoringService - 监控服务
- 健康检查
- 系统指标
- 警报管理
- 警报规则配置

#### SystemService - 系统服务
- 系统控制（启动/停止）
- 系统状态
- 配置管理
- 日志访问

## WebSocket事件

SDK支持以下WebSocket事件类型：

- `market_data` - 市场数据更新
- `orderbook` - 订单簿更新
- `arbitrage_opportunity` - 套利机会
- `alert` - 系统警报
- `system_status` - 系统状态变更
- `connected` - WebSocket连接成功
- `disconnected` - WebSocket连接断开
- `authenticated` - 认证成功
- `auth_failed` - 认证失败
- `error` - 错误事件

## 最佳实践

### 1. 错误处理
```typescript
// 始终处理可能的错误
try {
  const data = await sdk.qingxi.getMarketData();
  // 处理数据
} catch (error) {
  // 记录错误
  console.error('获取市场数据失败:', error);
  // 向用户显示友好的错误消息
}
```

### 2. 资源清理
```typescript
// 组件卸载时清理资源
useEffect(() => {
  const subscription = sdk.subscribeMarketData(handleMarketData);
  
  return () => {
    subscription.unsubscribe();
  };
}, []);
```

### 3. 性能优化
```typescript
// 使用批量操作减少网络请求
const batch = sdk.batch();
const [marketData, stats, alerts] = await Promise.all([
  sdk.qingxi.getMarketData(),
  sdk.dashboard.getDashboardStats(),
  sdk.monitoring.getAlerts({ limit: 5 }),
]);
```

### 4. 类型安全
```typescript
// 利用TypeScript类型检查
import { MarketData, SystemStatus } from '@arbitrage-system/frontend-sdk';

function processMarketData(data: MarketData): void {
  // TypeScript会提供完整的类型检查和自动完成
  console.log(`价格: ${data.price}, 成交量: ${data.volume}`);
}
```

## 许可证

MIT License

## 支持

如有问题或建议，请联系开发团队或提交Issue。