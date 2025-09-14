import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import type { 
  StrategyConfig, 
  RiskLimits, 
  StrategyPerformance,
  MLModelConfig,
  ProductionAPIConfig,
  ShadowTradingConfig,
  ApprovalWorkflowConfig,
  OrderInfo
} from '@/types/celue';

// CeLue模块状态类型
interface CeLueState {
  // AI/ML模型状态
  mlModels: {
    [modelId: string]: {
      id: string;
      name: string;
      type: string;
      status: 'training' | 'deployed' | 'inactive' | 'error' | 'validating';
      config: MLModelConfig;
      performance: {
        accuracy: number;
        precision: number;
        recall: number;
        f1Score: number;
        lastEvaluation: string;
      };
      training: {
        progress: number;
        currentEpoch: number;
        totalEpochs: number;
        loss: number;
        estimatedCompletion: string | null;
      };
      deployment: {
        endpoint: string | null;
        version: string;
        instances: number;
        healthStatus: 'healthy' | 'degraded' | 'unhealthy';
      };
    };
  };
  
  // 生产级API执行器状态
  productionAPI: {
    status: 'active' | 'inactive' | 'maintenance' | 'error';
    config: ProductionAPIConfig;
    stats: {
      totalTrades: number;
      successfulTrades: number;
      failedTrades: number;
      totalVolume: number;
      avgExecutionTime: number;
      profitLoss: number;
    };
    activeArbitrages: {
      [tradeId: string]: {
        id: string;
        status: 'executing' | 'completed' | 'failed' | 'cancelled';
        legs: Array<{
          exchange: string;
          symbol: string;
          side: string;
          quantity: number;
          price: number;
          status: string;
          orderId?: string;
        }>;
        estimatedProfit: number;
        actualProfit?: number;
        startTime: string;
        endTime?: string;
      };
    };
    exchanges: {
      [exchange: string]: {
        status: 'online' | 'offline' | 'degraded';
        latency: number;
        errorRate: number;
        rateLimitUsage: number;
        lastError?: string;
      };
    };
  };
  
  // 影子交易系统状态
  shadowTrading: {
    enabled: boolean;
    config: ShadowTradingConfig;
    status: 'running' | 'paused' | 'stopped' | 'error';
    virtualCapital: number;
    currentPnL: number;
    stats: {
      totalTrades: number;
      winRate: number;
      avgProfit: number;
      maxDrawdown: number;
      sharpeRatio: number;
    };
    comparison: {
      realVsShadow: {
        correlation: number;
        divergenceEvents: number;
        performanceDiff: number;
      };
    };
    backtests: {
      [backtestId: string]: {
        id: string;
        status: 'running' | 'completed' | 'failed' | 'cancelled';
        progress: number;
        results?: {
          totalReturn: number;
          sharpeRatio: number;
          maxDrawdown: number;
          totalTrades: number;
          winRate: number;
        };
        startTime: string;
        endTime?: string;
      };
    };
  };
  
  // 审批工作流系统状态
  approvalWorkflow: {
    enabled: boolean;
    config: ApprovalWorkflowConfig;
    workflows: {
      [workflowId: string]: {
        id: string;
        name: string;
        status: 'active' | 'inactive' | 'suspended';
        activeRequests: number;
        completionRate: number;
        avgProcessingTime: number;
      };
    };
    pendingApprovals: Array<{
      id: string;
      type: string;
      description: string;
      requestedBy: string;
      createdAt: string;
      urgency: 'low' | 'medium' | 'high' | 'critical';
      estimatedImpact: string;
      currentApprovers: string[];
      deadline: string;
    }>;
    approvalHistory: {
      totalRequests: number;
      approved: number;
      rejected: number;
      avgProcessingTime: number;
    };
  };
  
  // 策略编排器状态
  strategies: {
    [strategyId: string]: {
      id: string;
      name: string;
      type: string;
      status: 'active' | 'paused' | 'stopped' | 'error';
      config: StrategyConfig;
      performance: StrategyPerformance;
      risk: {
        currentExposure: number;
        maxExposure: number;
        riskScore: number;
        violations: string[];
      };
      lastUpdate: string;
    };
  };
  
  // 风险管理状态
  riskManager: {
    status: 'active' | 'inactive' | 'emergency_stop';
    limits: RiskLimits;
    currentPositions: Array<{
      exchange: string;
      symbol: string;
      position: number;
      marketValue: number;
      pnl: number;
      riskScore: number;
    }>;
    exposure: {
      totalExposure: number;
      byExchange: Record<string, number>;
      byAsset: Record<string, number>;
      concentrationRisk: number;
    };
    violations: Array<{
      id: string;
      type: string;
      severity: 'low' | 'medium' | 'high' | 'critical';
      message: string;
      timestamp: string;
      resolved: boolean;
    }>;
    circuitBreaker: {
      active: boolean;
      triggeredBy?: string;
      triggeredAt?: string;
      autoResetTime?: string;
    };
  };
  
  // 订单执行管理状态
  orderManager: {
    activeOrders: {
      [orderId: string]: OrderInfo;
    };
    orderHistory: {
      total: number;
      filled: number;
      cancelled: number;
      failed: number;
    };
    executionStats: {
      avgFillTime: number;
      totalVolume: number;
      totalFees: number;
      slippage: {
        avg: number;
        max: number;
        bySymbol: Record<string, number>;
      };
    };
    routing: {
      rules: Array<{
        condition: string;
        targetExchange: string;
        priority: number;
      }>;
      defaultExchange: string;
      optimization: 'latency' | 'fees' | 'liquidity';
    };
  };
  
  // 整体状态
  overallStatus: 'healthy' | 'warning' | 'critical' | 'maintenance';
  lastUpdate: string | null;
  activeAlerts: Array<{
    id: string;
    level: 'info' | 'warning' | 'error' | 'critical';
    component: string;
    message: string;
    timestamp: string;
    acknowledged: boolean;
  }>;
}

// 初始状态
const initialState: CeLueState = {
  mlModels: {},
  productionAPI: {
    status: 'inactive',
    config: {} as ProductionAPIConfig,
    stats: {
      totalTrades: 0,
      successfulTrades: 0,
      failedTrades: 0,
      totalVolume: 0,
      avgExecutionTime: 0,
      profitLoss: 0,
    },
    activeArbitrages: {},
    exchanges: {},
  },
  shadowTrading: {
    enabled: false,
    config: {} as ShadowTradingConfig,
    status: 'stopped',
    virtualCapital: 0,
    currentPnL: 0,
    stats: {
      totalTrades: 0,
      winRate: 0,
      avgProfit: 0,
      maxDrawdown: 0,
      sharpeRatio: 0,
    },
    comparison: {
      realVsShadow: {
        correlation: 0,
        divergenceEvents: 0,
        performanceDiff: 0,
      },
    },
    backtests: {},
  },
  approvalWorkflow: {
    enabled: false,
    config: {} as ApprovalWorkflowConfig,
    workflows: {},
    pendingApprovals: [],
    approvalHistory: {
      totalRequests: 0,
      approved: 0,
      rejected: 0,
      avgProcessingTime: 0,
    },
  },
  strategies: {},
  riskManager: {
    status: 'inactive',
    limits: {} as RiskLimits,
    currentPositions: [],
    exposure: {
      totalExposure: 0,
      byExchange: {},
      byAsset: {},
      concentrationRisk: 0,
    },
    violations: [],
    circuitBreaker: {
      active: false,
    },
  },
  orderManager: {
    activeOrders: {},
    orderHistory: {
      total: 0,
      filled: 0,
      cancelled: 0,
      failed: 0,
    },
    executionStats: {
      avgFillTime: 0,
      totalVolume: 0,
      totalFees: 0,
      slippage: {
        avg: 0,
        max: 0,
        bySymbol: {},
      },
    },
    routing: {
      rules: [],
      defaultExchange: '',
      optimization: 'latency',
    },
  },
  overallStatus: 'critical',
  lastUpdate: null,
  activeAlerts: [],
};

// CeLue slice
const celueSlice = createSlice({
  name: 'celue',
  initialState,
  reducers: {
    // ML模型管理
    addMLModel: (state, action: PayloadAction<CeLueState['mlModels'][string]>) => {
      const model = action.payload;
      state.mlModels[model.id] = model;
    },
    
    removeMLModel: (state, action: PayloadAction<string>) => {
      delete state.mlModels[action.payload];
    },
    
    updateMLModelStatus: (state, action: PayloadAction<{ modelId: string; status: CeLueState['mlModels'][string]['status'] }>) => {
      const { modelId, status } = action.payload;
      if (state.mlModels[modelId]) {
        state.mlModels[modelId].status = status;
      }
    },
    
    updateMLModelTraining: (state, action: PayloadAction<{ modelId: string; training: Partial<CeLueState['mlModels'][string]['training']> }>) => {
      const { modelId, training } = action.payload;
      if (state.mlModels[modelId]) {
        state.mlModels[modelId].training = { ...state.mlModels[modelId].training, ...training };
      }
    },
    
    updateMLModelPerformance: (state, action: PayloadAction<{ modelId: string; performance: Partial<CeLueState['mlModels'][string]['performance']> }>) => {
      const { modelId, performance } = action.payload;
      if (state.mlModels[modelId]) {
        state.mlModels[modelId].performance = { ...state.mlModels[modelId].performance, ...performance };
      }
    },
    
    // 生产API管理
    updateProductionAPIStatus: (state, action: PayloadAction<CeLueState['productionAPI']['status']>) => {
      state.productionAPI.status = action.payload;
    },
    
    updateProductionAPIStats: (state, action: PayloadAction<Partial<CeLueState['productionAPI']['stats']>>) => {
      state.productionAPI.stats = { ...state.productionAPI.stats, ...action.payload };
    },
    
    addActiveArbitrage: (state, action: PayloadAction<CeLueState['productionAPI']['activeArbitrages'][string]>) => {
      const arbitrage = action.payload;
      state.productionAPI.activeArbitrages[arbitrage.id] = arbitrage;
    },
    
    updateActiveArbitrage: (state, action: PayloadAction<{ tradeId: string; updates: Partial<CeLueState['productionAPI']['activeArbitrages'][string]> }>) => {
      const { tradeId, updates } = action.payload;
      if (state.productionAPI.activeArbitrages[tradeId]) {
        state.productionAPI.activeArbitrages[tradeId] = { ...state.productionAPI.activeArbitrages[tradeId], ...updates };
      }
    },
    
    removeActiveArbitrage: (state, action: PayloadAction<string>) => {
      delete state.productionAPI.activeArbitrages[action.payload];
    },
    
    updateExchangeStatus: (state, action: PayloadAction<{ exchange: string; status: Partial<CeLueState['productionAPI']['exchanges'][string]> }>) => {
      const { exchange, status } = action.payload;
      if (!state.productionAPI.exchanges[exchange]) {
        state.productionAPI.exchanges[exchange] = {
          status: 'offline',
          latency: 0,
          errorRate: 0,
          rateLimitUsage: 0,
        };
      }
      state.productionAPI.exchanges[exchange] = { ...state.productionAPI.exchanges[exchange], ...status };
    },
    
    // 影子交易管理
    updateShadowTradingStatus: (state, action: PayloadAction<{ enabled?: boolean; status?: CeLueState['shadowTrading']['status'] }>) => {
      if (action.payload.enabled !== undefined) {
        state.shadowTrading.enabled = action.payload.enabled;
      }
      if (action.payload.status !== undefined) {
        state.shadowTrading.status = action.payload.status;
      }
    },
    
    updateShadowTradingStats: (state, action: PayloadAction<Partial<CeLueState['shadowTrading']['stats']>>) => {
      state.shadowTrading.stats = { ...state.shadowTrading.stats, ...action.payload };
    },
    
    updateShadowTradingCapital: (state, action: PayloadAction<{ virtualCapital?: number; currentPnL?: number }>) => {
      if (action.payload.virtualCapital !== undefined) {
        state.shadowTrading.virtualCapital = action.payload.virtualCapital;
      }
      if (action.payload.currentPnL !== undefined) {
        state.shadowTrading.currentPnL = action.payload.currentPnL;
      }
    },
    
    addBacktest: (state, action: PayloadAction<CeLueState['shadowTrading']['backtests'][string]>) => {
      const backtest = action.payload;
      state.shadowTrading.backtests[backtest.id] = backtest;
    },
    
    updateBacktest: (state, action: PayloadAction<{ backtestId: string; updates: Partial<CeLueState['shadowTrading']['backtests'][string]> }>) => {
      const { backtestId, updates } = action.payload;
      if (state.shadowTrading.backtests[backtestId]) {
        state.shadowTrading.backtests[backtestId] = { ...state.shadowTrading.backtests[backtestId], ...updates };
      }
    },
    
    // 策略管理
    addStrategy: (state, action: PayloadAction<CeLueState['strategies'][string]>) => {
      const strategy = action.payload;
      state.strategies[strategy.id] = strategy;
    },
    
    removeStrategy: (state, action: PayloadAction<string>) => {
      delete state.strategies[action.payload];
    },
    
    updateStrategyStatus: (state, action: PayloadAction<{ strategyId: string; status: CeLueState['strategies'][string]['status'] }>) => {
      const { strategyId, status } = action.payload;
      if (state.strategies[strategyId]) {
        state.strategies[strategyId].status = status;
        state.strategies[strategyId].lastUpdate = new Date().toISOString();
      }
    },
    
    updateStrategyPerformance: (state, action: PayloadAction<{ strategyId: string; performance: Partial<StrategyPerformance> }>) => {
      const { strategyId, performance } = action.payload;
      if (state.strategies[strategyId]) {
        state.strategies[strategyId].performance = { ...state.strategies[strategyId].performance, ...performance };
        state.strategies[strategyId].lastUpdate = new Date().toISOString();
      }
    },
    
    updateStrategyRisk: (state, action: PayloadAction<{ strategyId: string; risk: Partial<CeLueState['strategies'][string]['risk']> }>) => {
      const { strategyId, risk } = action.payload;
      if (state.strategies[strategyId]) {
        state.strategies[strategyId].risk = { ...state.strategies[strategyId].risk, ...risk };
        state.strategies[strategyId].lastUpdate = new Date().toISOString();
      }
    },
    
    // 风险管理
    updateRiskManagerStatus: (state, action: PayloadAction<CeLueState['riskManager']['status']>) => {
      state.riskManager.status = action.payload;
    },
    
    updateRiskLimits: (state, action: PayloadAction<Partial<RiskLimits>>) => {
      state.riskManager.limits = { ...state.riskManager.limits, ...action.payload };
    },
    
    updateCurrentPositions: (state, action: PayloadAction<CeLueState['riskManager']['currentPositions']>) => {
      state.riskManager.currentPositions = action.payload;
    },
    
    updateExposure: (state, action: PayloadAction<Partial<CeLueState['riskManager']['exposure']>>) => {
      state.riskManager.exposure = { ...state.riskManager.exposure, ...action.payload };
    },
    
    addRiskViolation: (state, action: PayloadAction<CeLueState['riskManager']['violations'][number]>) => {
      state.riskManager.violations.push(action.payload);
    },
    
    resolveRiskViolation: (state, action: PayloadAction<string>) => {
      const violation = state.riskManager.violations.find(v => v.id === action.payload);
      if (violation) {
        violation.resolved = true;
      }
    },
    
    updateCircuitBreaker: (state, action: PayloadAction<Partial<CeLueState['riskManager']['circuitBreaker']>>) => {
      state.riskManager.circuitBreaker = { ...state.riskManager.circuitBreaker, ...action.payload };
    },
    
    // 订单管理
    addActiveOrder: (state, action: PayloadAction<OrderInfo>) => {
      state.orderManager.activeOrders[action.payload.id] = action.payload;
    },
    
    updateActiveOrder: (state, action: PayloadAction<{ orderId: string; updates: Partial<OrderInfo> }>) => {
      const { orderId, updates } = action.payload;
      if (state.orderManager.activeOrders[orderId]) {
        state.orderManager.activeOrders[orderId] = { ...state.orderManager.activeOrders[orderId], ...updates };
      }
    },
    
    removeActiveOrder: (state, action: PayloadAction<string>) => {
      delete state.orderManager.activeOrders[action.payload];
    },
    
    updateOrderHistory: (state, action: PayloadAction<Partial<CeLueState['orderManager']['orderHistory']>>) => {
      state.orderManager.orderHistory = { ...state.orderManager.orderHistory, ...action.payload };
    },
    
    updateExecutionStats: (state, action: PayloadAction<Partial<CeLueState['orderManager']['executionStats']>>) => {
      state.orderManager.executionStats = { ...state.orderManager.executionStats, ...action.payload };
    },
    
    // 审批工作流管理
    updateApprovalWorkflowStatus: (state, action: PayloadAction<{ enabled?: boolean; config?: Partial<ApprovalWorkflowConfig> }>) => {
      if (action.payload.enabled !== undefined) {
        state.approvalWorkflow.enabled = action.payload.enabled;
      }
      if (action.payload.config) {
        state.approvalWorkflow.config = { ...state.approvalWorkflow.config, ...action.payload.config };
      }
    },
    
    addPendingApproval: (state, action: PayloadAction<CeLueState['approvalWorkflow']['pendingApprovals'][number]>) => {
      state.approvalWorkflow.pendingApprovals.push(action.payload);
    },
    
    removePendingApproval: (state, action: PayloadAction<string>) => {
      state.approvalWorkflow.pendingApprovals = state.approvalWorkflow.pendingApprovals.filter(approval => approval.id !== action.payload);
    },
    
    updateApprovalHistory: (state, action: PayloadAction<Partial<CeLueState['approvalWorkflow']['approvalHistory']>>) => {
      state.approvalWorkflow.approvalHistory = { ...state.approvalWorkflow.approvalHistory, ...action.payload };
    },
    
    // 整体状态管理
    updateOverallStatus: (state, action: PayloadAction<CeLueState['overallStatus']>) => {
      state.overallStatus = action.payload;
      state.lastUpdate = new Date().toISOString();
    },
    
    // 告警管理
    addAlert: (state, action: PayloadAction<CeLueState['activeAlerts'][number]>) => {
      state.activeAlerts.push(action.payload);
    },
    
    removeAlert: (state, action: PayloadAction<string>) => {
      state.activeAlerts = state.activeAlerts.filter(alert => alert.id !== action.payload);
    },
    
    acknowledgeAlert: (state, action: PayloadAction<string>) => {
      const alert = state.activeAlerts.find(a => a.id === action.payload);
      if (alert) {
        alert.acknowledged = true;
      }
    },
    
    clearAllAlerts: (state) => {
      state.activeAlerts = [];
    },
    
    // 重置状态
    resetCeLueState: () => initialState,
  },
});

// 导出actions
export const {
  addMLModel,
  removeMLModel,
  updateMLModelStatus,
  updateMLModelTraining,
  updateMLModelPerformance,
  updateProductionAPIStatus,
  updateProductionAPIStats,
  addActiveArbitrage,
  updateActiveArbitrage,
  removeActiveArbitrage,
  updateExchangeStatus,
  updateShadowTradingStatus,
  updateShadowTradingStats,
  updateShadowTradingCapital,
  addBacktest,
  updateBacktest,
  addStrategy,
  removeStrategy,
  updateStrategyStatus,
  updateStrategyPerformance,
  updateStrategyRisk,
  updateRiskManagerStatus,
  updateRiskLimits,
  updateCurrentPositions,
  updateExposure,
  addRiskViolation,
  resolveRiskViolation,
  updateCircuitBreaker,
  addActiveOrder,
  updateActiveOrder,
  removeActiveOrder,
  updateOrderHistory,
  updateExecutionStats,
  updateApprovalWorkflowStatus,
  addPendingApproval,
  removePendingApproval,
  updateApprovalHistory,
  updateOverallStatus,
  addAlert,
  removeAlert,
  acknowledgeAlert,
  clearAllAlerts,
  resetCeLueState,
} = celueSlice.actions;

// 导出reducer
export default celueSlice.reducer;

// Selectors
export const selectCeLueOverallStatus = (state: { celue: CeLueState }) => state.celue.overallStatus;
export const selectMLModels = (state: { celue: CeLueState }) => state.celue.mlModels;
export const selectProductionAPI = (state: { celue: CeLueState }) => state.celue.productionAPI;
export const selectShadowTrading = (state: { celue: CeLueState }) => state.celue.shadowTrading;
export const selectApprovalWorkflow = (state: { celue: CeLueState }) => state.celue.approvalWorkflow;
export const selectStrategies = (state: { celue: CeLueState }) => state.celue.strategies;
export const selectRiskManager = (state: { celue: CeLueState }) => state.celue.riskManager;
export const selectOrderManager = (state: { celue: CeLueState }) => state.celue.orderManager;
export const selectCeLueAlerts = (state: { celue: CeLueState }) => state.celue.activeAlerts;