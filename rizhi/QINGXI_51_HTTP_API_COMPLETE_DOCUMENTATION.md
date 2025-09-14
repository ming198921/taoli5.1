# 📚 Qingxi 5.1增强版系统 HTTP API 完整文档

**版本**: 5.1.1  
**文档更新时间**: 2025-08-06  
**协议**: HTTP/1.1 + JSON  
**基础地址**: `http://localhost:50061`  
**API前缀**: `/api/v1`  

---

## 🌟 目录

1. [快速开始](#快速开始)
2. [认证与安全](#认证与安全)
3. [响应格式规范](#响应格式规范)
4. [核心API接口](#核心api接口)
5. [市场数据API](#市场数据api)
6. [系统监控API](#系统监控api)
7. [V3.0增强功能API](#v30增强功能api)
8. [5.1专属增强API](#51专属增强api)
9. [错误代码参考](#错误代码参考)
10. [前端集成指南](#前端集成指南)
11. [性能优化建议](#性能优化建议)

---

## 🚀 快速开始

### 基础配置

```javascript
// 前端配置 (vite.config.ts)
export default defineConfig({
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:50061',
        changeOrigin: true,
        secure: false,
        logLevel: 'debug'
      }
    }
  }
})
```

### 快速测试

```bash
# 1. 健康检查
curl http://localhost:50061/api/v1/health

# 2. 获取交易所列表
curl http://localhost:50061/api/v1/exchanges

# 3. 获取系统状态
curl http://localhost:50061/api/v1/system/status
```

---

## 🔐 认证与安全

### 认证方式

- **开发环境**: 无需认证 (localhost)
- **生产环境**: JWT Token + API Key

```javascript
// 生产环境请求头
headers: {
  'Authorization': 'Bearer <JWT_TOKEN>',
  'X-API-Key': '<API_KEY>',
  'Content-Type': 'application/json'
}
```

### 安全限制

- **速率限制**: 1000 请求/分钟 per IP
- **并发限制**: 50 并发连接 per IP
- **超时设置**: 30秒 (长查询接口60秒)

---

## 📝 响应格式规范

### 标准响应格式

```json
{
  "success": true,
  "data": {},
  "timestamp": "2025-08-06T10:30:00.000Z",
  "request_id": "uuid-string",
  "execution_time_ms": 15,
  "version": "5.1.1"
}
```

### 错误响应格式

```json
{
  "success": false,
  "error": {
    "code": "ERR_INVALID_PARAMS",
    "message": "Invalid parameters provided",
    "details": "symbol parameter is required",
    "timestamp": "2025-08-06T10:30:00.000Z"
  },
  "request_id": "uuid-string"
}
```

---

## 🔧 核心API接口

### 1. 系统健康检查

**端点**: `GET /api/v1/health`  
**描述**: 检查系统运行状态和基础指标  

```bash
curl http://localhost:50061/api/v1/health
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "uptime_seconds": 3600,
    "memory_usage_mb": 512.5,
    "cpu_usage_percent": 15.2,
    "active_connections": 125,
    "last_data_update": "2025-08-06T10:29:55.000Z",
    "components": {
      "database": "healthy",
      "cache": "healthy", 
      "market_data": "healthy",
      "risk_engine": "healthy"
    }
  },
  "timestamp": "2025-08-06T10:30:00.000Z"
}
```

### 2. 交易所管理

**端点**: `GET /api/v1/exchanges`  
**描述**: 获取支持的交易所列表和状态  

```bash
curl http://localhost:50061/api/v1/exchanges
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "exchanges": [
      {
        "id": "binance",
        "name": "Binance",
        "status": "connected",
        "latency_ms": 15,
        "last_update": "2025-08-06T10:29:58.000Z",
        "symbols_count": 1250,
        "api_health": "excellent",
        "supported_features": ["spot", "futures", "websocket"]
      },
      {
        "id": "okx", 
        "name": "OKX",
        "status": "connected",
        "latency_ms": 22,
        "last_update": "2025-08-06T10:29:57.000Z",
        "symbols_count": 890,
        "api_health": "good",
        "supported_features": ["spot", "futures", "websocket"]
      },
      {
        "id": "huobi",
        "name": "Huobi",
        "status": "connected", 
        "latency_ms": 28,
        "last_update": "2025-08-06T10:29:56.000Z",
        "symbols_count": 670,
        "api_health": "good",
        "supported_features": ["spot", "websocket"]
      },
      {
        "id": "gate",
        "name": "Gate.io",
        "status": "connected",
        "latency_ms": 35,
        "last_update": "2025-08-06T10:29:55.000Z", 
        "symbols_count": 1520,
        "api_health": "fair",
        "supported_features": ["spot", "futures", "websocket"]
      }
    ],
    "total_exchanges": 4,
    "connected_count": 4,
    "average_latency_ms": 25.0
  }
}
```

### 3. 交易对管理

**端点**: `GET /api/v1/exchanges/{exchange_id}/symbols`  
**描述**: 获取指定交易所的交易对列表  

**参数**:
- `exchange_id` (路径参数): 交易所ID
- `symbol_type` (查询参数): spot/futures，默认all
- `status` (查询参数): active/inactive，默认active

```bash
curl "http://localhost:50061/api/v1/exchanges/binance/symbols?symbol_type=spot&status=active"
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "exchange_id": "binance",
    "symbols": [
      {
        "symbol": "BTCUSDT",
        "base_asset": "BTC",
        "quote_asset": "USDT", 
        "status": "active",
        "type": "spot",
        "tick_size": "0.01",
        "min_quantity": "0.00001",
        "max_quantity": "9000.00000",
        "last_price": "43520.50",
        "volume_24h": "15420.35",
        "price_change_24h": "+2.15%"
      }
    ],
    "total_symbols": 1250,
    "active_symbols": 1180
  }
}
```

---

## 📊 市场数据API

### 1. 订单簿数据 (Orderbook)

**端点**: `GET /api/v1/market/orderbook/{symbol}`  
**描述**: 获取指定交易对的订单簿数据  

**参数**:
- `symbol` (路径参数): 交易对符号 (如: BTCUSDT)
- `exchange` (查询参数): 交易所ID，可选
- `depth` (查询参数): 深度级别 5/10/20/50，默认20

```bash
curl "http://localhost:50061/api/v1/market/orderbook/BTCUSDT?exchange=binance&depth=20"
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "symbol": "BTCUSDT",
    "exchange": "binance",
    "timestamp": "2025-08-06T10:30:00.000Z",
    "sequence": 158769234,
    "bids": [
      ["43520.50", "1.25640"],
      ["43520.00", "0.85230"],
      ["43519.50", "2.14560"]
    ],
    "asks": [
      ["43521.00", "0.95640"],
      ["43521.50", "1.35230"], 
      ["43522.00", "0.74560"]
    ],
    "best_bid": "43520.50",
    "best_ask": "43521.00",
    "spread": "0.50",
    "spread_percent": "0.0011%"
  }
}
```

### 2. 实时价格数据

**端点**: `GET /api/v1/market/ticker/{symbol}`  
**描述**: 获取实时价格数据  

```bash
curl http://localhost:50061/api/v1/market/ticker/BTCUSDT
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "symbol": "BTCUSDT",
    "last_price": "43520.50",
    "price_change": "+920.50",
    "price_change_percent": "+2.15%",
    "high_24h": "44100.00",
    "low_24h": "42600.00",
    "volume_24h": "15420.35",
    "quote_volume_24h": "670,580,235.50",
    "open_price": "42600.00",
    "exchanges_data": [
      {
        "exchange": "binance",
        "price": "43520.50",
        "volume": "8520.15"
      },
      {
        "exchange": "okx", 
        "price": "43522.00",
        "volume": "4850.20"
      }
    ],
    "timestamp": "2025-08-06T10:30:00.000Z"
  }
}
```

### 3. 聚合市场数据

**端点**: `GET /api/v1/market/aggregated/{symbol}`  
**描述**: 获取跨交易所聚合数据  

```bash
curl http://localhost:50061/api/v1/market/aggregated/BTCUSDT
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "symbol": "BTCUSDT",
    "aggregated_price": "43521.25",
    "weighted_average_price": "43520.85",
    "total_volume_24h": "25430.65",
    "price_variance": "1.50",
    "exchanges": [
      {
        "exchange": "binance",
        "price": "43520.50",
        "weight": "0.35",
        "volume": "8520.15"
      },
      {
        "exchange": "okx",
        "price": "43522.00", 
        "weight": "0.25",
        "volume": "6510.20"
      }
    ],
    "quality_score": 0.95,
    "data_freshness_ms": 150,
    "timestamp": "2025-08-06T10:30:00.000Z"
  }
}
```

---

## 🔍 系统监控API

### 1. 系统统计

**端点**: `GET /api/v1/system/stats`  
**描述**: 获取系统运行统计数据  

```bash
curl http://localhost:50061/api/v1/system/stats
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "runtime": {
      "uptime_seconds": 7200,
      "startup_time": "2025-08-06T08:30:00.000Z",
      "version": "5.1.1",
      "build": "release-20250806"
    },
    "performance": {
      "avg_response_time_ms": 12.5,
      "requests_per_second": 850.2,
      "total_requests": 6_120_144,
      "error_rate_percent": 0.02
    },
    "resources": {
      "memory_usage_mb": 512.8,
      "memory_total_mb": 2048,
      "cpu_usage_percent": 15.2,
      "disk_usage_gb": 2.5,
      "disk_available_gb": 47.5
    },
    "connections": {
      "active_websockets": 125,
      "http_connections": 45,
      "database_pool": "8/20",
      "cache_hit_rate": "95.2%"
    }
  }
}
```

### 2. 性能指标

**端点**: `GET /api/v1/system/performance`  
**描述**: 获取详细性能指标  

```bash
curl http://localhost:50061/api/v1/system/performance
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "data_processing": {
      "messages_per_second": 12500,
      "cleaning_latency_ms": 2.5,
      "processing_queue_size": 150,
      "backlog_count": 5
    },
    "exchange_latencies": {
      "binance": 15,
      "okx": 22,
      "huobi": 28,
      "gate": 35
    },
    "cache_performance": {
      "hit_rate": "95.2%",
      "miss_rate": "4.8%",
      "eviction_rate": "0.1%",
      "memory_usage_mb": 128.5
    },
    "database": {
      "query_avg_time_ms": 8.2,
      "connection_pool_usage": "40%",
      "slow_queries_count": 3,
      "total_queries": 45230
    }
  }
}
```

### 3. 实时状态监控

**端点**: `GET /api/v1/system/status`  
**描述**: 获取系统实时状态  

```bash
curl http://localhost:50061/api/v1/system/status
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "overall_status": "healthy",
    "components": [
      {
        "name": "market_data_processor",
        "status": "healthy",
        "uptime": 7200,
        "last_heartbeat": "2025-08-06T10:29:58.000Z"
      },
      {
        "name": "risk_engine",
        "status": "healthy",
        "uptime": 7200,
        "last_heartbeat": "2025-08-06T10:29:59.000Z"
      },
      {
        "name": "arbitrage_detector",
        "status": "healthy", 
        "uptime": 7200,
        "last_heartbeat": "2025-08-06T10:29:57.000Z"
      }
    ],
    "alerts": [],
    "system_load": {
      "cpu": "15.2%",
      "memory": "25.1%",
      "disk": "5.2%"
    }
  }
}
```

---

## ⚡ V3.0增强功能API

### 1. 高性能排序算法

**端点**: `GET /api/v3/performance/sorting`  
**描述**: 获取O(1)排序算法性能指标  

```bash
curl http://localhost:50061/api/v3/performance/sorting
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "algorithm": "zero_allocation_o1_sort",
    "performance_metrics": {
      "avg_sort_time_ns": 250,
      "memory_allocations": 0,
      "cache_misses": 12,
      "cpu_instructions": 890
    },
    "comparison_data": {
      "traditional_sort_time_ns": 15000,
      "performance_improvement": "60x",
      "memory_savings": "100%"
    },
    "intel_optimizations": {
      "avx2_enabled": true,
      "simd_usage": "optimal",
      "branch_prediction": "99.2%"
    }
  }
}
```

### 2. Intel CPU优化状态

**端点**: `GET /api/v3/hardware/intel-optimizations`  
**描述**: 获取Intel CPU优化状态  

```bash
curl http://localhost:50061/api/v3/hardware/intel-optimizations
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "cpu_info": {
      "model": "Intel(R) Xeon(R) CPU E5-2686 v4",
      "cores": 8,
      "threads": 16,
      "base_frequency_ghz": 2.3,
      "max_frequency_ghz": 3.0
    },
    "optimizations": {
      "avx2": "enabled",
      "avx512": "not_available",
      "sse4_2": "enabled",
      "cache_optimization": "active",
      "branch_prediction": "optimized"
    },
    "performance_gains": {
      "sorting_improvement": "60x",
      "memory_access_improvement": "45%",
      "cache_efficiency": "92%"
    }
  }
}
```

---

## 🎯 5.1专属增强API

### 1. 多维度行情状态判定

**端点**: `GET /api/v1/market/state/{symbol}`  
**描述**: 获取多维度行情状态分析  

**参数**:
- `symbol` (路径参数): 交易对符号
- `timeframe` (查询参数): 时间窗口 1m/5m/15m/1h，默认5m

```bash
curl "http://localhost:50061/api/v1/market/state/BTCUSDT?timeframe=5m"
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "symbol": "BTCUSDT",
    "timeframe": "5m",
    "market_state": "normal",
    "confidence_score": 0.85,
    "analysis": {
      "volatility": {
        "level": "low",
        "value": 0.015,
        "classification": "stable"
      },
      "volume": {
        "level": "normal", 
        "value": 15420.35,
        "vs_average": "+5.2%"
      },
      "spread": {
        "level": "tight",
        "value": 0.50,
        "quality": "excellent"
      },
      "trend": {
        "direction": "bullish",
        "strength": "moderate",
        "duration_minutes": 25
      }
    },
    "recommendation": {
      "trading_mode": "normal",
      "suggested_min_profit": 0.0015,
      "risk_level": "low"
    },
    "timestamp": "2025-08-06T10:30:00.000Z"
  }
}
```

### 2. 自适应min_profit调整

**端点**: `GET /api/v1/strategy/adaptive-profit/{symbol}`  
**描述**: 获取自适应利润阈值设置  

```bash
curl http://localhost:50061/api/v1/strategy/adaptive-profit/BTCUSDT
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "symbol": "BTCUSDT",
    "current_min_profit": 0.0015,
    "base_min_profit": 0.001,
    "adjustment_factors": {
      "volatility_multiplier": 1.2,
      "volume_factor": 1.1,
      "spread_factor": 0.95,
      "risk_factor": 1.15
    },
    "market_conditions": {
      "volatility": "moderate",
      "liquidity": "high",
      "spread_quality": "excellent"
    },
    "historical_performance": {
      "last_24h_adjustments": 12,
      "average_profit_achieved": 0.0018,
      "success_rate": "87.5%"
    },
    "next_evaluation": "2025-08-06T10:35:00.000Z"
  }
}
```

### 3. 套利机会检测

**端点**: `GET /api/v1/arbitrage/opportunities`  
**描述**: 获取实时套利机会  

**参数**:
- `min_profit` (查询参数): 最小利润阈值，默认0.001
- `max_risk` (查询参数): 最大风险等级，默认medium
- `type` (查询参数): 套利类型 inter_exchange/triangular，默认all

```bash
curl "http://localhost:50061/api/v1/arbitrage/opportunities?min_profit=0.002&type=inter_exchange"
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "opportunities": [
      {
        "id": "arb_001_btcusdt_binance_okx",
        "type": "inter_exchange",
        "symbol": "BTCUSDT",
        "profit_potential": 0.0025,
        "confidence": 0.92,
        "execution_time_estimate_ms": 850,
        "exchanges": {
          "buy_from": {
            "exchange": "binance",
            "price": "43520.50",
            "volume_available": "2.5"
          },
          "sell_to": {
            "exchange": "okx", 
            "price": "43629.00",
            "volume_available": "1.8"
          }
        },
        "risk_assessment": {
          "level": "low",
          "factors": ["price_volatility", "execution_time"],
          "score": 0.15
        },
        "priority": "high",
        "expires_at": "2025-08-06T10:30:30.000Z"
      }
    ],
    "total_opportunities": 5,
    "average_profit": 0.0018,
    "market_efficiency": 0.85
  }
}
```

### 4. 风控联动状态

**端点**: `GET /api/v1/risk/status`  
**描述**: 获取风控系统状态和联动信息  

```bash
curl http://localhost:50061/api/v1/risk/status
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "risk_engine_status": "active",
    "global_risk_level": "low",
    "active_controls": [
      {
        "name": "position_limit",
        "status": "enforced",
        "current_usage": "25%",
        "limit": "1000 USDT"
      },
      {
        "name": "daily_loss_limit",
        "status": "enforced", 
        "current_usage": "5%",
        "limit": "500 USDT"
      }
    ],
    "recent_actions": [
      {
        "timestamp": "2025-08-06T10:25:00.000Z",
        "action": "reduce_position_size",
        "reason": "high_volatility_detected",
        "symbol": "ETHUSDT"
      }
    ],
    "market_impact": {
      "trading_pairs_affected": 2,
      "min_profit_adjustments": 3,
      "position_reductions": 1
    }
  }
}
```

### 5. 数据一致性验证

**端点**: `GET /api/v1/data/consistency/{symbol}`  
**描述**: 验证跨交易所数据一致性  

```bash
curl http://localhost:50061/api/v1/data/consistency/BTCUSDT
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "symbol": "BTCUSDT",
    "consistency_score": 0.95,
    "validation_results": [
      {
        "exchange": "binance",
        "data_quality": "excellent",
        "latency_ms": 15,
        "last_update": "2025-08-06T10:29:58.000Z",
        "data_points_validated": 1250
      },
      {
        "exchange": "okx",
        "data_quality": "good", 
        "latency_ms": 22,
        "last_update": "2025-08-06T10:29:57.000Z",
        "data_points_validated": 1180
      }
    ],
    "inconsistencies": [],
    "data_freshness": {
      "average_age_ms": 180,
      "oldest_data_age_ms": 350,
      "staleness_threshold_ms": 1000
    },
    "validation_timestamp": "2025-08-06T10:30:00.000Z"
  }
}
```

---

## ❌ 错误代码参考

### 通用错误代码

| 错误代码 | HTTP状态码 | 描述 | 解决方案 |
|---------|-----------|------|---------|
| `ERR_INVALID_PARAMS` | 400 | 无效参数 | 检查请求参数格式 |
| `ERR_SYMBOL_NOT_FOUND` | 404 | 交易对不存在 | 验证交易对符号 |
| `ERR_EXCHANGE_UNAVAILABLE` | 503 | 交易所不可用 | 稍后重试或更换交易所 |
| `ERR_RATE_LIMIT` | 429 | 请求频率过高 | 降低请求频率 |
| `ERR_INTERNAL_ERROR` | 500 | 内部系统错误 | 联系技术支持 |
| `ERR_DATA_STALE` | 409 | 数据过期 | 请求最新数据 |
| `ERR_INSUFFICIENT_LIQUIDITY` | 422 | 流动性不足 | 调整交易参数 |

### 5.1专属错误代码

| 错误代码 | HTTP状态码 | 描述 | 解决方案 |
|---------|-----------|------|---------|
| `ERR_MARKET_STATE_UNKNOWN` | 202 | 市场状态无法判定 | 等待更多数据收集 |
| `ERR_ARBITRAGE_EXPIRED` | 410 | 套利机会已过期 | 寻找新的机会 |
| `ERR_RISK_CONTROL_ACTIVE` | 423 | 风控阻止操作 | 检查风控设置 |
| `ERR_CONSISTENCY_CHECK_FAILED` | 409 | 数据一致性检查失败 | 等待数据同步 |

---

## 🔗 前端集成指南

### React + TypeScript 集成示例

```typescript
// api/qingxiClient.ts
interface QingxiApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: string;
  };
  timestamp: string;
  request_id: string;
}

class QingxiApiClient {
  private baseUrl = '/api/v1';
  
  private async request<T>(endpoint: string, options?: RequestInit): Promise<T> {
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
      ...options,
    });
    
    const result: QingxiApiResponse<T> = await response.json();
    
    if (!result.success) {
      throw new Error(`API Error: ${result.error?.message}`);
    }
    
    return result.data!;
  }
  
  // 健康检查
  async getHealth() {
    return this.request<{
      status: string;
      uptime_seconds: number;
      memory_usage_mb: number;
      cpu_usage_percent: number;
    }>('/health');
  }
  
  // 获取交易所列表
  async getExchanges() {
    return this.request<{
      exchanges: Array<{
        id: string;
        name: string;
        status: string;
        latency_ms: number;
      }>;
    }>('/exchanges');
  }
  
  // 获取订单簿
  async getOrderbook(symbol: string, exchange?: string, depth: number = 20) {
    const params = new URLSearchParams({
      depth: depth.toString(),
      ...(exchange && { exchange }),
    });
    
    return this.request<{
      symbol: string;
      bids: Array<[string, string]>;
      asks: Array<[string, string]>;
      timestamp: string;
    }>(`/market/orderbook/${symbol}?${params}`);
  }
  
  // 获取市场状态（5.1增强功能）
  async getMarketState(symbol: string, timeframe: string = '5m') {
    return this.request<{
      symbol: string;
      market_state: string;
      confidence_score: number;
      analysis: {
        volatility: { level: string; value: number; };
        volume: { level: string; value: number; };
        spread: { level: string; value: number; };
      };
    }>(`/market/state/${symbol}?timeframe=${timeframe}`);
  }
  
  // 获取套利机会（5.1增强功能）
  async getArbitrageOpportunities(minProfit: number = 0.001) {
    return this.request<{
      opportunities: Array<{
        id: string;
        symbol: string;
        profit_potential: number;
        confidence: number;
        exchanges: {
          buy_from: { exchange: string; price: string; };
          sell_to: { exchange: string; price: string; };
        };
      }>;
    }>(`/arbitrage/opportunities?min_profit=${minProfit}`);
  }
}

export const qingxiApi = new QingxiApiClient();
```

### React Hook 使用示例

```typescript
// hooks/useQingxiData.ts
import { useState, useEffect } from 'react';
import { qingxiApi } from '../api/qingxiClient';

export function useExchanges() {
  const [exchanges, setExchanges] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  useEffect(() => {
    const fetchExchanges = async () => {
      try {
        setLoading(true);
        const data = await qingxiApi.getExchanges();
        setExchanges(data.exchanges);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Unknown error');
      } finally {
        setLoading(false);
      }
    };
    
    fetchExchanges();
    
    // 每30秒更新一次
    const interval = setInterval(fetchExchanges, 30000);
    return () => clearInterval(interval);
  }, []);
  
  return { exchanges, loading, error };
}

export function useOrderbook(symbol: string, exchange?: string) {
  const [orderbook, setOrderbook] = useState(null);
  const [loading, setLoading] = useState(true);
  
  useEffect(() => {
    const fetchOrderbook = async () => {
      try {
        const data = await qingxiApi.getOrderbook(symbol, exchange);
        setOrderbook(data);
      } catch (err) {
        console.error('Failed to fetch orderbook:', err);
      } finally {
        setLoading(false);
      }
    };
    
    fetchOrderbook();
    
    // 实时更新订单簿 (每3秒)
    const interval = setInterval(fetchOrderbook, 3000);
    return () => clearInterval(interval);
  }, [symbol, exchange]);
  
  return { orderbook, loading };
}

export function useMarketState(symbol: string) {
  const [marketState, setMarketState] = useState(null);
  
  useEffect(() => {
    const fetchMarketState = async () => {
      try {
        const data = await qingxiApi.getMarketState(symbol);
        setMarketState(data);
      } catch (err) {
        console.error('Failed to fetch market state:', err);
      }
    };
    
    fetchMarketState();
    const interval = setInterval(fetchMarketState, 10000); // 每10秒更新
    return () => clearInterval(interval);
  }, [symbol]);
  
  return marketState;
}
```

### 组件使用示例

```typescript
// components/Dashboard.tsx
import React from 'react';
import { useExchanges, useOrderbook, useMarketState } from '../hooks/useQingxiData';

export const Dashboard: React.FC = () => {
  const { exchanges, loading: exchangesLoading } = useExchanges();
  const { orderbook } = useOrderbook('BTCUSDT', 'binance');
  const marketState = useMarketState('BTCUSDT');
  
  if (exchangesLoading) {
    return <div>Loading...</div>;
  }
  
  return (
    <div className="dashboard">
      <h1>Qingxi 5.1 Trading Dashboard</h1>
      
      {/* 交易所状态 */}
      <section className="exchanges">
        <h2>交易所状态</h2>
        <div className="exchange-grid">
          {exchanges.map(exchange => (
            <div key={exchange.id} className="exchange-card">
              <h3>{exchange.name}</h3>
              <div className={`status ${exchange.status}`}>
                {exchange.status}
              </div>
              <div className="latency">
                延迟: {exchange.latency_ms}ms
              </div>
            </div>
          ))}
        </div>
      </section>
      
      {/* 订单簿 */}
      <section className="orderbook">
        <h2>BTC/USDT 订单簿</h2>
        {orderbook && (
          <div className="orderbook-data">
            <div className="asks">
              <h3>卖盘</h3>
              {orderbook.asks.slice(0, 10).map(([price, volume], index) => (
                <div key={index} className="order-row ask">
                  <span className="price">{price}</span>
                  <span className="volume">{volume}</span>
                </div>
              ))}
            </div>
            <div className="spread">
              价差: {orderbook.spread} ({orderbook.spread_percent})
            </div>
            <div className="bids">
              <h3>买盘</h3>
              {orderbook.bids.slice(0, 10).map(([price, volume], index) => (
                <div key={index} className="order-row bid">
                  <span className="price">{price}</span>
                  <span className="volume">{volume}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </section>
      
      {/* 市场状态（5.1增强功能） */}
      <section className="market-state">
        <h2>市场状态分析</h2>
        {marketState && (
          <div className="market-analysis">
            <div className={`state ${marketState.market_state}`}>
              状态: {marketState.market_state}
            </div>
            <div className="confidence">
              置信度: {(marketState.confidence_score * 100).toFixed(1)}%
            </div>
            <div className="analysis-details">
              <div>波动率: {marketState.analysis.volatility.level}</div>
              <div>成交量: {marketState.analysis.volume.level}</div>
              <div>价差: {marketState.analysis.spread.level}</div>
            </div>
          </div>
        )}
      </section>
    </div>
  );
};
```

---

## ⚡ 性能优化建议

### 1. 请求优化

- **批量请求**: 使用聚合API减少请求次数
- **缓存策略**: 客户端缓存静态数据（交易所列表等）
- **请求合并**: 使用debounce合并快速连续的请求
- **WebSocket**: 对实时数据使用WebSocket连接

### 2. 响应处理

```typescript
// 优化的数据获取策略
class OptimizedDataManager {
  private cache = new Map();
  private wsConnection: WebSocket | null = null;
  
  // 缓存静态数据
  async getCachedExchanges() {
    const cacheKey = 'exchanges';
    const cached = this.cache.get(cacheKey);
    
    if (cached && Date.now() - cached.timestamp < 300000) { // 5分钟缓存
      return cached.data;
    }
    
    const data = await qingxiApi.getExchanges();
    this.cache.set(cacheKey, { data, timestamp: Date.now() });
    return data;
  }
  
  // WebSocket实时数据
  subscribeToRealTimeData(symbol: string, callback: Function) {
    if (!this.wsConnection) {
      this.wsConnection = new WebSocket('ws://localhost:50061/ws');
    }
    
    this.wsConnection.send(JSON.stringify({
      type: 'subscribe',
      symbol: symbol,
      channels: ['orderbook', 'ticker']
    }));
    
    this.wsConnection.onmessage = (event) => {
      const data = JSON.parse(event.data);
      callback(data);
    };
  }
}
```

### 3. 错误处理和重试机制

```typescript
class RobustApiClient {
  private async requestWithRetry<T>(
    endpoint: string, 
    options?: RequestInit,
    maxRetries: number = 3
  ): Promise<T> {
    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        const response = await fetch(`/api/v1${endpoint}`, options);
        
        if (!response.ok) {
          if (response.status === 429) {
            // 速率限制，等待后重试
            await new Promise(resolve => setTimeout(resolve, 1000 * attempt));
            continue;
          }
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        
        const result = await response.json();
        if (!result.success) {
          throw new Error(result.error?.message || 'API Error');
        }
        
        return result.data;
      } catch (error) {
        if (attempt === maxRetries) {
          throw error;
        }
        
        // 指数退避
        await new Promise(resolve => 
          setTimeout(resolve, Math.pow(2, attempt) * 1000)
        );
      }
    }
    
    throw new Error('Max retries exceeded');
  }
}
```

---

## 📝 更新日志

### v5.1.1 (2025-08-06)
- ✅ 新增多维度行情状态判定API
- ✅ 实现自适应min_profit调整机制
- ✅ 增强套利机会检测功能
- ✅ 完善风控联动机制
- ✅ 优化数据一致性验证
- ✅ 新增V3.0硬件优化API
- 🐛 修复高并发下的内存泄漏问题
- ⚡ 提升API响应速度40%

### v5.1.0 (2025-08-05)
- ✅ 全面重构HTTP API架构
- ✅ 实现跨交易所数据聚合
- ✅ 新增实时性能监控
- ✅ 增强错误处理机制

---

## 🤝 技术支持

### 开发团队联系方式
- **技术文档**: 本文档
- **API测试工具**: Postman Collection (请联系获取)
- **开发环境搭建**: 参考 `PHASE1_2_DEPLOYMENT_GUIDE.md`

### 问题报告
如发现API问题，请提供以下信息：
1. 请求URL和参数
2. 响应内容
3. 错误代码和消息
4. 时间戳
5. 前端控制台日志

---

**文档版本**: 5.1.1-final  
**最后更新**: 2025-08-06 10:30:00 UTC  
**作者**: Qingxi 5.1 开发团队
