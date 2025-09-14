import { apiCall, HttpMethod } from '../api/apiClient';

// 交易服务相关类型定义
export interface Order {
  id: string;
  symbol: string;
  type: 'market' | 'limit' | 'stop';
  side: 'buy' | 'sell';
  amount: number;
  price?: number;
  status: 'pending' | 'filled' | 'cancelled' | 'rejected';
  fills: OrderFill[];
  created_at: string;
  updated_at: string;
}

export interface OrderFill {
  id: string;
  price: number;
  amount: number;
  fee: number;
  timestamp: string;
}

export interface Position {
  symbol: string;
  side: 'long' | 'short';
  size: number;
  entry_price: number;
  current_price: number;
  unrealized_pnl: number;
  realized_pnl: number;
  margin: number;
  leverage: number;
}

export interface FundInfo {
  total_balance: number;
  available_balance: number;
  locked_balance: number;
  currency: string;
  allocation: Record<string, number>;
}

export interface RiskMetrics {
  var: number; // Value at Risk
  max_drawdown: number;
  sharpe_ratio: number;
  exposure: number;
  leverage: number;
  concentration: number;
}

/**
 * 交易服务 - 41个API接口
 * 端口: 4005
 * 功能: 订单管理、仓位监控、风险控制、资金管理
 */
export class TradingService {
  
  // ==================== 订单监控API (15个) ====================
  
  /**
   * 获取活跃订单
   */
  async getActiveOrders(): Promise<Order[]> {
    return apiCall(HttpMethod.GET, '/orders/active');
  }
  
  /**
   * 获取历史订单
   */
  async getOrderHistory(limit: number = 100): Promise<Order[]> {
    return apiCall(HttpMethod.GET, `/orders/history?limit=${limit}`);
  }
  
  /**
   * 获取订单详情
   */
  async getOrder(id: string): Promise<Order> {
    return apiCall(HttpMethod.GET, `/orders/${id}`);
  }
  
  /**
   * 获取订单状态
   */
  async getOrderStatus(id: string): Promise<{ status: string }> {
    return apiCall(HttpMethod.GET, `/orders/${id}/status`);
  }
  
  /**
   * 获取订单成交
   */
  async getOrderFills(id: string): Promise<OrderFill[]> {
    return apiCall(HttpMethod.GET, `/orders/${id}/fills`);
  }
  
  /**
   * 取消订单
   */
  async cancelOrder(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/orders/${id}/cancel`);
  }
  
  /**
   * 修改订单
   */
  async modifyOrder(id: string, updates: { price?: number; amount?: number }): Promise<Order> {
    return apiCall(HttpMethod.PUT, `/orders/${id}/modify`, updates);
  }
  
  /**
   * 批量取消订单
   */
  async batchCancelOrders(ids: string[]): Promise<void> {
    return apiCall(HttpMethod.POST, '/orders/batch/cancel', { ids });
  }
  
  /**
   * 获取订单统计
   */
  async getOrderStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/orders/stats');
  }
  
  /**
   * 获取执行质量
   */
  async getExecutionQuality(): Promise<any> {
    return apiCall(HttpMethod.GET, '/orders/execution-quality');
  }
  
  /**
   * 获取订单延迟
   */
  async getOrderLatency(): Promise<{ latency: number }> {
    return apiCall(HttpMethod.GET, '/orders/latency');
  }
  
  /**
   * 分析滑点
   */
  async analyzeSlippage(): Promise<any> {
    return apiCall(HttpMethod.GET, '/orders/slippage');
  }
  
  /**
   * 获取被拒订单
   */
  async getRejectedOrders(): Promise<Order[]> {
    return apiCall(HttpMethod.GET, '/orders/rejected');
  }
  
  /**
   * 获取订单告警
   */
  async getOrderAlerts(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/orders/alerts');
  }
  
  /**
   * 获取执行性能
   */
  async getExecutionPerformance(): Promise<any> {
    return apiCall(HttpMethod.GET, '/orders/performance');
  }
  
  // ==================== 仓位监控API (12个) ====================
  
  /**
   * 获取当前仓位
   */
  async getCurrentPositions(): Promise<Position[]> {
    return apiCall(HttpMethod.GET, '/positions/current');
  }
  
  /**
   * 获取实时仓位
   */
  async getRealtimePositions(): Promise<Position[]> {
    return apiCall(HttpMethod.GET, '/positions/realtime');
  }
  
  /**
   * 按交易对获取仓位
   */
  async getPositionBySymbol(symbol: string): Promise<Position> {
    return apiCall(HttpMethod.GET, `/positions/${symbol}`);
  }
  
  /**
   * 获取仓位盈亏
   */
  async getPositionPnl(symbol: string): Promise<{ pnl: number; unrealized_pnl: number; realized_pnl: number }> {
    return apiCall(HttpMethod.GET, `/positions/${symbol}/pnl`);
  }
  
  /**
   * 获取仓位历史
   */
  async getPositionHistory(symbol: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/positions/${symbol}/history`);
  }
  
  /**
   * 获取总盈亏
   */
  async getTotalPnl(): Promise<{ total_pnl: number; unrealized_pnl: number; realized_pnl: number }> {
    return apiCall(HttpMethod.GET, '/positions/total-pnl');
  }
  
  /**
   * 获取风险敞口分析
   */
  async getExposureAnalysis(): Promise<any> {
    return apiCall(HttpMethod.GET, '/positions/exposure');
  }
  
  /**
   * 获取集中度分析
   */
  async getConcentrationAnalysis(): Promise<any> {
    return apiCall(HttpMethod.GET, '/positions/concentration');
  }
  
  /**
   * 获取相关性分析
   */
  async getCorrelationAnalysis(): Promise<any> {
    return apiCall(HttpMethod.GET, '/positions/correlation');
  }
  
  /**
   * 平仓
   */
  async closePosition(symbol: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/positions/${symbol}/close`);
  }
  
  /**
   * 对冲仓位
   */
  async hedgePosition(symbol: string): Promise<void> {
    return apiCall(HttpMethod.POST, '/positions/hedge', { symbol });
  }
  
  /**
   * 获取仓位汇总
   */
  async getPositionSummary(): Promise<any> {
    return apiCall(HttpMethod.GET, '/positions/summary');
  }
  
  // ==================== 资金管理API (14个) ====================
  
  /**
   * 获取账户余额
   */
  async getAccountBalance(): Promise<FundInfo> {
    return apiCall(HttpMethod.GET, '/funds/balance');
  }
  
  /**
   * 获取可用资金
   */
  async getAvailableFunds(): Promise<{ available: number; currency: string }> {
    return apiCall(HttpMethod.GET, '/funds/available');
  }
  
  /**
   * 获取冻结资金
   */
  async getLockedFunds(): Promise<{ locked: number; currency: string }> {
    return apiCall(HttpMethod.GET, '/funds/locked');
  }
  
  /**
   * 获取资金历史
   */
  async getFundsHistory(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/funds/history');
  }
  
  /**
   * 资金划转
   */
  async transferFunds(from: string, to: string, amount: number): Promise<void> {
    return apiCall(HttpMethod.POST, '/funds/transfer', { from, to, amount });
  }
  
  /**
   * 获取资金分配
   */
  async getFundsAllocation(): Promise<Record<string, number>> {
    return apiCall(HttpMethod.GET, '/funds/allocation');
  }
  
  /**
   * 设置资金分配
   */
  async setFundsAllocation(strategy: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/funds/allocation', { strategy });
  }
  
  /**
   * 获取资金利用率
   */
  async getFundsUtilization(): Promise<{ utilization: number }> {
    return apiCall(HttpMethod.GET, '/funds/utilization');
  }
  
  /**
   * 获取资金绩效
   */
  async getFundsPerformance(): Promise<any> {
    return apiCall(HttpMethod.GET, '/funds/performance');
  }
  
  /**
   * 资金再平衡
   */
  async rebalanceFunds(): Promise<void> {
    return apiCall(HttpMethod.POST, '/funds/rebalance');
  }
  
  /**
   * 获取资金限额
   */
  async getFundsLimits(): Promise<any> {
    return apiCall(HttpMethod.GET, '/funds/limits');
  }
  
  /**
   * 设置资金限额
   */
  async setFundsLimits(daily: number): Promise<void> {
    return apiCall(HttpMethod.PUT, '/funds/limits', { daily });
  }
  
  /**
   * 获取资金流向
   */
  async getFundsFlow(): Promise<any> {
    return apiCall(HttpMethod.GET, '/funds/flow');
  }
  
  /**
   * 优化资金配置
   */
  async optimizeFundsAllocation(): Promise<any> {
    return apiCall(HttpMethod.POST, '/funds/optimize');
  }
  
  // ==================== 风险控制API (未在文档中明确列出，但交易服务应包含) ====================
  
  /**
   * 获取风险指标
   */
  async getRiskMetrics(): Promise<RiskMetrics> {
    return apiCall(HttpMethod.GET, '/risk/metrics');
  }
  
  /**
   * 设置风险限额
   */
  async setRiskLimits(limits: any): Promise<void> {
    return apiCall(HttpMethod.PUT, '/risk/limits', limits);
  }
  
  /**
   * 获取风险告警
   */
  async getRiskAlerts(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/risk/alerts');
  }
  
  /**
   * 紧急止损
   */
  async emergencyStopLoss(): Promise<void> {
    return apiCall(HttpMethod.POST, '/risk/emergency-stop');
  }
}

// 导出单例实例
export const tradingService = new TradingService(); 

// 交易服务相关类型定义
export interface Order {
  id: string;
  symbol: string;
  type: 'market' | 'limit' | 'stop';
  side: 'buy' | 'sell';
  amount: number;
  price?: number;
  status: 'pending' | 'filled' | 'cancelled' | 'rejected';
  fills: OrderFill[];
  created_at: string;
  updated_at: string;
}

export interface OrderFill {
  id: string;
  price: number;
  amount: number;
  fee: number;
  timestamp: string;
}

export interface Position {
  symbol: string;
  side: 'long' | 'short';
  size: number;
  entry_price: number;
  current_price: number;
  unrealized_pnl: number;
  realized_pnl: number;
  margin: number;
  leverage: number;
}

export interface FundInfo {
  total_balance: number;
  available_balance: number;
  locked_balance: number;
  currency: string;
  allocation: Record<string, number>;
}

export interface RiskMetrics {
  var: number; // Value at Risk
  max_drawdown: number;
  sharpe_ratio: number;
  exposure: number;
  leverage: number;
  concentration: number;
}

/**
 * 交易服务 - 41个API接口
 * 端口: 4005
 * 功能: 订单管理、仓位监控、风险控制、资金管理
 */
export class TradingService {
  
  // ==================== 订单监控API (15个) ====================
  
  /**
   * 获取活跃订单
   */
  async getActiveOrders(): Promise<Order[]> {
    return apiCall(HttpMethod.GET, '/orders/active');
  }
  
  /**
   * 获取历史订单
   */
  async getOrderHistory(limit: number = 100): Promise<Order[]> {
    return apiCall(HttpMethod.GET, `/orders/history?limit=${limit}`);
  }
  
  /**
   * 获取订单详情
   */
  async getOrder(id: string): Promise<Order> {
    return apiCall(HttpMethod.GET, `/orders/${id}`);
  }
  
  /**
   * 获取订单状态
   */
  async getOrderStatus(id: string): Promise<{ status: string }> {
    return apiCall(HttpMethod.GET, `/orders/${id}/status`);
  }
  
  /**
   * 获取订单成交
   */
  async getOrderFills(id: string): Promise<OrderFill[]> {
    return apiCall(HttpMethod.GET, `/orders/${id}/fills`);
  }
  
  /**
   * 取消订单
   */
  async cancelOrder(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/orders/${id}/cancel`);
  }
  
  /**
   * 修改订单
   */
  async modifyOrder(id: string, updates: { price?: number; amount?: number }): Promise<Order> {
    return apiCall(HttpMethod.PUT, `/orders/${id}/modify`, updates);
  }
  
  /**
   * 批量取消订单
   */
  async batchCancelOrders(ids: string[]): Promise<void> {
    return apiCall(HttpMethod.POST, '/orders/batch/cancel', { ids });
  }
  
  /**
   * 获取订单统计
   */
  async getOrderStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/orders/stats');
  }
  
  /**
   * 获取执行质量
   */
  async getExecutionQuality(): Promise<any> {
    return apiCall(HttpMethod.GET, '/orders/execution-quality');
  }
  
  /**
   * 获取订单延迟
   */
  async getOrderLatency(): Promise<{ latency: number }> {
    return apiCall(HttpMethod.GET, '/orders/latency');
  }
  
  /**
   * 分析滑点
   */
  async analyzeSlippage(): Promise<any> {
    return apiCall(HttpMethod.GET, '/orders/slippage');
  }
  
  /**
   * 获取被拒订单
   */
  async getRejectedOrders(): Promise<Order[]> {
    return apiCall(HttpMethod.GET, '/orders/rejected');
  }
  
  /**
   * 获取订单告警
   */
  async getOrderAlerts(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/orders/alerts');
  }
  
  /**
   * 获取执行性能
   */
  async getExecutionPerformance(): Promise<any> {
    return apiCall(HttpMethod.GET, '/orders/performance');
  }
  
  // ==================== 仓位监控API (12个) ====================
  
  /**
   * 获取当前仓位
   */
  async getCurrentPositions(): Promise<Position[]> {
    return apiCall(HttpMethod.GET, '/positions/current');
  }
  
  /**
   * 获取实时仓位
   */
  async getRealtimePositions(): Promise<Position[]> {
    return apiCall(HttpMethod.GET, '/positions/realtime');
  }
  
  /**
   * 按交易对获取仓位
   */
  async getPositionBySymbol(symbol: string): Promise<Position> {
    return apiCall(HttpMethod.GET, `/positions/${symbol}`);
  }
  
  /**
   * 获取仓位盈亏
   */
  async getPositionPnl(symbol: string): Promise<{ pnl: number; unrealized_pnl: number; realized_pnl: number }> {
    return apiCall(HttpMethod.GET, `/positions/${symbol}/pnl`);
  }
  
  /**
   * 获取仓位历史
   */
  async getPositionHistory(symbol: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/positions/${symbol}/history`);
  }
  
  /**
   * 获取总盈亏
   */
  async getTotalPnl(): Promise<{ total_pnl: number; unrealized_pnl: number; realized_pnl: number }> {
    return apiCall(HttpMethod.GET, '/positions/total-pnl');
  }
  
  /**
   * 获取风险敞口分析
   */
  async getExposureAnalysis(): Promise<any> {
    return apiCall(HttpMethod.GET, '/positions/exposure');
  }
  
  /**
   * 获取集中度分析
   */
  async getConcentrationAnalysis(): Promise<any> {
    return apiCall(HttpMethod.GET, '/positions/concentration');
  }
  
  /**
   * 获取相关性分析
   */
  async getCorrelationAnalysis(): Promise<any> {
    return apiCall(HttpMethod.GET, '/positions/correlation');
  }
  
  /**
   * 平仓
   */
  async closePosition(symbol: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/positions/${symbol}/close`);
  }
  
  /**
   * 对冲仓位
   */
  async hedgePosition(symbol: string): Promise<void> {
    return apiCall(HttpMethod.POST, '/positions/hedge', { symbol });
  }
  
  /**
   * 获取仓位汇总
   */
  async getPositionSummary(): Promise<any> {
    return apiCall(HttpMethod.GET, '/positions/summary');
  }
  
  // ==================== 资金管理API (14个) ====================
  
  /**
   * 获取账户余额
   */
  async getAccountBalance(): Promise<FundInfo> {
    return apiCall(HttpMethod.GET, '/funds/balance');
  }
  
  /**
   * 获取可用资金
   */
  async getAvailableFunds(): Promise<{ available: number; currency: string }> {
    return apiCall(HttpMethod.GET, '/funds/available');
  }
  
  /**
   * 获取冻结资金
   */
  async getLockedFunds(): Promise<{ locked: number; currency: string }> {
    return apiCall(HttpMethod.GET, '/funds/locked');
  }
  
  /**
   * 获取资金历史
   */
  async getFundsHistory(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/funds/history');
  }
  
  /**
   * 资金划转
   */
  async transferFunds(from: string, to: string, amount: number): Promise<void> {
    return apiCall(HttpMethod.POST, '/funds/transfer', { from, to, amount });
  }
  
  /**
   * 获取资金分配
   */
  async getFundsAllocation(): Promise<Record<string, number>> {
    return apiCall(HttpMethod.GET, '/funds/allocation');
  }
  
  /**
   * 设置资金分配
   */
  async setFundsAllocation(strategy: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/funds/allocation', { strategy });
  }
  
  /**
   * 获取资金利用率
   */
  async getFundsUtilization(): Promise<{ utilization: number }> {
    return apiCall(HttpMethod.GET, '/funds/utilization');
  }
  
  /**
   * 获取资金绩效
   */
  async getFundsPerformance(): Promise<any> {
    return apiCall(HttpMethod.GET, '/funds/performance');
  }
  
  /**
   * 资金再平衡
   */
  async rebalanceFunds(): Promise<void> {
    return apiCall(HttpMethod.POST, '/funds/rebalance');
  }
  
  /**
   * 获取资金限额
   */
  async getFundsLimits(): Promise<any> {
    return apiCall(HttpMethod.GET, '/funds/limits');
  }
  
  /**
   * 设置资金限额
   */
  async setFundsLimits(daily: number): Promise<void> {
    return apiCall(HttpMethod.PUT, '/funds/limits', { daily });
  }
  
  /**
   * 获取资金流向
   */
  async getFundsFlow(): Promise<any> {
    return apiCall(HttpMethod.GET, '/funds/flow');
  }
  
  /**
   * 优化资金配置
   */
  async optimizeFundsAllocation(): Promise<any> {
    return apiCall(HttpMethod.POST, '/funds/optimize');
  }
  
  // ==================== 风险控制API (未在文档中明确列出，但交易服务应包含) ====================
  
  /**
   * 获取风险指标
   */
  async getRiskMetrics(): Promise<RiskMetrics> {
    return apiCall(HttpMethod.GET, '/risk/metrics');
  }
  
  /**
   * 设置风险限额
   */
  async setRiskLimits(limits: any): Promise<void> {
    return apiCall(HttpMethod.PUT, '/risk/limits', limits);
  }
  
  /**
   * 获取风险告警
   */
  async getRiskAlerts(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/risk/alerts');
  }
  
  /**
   * 紧急止损
   */
  async emergencyStopLoss(): Promise<void> {
    return apiCall(HttpMethod.POST, '/risk/emergency-stop');
  }
}

// 导出单例实例
export const tradingService = new TradingService(); 