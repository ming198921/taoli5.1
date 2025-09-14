# 前后端数据结构一致性分析报告

## 1. 前端数据结构 (TypeScript)

### ArbitrageOpportunity (前端)
```typescript
export interface ArbitrageOpportunity {
  id: string;
  symbol: string;
  buy_exchange: string;
  sell_exchange: string;
  buy_price: number;
  sell_price: number;
  profit_usd: number;
  profit_percent: number;
  volume_available: number;
  detected_at: string;
  expires_at: string;
  status: 'active' | 'executed' | 'expired' | 'cancelled';
}
```

## 2. 后端数据结构 (Rust) - 发现多个不一致定义！

### 定义1: /architecture/src/types.rs
```rust
pub struct ArbitrageOpportunity {
    pub id: String,
    pub strategy_type: StrategyType,
    pub exchange_pair: Option<(String, String)>,
    pub triangle_path: Option<Vec<String>>,
    pub symbol: String,
    pub estimated_profit: f64,
    pub net_profit: f64,
    pub profit_bps: f64,
    pub liquidity_score: f64,
    pub confidence_score: f64,
    pub estimated_latency_ms: u64,
    pub risk_score: f64,
    pub required_funds: HashMap<String, f64>,
    pub market_impact: f64,
    pub slippage_estimate: f64,
    // ... 更多字段
}
```

### 定义2: /celue/orchestrator/src/processor.rs
```rust
pub struct ArbitrageOpportunity {
    pub id: String,
    pub buy_exchange: String,
    pub sell_exchange: String,
    pub symbol: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub profit: f64,
    pub profit_pct: f64,
    pub volume: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub is_viable: bool,
}
```

### 定义3: /celue/common/src/arbitrage.rs
```rust
pub struct ArbitrageOpportunity {
    pub id: Uuid,
    pub strategy_name: String,
    pub legs: Vec<ArbitrageLeg>,
    pub gross_profit: FixedPrice,
    pub net_profit: FixedPrice,
    pub net_profit_pct: FixedPrice,
    pub created_at_ns: u64,
    pub ttl_ns: u64,
    pub tags: HashMap<String, String>,
}
```

## 3. 数据结构不一致性问题分析

### 主要问题：
1. **多重定义冲突**：后端存在至少8个不同的ArbitrageOpportunity结构定义
2. **字段名不匹配**：
   - 前端: `profit_usd` vs 后端: `estimated_profit` / `profit` / `gross_profit`
   - 前端: `profit_percent` vs 后端: `profit_pct` / `net_profit_pct` / `profit_bps`
   - 前端: `detected_at` vs 后端: `timestamp` / `created_at_ns`
3. **数据类型不匹配**：
   - ID字段：前端String vs 后端Uuid/String混合
   - 时间戳：前端string vs 后端DateTime/u64混合
4. **缺失字段**：前端有`status`, `expires_at`, `volume_available`等字段在某些后端定义中缺失

## 4. 兼容性影响

### 严重程度：🔴 CRITICAL
- API序列化/反序列化会失败
- 前端无法正确显示套利机会数据
- 数据类型转换错误可能导致系统崩溃
