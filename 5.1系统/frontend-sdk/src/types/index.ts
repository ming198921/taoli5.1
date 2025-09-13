/**
 * 5.1套利系统前端SDK类型定义
 * 完整映射后端所有数据结构和API响应
 */

// ========== 通用类型 ==========
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  message?: string;
  timestamp: number;
}

export interface PaginationQuery {
  page?: number;
  limit?: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  limit: number;
  totalPages: number;
}

// ========== 认证系统类型 ==========
export interface LoginRequest {
  username: string;
  password: string;
  remember?: boolean;
}

export interface LoginResponse {
  token: string;
  refresh_token: string;
  user: UserInfo;
  expires_in: number;
}

export interface UserInfo {
  id: string;
  username: string;
  email: string;
  role: UserRole;
  permissions: string[];
  created_at: string;
  last_login: string;
  is_active: boolean;
}

export enum UserRole {
  SuperAdmin = 'super_admin',
  Admin = 'admin',
  Trader = 'trader',
  Analyst = 'analyst',
  Viewer = 'viewer'
}

export interface RefreshTokenRequest {
  refresh_token: string;
}

export interface ChangePasswordRequest {
  current_password: string;
  new_password: string;
}

export interface CreateUserRequest {
  username: string;
  email: string;
  password: string;
  role: UserRole;
}

// ========== QingXi数据服务类型 ==========
export interface MarketData {
  symbol: string;
  exchange: string;
  price: number;
  volume: number;
  timestamp: number;
  bid: number;
  ask: number;
  high_24h: number;
  low_24h: number;
  change_24h: number;
  change_percent_24h: number;
}

export interface OrderBookLevel {
  price: number;
  quantity: number;
}

export interface OrderBook {
  symbol: string;
  exchange: string;
  bids: OrderBookLevel[];
  asks: OrderBookLevel[];
  timestamp: number;
  sequence: number;
}

export interface CollectorStatus {
  id: string;
  name: string;
  exchange: string;
  status: 'running' | 'stopped' | 'error' | 'connecting';
  last_update: string;
  total_symbols: number;
  active_connections: number;
  error_count: number;
  uptime_seconds: number;
}

export interface ArbitrageOpportunity {
  id: string;
  symbol: string;
  buy_exchange: string;
  sell_exchange: string;
  buy_price: number;
  sell_price: number;
  profit_amount: number;
  profit_percentage: number;
  volume: number;
  timestamp: string;
  status: 'active' | 'expired' | 'executed';
  risk_score: number;
}

// ========== 仪表板服务类型 ==========
export interface SankeyNodeData {
  id: string;
  name: string;
  color: string;
}

export interface SankeyLinkData {
  source: string;
  target: string;
  value: number;
  color: string;
}

export interface SankeyData {
  nodes: SankeyNodeData[];
  links: SankeyLinkData[];
}

export interface ProfitCurvePoint {
  timestamp: number;
  cumulative_profit: number;
  daily_profit: number;
  trade_count: number;
}

export interface ProfitCurveData {
  points: ProfitCurvePoint[];
  total_profit: number;
  total_trades: number;
  win_rate: number;
  max_drawdown: number;
  sharpe_ratio: number;
}

export interface FlowHistoryItem {
  id: string;
  timestamp: string;
  from_exchange: string;
  to_exchange: string;
  symbol: string;
  amount: number;
  profit: number;
  status: 'pending' | 'completed' | 'failed';
  execution_time_ms: number;
}

export interface DashboardStats {
  total_profit_24h: number;
  total_trades_24h: number;
  active_opportunities: number;
  system_uptime: number;
  success_rate: number;
  average_profit_per_trade: number;
}

// ========== 监控系统类型 ==========
export interface SystemMetrics {
  cpu_usage_percent: number;
  memory_usage_percent: number;
  disk_usage_percent: number;
  network_in_mbps: number;
  network_out_mbps: number;
  active_connections: number;
  response_time_ms: number;
  timestamp: number;
}

export interface HealthCheck {
  component: string;
  status: 'healthy' | 'unhealthy' | 'degraded';
  last_check: string;
  response_time_ms?: number;
  error_message?: string;
  uptime_percentage: number;
}

export interface Alert {
  id: string;
  type: 'info' | 'warning' | 'error' | 'critical';
  title: string;
  message: string;
  component: string;
  timestamp: string;
  acknowledged: boolean;
  resolved: boolean;
  metadata?: Record<string, any>;
}

export interface AlertRule {
  id: string;
  name: string;
  description: string;
  metric: string;
  operator: 'gt' | 'lt' | 'eq' | 'ne' | 'gte' | 'lte';
  threshold: number;
  severity: 'info' | 'warning' | 'error' | 'critical';
  enabled: boolean;
  cooldown_minutes: number;
}

export interface CreateAlertRuleRequest {
  name: string;
  description: string;
  metric: string;
  operator: 'gt' | 'lt' | 'eq' | 'ne' | 'gte' | 'lte';
  threshold: number;
  severity: 'info' | 'warning' | 'error' | 'critical';
  cooldown_minutes: number;
}

// ========== 系统控制类型 ==========
export interface SystemStatus {
  isRunning: boolean;
  status: 'running' | 'stopped' | 'starting' | 'stopping';
  uptime?: number;
  components: {
    qingxi: ComponentStatus;
    celue: ComponentStatus;
    orchestrator: ComponentStatus;
    monitoring: ComponentStatus;
    api_gateway: ComponentStatus;
  };
}

export interface ComponentStatus {
  status: 'running' | 'stopped' | 'error';
  lastHeartbeat?: number;
  errorMessage?: string;
}

// ========== WebSocket消息类型 ==========
export interface WebSocketMessage {
  type: string;
  data: any;
  timestamp: number;
}

export interface MarketDataMessage extends WebSocketMessage {
  type: 'market_data';
  data: MarketData;
}

export interface OrderBookMessage extends WebSocketMessage {
  type: 'orderbook';
  data: OrderBook;
}

export interface ArbitrageOpportunityMessage extends WebSocketMessage {
  type: 'arbitrage_opportunity';
  data: ArbitrageOpportunity;
}

export interface AlertMessage extends WebSocketMessage {
  type: 'alert';
  data: Alert;
}

export interface SystemStatusMessage extends WebSocketMessage {
  type: 'system_status';
  data: SystemStatus;
}

// ========== 错误类型 ==========
export interface ApiError {
  code: string;
  message: string;
  details?: any;
}

export class ArbitrageSDKError extends Error {
  public code: string;
  public details?: any;

  constructor(message: string, code: string = 'UNKNOWN_ERROR', details?: any) {
    super(message);
    this.name = 'ArbitrageSDKError';
    this.code = code;
    this.details = details;
  }
}

// ========== 配置类型 ==========
export interface SDKConfig {
  baseUrl: string;
  wsUrl?: string;
  apiKey?: string;
  timeout?: number;
  retryAttempts?: number;
  retryDelay?: number;
  enableLogging?: boolean;
}

// ========== 事件类型 ==========
export type EventCallback<T = any> = (data: T) => void;

export interface EventSubscription {
  unsubscribe: () => void;
}