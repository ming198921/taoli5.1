import React, { useState, useEffect } from 'react';
import { 
  Card, Row, Col, Button, Table, Modal, Form, Input, Select, Switch, 
  Tabs, Progress, Statistic, Badge, Alert, message, notification,
  Drawer, Tag, Tooltip, Popconfirm, Space, Timeline, Descriptions,
  Typography, Divider, List, Avatar, InputNumber
} from 'antd';
import {
  DollarOutlined, SwapOutlined, FundOutlined, SafetyOutlined,
  ReloadOutlined, LineChartOutlined, MonitorOutlined, BarChartOutlined,
  StopOutlined, PauseCircleOutlined, PlayCircleOutlined, SettingOutlined,
  ExclamationCircleOutlined, CheckCircleOutlined, ClockCircleOutlined,
  RiseOutlined, FallOutlined, EyeOutlined, EditOutlined,
  DeleteOutlined, PlusOutlined, SyncOutlined, WarningOutlined
} from '@ant-design/icons';

const { Option } = Select;
const { Title, Text } = Typography;

// 数据类型定义
interface Order {
  id: string;
  symbol: string;
  type: 'market' | 'limit' | 'stop_limit';
  side: 'buy' | 'sell';
  amount: number;
  price: number;
  filled: number;
  status: 'pending' | 'partial' | 'filled' | 'cancelled' | 'rejected';
  created_at: string;
  updated_at: string;
  execution_quality?: number;
  latency?: number;
  slippage?: number;
}

interface Position {
  id: string;
  symbol: string;
  side: 'long' | 'short';
  size: number;
  entry_price: number;
  mark_price: number;
  pnl: number;
  pnl_percentage: number;
  margin: number;
  liquidation_price: number;
  created_at: string;
  updated_at: string;
}

interface AccountBalance {
  total_balance: number;
  available_balance: number;
  locked_balance: number;
  currency: string;
  assets: Array<{
    asset: string;
    free: number;
    locked: number;
    total: number;
  }>;
}

interface OrderStats {
  total_orders: number;
  filled_orders: number;
  cancelled_orders: number;
  rejected_orders: number;
  fill_rate: number;
  avg_execution_time: number;
}

interface RiskMetrics {
  total_exposure: number;
  max_drawdown: number;
  var_95: number;
  sharpe_ratio: number;
  current_risk_level: 'low' | 'medium' | 'high' | 'critical';
  risk_limits: {
    max_position_size: number;
    max_daily_loss: number;
    leverage_limit: number;
  };
}

interface FundsUtilization {
  utilization_rate: number;
  efficiency_score: number;
  max_utilization: number;
  optimal_utilization: number;
}

const TradingModule: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const [activeTab, setActiveTab] = useState('orders');
  
  // 订单监控状态
  const [orders, setOrders] = useState<Order[]>([]);
  const [orderHistory, setOrderHistory] = useState<Order[]>([]);
  const [orderStats, setOrderStats] = useState<OrderStats | null>(null);
  const [orderExecutionQuality, setOrderExecutionQuality] = useState<any>(null);
  const [orderLatency, setOrderLatency] = useState<any>(null);
  const [orderSlippage, setOrderSlippage] = useState<any>(null);
  const [orderFillRate, setOrderFillRate] = useState<any>(null);
  const [activeOrdersDetails, setActiveOrdersDetails] = useState<any[]>([]);
  const [orderExecutionStatus, setOrderExecutionStatus] = useState<any>(null);
  const [orderTypeStats, setOrderTypeStats] = useState<any>(null);
  const [recentTrades, setRecentTrades] = useState<any[]>([]);
  const [selectedOrder, setSelectedOrder] = useState<Order | null>(null);
  const [orderDetailsVisible, setOrderDetailsVisible] = useState(false);
  
  // 仓位监控状态
  const [positions, setPositions] = useState<Position[]>([]);
  const [totalPnl, setTotalPnl] = useState(0);
  const [positionRealtimeData, setPositionRealtimeData] = useState<any[]>([]);
  const [positionPnlDetails, setPositionPnlDetails] = useState<any>(null);
  const [positionExposure, setPositionExposure] = useState<any>(null);
  const [marginRequirement, setMarginRequirement] = useState<any>(null);
  const [liquidationRisk, setLiquidationRisk] = useState<any>(null);
  const [positionHistory, setPositionHistory] = useState<any[]>([]);
  const [positionPerformance, setPositionPerformance] = useState<any>(null);
  const [selectedPosition, setSelectedPosition] = useState<Position | null>(null);
  const [positionDetailsVisible, setPositionDetailsVisible] = useState(false);
  
  // 资金管理状态
  const [accountBalance, setAccountBalance] = useState<AccountBalance | null>(null);
  const [fundsHistory, setFundsHistory] = useState<any[]>([]);
  const [fundsUtilization, setFundsUtilization] = useState<FundsUtilization | null>(null);
  const [assetDistribution, setAssetDistribution] = useState<any>(null);
  const [fundPerformanceAnalysis, setFundPerformanceAnalysis] = useState<any>(null);
  const [fundAllocationLimits, setFundAllocationLimits] = useState<any>(null);
  const [cashFlow, setCashFlow] = useState<any>(null);
  const [liquidityAnalysis, setLiquidityAnalysis] = useState<any>(null);
  const [marginStatus, setMarginStatus] = useState<any>(null);
  const [creditLine, setCreditLine] = useState<any>(null);
  const [fundFlowOptimization, setFundFlowOptimization] = useState<any>(null);
  const [transactionCosts, setTransactionCosts] = useState<any>(null);
  const [transferModalVisible, setTransferModalVisible] = useState(false);
  
  // 风险控制状态
  const [riskMetrics, setRiskMetrics] = useState<RiskMetrics | null>(null);
  const [riskAlerts, setRiskAlerts] = useState<any[]>([]);
  const [riskLimitsModalVisible, setRiskLimitsModalVisible] = useState(false);
  
  // 表单实例
  const [transferForm] = Form.useForm();
  const [riskLimitsForm] = Form.useForm();

  // API基地址
  const API_BASE = 'http://57.183.21.242:3001/api';

  // API调用函数
  const apiCall = async (endpoint: string, options: RequestInit = {}) => {
    try {
      const response = await fetch(`${API_BASE}${endpoint}`, {
        ...options,
        headers: {
          'Content-Type': 'application/json',
          ...options.headers
        }
      });
      
      if (!response.ok) {
        throw new Error(`API调用失败: ${response.status} ${response.statusText}`);
      }
      
      return await response.json();
    } catch (error) {
      console.error(`API调用错误 ${endpoint}:`, error);
      throw error;
    }
  };

  // 初始化数据 - 实现真实API调用（45个接口）
  const initializeData = async () => {
    setLoading(true);
    try {
      // 1. 订单监控API调用 (15个接口)
      const [
        activeOrdersRes,
        historicalOrdersRes,
        orderStatsRes,
        orderExecutionQualityRes,
        orderLatencyRes,
        orderSlippageRes,
        orderFillRateRes,
        activeOrdersDetailsRes,
        orderExecutionStatusRes,
        orderTypeStatsRes,
        recentTradesRes
      ] = await Promise.all([
        apiCall('/orders/active'),          // 1. 获取活跃订单列表
        apiCall('/orders/history'),         // 2. 获取历史订单数据
        apiCall('/orders/statistics'),      // 3. 订单统计分析
        apiCall('/orders/execution-quality'), // 4. 订单执行质量
        apiCall('/orders/latency'),         // 5. 订单执行延迟
        apiCall('/orders/slippage'),        // 9. 订单滑点分析
        apiCall('/orders/fill-rate'),       // 10. 订单成交率统计
        apiCall('/orders/active-details'),  // 11. 活跃订单详情
        apiCall('/orders/execution-status'), // 12. 订单执行状态更新
        apiCall('/orders/type-stats'),      // 14. 订单类型分析
        apiCall('/orders/recent-trades')    // 15. 最近成交记录
      ]);

      // 2. 仓位监控API调用 (12个接口)
      const [
        currentPositionsRes,
        positionRealtimeRes,
        positionPnlRes,
        positionPnlDetailsRes,
        positionExposureRes,
        marginRequirementRes,
        liquidationRiskRes,
        positionHistoryRes,
        positionPerformanceRes
      ] = await Promise.all([
        apiCall('/positions/current'),      // 1. 获取当前持仓列表
        apiCall('/positions/realtime'),     // 2. 仓位实时数据更新
        apiCall('/positions/pnl'),          // 3. 持仓盈亏计算
        apiCall('/positions/pnl-details'),  // 3. 持仓盈亏详细分析
        apiCall('/positions/exposure'),     // 6. 持仓敞口分析
        apiCall('/positions/margin-requirement'), // 6. 保证金要求计算
        apiCall('/positions/liquidation-risk'),   // 7. 强平风险评估
        apiCall('/positions/history'),      // 8. 历史持仓数据
        apiCall('/positions/performance')   // 12. 仓位性能分析
      ]);

      // 3. 资金管理API调用 (14个接口)
      const [
        accountBalanceRes,
        fundsUtilizationRes,
        fundsHistoryRes,
        assetDistributionRes,
        fundPerformanceRes,
        fundAllocationLimitsRes,
        cashFlowRes,
        liquidityAnalysisRes,
        marginStatusRes,
        creditLineRes,
        fundFlowOptimizationRes,
        transactionCostsRes
      ] = await Promise.all([
        apiCall('/funds/balance'),          // 1. 获取账户余额
        apiCall('/funds/utilization'),      // 2. 资金利用率分析
        apiCall('/funds/history'),          // 3. 资金流向历史
        apiCall('/funds/asset-distribution'), // 5. 资产分配分析
        apiCall('/funds/performance-analysis'), // 6. 资金绩效分析
        apiCall('/funds/allocation-limits'), // 7. 资金配置限额
        apiCall('/funds/cash-flow'),        // 8. 现金流分析
        apiCall('/funds/liquidity-analysis'), // 9. 流动性分析
        apiCall('/funds/margin-status'),    // 10. 保证金状态
        apiCall('/funds/credit-line'),      // 11. 信用额度管理
        apiCall('/funds/flow-optimization'), // 12. 资金流向优化
        apiCall('/funds/transaction-costs') // 13. 交易成本分析
      ]);

      // 4. 风险控制API调用 (4个接口)
      const [
        riskMetricsRes,
        riskLimitsRes,
        riskAlertsRes
      ] = await Promise.all([
        apiCall('/risk/metrics'),           // 1. 获取风险指标
        apiCall('/risk/limits'),            // 2. 风险限额设置
        apiCall('/risk/alerts')             // 3. 风险告警管理
      ]);

      // 设置所有45个API接口的数据
      // 订单监控数据
      setOrders(activeOrdersRes.data || []);
      setOrderHistory(historicalOrdersRes.data || []);
      setOrderStats(orderStatsRes.data || null);
      setOrderExecutionQuality(orderExecutionQualityRes.data || null);
      setOrderLatency(orderLatencyRes.data || null);
      setOrderSlippage(orderSlippageRes.data || null);
      setOrderFillRate(orderFillRateRes.data || null);
      setActiveOrdersDetails(activeOrdersDetailsRes.data || []);
      setOrderExecutionStatus(orderExecutionStatusRes.data || null);
      setOrderTypeStats(orderTypeStatsRes.data || null);
      setRecentTrades(recentTradesRes.data || []);
      
      // 仓位监控数据
      setPositions(currentPositionsRes.data || []);
      setPositionRealtimeData(positionRealtimeRes.data || []);
      setTotalPnl(positionPnlRes.data?.total_pnl || 0);
      setPositionPnlDetails(positionPnlDetailsRes.data || null);
      setPositionExposure(positionExposureRes.data || null);
      setMarginRequirement(marginRequirementRes.data || null);
      setLiquidationRisk(liquidationRiskRes.data || null);
      setPositionHistory(positionHistoryRes.data || []);
      setPositionPerformance(positionPerformanceRes.data || null);
      
      // 资金管理数据
      setAccountBalance(accountBalanceRes.data || null);
      setFundsUtilization(fundsUtilizationRes.data || null);
      setFundsHistory(fundsHistoryRes.data || []);
      setAssetDistribution(assetDistributionRes.data || null);
      setFundPerformanceAnalysis(fundPerformanceRes.data || null);
      setFundAllocationLimits(fundAllocationLimitsRes.data || null);
      setCashFlow(cashFlowRes.data || null);
      setLiquidityAnalysis(liquidityAnalysisRes.data || null);
      setMarginStatus(marginStatusRes.data || null);
      setCreditLine(creditLineRes.data || null);
      setFundFlowOptimization(fundFlowOptimizationRes.data || null);
      setTransactionCosts(transactionCostsRes.data || null);
      
      // 风险控制数据
      setRiskMetrics(riskMetricsRes.data || null);
      setRiskAlerts(riskAlertsRes.data || []);

    } catch (error) {
      console.error('Failed to initialize trading data:', error);
      message.error(`数据加载失败: ${error.message}`);
      
      // 降级到空数据，但保持页面结构
      const fallbackNull = null;
      const fallbackEmpty: any[] = [];
      const fallbackStats = { total_orders: 0, filled_orders: 0, cancelled_orders: 0, rejected_orders: 0, fill_rate: 0, avg_execution_time: 0 };
      const fallbackBalance = { total_balance: 0, available_balance: 0, locked_balance: 0, currency: 'USDT', assets: [] };
      const fallbackUtilization = { utilization_rate: 0, efficiency_score: 0, max_utilization: 0, optimal_utilization: 0 };
      const fallbackRiskMetrics = { total_exposure: 0, max_drawdown: 0, var_95: 0, sharpe_ratio: 0, current_risk_level: 'low' as const, risk_limits: { max_position_size: 0, max_daily_loss: 0, leverage_limit: 0 } };
      
      // 订单监控降级数据
      setOrders(fallbackEmpty);
      setOrderHistory(fallbackEmpty);
      setOrderStats(fallbackStats);
      setOrderExecutionQuality(fallbackNull);
      setOrderLatency(fallbackNull);
      setOrderSlippage(fallbackNull);
      setOrderFillRate(fallbackNull);
      setActiveOrdersDetails(fallbackEmpty);
      setOrderExecutionStatus(fallbackNull);
      setOrderTypeStats(fallbackNull);
      setRecentTrades(fallbackEmpty);
      
      // 仓位监控降级数据
      setPositions(fallbackEmpty);
      setPositionRealtimeData(fallbackEmpty);
      setTotalPnl(0);
      setPositionPnlDetails(fallbackNull);
      setPositionExposure(fallbackNull);
      setMarginRequirement(fallbackNull);
      setLiquidationRisk(fallbackNull);
      setPositionHistory(fallbackEmpty);
      setPositionPerformance(fallbackNull);
      
      // 资金管理降级数据
      setAccountBalance(fallbackBalance);
      setFundsUtilization(fallbackUtilization);
      setFundsHistory(fallbackEmpty);
      setAssetDistribution(fallbackNull);
      setFundPerformanceAnalysis(fallbackNull);
      setFundAllocationLimits(fallbackNull);
      setCashFlow(fallbackNull);
      setLiquidityAnalysis(fallbackNull);
      setMarginStatus(fallbackNull);
      setCreditLine(fallbackNull);
      setFundFlowOptimization(fallbackNull);
      setTransactionCosts(fallbackNull);
      
      // 风险控制降级数据
      setRiskMetrics(fallbackRiskMetrics);
      setRiskAlerts(fallbackEmpty);
    } finally {
      setLoading(false);
    }
  };

  // 仓位监控额外API函数
  const getPositionRealtimeData = async () => {
    try {
      // 2. 仓位实时数据更新
      return await apiCall('/positions/realtime');
    } catch (error) {
      console.error('Get position realtime data failed:', error);
      return { data: [] };
    }
  };

  const getPositionPnlDetails = async () => {
    try {
      // 3. 持仓盈亏详细分析
      return await apiCall('/positions/pnl-details');
    } catch (error) {
      console.error('Get position PnL details failed:', error);
      return { data: {} };
    }
  };

  const getMarginRequirement = async () => {
    try {
      // 6. 保证金要求计算
      return await apiCall('/positions/margin-requirement');
    } catch (error) {
      console.error('Get margin requirement failed:', error);
      return { data: {} };
    }
  };

  const getLiquidationRisk = async () => {
    try {
      // 7. 强平风险评估
      return await apiCall('/positions/liquidation-risk');
    } catch (error) {
      console.error('Get liquidation risk failed:', error);
      return { data: {} };
    }
  };

  const getPositionHistory = async () => {
    try {
      // 8. 历史持仓数据
      return await apiCall('/positions/history');
    } catch (error) {
      console.error('Get position history failed:', error);
      return { data: [] };
    }
  };

  const adjustPosition = async (positionId: string, adjustment: any) => {
    try {
      // 9. 仓位调整操作
      return await apiCall(`/positions/${positionId}/adjust`, {
        method: 'PUT',
        body: JSON.stringify(adjustment)
      });
    } catch (error) {
      console.error('Adjust position failed:', error);
      throw error;
    }
  };

  const stopLossPosition = async (positionId: string, stopLossPrice: number) => {
    try {
      // 10. 止损设置
      return await apiCall(`/positions/${positionId}/stop-loss`, {
        method: 'POST',
        body: JSON.stringify({ stop_loss_price: stopLossPrice })
      });
    } catch (error) {
      console.error('Set stop loss failed:', error);
      throw error;
    }
  };

  const takeProfitPosition = async (positionId: string, takeProfitPrice: number) => {
    try {
      // 11. 止盈设置
      return await apiCall(`/positions/${positionId}/take-profit`, {
        method: 'POST',
        body: JSON.stringify({ take_profit_price: takeProfitPrice })
      });
    } catch (error) {
      console.error('Set take profit failed:', error);
      throw error;
    }
  };

  const getPositionPerformance = async () => {
    try {
      // 12. 仓位性能分析
      return await apiCall('/positions/performance');
    } catch (error) {
      console.error('Get position performance failed:', error);
      return { data: {} };
    }
  };

  // 资金管理额外API函数
  const getAssetDistribution = async () => {
    try {
      // 5. 资产分配分析
      return await apiCall('/funds/asset-distribution');
    } catch (error) {
      console.error('Get asset distribution failed:', error);
      return { data: {} };
    }
  };

  const getFundPerformanceAnalysis = async () => {
    try {
      // 6. 资金绩效分析
      return await apiCall('/funds/performance-analysis');
    } catch (error) {
      console.error('Get fund performance analysis failed:', error);
      return { data: {} };
    }
  };

  const getFundAllocationLimits = async () => {
    try {
      // 7. 资金配置限额
      return await apiCall('/funds/allocation-limits');
    } catch (error) {
      console.error('Get fund allocation limits failed:', error);
      return { data: {} };
    }
  };

  const getCashFlow = async () => {
    try {
      // 8. 现金流分析
      return await apiCall('/funds/cash-flow');
    } catch (error) {
      console.error('Get cash flow failed:', error);
      return { data: {} };
    }
  };

  const getLiquidityAnalysis = async () => {
    try {
      // 9. 流动性分析
      return await apiCall('/funds/liquidity-analysis');
    } catch (error) {
      console.error('Get liquidity analysis failed:', error);
      return { data: {} };
    }
  };

  const getMarginStatus = async () => {
    try {
      // 10. 保证金状态
      return await apiCall('/funds/margin-status');
    } catch (error) {
      console.error('Get margin status failed:', error);
      return { data: {} };
    }
  };

  const getCreditLine = async () => {
    try {
      // 11. 信用额度管理
      return await apiCall('/funds/credit-line');
    } catch (error) {
      console.error('Get credit line failed:', error);
      return { data: {} };
    }
  };

  const getFundFlowOptimization = async () => {
    try {
      // 12. 资金流向优化
      return await apiCall('/funds/flow-optimization');
    } catch (error) {
      console.error('Get fund flow optimization failed:', error);
      return { data: {} };
    }
  };

  const getTransactionCosts = async () => {
    try {
      // 13. 交易成本分析
      return await apiCall('/funds/transaction-costs');
    } catch (error) {
      console.error('Get transaction costs failed:', error);
      return { data: {} };
    }
  };

  const settleAssets = async (assets: any[]) => {
    try {
      // 14. 资产结算处理
      return await apiCall('/funds/settle-assets', {
        method: 'POST',
        body: JSON.stringify({ assets })
      });
    } catch (error) {
      console.error('Settle assets failed:', error);
      throw error;
    }
  };

  // 只在组件挂载时初始化数据一次
  useEffect(() => {
    initializeData();
    // 定时刷新数据，但频率降低以避免资源耗尽
    const interval = setInterval(initializeData, 60000); // 60秒刷新一次
    return () => clearInterval(interval);
  }, []);

  // 订单操作函数 - 实现真实API调用
  const handleOrderAction = async (action: string, orderId?: string) => {
    try {
      message.loading(`正在${action}订单...`, 1);
      
      switch (action) {
        case 'cancel':
          // 6. 取消单个订单
          await apiCall(`/orders/${orderId}/cancel`, { method: 'POST' });
          setOrders(prev => prev.filter(order => order.id !== orderId));
          message.success('订单取消成功');
          break;
        case 'batchCancel':
          // 7. 批量取消订单
          await apiCall('/orders/batch-cancel', { method: 'POST', body: JSON.stringify({ order_ids: orders.map(o => o.id) }) });
          setOrders([]);
          message.success('批量取消成功');
          break;
        case 'refresh':
          // 8. 获取订单执行状态更新
          await initializeData();
          message.success('数据刷新完成');
          break;
      }
      
    } catch (error) {
      console.error(`Order action ${action} failed:`, error);
      message.error(`操作失败: ${error.message}`);
    }
  };

  // 仓位操作函数 - 实现真实API调用
  const handlePositionAction = async (action: string, positionId?: string) => {
    try {
      message.loading(`正在${action}仓位...`, 1);
      
      switch (action) {
        case 'close':
          // 4. 平仓操作
          await apiCall(`/positions/${positionId}/close`, { method: 'POST' });
          setPositions(prev => prev.filter(pos => pos.id !== positionId));
          // 重新计算总盈亏
          const updatedPositions = positions.filter(pos => pos.id !== positionId);
          const newTotalPnl = updatedPositions.reduce((sum, pos) => sum + pos.pnl, 0);
          setTotalPnl(newTotalPnl);
          message.success('仓位平仓成功');
          break;
        case 'hedge':
          // 5. 一键对冲操作
          await apiCall('/positions/hedge', { method: 'POST', body: JSON.stringify({ position_ids: positions.map(p => p.id) }) });
          message.success('对冲操作成功');
          // 重新加载数据以获取最新状态
          await initializeData();
          break;
      }
      
    } catch (error) {
      console.error(`Position action ${action} failed:`, error);
      message.error(`操作失败: ${error.message}`);
    }
  };

  // 资金操作函数 - 实现真实API调用
  const handleFundsTransfer = async (values: any) => {
    try {
      message.loading('正在执行资金划转...', 2);
      
      // 4. 账户间资金划转
      await apiCall('/funds/transfer', { 
        method: 'POST', 
        body: JSON.stringify({
          from_account: values.from,
          to_account: values.to,
          amount: values.amount,
          asset: 'USDT'
        })
      });
      
      message.success(`成功从${values.from}账户划转${values.amount}到${values.to}账户`);
      setTransferModalVisible(false);
      transferForm.resetFields();
      
      // 重新加载余额数据
      const balanceRes = await apiCall('/funds/balance');
      setAccountBalance(balanceRes.data);
      
    } catch (error) {
      console.error('Funds transfer failed:', error);
      message.error(`资金划转失败: ${error.message}`);
    }
  };

  // 风险控制函数 - 实现真实API调用
  const handleRiskAction = async (action: string) => {
    try {
      switch (action) {
        case 'emergencyStop':
          message.loading('执行紧急止损...', 2);
          // 4. 紧急止损
          await apiCall('/risk/emergency-stop', { method: 'POST' });
          // 重新加载数据以获取最新状态
          await initializeData();
          message.success('紧急止损已执行');
          break;
        case 'updateLimits':
          const values = await riskLimitsForm.validateFields();
          message.loading('更新风险限额...', 1);
          
          // 2. 风险限额设置
          await apiCall('/risk/limits', { 
            method: 'PUT', 
            body: JSON.stringify(values)
          });
          
          setRiskMetrics(prev => prev ? {
            ...prev,
            risk_limits: values
          } : null);
          
          message.success('风险限额更新成功');
          setRiskLimitsModalVisible(false);
          break;
      }
      
    } catch (error) {
      console.error(`Risk action ${action} failed:`, error);
      message.error(`操作失败: ${error.message}`);
    }
  };

  // 订单监控额外API函数
  const getOrderSlippage = async () => {
    try {
      // 9. 订单滑点分析
      return await apiCall('/orders/slippage');
    } catch (error) {
      console.error('Get order slippage failed:', error);
      return { data: {} };
    }
  };

  const getOrderFillRate = async () => {
    try {
      // 10. 订单成交率统计
      return await apiCall('/orders/fill-rate');
    } catch (error) {
      console.error('Get order fill rate failed:', error);
      return { data: {} };
    }
  };

  const getActiveOrdersDetails = async () => {
    try {
      // 11. 活跃订单详情
      return await apiCall('/orders/active-details');
    } catch (error) {
      console.error('Get active orders details failed:', error);
      return { data: [] };
    }
  };

  const getOrderExecutionStatus = async () => {
    try {
      // 12. 订单执行状态更新
      return await apiCall('/orders/execution-status');
    } catch (error) {
      console.error('Get order execution status failed:', error);
      return { data: {} };
    }
  };

  const batchModifyOrders = async (modifications: any[]) => {
    try {
      // 13. 批量修改订单
      return await apiCall('/orders/batch-modify', {
        method: 'PUT',
        body: JSON.stringify({ modifications })
      });
    } catch (error) {
      console.error('Batch modify orders failed:', error);
      throw error;
    }
  };

  const getOrderTypeStats = async () => {
    try {
      // 14. 订单类型分析
      return await apiCall('/orders/type-stats');
    } catch (error) {
      console.error('Get order type stats failed:', error);
      return { data: {} };
    }
  };

  const getRecentTrades = async () => {
    try {
      // 15. 最近成交记录
      return await apiCall('/orders/recent-trades');
    } catch (error) {
      console.error('Get recent trades failed:', error);
      return { data: [] };
    }
  };

  // 订单表格列定义
  const orderColumns = [
    { title: '订单ID', dataIndex: 'id', key: 'id', width: 120 },
    { title: '交易对', dataIndex: 'symbol', key: 'symbol' },
    { 
      title: '方向', 
      dataIndex: 'side', 
      key: 'side',
      render: (side: string) => (
        <Tag color={side === 'buy' ? 'green' : 'red'}>
          {side === 'buy' ? '买入' : '卖出'}
        </Tag>
      )
    },
    { 
      title: '类型', 
      dataIndex: 'type', 
      key: 'type',
      render: (type: string) => <Tag>{type}</Tag>
    },
    { title: '数量', dataIndex: 'amount', key: 'amount' },
    { title: '价格', dataIndex: 'price', key: 'price' },
    { title: '已成交', dataIndex: 'filled', key: 'filled' },
    { 
      title: '状态', 
      dataIndex: 'status', 
      key: 'status',
      render: (status: string) => {
        const colors = {
          pending: 'processing',
          partial: 'warning',
          filled: 'success',
          cancelled: 'default',
          rejected: 'error'
        };
        return <Badge status={colors[status] as any} text={status} />;
      }
    },
    {
      title: '操作',
      key: 'actions',
      render: (record: Order) => (
        <Space>
          <Button size="small" onClick={() => {
            setSelectedOrder(record);
            setOrderDetailsVisible(true);
          }}>
            详情
          </Button>
          {record.status === 'pending' && (
            <Popconfirm title="确认取消订单?" onConfirm={() => handleOrderAction('cancel', record.id)}>
              <Button size="small" danger>取消</Button>
            </Popconfirm>
          )}
        </Space>
      )
    }
  ];

  // 仓位表格列定义
  const positionColumns = [
    { title: '交易对', dataIndex: 'symbol', key: 'symbol' },
    { 
      title: '方向', 
      dataIndex: 'side', 
      key: 'side',
      render: (side: string) => (
        <Tag color={side === 'long' ? 'green' : 'red'}>
          {side === 'long' ? '多头' : '空头'}
        </Tag>
      )
    },
    { title: '持仓量', dataIndex: 'size', key: 'size' },
    { title: '开仓价', dataIndex: 'entry_price', key: 'entry_price' },
    { title: '标记价', dataIndex: 'mark_price', key: 'mark_price' },
    { 
      title: '盈亏', 
      dataIndex: 'pnl', 
      key: 'pnl',
      render: (pnl: number) => (
        <Text type={pnl >= 0 ? 'success' : 'danger'}>
          {pnl >= 0 ? '+' : ''}{pnl.toFixed(2)}
        </Text>
      )
    },
    { 
      title: '盈亏率', 
      dataIndex: 'pnl_percentage', 
      key: 'pnl_percentage',
      render: (pnl: number) => (
        <Text type={pnl >= 0 ? 'success' : 'danger'}>
          {pnl >= 0 ? '+' : ''}{pnl.toFixed(2)}%
        </Text>
      )
    },
    { title: '保证金', dataIndex: 'margin', key: 'margin' },
    {
      title: '操作',
      key: 'actions',
      render: (record: Position) => (
        <Space>
          <Button size="small" onClick={() => {
            setSelectedPosition(record);
            setPositionDetailsVisible(true);
          }}>
            详情
          </Button>
          <Popconfirm title="确认平仓?" onConfirm={() => handlePositionAction('close', record.id)}>
            <Button size="small" danger>平仓</Button>
          </Popconfirm>
        </Space>
      )
    }
  ];

  return (
    <div style={{ padding: '24px' }}>
      <div style={{ marginBottom: '24px' }}>
        <Title level={2}>交易管理中心</Title>
        <Text type="secondary">订单监控、仓位管理、资金管理、风险控制</Text>
      </div>

      <Tabs 
        activeKey={activeTab} 
        onChange={setActiveTab} 
        size="large"
        items={[
          {
            key: 'orders',
            label: `订单监控 (${orders.length})`,
            children: (
              <>
                {/* 概览统计 - 扩展显示更多API数据 */}
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="活跃订单"
                        value={orders.length}
                        prefix={<SwapOutlined />}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="成交率"
                        value={orderStats?.fill_rate || 0}
                        precision={1}
                        suffix="%"
                        prefix={<CheckCircleOutlined />}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="平均执行时间"
                        value={orderLatency?.avg_latency || orderStats?.avg_execution_time || 0}
                        suffix="ms"
                        prefix={<ClockCircleOutlined />}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="平均滑点"
                        value={orderSlippage?.avg_slippage || 0}
                        precision={3}
                        suffix="%"
                        prefix={<WarningOutlined />}
                        valueStyle={{ color: orderSlippage?.avg_slippage > 0.1 ? '#cf1322' : '#3f8600' }}
                      />
                    </Card>
                  </Col>
                </Row>

                {/* 订单执行质量和类型分析 */}
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} md={12}>
                    <Card title="订单执行质量分析" size="small">
                      <Row gutter={16}>
                        <Col span={12}>
                          <Statistic
                            title="执行质量分数"
                            value={orderExecutionQuality?.avg_quality || 0}
                            precision={1}
                            suffix="/100"
                          />
                        </Col>
                        <Col span={12}>
                          <Statistic
                            title="最优价格达成率"
                            value={orderExecutionQuality?.optimal_price_rate || 0}
                            precision={1}
                            suffix="%"
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card title="订单类型分布" size="small">
                      <Row gutter={16}>
                        <Col span={8}>
                          <Statistic
                            title="限价单"
                            value={orderTypeStats?.limit_orders || 0}
                          />
                        </Col>
                        <Col span={8}>
                          <Statistic
                            title="市价单"
                            value={orderTypeStats?.market_orders || 0}
                          />
                        </Col>
                        <Col span={8}>
                          <Statistic
                            title="止损单"
                            value={orderTypeStats?.stop_orders || 0}
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                </Row>

                {/* 最近成交记录 */}
                {recentTrades && recentTrades.length > 0 && (
                  <Card title="最近成交记录" style={{ marginBottom: '24px' }}>
                    <List
                      size="small"
                      dataSource={recentTrades.slice(0, 5)}
                      renderItem={(trade: any) => (
                        <List.Item>
                          <List.Item.Meta
                            title={`${trade.symbol} ${trade.side === 'buy' ? '买入' : '卖出'}`}
                            description={`成交价: ${trade.price} | 数量: ${trade.quantity} | 时间: ${new Date(trade.timestamp).toLocaleTimeString()}`}
                          />
                          <Text type={trade.side === 'buy' ? 'success' : 'danger'}>
                            {trade.amount}
                          </Text>
                        </List.Item>
                      )}
                    />
                  </Card>
                )}

                {/* 订单列表 */}
                <Card
                  title="活跃订单"
                  extra={
                    <Space>
                      <Button 
                        icon={<DeleteOutlined />} 
                        onClick={() => handleOrderAction('batchCancel')}
                        danger
                      >
                        批量取消
                      </Button>
                      <Button 
                        icon={<ReloadOutlined />} 
                        onClick={() => handleOrderAction('refresh')} 
                        loading={loading}
                      >
                        刷新
                      </Button>
                    </Space>
                  }
                >
                  <Table
                    dataSource={orders}
                    columns={orderColumns}
                    rowKey="id"
                    loading={loading}
                    pagination={{ pageSize: 10 }}
                    scroll={{ x: 1200 }}
                  />
                </Card>
              </>
            )
          },
          {
            key: 'positions',
            label: `仓位监控 (${positions.length})`,
            children: (
              <>
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="持仓数量"
                        value={positions.length}
                        prefix={<FundOutlined />}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="总盈亏"
                        value={totalPnl}
                        precision={2}
                        prefix={totalPnl >= 0 ? <RiseOutlined /> : <FallOutlined />}
                        valueStyle={{ color: totalPnl >= 0 ? '#3f8600' : '#cf1322' }}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="多头敞口"
                        value={positionExposure?.long_exposure || 0}
                        precision={2}
                        prefix={<RiseOutlined />}
                        valueStyle={{ color: '#3f8600' }}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="空头敞口"
                        value={positionExposure?.short_exposure || 0}
                        precision={2}
                        prefix={<FallOutlined />}
                        valueStyle={{ color: '#cf1322' }}
                      />
                    </Card>
                  </Col>
                </Row>

                {/* 仓位风险分析 */}
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} md={12}>
                    <Card title="保证金状态" size="small">
                      <Row gutter={16}>
                        <Col span={12}>
                          <Statistic
                            title="总保证金"
                            value={marginRequirement?.total_margin || 0}
                            precision={2}
                            suffix="USDT"
                          />
                        </Col>
                        <Col span={12}>
                          <Statistic
                            title="维持保证金"
                            value={marginRequirement?.maintenance_margin || 0}
                            precision={2}
                            suffix="USDT"
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card title="强平风险评估" size="small">
                      <Row gutter={16}>
                        <Col span={12}>
                          <Statistic
                            title="风险等级"
                            value={liquidationRisk?.risk_level || 'low'}
                            valueStyle={{ 
                              color: {
                                low: '#3f8600',
                                medium: '#fa8c16',
                                high: '#cf1322'
                              }[liquidationRisk?.risk_level] || '#3f8600'
                            }}
                          />
                        </Col>
                        <Col span={12}>
                          <Statistic
                            title="强平距离"
                            value={liquidationRisk?.distance_to_liquidation || 0}
                            precision={2}
                            suffix="%"
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                </Row>

                {/* 仓位性能分析 */}
                {positionPerformance && (
                  <Card title="仓位性能分析" style={{ marginBottom: '24px' }}>
                    <Row gutter={16}>
                      <Col span={6}>
                        <Statistic
                          title="胜率"
                          value={positionPerformance.win_rate || 0}
                          precision={1}
                          suffix="%"
                        />
                      </Col>
                      <Col span={6}>
                        <Statistic
                          title="平均持仓时间"
                          value={positionPerformance.avg_holding_time || 0}
                          suffix="小时"
                        />
                      </Col>
                      <Col span={6}>
                        <Statistic
                          title="最大单笔盈利"
                          value={positionPerformance.max_profit || 0}
                          precision={2}
                          valueStyle={{ color: '#3f8600' }}
                        />
                      </Col>
                      <Col span={6}>
                        <Statistic
                          title="最大单笔亏损"
                          value={positionPerformance.max_loss || 0}
                          precision={2}
                          valueStyle={{ color: '#cf1322' }}
                        />
                      </Col>
                    </Row>
                  </Card>
                )}

                <Card
                  title="当前持仓"
                  extra={
                    <Space>
                      <Button 
                        icon={<SafetyOutlined />} 
                        onClick={() => handlePositionAction('hedge')}
                      >
                        一键对冲
                      </Button>
                      <Button 
                        icon={<ReloadOutlined />} 
                        onClick={initializeData} 
                        loading={loading}
                      >
                        刷新
                      </Button>
                    </Space>
                  }
                >
                  <Table
                    dataSource={positions}
                    columns={positionColumns}
                    rowKey="id"
                    loading={loading}
                    pagination={{ pageSize: 10 }}
                    scroll={{ x: 1200 }}
                  />
                </Card>
              </>
            )
          },
          {
            key: 'funds',
            label: '资金管理',
            children: (
              <>
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} sm={8}>
                    <Card>
                      <Statistic
                        title="账户总余额"
                        value={accountBalance?.total_balance || 0}
                        precision={2}
                        suffix="USDT"
                        prefix={<DollarOutlined />}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={8}>
                    <Card>
                      <Statistic
                        title="可用余额"
                        value={accountBalance?.available_balance || 0}
                        precision={2}
                        suffix="USDT"
                        prefix={<FundOutlined />}
                        valueStyle={{ color: '#3f8600' }}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={8}>
                    <Card>
                      <Statistic
                        title="冻结余额"
                        value={accountBalance?.locked_balance || 0}
                        precision={2}
                        suffix="USDT"
                        prefix={<SafetyOutlined />}
                        valueStyle={{ color: '#fa8c16' }}
                      />
                    </Card>
                  </Col>
                </Row>

                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} md={12}>
                    <Card title="资金利用率" size="small">
                      <Progress
                        type="circle"
                        percent={Math.round((fundsUtilization?.utilization_rate || 0) * 100)}
                        format={percent => `${percent}%`}
                        status={
                          (fundsUtilization?.utilization_rate || 0) > 0.8 ? 'exception' :
                          (fundsUtilization?.utilization_rate || 0) > 0.6 ? 'active' : 'success'
                        }
                      />
                      <div style={{ marginTop: '16px', textAlign: 'center' }}>
                        <Text>效率分数: {fundsUtilization?.efficiency_score || 0}</Text>
                      </div>
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card 
                      title="快速操作" 
                      size="small"
                      extra={
                        <Button 
                          type="primary" 
                          icon={<SwapOutlined />}
                          onClick={() => setTransferModalVisible(true)}
                        >
                          资金划转
                        </Button>
                      }
                    >
                      <div style={{ lineHeight: '2.5' }}>
                        <div>最大利用率: {Math.round((fundsUtilization?.max_utilization || 0) * 100)}%</div>
                        <div>最优利用率: {Math.round((fundsUtilization?.optimal_utilization || 0) * 100)}%</div>
                        <div>当前状态: <Tag color="processing">正常</Tag></div>
                      </div>
                    </Card>
                  </Col>
                </Row>

                {/* 资产分配和绩效分析 */}
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} md={12}>
                    <Card title="资产分配分析" size="small">
                      <Row gutter={16}>
                        <Col span={8}>
                          <Statistic
                            title="现货占比"
                            value={assetDistribution?.spot_ratio || 0}
                            precision={1}
                            suffix="%"
                          />
                        </Col>
                        <Col span={8}>
                          <Statistic
                            title="期货占比"
                            value={assetDistribution?.futures_ratio || 0}
                            precision={1}
                            suffix="%"
                          />
                        </Col>
                        <Col span={8}>
                          <Statistic
                            title="杠杆占比"
                            value={assetDistribution?.margin_ratio || 0}
                            precision={1}
                            suffix="%"
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card title="资金绩效分析" size="small">
                      <Row gutter={16}>
                        <Col span={12}>
                          <Statistic
                            title="年化收益率"
                            value={fundPerformanceAnalysis?.annual_return || 0}
                            precision={2}
                            suffix="%"
                            valueStyle={{ color: fundPerformanceAnalysis?.annual_return > 0 ? '#3f8600' : '#cf1322' }}
                          />
                        </Col>
                        <Col span={12}>
                          <Statistic
                            title="波动率"
                            value={fundPerformanceAnalysis?.volatility || 0}
                            precision={2}
                            suffix="%"
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                </Row>

                {/* 现金流和交易成本分析 */}
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} md={12}>
                    <Card title="现金流分析" size="small">
                      <Row gutter={16}>
                        <Col span={12}>
                          <Statistic
                            title="净流入"
                            value={cashFlow?.net_inflow || 0}
                            precision={2}
                            suffix="USDT"
                            valueStyle={{ color: '#3f8600' }}
                          />
                        </Col>
                        <Col span={12}>
                          <Statistic
                            title="净流出"
                            value={cashFlow?.net_outflow || 0}
                            precision={2}
                            suffix="USDT"
                            valueStyle={{ color: '#cf1322' }}
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card title="交易成本分析" size="small">
                      <Row gutter={16}>
                        <Col span={12}>
                          <Statistic
                            title="总交易费用"
                            value={transactionCosts?.total_fees || 0}
                            precision={2}
                            suffix="USDT"
                          />
                        </Col>
                        <Col span={12}>
                          <Statistic
                            title="费率优化"
                            value={transactionCosts?.fee_optimization || 0}
                            precision={1}
                            suffix="%"
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                </Row>

                {/* 流动性和信用状态 */}
                <Row gutter={[16, 16]}>
                  <Col xs={24} md={12}>
                    <Card title="流动性分析" size="small">
                      <Row gutter={16}>
                        <Col span={12}>
                          <Statistic
                            title="流动性评分"
                            value={liquidityAnalysis?.liquidity_score || 0}
                            precision={1}
                            suffix="/100"
                          />
                        </Col>
                        <Col span={12}>
                          <Statistic
                            title="可变现比例"
                            value={liquidityAnalysis?.liquidatable_ratio || 0}
                            precision={1}
                            suffix="%"
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card title="信用额度管理" size="small">
                      <Row gutter={16}>
                        <Col span={12}>
                          <Statistic
                            title="可用信用额度"
                            value={creditLine?.available_credit || 0}
                            precision={2}
                            suffix="USDT"
                          />
                        </Col>
                        <Col span={12}>
                          <Statistic
                            title="信用使用率"
                            value={creditLine?.credit_utilization || 0}
                            precision={1}
                            suffix="%"
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                </Row>
              </>
            )
          },
          {
            key: 'risk',
            label: '风险控制',
            children: (
              <>
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="当前风险等级"
                        value={riskMetrics?.current_risk_level || 'low'}
                        valueStyle={{ 
                          color: {
                            low: '#3f8600',
                            medium: '#fa8c16', 
                            high: '#ff7875',
                            critical: '#cf1322'
                          }[riskMetrics?.current_risk_level || 'low']
                        }}
                        prefix={<SafetyOutlined />}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="总敞口"
                        value={riskMetrics?.total_exposure || 0}
                        precision={2}
                        prefix={<BarChartOutlined />}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="最大回撤"
                        value={Math.abs(riskMetrics?.max_drawdown || 0)}
                        precision={2}
                        suffix="%"
                        prefix={<FallOutlined />}
                        valueStyle={{ color: '#cf1322' }}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="夏普比率"
                        value={riskMetrics?.sharpe_ratio || 0}
                        precision={2}
                        prefix={<LineChartOutlined />}
                      />
                    </Card>
                  </Col>
                </Row>

                <Row gutter={[16, 16]}>
                  <Col xs={24} md={12}>
                    <Card 
                      title="风险告警" 
                      size="small"
                      extra={
                        <Button 
                          danger 
                          icon={<StopOutlined />}
                          onClick={() => handleRiskAction('emergencyStop')}
                        >
                          紧急止损
                        </Button>
                      }
                    >
                      {riskAlerts.length > 0 ? (
                        riskAlerts.map(alert => (
                          <Alert
                            key={alert.id}
                            message={alert.message}
                            type={alert.type}
                            showIcon
                            style={{ marginBottom: 8 }}
                          />
                        ))
                      ) : (
                        <Text type="secondary">暂无风险告警</Text>
                      )}
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card 
                      title="风险限额设置" 
                      size="small"
                      extra={
                        <Button 
                          icon={<SettingOutlined />}
                          onClick={() => setRiskLimitsModalVisible(true)}
                        >
                          设置限额
                        </Button>
                      }
                    >
                      <div style={{ lineHeight: '2.5' }}>
                        <div>最大持仓: {riskMetrics?.risk_limits?.max_position_size || 0}</div>
                        <div>最大日亏损: {riskMetrics?.risk_limits?.max_daily_loss || 0}</div>
                        <div>杠杆限制: {riskMetrics?.risk_limits?.leverage_limit || 0}x</div>
                      </div>
                    </Card>
                  </Col>
                </Row>
              </>
            )
          }
        ]}
      />

      {/* 资金划转模态框 */}
      <Modal
        title="资金划转"
        open={transferModalVisible}
        onCancel={() => setTransferModalVisible(false)}
        onOk={() => transferForm.submit()}
      >
        <Form form={transferForm} onFinish={handleFundsTransfer} layout="vertical">
          <Form.Item name="from" label="从" rules={[{ required: true }]}>
            <Select placeholder="选择源账户">
              <Option value="spot">现货账户</Option>
              <Option value="futures">期货账户</Option>
              <Option value="margin">杠杆账户</Option>
            </Select>
          </Form.Item>
          <Form.Item name="to" label="到" rules={[{ required: true }]}>
            <Select placeholder="选择目标账户">
              <Option value="spot">现货账户</Option>
              <Option value="futures">期货账户</Option>
              <Option value="margin">杠杆账户</Option>
            </Select>
          </Form.Item>
          <Form.Item name="amount" label="金额" rules={[{ required: true }]}>
            <InputNumber 
              style={{ width: '100%' }} 
              placeholder="输入划转金额"
              min={0}
              precision={2}
            />
          </Form.Item>
        </Form>
      </Modal>

      {/* 风险限额设置模态框 */}
      <Modal
        title="风险限额设置"
        open={riskLimitsModalVisible}
        onCancel={() => setRiskLimitsModalVisible(false)}
        onOk={() => handleRiskAction('updateLimits')}
      >
        <Form 
          form={riskLimitsForm} 
          layout="vertical"
          initialValues={riskMetrics?.risk_limits}
        >
          <Form.Item name="max_position_size" label="最大持仓限制">
            <InputNumber style={{ width: '100%' }} min={0} />
          </Form.Item>
          <Form.Item name="max_daily_loss" label="最大日亏损限制">
            <InputNumber style={{ width: '100%' }} min={0} />
          </Form.Item>
          <Form.Item name="leverage_limit" label="杠杆限制">
            <InputNumber style={{ width: '100%' }} min={1} max={100} />
          </Form.Item>
        </Form>
      </Modal>

      {/* 订单详情模态框 */}
      <Modal
        title="订单详情"
        open={orderDetailsVisible}
        onCancel={() => setOrderDetailsVisible(false)}
        footer={null}
        width={600}
      >
        {selectedOrder && (
          <Descriptions column={2} bordered>
            <Descriptions.Item label="订单ID">{selectedOrder.id}</Descriptions.Item>
            <Descriptions.Item label="交易对">{selectedOrder.symbol}</Descriptions.Item>
            <Descriptions.Item label="方向">
              <Tag color={selectedOrder.side === 'buy' ? 'green' : 'red'}>
                {selectedOrder.side === 'buy' ? '买入' : '卖出'}
              </Tag>
            </Descriptions.Item>
            <Descriptions.Item label="类型">{selectedOrder.type}</Descriptions.Item>
            <Descriptions.Item label="数量">{selectedOrder.amount}</Descriptions.Item>
            <Descriptions.Item label="价格">{selectedOrder.price}</Descriptions.Item>
            <Descriptions.Item label="已成交">{selectedOrder.filled}</Descriptions.Item>
            <Descriptions.Item label="状态">
              <Badge status="processing" text={selectedOrder.status} />
            </Descriptions.Item>
            <Descriptions.Item label="执行质量">{selectedOrder.execution_quality || 'N/A'}</Descriptions.Item>
            <Descriptions.Item label="延迟">{selectedOrder.latency || 'N/A'}ms</Descriptions.Item>
            <Descriptions.Item label="滑点">{selectedOrder.slippage || 'N/A'}%</Descriptions.Item>
            <Descriptions.Item label="创建时间">
              {new Date(selectedOrder.created_at).toLocaleString()}
            </Descriptions.Item>
          </Descriptions>
        )}
      </Modal>

      {/* 仓位详情模态框 */}
      <Modal
        title="仓位详情"
        open={positionDetailsVisible}
        onCancel={() => setPositionDetailsVisible(false)}
        footer={null}
        width={600}
      >
        {selectedPosition && (
          <Descriptions column={2} bordered>
            <Descriptions.Item label="交易对">{selectedPosition.symbol}</Descriptions.Item>
            <Descriptions.Item label="方向">
              <Tag color={selectedPosition.side === 'long' ? 'green' : 'red'}>
                {selectedPosition.side === 'long' ? '多头' : '空头'}
              </Tag>
            </Descriptions.Item>
            <Descriptions.Item label="持仓量">{selectedPosition.size}</Descriptions.Item>
            <Descriptions.Item label="开仓价">{selectedPosition.entry_price}</Descriptions.Item>
            <Descriptions.Item label="标记价">{selectedPosition.mark_price}</Descriptions.Item>
            <Descriptions.Item label="盈亏">
              <Text type={selectedPosition.pnl >= 0 ? 'success' : 'danger'}>
                {selectedPosition.pnl >= 0 ? '+' : ''}{selectedPosition.pnl.toFixed(2)}
              </Text>
            </Descriptions.Item>
            <Descriptions.Item label="盈亏率">
              <Text type={selectedPosition.pnl_percentage >= 0 ? 'success' : 'danger'}>
                {selectedPosition.pnl_percentage >= 0 ? '+' : ''}{selectedPosition.pnl_percentage.toFixed(2)}%
              </Text>
            </Descriptions.Item>
            <Descriptions.Item label="保证金">{selectedPosition.margin}</Descriptions.Item>
            <Descriptions.Item label="强平价">{selectedPosition.liquidation_price}</Descriptions.Item>
            <Descriptions.Item label="创建时间">
              {new Date(selectedPosition.created_at).toLocaleString()}
            </Descriptions.Item>
          </Descriptions>
        )}
      </Modal>
    </div>
  );
};

export default TradingModule;
import { 
  Card, Row, Col, Button, Table, Modal, Form, Input, Select, Switch, 
  Tabs, Progress, Statistic, Badge, Alert, message, notification,
  Drawer, Tag, Tooltip, Popconfirm, Space, Timeline, Descriptions,
  Typography, Divider, List, Avatar, InputNumber
} from 'antd';
import {
  DollarOutlined, SwapOutlined, FundOutlined, SafetyOutlined,
  ReloadOutlined, LineChartOutlined, MonitorOutlined, BarChartOutlined,
  StopOutlined, PauseCircleOutlined, PlayCircleOutlined, SettingOutlined,
  ExclamationCircleOutlined, CheckCircleOutlined, ClockCircleOutlined,
  RiseOutlined, FallOutlined, EyeOutlined, EditOutlined,
  DeleteOutlined, PlusOutlined, SyncOutlined, WarningOutlined
} from '@ant-design/icons';

const { Option } = Select;
const { Title, Text } = Typography;

// 数据类型定义
interface Order {
  id: string;
  symbol: string;
  type: 'market' | 'limit' | 'stop_limit';
  side: 'buy' | 'sell';
  amount: number;
  price: number;
  filled: number;
  status: 'pending' | 'partial' | 'filled' | 'cancelled' | 'rejected';
  created_at: string;
  updated_at: string;
  execution_quality?: number;
  latency?: number;
  slippage?: number;
}

interface Position {
  id: string;
  symbol: string;
  side: 'long' | 'short';
  size: number;
  entry_price: number;
  mark_price: number;
  pnl: number;
  pnl_percentage: number;
  margin: number;
  liquidation_price: number;
  created_at: string;
  updated_at: string;
}

interface AccountBalance {
  total_balance: number;
  available_balance: number;
  locked_balance: number;
  currency: string;
  assets: Array<{
    asset: string;
    free: number;
    locked: number;
    total: number;
  }>;
}

interface OrderStats {
  total_orders: number;
  filled_orders: number;
  cancelled_orders: number;
  rejected_orders: number;
  fill_rate: number;
  avg_execution_time: number;
}

interface RiskMetrics {
  total_exposure: number;
  max_drawdown: number;
  var_95: number;
  sharpe_ratio: number;
  current_risk_level: 'low' | 'medium' | 'high' | 'critical';
  risk_limits: {
    max_position_size: number;
    max_daily_loss: number;
    leverage_limit: number;
  };
}

interface FundsUtilization {
  utilization_rate: number;
  efficiency_score: number;
  max_utilization: number;
  optimal_utilization: number;
}

const TradingModule: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const [activeTab, setActiveTab] = useState('orders');
  
  // 订单监控状态
  const [orders, setOrders] = useState<Order[]>([]);
  const [orderHistory, setOrderHistory] = useState<Order[]>([]);
  const [orderStats, setOrderStats] = useState<OrderStats | null>(null);
  const [orderExecutionQuality, setOrderExecutionQuality] = useState<any>(null);
  const [orderLatency, setOrderLatency] = useState<any>(null);
  const [orderSlippage, setOrderSlippage] = useState<any>(null);
  const [orderFillRate, setOrderFillRate] = useState<any>(null);
  const [activeOrdersDetails, setActiveOrdersDetails] = useState<any[]>([]);
  const [orderExecutionStatus, setOrderExecutionStatus] = useState<any>(null);
  const [orderTypeStats, setOrderTypeStats] = useState<any>(null);
  const [recentTrades, setRecentTrades] = useState<any[]>([]);
  const [selectedOrder, setSelectedOrder] = useState<Order | null>(null);
  const [orderDetailsVisible, setOrderDetailsVisible] = useState(false);
  
  // 仓位监控状态
  const [positions, setPositions] = useState<Position[]>([]);
  const [totalPnl, setTotalPnl] = useState(0);
  const [positionRealtimeData, setPositionRealtimeData] = useState<any[]>([]);
  const [positionPnlDetails, setPositionPnlDetails] = useState<any>(null);
  const [positionExposure, setPositionExposure] = useState<any>(null);
  const [marginRequirement, setMarginRequirement] = useState<any>(null);
  const [liquidationRisk, setLiquidationRisk] = useState<any>(null);
  const [positionHistory, setPositionHistory] = useState<any[]>([]);
  const [positionPerformance, setPositionPerformance] = useState<any>(null);
  const [selectedPosition, setSelectedPosition] = useState<Position | null>(null);
  const [positionDetailsVisible, setPositionDetailsVisible] = useState(false);
  
  // 资金管理状态
  const [accountBalance, setAccountBalance] = useState<AccountBalance | null>(null);
  const [fundsHistory, setFundsHistory] = useState<any[]>([]);
  const [fundsUtilization, setFundsUtilization] = useState<FundsUtilization | null>(null);
  const [assetDistribution, setAssetDistribution] = useState<any>(null);
  const [fundPerformanceAnalysis, setFundPerformanceAnalysis] = useState<any>(null);
  const [fundAllocationLimits, setFundAllocationLimits] = useState<any>(null);
  const [cashFlow, setCashFlow] = useState<any>(null);
  const [liquidityAnalysis, setLiquidityAnalysis] = useState<any>(null);
  const [marginStatus, setMarginStatus] = useState<any>(null);
  const [creditLine, setCreditLine] = useState<any>(null);
  const [fundFlowOptimization, setFundFlowOptimization] = useState<any>(null);
  const [transactionCosts, setTransactionCosts] = useState<any>(null);
  const [transferModalVisible, setTransferModalVisible] = useState(false);
  
  // 风险控制状态
  const [riskMetrics, setRiskMetrics] = useState<RiskMetrics | null>(null);
  const [riskAlerts, setRiskAlerts] = useState<any[]>([]);
  const [riskLimitsModalVisible, setRiskLimitsModalVisible] = useState(false);
  
  // 表单实例
  const [transferForm] = Form.useForm();
  const [riskLimitsForm] = Form.useForm();

  // API基地址
  const API_BASE = 'http://57.183.21.242:3001/api';

  // API调用函数
  const apiCall = async (endpoint: string, options: RequestInit = {}) => {
    try {
      const response = await fetch(`${API_BASE}${endpoint}`, {
        ...options,
        headers: {
          'Content-Type': 'application/json',
          ...options.headers
        }
      });
      
      if (!response.ok) {
        throw new Error(`API调用失败: ${response.status} ${response.statusText}`);
      }
      
      return await response.json();
    } catch (error) {
      console.error(`API调用错误 ${endpoint}:`, error);
      throw error;
    }
  };

  // 初始化数据 - 实现真实API调用（45个接口）
  const initializeData = async () => {
    setLoading(true);
    try {
      // 1. 订单监控API调用 (15个接口)
      const [
        activeOrdersRes,
        historicalOrdersRes,
        orderStatsRes,
        orderExecutionQualityRes,
        orderLatencyRes,
        orderSlippageRes,
        orderFillRateRes,
        activeOrdersDetailsRes,
        orderExecutionStatusRes,
        orderTypeStatsRes,
        recentTradesRes
      ] = await Promise.all([
        apiCall('/orders/active'),          // 1. 获取活跃订单列表
        apiCall('/orders/history'),         // 2. 获取历史订单数据
        apiCall('/orders/statistics'),      // 3. 订单统计分析
        apiCall('/orders/execution-quality'), // 4. 订单执行质量
        apiCall('/orders/latency'),         // 5. 订单执行延迟
        apiCall('/orders/slippage'),        // 9. 订单滑点分析
        apiCall('/orders/fill-rate'),       // 10. 订单成交率统计
        apiCall('/orders/active-details'),  // 11. 活跃订单详情
        apiCall('/orders/execution-status'), // 12. 订单执行状态更新
        apiCall('/orders/type-stats'),      // 14. 订单类型分析
        apiCall('/orders/recent-trades')    // 15. 最近成交记录
      ]);

      // 2. 仓位监控API调用 (12个接口)
      const [
        currentPositionsRes,
        positionRealtimeRes,
        positionPnlRes,
        positionPnlDetailsRes,
        positionExposureRes,
        marginRequirementRes,
        liquidationRiskRes,
        positionHistoryRes,
        positionPerformanceRes
      ] = await Promise.all([
        apiCall('/positions/current'),      // 1. 获取当前持仓列表
        apiCall('/positions/realtime'),     // 2. 仓位实时数据更新
        apiCall('/positions/pnl'),          // 3. 持仓盈亏计算
        apiCall('/positions/pnl-details'),  // 3. 持仓盈亏详细分析
        apiCall('/positions/exposure'),     // 6. 持仓敞口分析
        apiCall('/positions/margin-requirement'), // 6. 保证金要求计算
        apiCall('/positions/liquidation-risk'),   // 7. 强平风险评估
        apiCall('/positions/history'),      // 8. 历史持仓数据
        apiCall('/positions/performance')   // 12. 仓位性能分析
      ]);

      // 3. 资金管理API调用 (14个接口)
      const [
        accountBalanceRes,
        fundsUtilizationRes,
        fundsHistoryRes,
        assetDistributionRes,
        fundPerformanceRes,
        fundAllocationLimitsRes,
        cashFlowRes,
        liquidityAnalysisRes,
        marginStatusRes,
        creditLineRes,
        fundFlowOptimizationRes,
        transactionCostsRes
      ] = await Promise.all([
        apiCall('/funds/balance'),          // 1. 获取账户余额
        apiCall('/funds/utilization'),      // 2. 资金利用率分析
        apiCall('/funds/history'),          // 3. 资金流向历史
        apiCall('/funds/asset-distribution'), // 5. 资产分配分析
        apiCall('/funds/performance-analysis'), // 6. 资金绩效分析
        apiCall('/funds/allocation-limits'), // 7. 资金配置限额
        apiCall('/funds/cash-flow'),        // 8. 现金流分析
        apiCall('/funds/liquidity-analysis'), // 9. 流动性分析
        apiCall('/funds/margin-status'),    // 10. 保证金状态
        apiCall('/funds/credit-line'),      // 11. 信用额度管理
        apiCall('/funds/flow-optimization'), // 12. 资金流向优化
        apiCall('/funds/transaction-costs') // 13. 交易成本分析
      ]);

      // 4. 风险控制API调用 (4个接口)
      const [
        riskMetricsRes,
        riskLimitsRes,
        riskAlertsRes
      ] = await Promise.all([
        apiCall('/risk/metrics'),           // 1. 获取风险指标
        apiCall('/risk/limits'),            // 2. 风险限额设置
        apiCall('/risk/alerts')             // 3. 风险告警管理
      ]);

      // 设置所有45个API接口的数据
      // 订单监控数据
      setOrders(activeOrdersRes.data || []);
      setOrderHistory(historicalOrdersRes.data || []);
      setOrderStats(orderStatsRes.data || null);
      setOrderExecutionQuality(orderExecutionQualityRes.data || null);
      setOrderLatency(orderLatencyRes.data || null);
      setOrderSlippage(orderSlippageRes.data || null);
      setOrderFillRate(orderFillRateRes.data || null);
      setActiveOrdersDetails(activeOrdersDetailsRes.data || []);
      setOrderExecutionStatus(orderExecutionStatusRes.data || null);
      setOrderTypeStats(orderTypeStatsRes.data || null);
      setRecentTrades(recentTradesRes.data || []);
      
      // 仓位监控数据
      setPositions(currentPositionsRes.data || []);
      setPositionRealtimeData(positionRealtimeRes.data || []);
      setTotalPnl(positionPnlRes.data?.total_pnl || 0);
      setPositionPnlDetails(positionPnlDetailsRes.data || null);
      setPositionExposure(positionExposureRes.data || null);
      setMarginRequirement(marginRequirementRes.data || null);
      setLiquidationRisk(liquidationRiskRes.data || null);
      setPositionHistory(positionHistoryRes.data || []);
      setPositionPerformance(positionPerformanceRes.data || null);
      
      // 资金管理数据
      setAccountBalance(accountBalanceRes.data || null);
      setFundsUtilization(fundsUtilizationRes.data || null);
      setFundsHistory(fundsHistoryRes.data || []);
      setAssetDistribution(assetDistributionRes.data || null);
      setFundPerformanceAnalysis(fundPerformanceRes.data || null);
      setFundAllocationLimits(fundAllocationLimitsRes.data || null);
      setCashFlow(cashFlowRes.data || null);
      setLiquidityAnalysis(liquidityAnalysisRes.data || null);
      setMarginStatus(marginStatusRes.data || null);
      setCreditLine(creditLineRes.data || null);
      setFundFlowOptimization(fundFlowOptimizationRes.data || null);
      setTransactionCosts(transactionCostsRes.data || null);
      
      // 风险控制数据
      setRiskMetrics(riskMetricsRes.data || null);
      setRiskAlerts(riskAlertsRes.data || []);

    } catch (error) {
      console.error('Failed to initialize trading data:', error);
      message.error(`数据加载失败: ${error.message}`);
      
      // 降级到空数据，但保持页面结构
      const fallbackNull = null;
      const fallbackEmpty: any[] = [];
      const fallbackStats = { total_orders: 0, filled_orders: 0, cancelled_orders: 0, rejected_orders: 0, fill_rate: 0, avg_execution_time: 0 };
      const fallbackBalance = { total_balance: 0, available_balance: 0, locked_balance: 0, currency: 'USDT', assets: [] };
      const fallbackUtilization = { utilization_rate: 0, efficiency_score: 0, max_utilization: 0, optimal_utilization: 0 };
      const fallbackRiskMetrics = { total_exposure: 0, max_drawdown: 0, var_95: 0, sharpe_ratio: 0, current_risk_level: 'low' as const, risk_limits: { max_position_size: 0, max_daily_loss: 0, leverage_limit: 0 } };
      
      // 订单监控降级数据
      setOrders(fallbackEmpty);
      setOrderHistory(fallbackEmpty);
      setOrderStats(fallbackStats);
      setOrderExecutionQuality(fallbackNull);
      setOrderLatency(fallbackNull);
      setOrderSlippage(fallbackNull);
      setOrderFillRate(fallbackNull);
      setActiveOrdersDetails(fallbackEmpty);
      setOrderExecutionStatus(fallbackNull);
      setOrderTypeStats(fallbackNull);
      setRecentTrades(fallbackEmpty);
      
      // 仓位监控降级数据
      setPositions(fallbackEmpty);
      setPositionRealtimeData(fallbackEmpty);
      setTotalPnl(0);
      setPositionPnlDetails(fallbackNull);
      setPositionExposure(fallbackNull);
      setMarginRequirement(fallbackNull);
      setLiquidationRisk(fallbackNull);
      setPositionHistory(fallbackEmpty);
      setPositionPerformance(fallbackNull);
      
      // 资金管理降级数据
      setAccountBalance(fallbackBalance);
      setFundsUtilization(fallbackUtilization);
      setFundsHistory(fallbackEmpty);
      setAssetDistribution(fallbackNull);
      setFundPerformanceAnalysis(fallbackNull);
      setFundAllocationLimits(fallbackNull);
      setCashFlow(fallbackNull);
      setLiquidityAnalysis(fallbackNull);
      setMarginStatus(fallbackNull);
      setCreditLine(fallbackNull);
      setFundFlowOptimization(fallbackNull);
      setTransactionCosts(fallbackNull);
      
      // 风险控制降级数据
      setRiskMetrics(fallbackRiskMetrics);
      setRiskAlerts(fallbackEmpty);
    } finally {
      setLoading(false);
    }
  };

  // 仓位监控额外API函数
  const getPositionRealtimeData = async () => {
    try {
      // 2. 仓位实时数据更新
      return await apiCall('/positions/realtime');
    } catch (error) {
      console.error('Get position realtime data failed:', error);
      return { data: [] };
    }
  };

  const getPositionPnlDetails = async () => {
    try {
      // 3. 持仓盈亏详细分析
      return await apiCall('/positions/pnl-details');
    } catch (error) {
      console.error('Get position PnL details failed:', error);
      return { data: {} };
    }
  };

  const getMarginRequirement = async () => {
    try {
      // 6. 保证金要求计算
      return await apiCall('/positions/margin-requirement');
    } catch (error) {
      console.error('Get margin requirement failed:', error);
      return { data: {} };
    }
  };

  const getLiquidationRisk = async () => {
    try {
      // 7. 强平风险评估
      return await apiCall('/positions/liquidation-risk');
    } catch (error) {
      console.error('Get liquidation risk failed:', error);
      return { data: {} };
    }
  };

  const getPositionHistory = async () => {
    try {
      // 8. 历史持仓数据
      return await apiCall('/positions/history');
    } catch (error) {
      console.error('Get position history failed:', error);
      return { data: [] };
    }
  };

  const adjustPosition = async (positionId: string, adjustment: any) => {
    try {
      // 9. 仓位调整操作
      return await apiCall(`/positions/${positionId}/adjust`, {
        method: 'PUT',
        body: JSON.stringify(adjustment)
      });
    } catch (error) {
      console.error('Adjust position failed:', error);
      throw error;
    }
  };

  const stopLossPosition = async (positionId: string, stopLossPrice: number) => {
    try {
      // 10. 止损设置
      return await apiCall(`/positions/${positionId}/stop-loss`, {
        method: 'POST',
        body: JSON.stringify({ stop_loss_price: stopLossPrice })
      });
    } catch (error) {
      console.error('Set stop loss failed:', error);
      throw error;
    }
  };

  const takeProfitPosition = async (positionId: string, takeProfitPrice: number) => {
    try {
      // 11. 止盈设置
      return await apiCall(`/positions/${positionId}/take-profit`, {
        method: 'POST',
        body: JSON.stringify({ take_profit_price: takeProfitPrice })
      });
    } catch (error) {
      console.error('Set take profit failed:', error);
      throw error;
    }
  };

  const getPositionPerformance = async () => {
    try {
      // 12. 仓位性能分析
      return await apiCall('/positions/performance');
    } catch (error) {
      console.error('Get position performance failed:', error);
      return { data: {} };
    }
  };

  // 资金管理额外API函数
  const getAssetDistribution = async () => {
    try {
      // 5. 资产分配分析
      return await apiCall('/funds/asset-distribution');
    } catch (error) {
      console.error('Get asset distribution failed:', error);
      return { data: {} };
    }
  };

  const getFundPerformanceAnalysis = async () => {
    try {
      // 6. 资金绩效分析
      return await apiCall('/funds/performance-analysis');
    } catch (error) {
      console.error('Get fund performance analysis failed:', error);
      return { data: {} };
    }
  };

  const getFundAllocationLimits = async () => {
    try {
      // 7. 资金配置限额
      return await apiCall('/funds/allocation-limits');
    } catch (error) {
      console.error('Get fund allocation limits failed:', error);
      return { data: {} };
    }
  };

  const getCashFlow = async () => {
    try {
      // 8. 现金流分析
      return await apiCall('/funds/cash-flow');
    } catch (error) {
      console.error('Get cash flow failed:', error);
      return { data: {} };
    }
  };

  const getLiquidityAnalysis = async () => {
    try {
      // 9. 流动性分析
      return await apiCall('/funds/liquidity-analysis');
    } catch (error) {
      console.error('Get liquidity analysis failed:', error);
      return { data: {} };
    }
  };

  const getMarginStatus = async () => {
    try {
      // 10. 保证金状态
      return await apiCall('/funds/margin-status');
    } catch (error) {
      console.error('Get margin status failed:', error);
      return { data: {} };
    }
  };

  const getCreditLine = async () => {
    try {
      // 11. 信用额度管理
      return await apiCall('/funds/credit-line');
    } catch (error) {
      console.error('Get credit line failed:', error);
      return { data: {} };
    }
  };

  const getFundFlowOptimization = async () => {
    try {
      // 12. 资金流向优化
      return await apiCall('/funds/flow-optimization');
    } catch (error) {
      console.error('Get fund flow optimization failed:', error);
      return { data: {} };
    }
  };

  const getTransactionCosts = async () => {
    try {
      // 13. 交易成本分析
      return await apiCall('/funds/transaction-costs');
    } catch (error) {
      console.error('Get transaction costs failed:', error);
      return { data: {} };
    }
  };

  const settleAssets = async (assets: any[]) => {
    try {
      // 14. 资产结算处理
      return await apiCall('/funds/settle-assets', {
        method: 'POST',
        body: JSON.stringify({ assets })
      });
    } catch (error) {
      console.error('Settle assets failed:', error);
      throw error;
    }
  };

  // 只在组件挂载时初始化数据一次
  useEffect(() => {
    initializeData();
    // 定时刷新数据，但频率降低以避免资源耗尽
    const interval = setInterval(initializeData, 60000); // 60秒刷新一次
    return () => clearInterval(interval);
  }, []);

  // 订单操作函数 - 实现真实API调用
  const handleOrderAction = async (action: string, orderId?: string) => {
    try {
      message.loading(`正在${action}订单...`, 1);
      
      switch (action) {
        case 'cancel':
          // 6. 取消单个订单
          await apiCall(`/orders/${orderId}/cancel`, { method: 'POST' });
          setOrders(prev => prev.filter(order => order.id !== orderId));
          message.success('订单取消成功');
          break;
        case 'batchCancel':
          // 7. 批量取消订单
          await apiCall('/orders/batch-cancel', { method: 'POST', body: JSON.stringify({ order_ids: orders.map(o => o.id) }) });
          setOrders([]);
          message.success('批量取消成功');
          break;
        case 'refresh':
          // 8. 获取订单执行状态更新
          await initializeData();
          message.success('数据刷新完成');
          break;
      }
      
    } catch (error) {
      console.error(`Order action ${action} failed:`, error);
      message.error(`操作失败: ${error.message}`);
    }
  };

  // 仓位操作函数 - 实现真实API调用
  const handlePositionAction = async (action: string, positionId?: string) => {
    try {
      message.loading(`正在${action}仓位...`, 1);
      
      switch (action) {
        case 'close':
          // 4. 平仓操作
          await apiCall(`/positions/${positionId}/close`, { method: 'POST' });
          setPositions(prev => prev.filter(pos => pos.id !== positionId));
          // 重新计算总盈亏
          const updatedPositions = positions.filter(pos => pos.id !== positionId);
          const newTotalPnl = updatedPositions.reduce((sum, pos) => sum + pos.pnl, 0);
          setTotalPnl(newTotalPnl);
          message.success('仓位平仓成功');
          break;
        case 'hedge':
          // 5. 一键对冲操作
          await apiCall('/positions/hedge', { method: 'POST', body: JSON.stringify({ position_ids: positions.map(p => p.id) }) });
          message.success('对冲操作成功');
          // 重新加载数据以获取最新状态
          await initializeData();
          break;
      }
      
    } catch (error) {
      console.error(`Position action ${action} failed:`, error);
      message.error(`操作失败: ${error.message}`);
    }
  };

  // 资金操作函数 - 实现真实API调用
  const handleFundsTransfer = async (values: any) => {
    try {
      message.loading('正在执行资金划转...', 2);
      
      // 4. 账户间资金划转
      await apiCall('/funds/transfer', { 
        method: 'POST', 
        body: JSON.stringify({
          from_account: values.from,
          to_account: values.to,
          amount: values.amount,
          asset: 'USDT'
        })
      });
      
      message.success(`成功从${values.from}账户划转${values.amount}到${values.to}账户`);
      setTransferModalVisible(false);
      transferForm.resetFields();
      
      // 重新加载余额数据
      const balanceRes = await apiCall('/funds/balance');
      setAccountBalance(balanceRes.data);
      
    } catch (error) {
      console.error('Funds transfer failed:', error);
      message.error(`资金划转失败: ${error.message}`);
    }
  };

  // 风险控制函数 - 实现真实API调用
  const handleRiskAction = async (action: string) => {
    try {
      switch (action) {
        case 'emergencyStop':
          message.loading('执行紧急止损...', 2);
          // 4. 紧急止损
          await apiCall('/risk/emergency-stop', { method: 'POST' });
          // 重新加载数据以获取最新状态
          await initializeData();
          message.success('紧急止损已执行');
          break;
        case 'updateLimits':
          const values = await riskLimitsForm.validateFields();
          message.loading('更新风险限额...', 1);
          
          // 2. 风险限额设置
          await apiCall('/risk/limits', { 
            method: 'PUT', 
            body: JSON.stringify(values)
          });
          
          setRiskMetrics(prev => prev ? {
            ...prev,
            risk_limits: values
          } : null);
          
          message.success('风险限额更新成功');
          setRiskLimitsModalVisible(false);
          break;
      }
      
    } catch (error) {
      console.error(`Risk action ${action} failed:`, error);
      message.error(`操作失败: ${error.message}`);
    }
  };

  // 订单监控额外API函数
  const getOrderSlippage = async () => {
    try {
      // 9. 订单滑点分析
      return await apiCall('/orders/slippage');
    } catch (error) {
      console.error('Get order slippage failed:', error);
      return { data: {} };
    }
  };

  const getOrderFillRate = async () => {
    try {
      // 10. 订单成交率统计
      return await apiCall('/orders/fill-rate');
    } catch (error) {
      console.error('Get order fill rate failed:', error);
      return { data: {} };
    }
  };

  const getActiveOrdersDetails = async () => {
    try {
      // 11. 活跃订单详情
      return await apiCall('/orders/active-details');
    } catch (error) {
      console.error('Get active orders details failed:', error);
      return { data: [] };
    }
  };

  const getOrderExecutionStatus = async () => {
    try {
      // 12. 订单执行状态更新
      return await apiCall('/orders/execution-status');
    } catch (error) {
      console.error('Get order execution status failed:', error);
      return { data: {} };
    }
  };

  const batchModifyOrders = async (modifications: any[]) => {
    try {
      // 13. 批量修改订单
      return await apiCall('/orders/batch-modify', {
        method: 'PUT',
        body: JSON.stringify({ modifications })
      });
    } catch (error) {
      console.error('Batch modify orders failed:', error);
      throw error;
    }
  };

  const getOrderTypeStats = async () => {
    try {
      // 14. 订单类型分析
      return await apiCall('/orders/type-stats');
    } catch (error) {
      console.error('Get order type stats failed:', error);
      return { data: {} };
    }
  };

  const getRecentTrades = async () => {
    try {
      // 15. 最近成交记录
      return await apiCall('/orders/recent-trades');
    } catch (error) {
      console.error('Get recent trades failed:', error);
      return { data: [] };
    }
  };

  // 订单表格列定义
  const orderColumns = [
    { title: '订单ID', dataIndex: 'id', key: 'id', width: 120 },
    { title: '交易对', dataIndex: 'symbol', key: 'symbol' },
    { 
      title: '方向', 
      dataIndex: 'side', 
      key: 'side',
      render: (side: string) => (
        <Tag color={side === 'buy' ? 'green' : 'red'}>
          {side === 'buy' ? '买入' : '卖出'}
        </Tag>
      )
    },
    { 
      title: '类型', 
      dataIndex: 'type', 
      key: 'type',
      render: (type: string) => <Tag>{type}</Tag>
    },
    { title: '数量', dataIndex: 'amount', key: 'amount' },
    { title: '价格', dataIndex: 'price', key: 'price' },
    { title: '已成交', dataIndex: 'filled', key: 'filled' },
    { 
      title: '状态', 
      dataIndex: 'status', 
      key: 'status',
      render: (status: string) => {
        const colors = {
          pending: 'processing',
          partial: 'warning',
          filled: 'success',
          cancelled: 'default',
          rejected: 'error'
        };
        return <Badge status={colors[status] as any} text={status} />;
      }
    },
    {
      title: '操作',
      key: 'actions',
      render: (record: Order) => (
        <Space>
          <Button size="small" onClick={() => {
            setSelectedOrder(record);
            setOrderDetailsVisible(true);
          }}>
            详情
          </Button>
          {record.status === 'pending' && (
            <Popconfirm title="确认取消订单?" onConfirm={() => handleOrderAction('cancel', record.id)}>
              <Button size="small" danger>取消</Button>
            </Popconfirm>
          )}
        </Space>
      )
    }
  ];

  // 仓位表格列定义
  const positionColumns = [
    { title: '交易对', dataIndex: 'symbol', key: 'symbol' },
    { 
      title: '方向', 
      dataIndex: 'side', 
      key: 'side',
      render: (side: string) => (
        <Tag color={side === 'long' ? 'green' : 'red'}>
          {side === 'long' ? '多头' : '空头'}
        </Tag>
      )
    },
    { title: '持仓量', dataIndex: 'size', key: 'size' },
    { title: '开仓价', dataIndex: 'entry_price', key: 'entry_price' },
    { title: '标记价', dataIndex: 'mark_price', key: 'mark_price' },
    { 
      title: '盈亏', 
      dataIndex: 'pnl', 
      key: 'pnl',
      render: (pnl: number) => (
        <Text type={pnl >= 0 ? 'success' : 'danger'}>
          {pnl >= 0 ? '+' : ''}{pnl.toFixed(2)}
        </Text>
      )
    },
    { 
      title: '盈亏率', 
      dataIndex: 'pnl_percentage', 
      key: 'pnl_percentage',
      render: (pnl: number) => (
        <Text type={pnl >= 0 ? 'success' : 'danger'}>
          {pnl >= 0 ? '+' : ''}{pnl.toFixed(2)}%
        </Text>
      )
    },
    { title: '保证金', dataIndex: 'margin', key: 'margin' },
    {
      title: '操作',
      key: 'actions',
      render: (record: Position) => (
        <Space>
          <Button size="small" onClick={() => {
            setSelectedPosition(record);
            setPositionDetailsVisible(true);
          }}>
            详情
          </Button>
          <Popconfirm title="确认平仓?" onConfirm={() => handlePositionAction('close', record.id)}>
            <Button size="small" danger>平仓</Button>
          </Popconfirm>
        </Space>
      )
    }
  ];

  return (
    <div style={{ padding: '24px' }}>
      <div style={{ marginBottom: '24px' }}>
        <Title level={2}>交易管理中心</Title>
        <Text type="secondary">订单监控、仓位管理、资金管理、风险控制</Text>
      </div>

      <Tabs 
        activeKey={activeTab} 
        onChange={setActiveTab} 
        size="large"
        items={[
          {
            key: 'orders',
            label: `订单监控 (${orders.length})`,
            children: (
              <>
                {/* 概览统计 - 扩展显示更多API数据 */}
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="活跃订单"
                        value={orders.length}
                        prefix={<SwapOutlined />}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="成交率"
                        value={orderStats?.fill_rate || 0}
                        precision={1}
                        suffix="%"
                        prefix={<CheckCircleOutlined />}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="平均执行时间"
                        value={orderLatency?.avg_latency || orderStats?.avg_execution_time || 0}
                        suffix="ms"
                        prefix={<ClockCircleOutlined />}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="平均滑点"
                        value={orderSlippage?.avg_slippage || 0}
                        precision={3}
                        suffix="%"
                        prefix={<WarningOutlined />}
                        valueStyle={{ color: orderSlippage?.avg_slippage > 0.1 ? '#cf1322' : '#3f8600' }}
                      />
                    </Card>
                  </Col>
                </Row>

                {/* 订单执行质量和类型分析 */}
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} md={12}>
                    <Card title="订单执行质量分析" size="small">
                      <Row gutter={16}>
                        <Col span={12}>
                          <Statistic
                            title="执行质量分数"
                            value={orderExecutionQuality?.avg_quality || 0}
                            precision={1}
                            suffix="/100"
                          />
                        </Col>
                        <Col span={12}>
                          <Statistic
                            title="最优价格达成率"
                            value={orderExecutionQuality?.optimal_price_rate || 0}
                            precision={1}
                            suffix="%"
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card title="订单类型分布" size="small">
                      <Row gutter={16}>
                        <Col span={8}>
                          <Statistic
                            title="限价单"
                            value={orderTypeStats?.limit_orders || 0}
                          />
                        </Col>
                        <Col span={8}>
                          <Statistic
                            title="市价单"
                            value={orderTypeStats?.market_orders || 0}
                          />
                        </Col>
                        <Col span={8}>
                          <Statistic
                            title="止损单"
                            value={orderTypeStats?.stop_orders || 0}
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                </Row>

                {/* 最近成交记录 */}
                {recentTrades && recentTrades.length > 0 && (
                  <Card title="最近成交记录" style={{ marginBottom: '24px' }}>
                    <List
                      size="small"
                      dataSource={recentTrades.slice(0, 5)}
                      renderItem={(trade: any) => (
                        <List.Item>
                          <List.Item.Meta
                            title={`${trade.symbol} ${trade.side === 'buy' ? '买入' : '卖出'}`}
                            description={`成交价: ${trade.price} | 数量: ${trade.quantity} | 时间: ${new Date(trade.timestamp).toLocaleTimeString()}`}
                          />
                          <Text type={trade.side === 'buy' ? 'success' : 'danger'}>
                            {trade.amount}
                          </Text>
                        </List.Item>
                      )}
                    />
                  </Card>
                )}

                {/* 订单列表 */}
                <Card
                  title="活跃订单"
                  extra={
                    <Space>
                      <Button 
                        icon={<DeleteOutlined />} 
                        onClick={() => handleOrderAction('batchCancel')}
                        danger
                      >
                        批量取消
                      </Button>
                      <Button 
                        icon={<ReloadOutlined />} 
                        onClick={() => handleOrderAction('refresh')} 
                        loading={loading}
                      >
                        刷新
                      </Button>
                    </Space>
                  }
                >
                  <Table
                    dataSource={orders}
                    columns={orderColumns}
                    rowKey="id"
                    loading={loading}
                    pagination={{ pageSize: 10 }}
                    scroll={{ x: 1200 }}
                  />
                </Card>
              </>
            )
          },
          {
            key: 'positions',
            label: `仓位监控 (${positions.length})`,
            children: (
              <>
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="持仓数量"
                        value={positions.length}
                        prefix={<FundOutlined />}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="总盈亏"
                        value={totalPnl}
                        precision={2}
                        prefix={totalPnl >= 0 ? <RiseOutlined /> : <FallOutlined />}
                        valueStyle={{ color: totalPnl >= 0 ? '#3f8600' : '#cf1322' }}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="多头敞口"
                        value={positionExposure?.long_exposure || 0}
                        precision={2}
                        prefix={<RiseOutlined />}
                        valueStyle={{ color: '#3f8600' }}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="空头敞口"
                        value={positionExposure?.short_exposure || 0}
                        precision={2}
                        prefix={<FallOutlined />}
                        valueStyle={{ color: '#cf1322' }}
                      />
                    </Card>
                  </Col>
                </Row>

                {/* 仓位风险分析 */}
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} md={12}>
                    <Card title="保证金状态" size="small">
                      <Row gutter={16}>
                        <Col span={12}>
                          <Statistic
                            title="总保证金"
                            value={marginRequirement?.total_margin || 0}
                            precision={2}
                            suffix="USDT"
                          />
                        </Col>
                        <Col span={12}>
                          <Statistic
                            title="维持保证金"
                            value={marginRequirement?.maintenance_margin || 0}
                            precision={2}
                            suffix="USDT"
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card title="强平风险评估" size="small">
                      <Row gutter={16}>
                        <Col span={12}>
                          <Statistic
                            title="风险等级"
                            value={liquidationRisk?.risk_level || 'low'}
                            valueStyle={{ 
                              color: {
                                low: '#3f8600',
                                medium: '#fa8c16',
                                high: '#cf1322'
                              }[liquidationRisk?.risk_level] || '#3f8600'
                            }}
                          />
                        </Col>
                        <Col span={12}>
                          <Statistic
                            title="强平距离"
                            value={liquidationRisk?.distance_to_liquidation || 0}
                            precision={2}
                            suffix="%"
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                </Row>

                {/* 仓位性能分析 */}
                {positionPerformance && (
                  <Card title="仓位性能分析" style={{ marginBottom: '24px' }}>
                    <Row gutter={16}>
                      <Col span={6}>
                        <Statistic
                          title="胜率"
                          value={positionPerformance.win_rate || 0}
                          precision={1}
                          suffix="%"
                        />
                      </Col>
                      <Col span={6}>
                        <Statistic
                          title="平均持仓时间"
                          value={positionPerformance.avg_holding_time || 0}
                          suffix="小时"
                        />
                      </Col>
                      <Col span={6}>
                        <Statistic
                          title="最大单笔盈利"
                          value={positionPerformance.max_profit || 0}
                          precision={2}
                          valueStyle={{ color: '#3f8600' }}
                        />
                      </Col>
                      <Col span={6}>
                        <Statistic
                          title="最大单笔亏损"
                          value={positionPerformance.max_loss || 0}
                          precision={2}
                          valueStyle={{ color: '#cf1322' }}
                        />
                      </Col>
                    </Row>
                  </Card>
                )}

                <Card
                  title="当前持仓"
                  extra={
                    <Space>
                      <Button 
                        icon={<SafetyOutlined />} 
                        onClick={() => handlePositionAction('hedge')}
                      >
                        一键对冲
                      </Button>
                      <Button 
                        icon={<ReloadOutlined />} 
                        onClick={initializeData} 
                        loading={loading}
                      >
                        刷新
                      </Button>
                    </Space>
                  }
                >
                  <Table
                    dataSource={positions}
                    columns={positionColumns}
                    rowKey="id"
                    loading={loading}
                    pagination={{ pageSize: 10 }}
                    scroll={{ x: 1200 }}
                  />
                </Card>
              </>
            )
          },
          {
            key: 'funds',
            label: '资金管理',
            children: (
              <>
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} sm={8}>
                    <Card>
                      <Statistic
                        title="账户总余额"
                        value={accountBalance?.total_balance || 0}
                        precision={2}
                        suffix="USDT"
                        prefix={<DollarOutlined />}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={8}>
                    <Card>
                      <Statistic
                        title="可用余额"
                        value={accountBalance?.available_balance || 0}
                        precision={2}
                        suffix="USDT"
                        prefix={<FundOutlined />}
                        valueStyle={{ color: '#3f8600' }}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={8}>
                    <Card>
                      <Statistic
                        title="冻结余额"
                        value={accountBalance?.locked_balance || 0}
                        precision={2}
                        suffix="USDT"
                        prefix={<SafetyOutlined />}
                        valueStyle={{ color: '#fa8c16' }}
                      />
                    </Card>
                  </Col>
                </Row>

                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} md={12}>
                    <Card title="资金利用率" size="small">
                      <Progress
                        type="circle"
                        percent={Math.round((fundsUtilization?.utilization_rate || 0) * 100)}
                        format={percent => `${percent}%`}
                        status={
                          (fundsUtilization?.utilization_rate || 0) > 0.8 ? 'exception' :
                          (fundsUtilization?.utilization_rate || 0) > 0.6 ? 'active' : 'success'
                        }
                      />
                      <div style={{ marginTop: '16px', textAlign: 'center' }}>
                        <Text>效率分数: {fundsUtilization?.efficiency_score || 0}</Text>
                      </div>
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card 
                      title="快速操作" 
                      size="small"
                      extra={
                        <Button 
                          type="primary" 
                          icon={<SwapOutlined />}
                          onClick={() => setTransferModalVisible(true)}
                        >
                          资金划转
                        </Button>
                      }
                    >
                      <div style={{ lineHeight: '2.5' }}>
                        <div>最大利用率: {Math.round((fundsUtilization?.max_utilization || 0) * 100)}%</div>
                        <div>最优利用率: {Math.round((fundsUtilization?.optimal_utilization || 0) * 100)}%</div>
                        <div>当前状态: <Tag color="processing">正常</Tag></div>
                      </div>
                    </Card>
                  </Col>
                </Row>

                {/* 资产分配和绩效分析 */}
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} md={12}>
                    <Card title="资产分配分析" size="small">
                      <Row gutter={16}>
                        <Col span={8}>
                          <Statistic
                            title="现货占比"
                            value={assetDistribution?.spot_ratio || 0}
                            precision={1}
                            suffix="%"
                          />
                        </Col>
                        <Col span={8}>
                          <Statistic
                            title="期货占比"
                            value={assetDistribution?.futures_ratio || 0}
                            precision={1}
                            suffix="%"
                          />
                        </Col>
                        <Col span={8}>
                          <Statistic
                            title="杠杆占比"
                            value={assetDistribution?.margin_ratio || 0}
                            precision={1}
                            suffix="%"
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card title="资金绩效分析" size="small">
                      <Row gutter={16}>
                        <Col span={12}>
                          <Statistic
                            title="年化收益率"
                            value={fundPerformanceAnalysis?.annual_return || 0}
                            precision={2}
                            suffix="%"
                            valueStyle={{ color: fundPerformanceAnalysis?.annual_return > 0 ? '#3f8600' : '#cf1322' }}
                          />
                        </Col>
                        <Col span={12}>
                          <Statistic
                            title="波动率"
                            value={fundPerformanceAnalysis?.volatility || 0}
                            precision={2}
                            suffix="%"
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                </Row>

                {/* 现金流和交易成本分析 */}
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} md={12}>
                    <Card title="现金流分析" size="small">
                      <Row gutter={16}>
                        <Col span={12}>
                          <Statistic
                            title="净流入"
                            value={cashFlow?.net_inflow || 0}
                            precision={2}
                            suffix="USDT"
                            valueStyle={{ color: '#3f8600' }}
                          />
                        </Col>
                        <Col span={12}>
                          <Statistic
                            title="净流出"
                            value={cashFlow?.net_outflow || 0}
                            precision={2}
                            suffix="USDT"
                            valueStyle={{ color: '#cf1322' }}
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card title="交易成本分析" size="small">
                      <Row gutter={16}>
                        <Col span={12}>
                          <Statistic
                            title="总交易费用"
                            value={transactionCosts?.total_fees || 0}
                            precision={2}
                            suffix="USDT"
                          />
                        </Col>
                        <Col span={12}>
                          <Statistic
                            title="费率优化"
                            value={transactionCosts?.fee_optimization || 0}
                            precision={1}
                            suffix="%"
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                </Row>

                {/* 流动性和信用状态 */}
                <Row gutter={[16, 16]}>
                  <Col xs={24} md={12}>
                    <Card title="流动性分析" size="small">
                      <Row gutter={16}>
                        <Col span={12}>
                          <Statistic
                            title="流动性评分"
                            value={liquidityAnalysis?.liquidity_score || 0}
                            precision={1}
                            suffix="/100"
                          />
                        </Col>
                        <Col span={12}>
                          <Statistic
                            title="可变现比例"
                            value={liquidityAnalysis?.liquidatable_ratio || 0}
                            precision={1}
                            suffix="%"
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card title="信用额度管理" size="small">
                      <Row gutter={16}>
                        <Col span={12}>
                          <Statistic
                            title="可用信用额度"
                            value={creditLine?.available_credit || 0}
                            precision={2}
                            suffix="USDT"
                          />
                        </Col>
                        <Col span={12}>
                          <Statistic
                            title="信用使用率"
                            value={creditLine?.credit_utilization || 0}
                            precision={1}
                            suffix="%"
                          />
                        </Col>
                      </Row>
                    </Card>
                  </Col>
                </Row>
              </>
            )
          },
          {
            key: 'risk',
            label: '风险控制',
            children: (
              <>
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="当前风险等级"
                        value={riskMetrics?.current_risk_level || 'low'}
                        valueStyle={{ 
                          color: {
                            low: '#3f8600',
                            medium: '#fa8c16', 
                            high: '#ff7875',
                            critical: '#cf1322'
                          }[riskMetrics?.current_risk_level || 'low']
                        }}
                        prefix={<SafetyOutlined />}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="总敞口"
                        value={riskMetrics?.total_exposure || 0}
                        precision={2}
                        prefix={<BarChartOutlined />}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="最大回撤"
                        value={Math.abs(riskMetrics?.max_drawdown || 0)}
                        precision={2}
                        suffix="%"
                        prefix={<FallOutlined />}
                        valueStyle={{ color: '#cf1322' }}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="夏普比率"
                        value={riskMetrics?.sharpe_ratio || 0}
                        precision={2}
                        prefix={<LineChartOutlined />}
                      />
                    </Card>
                  </Col>
                </Row>

                <Row gutter={[16, 16]}>
                  <Col xs={24} md={12}>
                    <Card 
                      title="风险告警" 
                      size="small"
                      extra={
                        <Button 
                          danger 
                          icon={<StopOutlined />}
                          onClick={() => handleRiskAction('emergencyStop')}
                        >
                          紧急止损
                        </Button>
                      }
                    >
                      {riskAlerts.length > 0 ? (
                        riskAlerts.map(alert => (
                          <Alert
                            key={alert.id}
                            message={alert.message}
                            type={alert.type}
                            showIcon
                            style={{ marginBottom: 8 }}
                          />
                        ))
                      ) : (
                        <Text type="secondary">暂无风险告警</Text>
                      )}
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card 
                      title="风险限额设置" 
                      size="small"
                      extra={
                        <Button 
                          icon={<SettingOutlined />}
                          onClick={() => setRiskLimitsModalVisible(true)}
                        >
                          设置限额
                        </Button>
                      }
                    >
                      <div style={{ lineHeight: '2.5' }}>
                        <div>最大持仓: {riskMetrics?.risk_limits?.max_position_size || 0}</div>
                        <div>最大日亏损: {riskMetrics?.risk_limits?.max_daily_loss || 0}</div>
                        <div>杠杆限制: {riskMetrics?.risk_limits?.leverage_limit || 0}x</div>
                      </div>
                    </Card>
                  </Col>
                </Row>
              </>
            )
          }
        ]}
      />

      {/* 资金划转模态框 */}
      <Modal
        title="资金划转"
        open={transferModalVisible}
        onCancel={() => setTransferModalVisible(false)}
        onOk={() => transferForm.submit()}
      >
        <Form form={transferForm} onFinish={handleFundsTransfer} layout="vertical">
          <Form.Item name="from" label="从" rules={[{ required: true }]}>
            <Select placeholder="选择源账户">
              <Option value="spot">现货账户</Option>
              <Option value="futures">期货账户</Option>
              <Option value="margin">杠杆账户</Option>
            </Select>
          </Form.Item>
          <Form.Item name="to" label="到" rules={[{ required: true }]}>
            <Select placeholder="选择目标账户">
              <Option value="spot">现货账户</Option>
              <Option value="futures">期货账户</Option>
              <Option value="margin">杠杆账户</Option>
            </Select>
          </Form.Item>
          <Form.Item name="amount" label="金额" rules={[{ required: true }]}>
            <InputNumber 
              style={{ width: '100%' }} 
              placeholder="输入划转金额"
              min={0}
              precision={2}
            />
          </Form.Item>
        </Form>
      </Modal>

      {/* 风险限额设置模态框 */}
      <Modal
        title="风险限额设置"
        open={riskLimitsModalVisible}
        onCancel={() => setRiskLimitsModalVisible(false)}
        onOk={() => handleRiskAction('updateLimits')}
      >
        <Form 
          form={riskLimitsForm} 
          layout="vertical"
          initialValues={riskMetrics?.risk_limits}
        >
          <Form.Item name="max_position_size" label="最大持仓限制">
            <InputNumber style={{ width: '100%' }} min={0} />
          </Form.Item>
          <Form.Item name="max_daily_loss" label="最大日亏损限制">
            <InputNumber style={{ width: '100%' }} min={0} />
          </Form.Item>
          <Form.Item name="leverage_limit" label="杠杆限制">
            <InputNumber style={{ width: '100%' }} min={1} max={100} />
          </Form.Item>
        </Form>
      </Modal>

      {/* 订单详情模态框 */}
      <Modal
        title="订单详情"
        open={orderDetailsVisible}
        onCancel={() => setOrderDetailsVisible(false)}
        footer={null}
        width={600}
      >
        {selectedOrder && (
          <Descriptions column={2} bordered>
            <Descriptions.Item label="订单ID">{selectedOrder.id}</Descriptions.Item>
            <Descriptions.Item label="交易对">{selectedOrder.symbol}</Descriptions.Item>
            <Descriptions.Item label="方向">
              <Tag color={selectedOrder.side === 'buy' ? 'green' : 'red'}>
                {selectedOrder.side === 'buy' ? '买入' : '卖出'}
              </Tag>
            </Descriptions.Item>
            <Descriptions.Item label="类型">{selectedOrder.type}</Descriptions.Item>
            <Descriptions.Item label="数量">{selectedOrder.amount}</Descriptions.Item>
            <Descriptions.Item label="价格">{selectedOrder.price}</Descriptions.Item>
            <Descriptions.Item label="已成交">{selectedOrder.filled}</Descriptions.Item>
            <Descriptions.Item label="状态">
              <Badge status="processing" text={selectedOrder.status} />
            </Descriptions.Item>
            <Descriptions.Item label="执行质量">{selectedOrder.execution_quality || 'N/A'}</Descriptions.Item>
            <Descriptions.Item label="延迟">{selectedOrder.latency || 'N/A'}ms</Descriptions.Item>
            <Descriptions.Item label="滑点">{selectedOrder.slippage || 'N/A'}%</Descriptions.Item>
            <Descriptions.Item label="创建时间">
              {new Date(selectedOrder.created_at).toLocaleString()}
            </Descriptions.Item>
          </Descriptions>
        )}
      </Modal>

      {/* 仓位详情模态框 */}
      <Modal
        title="仓位详情"
        open={positionDetailsVisible}
        onCancel={() => setPositionDetailsVisible(false)}
        footer={null}
        width={600}
      >
        {selectedPosition && (
          <Descriptions column={2} bordered>
            <Descriptions.Item label="交易对">{selectedPosition.symbol}</Descriptions.Item>
            <Descriptions.Item label="方向">
              <Tag color={selectedPosition.side === 'long' ? 'green' : 'red'}>
                {selectedPosition.side === 'long' ? '多头' : '空头'}
              </Tag>
            </Descriptions.Item>
            <Descriptions.Item label="持仓量">{selectedPosition.size}</Descriptions.Item>
            <Descriptions.Item label="开仓价">{selectedPosition.entry_price}</Descriptions.Item>
            <Descriptions.Item label="标记价">{selectedPosition.mark_price}</Descriptions.Item>
            <Descriptions.Item label="盈亏">
              <Text type={selectedPosition.pnl >= 0 ? 'success' : 'danger'}>
                {selectedPosition.pnl >= 0 ? '+' : ''}{selectedPosition.pnl.toFixed(2)}
              </Text>
            </Descriptions.Item>
            <Descriptions.Item label="盈亏率">
              <Text type={selectedPosition.pnl_percentage >= 0 ? 'success' : 'danger'}>
                {selectedPosition.pnl_percentage >= 0 ? '+' : ''}{selectedPosition.pnl_percentage.toFixed(2)}%
              </Text>
            </Descriptions.Item>
            <Descriptions.Item label="保证金">{selectedPosition.margin}</Descriptions.Item>
            <Descriptions.Item label="强平价">{selectedPosition.liquidation_price}</Descriptions.Item>
            <Descriptions.Item label="创建时间">
              {new Date(selectedPosition.created_at).toLocaleString()}
            </Descriptions.Item>
          </Descriptions>
        )}
      </Modal>
    </div>
  );
};

export default TradingModule;