// CeLue策略执行模块相关类型定义

export interface StrategyConfig {
  id: string;
  name: string;
  type: 'cross_exchange' | 'triangular' | 'market_making' | 'statistical_arbitrage';
  enabled: boolean;
  priority: number;
  description: string;
  parameters: Record<string, any>;
  risk_limits: RiskLimits;
  exchanges: string[];
  symbols: string[];
  created_at: string;
  updated_at: string;
  performance_metrics?: StrategyPerformance;
}

export interface RiskLimits {
  max_position_size_usd: number;
  max_daily_loss_usd: number;
  max_exposure_per_exchange: number;
  max_correlation_risk: number;
  stop_loss_percentage: number;
  max_drawdown_percentage: number;
  position_concentration_limit: number;
  leverage_limit: number;
}

export interface StrategyPerformance {
  total_pnl_usd: number;
  daily_pnl_usd: number;
  win_rate_percent: number;
  sharpe_ratio: number;
  max_drawdown_percent: number;
  total_trades: number;
  successful_trades: number;
  average_trade_duration_minutes: number;
  return_on_capital_percent: number;
}

export interface MLModelConfig {
  id: string;
  name: string;
  type: 'classification' | 'regression' | 'reinforcement' | 'time_series';
  framework: 'tensorflow' | 'pytorch' | 'sklearn' | 'lightgbm' | 'xgboost';
  version: string;
  status: 'training' | 'ready' | 'deployed' | 'deprecated' | 'failed';
  accuracy: number;
  training_data_size: number;
  training_started_at?: string;
  training_completed_at?: string;
  deployment_date?: string;
  hyperparameters: Record<string, any>;
  feature_importance?: Record<string, number>;
}

export interface ProductionAPIConfig {
  exchange: string;
  api_key: string;
  api_secret: string;
  passphrase?: string;
  sandbox_mode: boolean;
  rate_limit_per_second: number;
  timeout_seconds: number;
  retry_attempts: number;
  health_check_interval: number;
  order_types_supported: string[];
  min_order_sizes: Record<string, number>;
}

export interface ShadowTradingConfig {
  enabled: boolean;
  mode: 'simulation' | 'paper' | 'comparison';
  capital_usd: number;
  latency_simulation_ms: number;
  slippage_simulation_percent: number;
  commission_simulation_percent: number;
  market_impact_simulation: boolean;
  risk_free_rate_percent: number;
  benchmark_symbol: string;
}

export interface ApprovalWorkflowConfig {
  enabled: boolean;
  rules: ApprovalRule[];
  approvers: ApprovalUser[];
  default_timeout_hours: number;
  require_digital_signature: boolean;
  audit_log_retention_days: number;
  notification_channels: string[];
}

export interface ApprovalRule {
  id: string;
  name: string;
  description: string;
  config_type_pattern: string;
  change_types: ('create' | 'update' | 'delete')[];
  required_levels: string[];
  min_approvers: number;
  all_levels_required: boolean;
  timeout_seconds: number;
  auto_approve_conditions?: Record<string, any>;
}

export interface ApprovalUser {
  id: string;
  name: string;
  email: string;
  level: string;
  role: string;
  departments: string[];
  max_approval_amount_usd?: number;
  signature_required: boolean;
  notification_preferences: {
    email: boolean;
    sms: boolean;
    slack: boolean;
  };
}

export interface OrderInfo {
  order_id: string;
  client_order_id: string;
  exchange: string;
  symbol: string;
  side: 'buy' | 'sell';
  type: 'market' | 'limit' | 'stop_loss' | 'take_profit';
  quantity: number;
  price?: number;
  status: 'new' | 'partially_filled' | 'filled' | 'canceled' | 'rejected' | 'expired';
  filled_quantity: number;
  average_price?: number;
  fees_paid: number;
  created_at: string;
  updated_at: string;
  execution_report?: ExecutionReport;
}

export interface ExecutionReport {
  execution_id: string;
  order_id: string;
  trade_id: string;
  quantity: number;
  price: number;
  fees: number;
  liquidity: 'maker' | 'taker';
  executed_at: string;
  counterparty?: string;
  market_impact_bps?: number;
  slippage_bps?: number;
}

export interface ArbitrageTrade {
  id: string;
  strategy_id: string;
  opportunity_id: string;
  legs: TradeLeg[];
  status: 'pending' | 'executing' | 'completed' | 'failed' | 'partial';
  expected_profit_usd: number;
  actual_profit_usd?: number;
  execution_time_ms?: number;
  started_at: string;
  completed_at?: string;
  error_message?: string;
}

export interface TradeLeg {
  exchange: string;
  symbol: string;
  side: 'buy' | 'sell';
  quantity: number;
  price: number;
  order_id?: string;
  status: 'pending' | 'submitted' | 'filled' | 'failed';
  filled_quantity?: number;
  average_price?: number;
  fees?: number;
}