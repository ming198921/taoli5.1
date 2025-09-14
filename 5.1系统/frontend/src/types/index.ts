// 主要类型导出
export * from './api';
export * from './celue';
export * from './architecture';
export * from './observability';
// 仅导出qingxi中不冲突的类型
export type { 
  QingXiConfig, 
  QingXiStatus, 
  QingXiMetrics, 
  DataCollectorConfig 
} from './qingxi';

// 通用类型定义
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: string;
}

export interface PaginatedResponse<T = any> {
  data: T[];
  total: number;
  page: number;
  pageSize: number;
}

export interface SystemStatus {
  isRunning: boolean;
  qingxi: 'running' | 'stopped' | 'error' | 'warning';
  celue: 'running' | 'stopped' | 'error' | 'warning';
  architecture: 'running' | 'stopped' | 'error' | 'warning';
  observability: 'running' | 'stopped' | 'error' | 'warning';
  uptime: number;
  lastUpdate: string;
}

export interface PerformanceMetrics {
  latency: {
    avg: number;
    p95: number;
    p99: number;
  };
  throughput: {
    requests_per_second: number;
    messages_per_second: number;
  };
  resource_usage: {
    cpu_percent: number;
    memory_usage_mb: number;
    network_io_mbps: number;
  };
}

// 统一的市场数据接口 - 与后端architecture::types::MarketData对应
export interface MarketData {
  symbol: string;
  exchanges: Record<string, ExchangeMarketData>;
  aggregated_price?: number;
  price_variance: number;
  volume_24h: number;
  timestamp: string;
  data_quality_score: number;
}

// 单个交易所的市场数据 - 与后端ExchangeMarketData对应
export interface ExchangeMarketData {
  exchange: string;
  symbol: string;
  last_price: number;
  best_bid: number;
  best_ask: number;
  bid_volume: number;
  ask_volume: number;
  volume_24h: number;
  price_change_24h: number;
  price_change_percent_24h: number;
  high_24h: number;
  low_24h: number;
  timestamp: string;
  orderbook?: OrderBookDepth;
  recent_trades: TradeInfo[];
}

// 订单簿深度数据
export interface OrderBookDepth {
  bids: [number, number][]; // [价格, 数量]
  asks: [number, number][]; // [价格, 数量]
  timestamp: string;
  exchange: string;
  symbol: string;
}

// 交易信息
export interface TradeInfo {
  trade_id: string;
  price: number;
  quantity: number;
  fee: number;
  fee_asset: string;
  timestamp: string;
  is_maker: boolean;
}

// 简化的MarketData接口 - 向后兼容
export interface SimpleMarketData {
  symbol: string;
  exchange: string;
  price: number;
  volume: number;
  timestamp: string;
  bid: number;
  ask: number;
  spread: number;
}

// 工具函数：从统一MarketData提取简化数据
export function extractSimpleMarketData(marketData: MarketData, exchange: string): SimpleMarketData | null {
  const exchangeData = marketData.exchanges[exchange];
  if (!exchangeData) return null;
  
  return {
    symbol: marketData.symbol,
    exchange: exchangeData.exchange,
    price: exchangeData.last_price,
    volume: exchangeData.volume_24h,
    timestamp: exchangeData.timestamp,
    bid: exchangeData.best_bid,
    ask: exchangeData.best_ask,
    spread: exchangeData.best_ask - exchangeData.best_bid,
  };
}

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

export interface RiskAlert {
  id: string;
  type: 'position_limit' | 'loss_limit' | 'exposure_limit' | 'correlation_risk' | 'system_error';
  severity: 'low' | 'medium' | 'high' | 'critical';
  message: string;
  details: Record<string, any>;
  created_at: string;
  resolved_at?: string;
  status: 'active' | 'resolved' | 'ignored';
}

export interface SystemEvent {
  id: string;
  type: 'system_start' | 'system_stop' | 'strategy_start' | 'strategy_stop' | 'error' | 'warning' | 'info';
  source: string;
  message: string;
  data?: Record<string, any>;
  timestamp: string;
}

export interface TimeSeriesData {
  timestamp: string;
  value: number;
  label?: string;
}

// 用户权限和认证
export interface User {
  id: string;
  username: string;
  email: string;
  role: 'admin' | 'trader' | 'viewer' | 'auditor';
  permissions: string[];
  lastLogin: string;
}

export interface AuthToken {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  token_type: string;
}