// CeLue策略执行模块API
import { apiClient } from './client';
import type {
  StrategyConfig,
  RiskLimits,
  StrategyPerformance,
  MLModelConfig,
  ShadowTradingConfig,
  ApprovalWorkflowConfig,
  ApprovalRule,
  OrderInfo,
  TradeLeg
} from '@/types/celue';

export const celueAPI = {
  // 2.1 AI/ML模型管理
  ml: {
    // 模型训练控制
    trainModel: (config: { name: string; type: string; data_source: string; hyperparameters: Record<string, any> }) =>
      apiClient.post<{ model_id: string; training_job_id: string; estimated_duration_minutes: number }>('/api/ml/models/train', config),
    
    getTrainingStatus: (modelId: string) =>
      apiClient.get<{ status: string; progress_percent: number; current_epoch: number; loss: number; accuracy: number; eta_minutes: number }>(`/api/ml/models/${modelId}/training-status`),
    
    stopTraining: (modelId: string) =>
      apiClient.post(`/api/ml/models/${modelId}/stop-training`),
    
    getTrainingLogs: (modelId: string) =>
      apiClient.get<Array<{ timestamp: string; epoch: number; loss: number; accuracy: number; metrics: Record<string, number> }>>(`/api/ml/models/${modelId}/training-logs`),
    
    // 模型生命周期管理
    listModels: () =>
      apiClient.get<MLModelConfig[]>('/api/ml/models/list'),
    
    deployModel: (modelId: string) =>
      apiClient.post<{ deployment_id: string; endpoint: string; status: string }>(`/api/ml/models/${modelId}/deploy`),
    
    rollbackModel: (modelId: string, versionId?: string) =>
      apiClient.post(`/api/ml/models/${modelId}/rollback`, { version_id: versionId }),
    
    deleteModel: (modelId: string) =>
      apiClient.delete(`/api/ml/models/${modelId}`),
    
    getModelPerformance: (modelId: string) =>
      apiClient.get<{ accuracy: number; precision: number; recall: number; f1_score: number; auc: number; confusion_matrix: number[][] }>(`/api/ml/models/${modelId}/performance`),
    
    // 模型持久化
    saveModel: (modelId: string, name?: string) =>
      apiClient.post(`/api/ml/models/${modelId}/save`, { name }),
    
    loadModel: (modelId: string, versionId?: string) =>
      apiClient.post(`/api/ml/models/${modelId}/load`, { version_id: versionId }),
    
    getModelVersions: (modelId: string) =>
      apiClient.get<Array<{ version_id: string; created_at: string; accuracy: number; size_mb: number; description: string }>>(`/api/ml/models/${modelId}/versions`),
    
    // 模型验证
    validateModel: (modelId: string, testData: any[]) =>
      apiClient.post<{ validation_score: number; detailed_metrics: Record<string, number>; recommendations: string[] }>(`/api/ml/models/${modelId}/validate`, { test_data: testData }),
    
    getValidationReport: (modelId: string) =>
      apiClient.get<{ overall_score: number; cross_validation_scores: number[]; feature_importance: Record<string, number>; model_complexity: number }>(`/api/ml/models/${modelId}/validation-report`),
    
    crossValidateModel: (modelId: string, folds: number) =>
      apiClient.post<{ cv_scores: number[]; mean_score: number; std_score: number; best_fold: number }>(`/api/ml/models/${modelId}/cross-validate`, { folds }),
    
    // 在线学习控制
    enableOnlineLearning: (modelId: string) =>
      apiClient.post(`/api/ml/online-learning/enable`, { model_id: modelId }),
    
    disableOnlineLearning: (modelId: string) =>
      apiClient.post(`/api/ml/online-learning/disable`, { model_id: modelId }),
    
    updateOnlineLearningParams: (config: { learning_rate: number; batch_size: number; update_frequency: number }) =>
      apiClient.put('/api/ml/online-learning/parameters', config),
    
    getOnlineLearningMetrics: (modelId?: string) =>
      apiClient.get<{ models_learning: number; total_updates: number; avg_learning_rate: number; performance_trend: Array<{ timestamp: string; accuracy: number }> }>('/api/ml/online-learning/metrics', { params: { model_id: modelId } }),
    
    retrainOnline: (modelId: string) =>
      apiClient.post(`/api/ml/online-learning/retrain`, { model_id: modelId }),
  },

  // 2.2 生产级API执行器控制
  production: {
    // 原子性套利执行
    executeArbitrage: (legs: Array<{ exchange: string; symbol: string; side: string; quantity: number; price: number }>) =>
      apiClient.post<{ trade_id: string; estimated_profit_usd: number; legs: Array<{ order_id: string; status: string }> }>('/api/production/arbitrage/execute', { legs }),
    
    getArbitrageStatus: (tradeId: string) =>
      apiClient.get<{ trade_id: string; status: string; completed_legs: number; total_legs: number; current_profit_usd: number; execution_time_ms: number }>(`/api/production/arbitrage/${tradeId}/status`),
    
    cancelArbitrage: (tradeId: string) =>
      apiClient.post(`/api/production/arbitrage/${tradeId}/cancel`),
    
    getArbitrageLegs: (tradeId: string) =>
      apiClient.get<TradeLeg[]>(`/api/production/arbitrage/${tradeId}/legs`),
    
    // 订单管理
    getActiveOrders: (exchange?: string) =>
      apiClient.get<OrderInfo[]>('/api/production/orders/active', { params: { exchange } }),
    
    cancelAllOrders: (exchange?: string) =>
      apiClient.post('/api/production/orders/cancel-all', { exchange }),
    
    getOrderFills: (orderId: string) =>
      apiClient.get<Array<{ fill_id: string; quantity: number; price: number; fees: number; timestamp: string }>>(`/api/production/orders/${orderId}/fills`),
    
    batchCancelOrders: (orderIds: string[]) =>
      apiClient.post('/api/production/orders/batch-cancel', { order_ids: orderIds }),
    
    // 执行监控
    getExecutionLatency: () =>
      apiClient.get<{ avg_latency_ms: number; p95_latency_ms: number; p99_latency_ms: number; by_exchange: Record<string, number> }>('/api/production/execution/latency'),
    
    getExecutionSuccessRate: () =>
      apiClient.get<{ overall_success_rate: number; by_exchange: Record<string, number>; by_strategy: Record<string, number> }>('/api/production/execution/success-rate'),
    
    getExecutionSlippage: () =>
      apiClient.get<{ avg_slippage_bps: number; max_slippage_bps: number; by_symbol: Record<string, number> }>('/api/production/execution/slippage'),
    
    optimizeExecution: (config: { target_latency_ms: number; max_slippage_bps: number }) =>
      apiClient.post<{ optimization_applied: string[]; expected_improvement: Record<string, number> }>('/api/production/execution/optimize', config),
    
    // API健康监控
    getExchangeHealth: () =>
      apiClient.get<Array<{ exchange: string; status: string; latency_ms: number; error_rate: number; last_error: string; uptime_percent: number }>>('/api/production/exchanges/health'),
    
    testExchange: (exchangeId: string) =>
      apiClient.post<{ success: boolean; latency_ms: number; api_endpoints_status: Record<string, boolean>; error_message?: string }>(`/api/production/exchanges/${exchangeId}/test`),
    
    getRateLimitStatus: () =>
      apiClient.get<Array<{ exchange: string; current_usage: number; limit: number; reset_time: string; overage_protection: boolean }>>('/api/production/rate-limits/status'),
  },

  // 2.3 影子交易系统控制
  shadow: {
    // 影子交易模式控制
    enable: (config: ShadowTradingConfig) =>
      apiClient.post('/api/shadow/enable', config),
    
    disable: () =>
      apiClient.post('/api/shadow/disable'),
    
    getStatus: () =>
      apiClient.get<{ enabled: boolean; mode: string; virtual_capital_usd: number; trades_executed: number; current_pnl_usd: number; uptime_hours: number }>('/api/shadow/status'),
    
    updateConfig: (config: Partial<ShadowTradingConfig>) =>
      apiClient.put('/api/shadow/config', config),
    
    // 对比分析
    getComparison: (period: string) =>
      apiClient.get<{
        period: string;
        real_trades: { count: number; pnl_usd: number; win_rate: number };
        shadow_trades: { count: number; pnl_usd: number; win_rate: number };
        divergence_analysis: { avg_difference_percent: number; max_difference_percent: number };
      }>(`/api/shadow/comparison/${period}`),
    
    getRealVsShadowPerformance: () =>
      apiClient.get<{ correlation_coefficient: number; alpha: number; beta: number; tracking_error: number; information_ratio: number }>('/api/shadow/performance/real-vs-shadow'),
    
    getDivergenceAnalysis: () =>
      apiClient.get<{ divergence_events: Array<{ timestamp: string; real_action: string; shadow_action: string; reason: string; impact_usd: number }>; total_divergences: number }>('/api/shadow/divergence/analysis'),
    
    // 回测管理
    startBacktest: (config: { strategy_id: string; start_date: string; end_date: string; initial_capital: number; benchmark?: string }) =>
      apiClient.post<{ backtest_id: string; estimated_duration_minutes: number }>('/api/shadow/backtest/start', config),
    
    getBacktestResults: (backtestId: string) =>
      apiClient.get<{
        backtest_id: string;
        status: string;
        total_return_percent: number;
        annual_return_percent: number;
        sharpe_ratio: number;
        max_drawdown_percent: number;
        total_trades: number;
        win_rate: number;
        profit_factor: number;
      }>(`/api/shadow/backtest/${backtestId}/results`),
    
    stopBacktest: (backtestId: string) =>
      apiClient.post(`/api/shadow/backtest/${backtestId}/stop`),
    
    getBacktestHistory: () =>
      apiClient.get<Array<{ backtest_id: string; strategy: string; period: string; return_percent: number; sharpe_ratio: number; completed_at: string }>>('/api/shadow/backtest/history'),
    
    // 风险测试环境
    createRiskScenario: (scenario: { name: string; market_conditions: Record<string, any>; stress_factors: Record<string, number> }) =>
      apiClient.post<{ scenario_id: string; estimated_completion_time: string }>('/api/shadow/risk-test/scenario', scenario),
    
    getRiskTestResults: (scenarioId: string) =>
      apiClient.get<{ scenario_id: string; pnl_under_stress: number; max_loss: number; var_95: number; expected_shortfall: number; stress_test_passed: boolean }>(`/api/shadow/risk-test/${scenarioId}/results`),
    
    runStressTest: (config: { scenarios: string[]; confidence_level: number; time_horizon_days: number }) =>
      apiClient.post<{ stress_test_id: string; worst_case_loss: number; scenarios_results: Array<{ scenario: string; pnl: number; probability: number }> }>('/api/shadow/stress-test/run', config),
  },

  // 2.4 审批工作流系统控制
  approval: {
    // 工作流管理
    createWorkflow: (workflow: { name: string; description: string; rules: ApprovalRule[]; approvers: string[] }) =>
      apiClient.post<{ workflow_id: string; status: string }>('/api/approval/workflow/create', workflow),
    
    listWorkflows: () =>
      apiClient.get<Array<{ workflow_id: string; name: string; status: string; created_at: string; active_requests: number }>>('/api/approval/workflow/list'),
    
    updateWorkflowConfig: (workflowId: string, config: Partial<ApprovalWorkflowConfig>) =>
      apiClient.put(`/api/approval/workflow/${workflowId}/config`, config),
    
    deleteWorkflow: (workflowId: string) =>
      apiClient.delete(`/api/approval/workflow/${workflowId}`),
    
    // 审批流程控制
    getPendingApprovals: (userId?: string) =>
      apiClient.get<Array<{
        request_id: string;
        type: string;
        description: string;
        requested_by: string;
        created_at: string;
        urgency: string;
        estimated_impact: string;
      }>>('/api/approval/pending', { params: { user_id: userId } }),
    
    approveRequest: (requestId: string, comment?: string, signature?: string) =>
      apiClient.post(`/api/approval/${requestId}/approve`, { comment, signature }),
    
    rejectRequest: (requestId: string, reason: string, comment?: string) =>
      apiClient.post(`/api/approval/${requestId}/reject`, { reason, comment }),
    
    getApprovalStatus: (requestId: string) =>
      apiClient.get<{
        request_id: string;
        status: string;
        current_step: string;
        approvals_received: number;
        approvals_required: number;
        next_approvers: string[];
        deadline: string;
      }>(`/api/approval/${requestId}/status`),
    
    // 审批历史
    getApprovalHistory: (requestId: string) =>
      apiClient.get<Array<{
        step: string;
        approver: string;
        action: string;
        timestamp: string;
        comment?: string;
        processing_time_minutes: number;
      }>>(`/api/approval/${requestId}/history`),
    
    getApprovalReports: (timeRange: { start: string; end: string }) =>
      apiClient.get<{
        total_requests: number;
        approved: number;
        rejected: number;
        pending: number;
        avg_processing_time_hours: number;
        by_type: Record<string, { count: number; avg_time_hours: number; approval_rate: number }>;
      }>('/api/approval/reports/summary', { params: timeRange }),
    
    getApprovalAnalytics: () =>
      apiClient.get<{
        bottlenecks: Array<{ step: string; avg_delay_hours: number; affected_requests: number }>;
        approver_performance: Array<{ approver: string; avg_response_time_hours: number; approval_rate: number }>;
        trends: Array<{ date: string; requests: number; approval_rate: number }>;
      }>('/api/approval/analytics/performance'),
    
    // 权限管理
    listRoles: () =>
      apiClient.get<Array<{ role: string; permissions: string[]; user_count: number; description: string }>>('/api/approval/roles/list'),
    
    assignRole: (userId: string, role: string) =>
      apiClient.post(`/api/approval/roles/${userId}/assign`, { role }),
    
    getUserPermissions: (userId: string) =>
      apiClient.get<{ user_id: string; roles: string[]; permissions: string[]; approval_limits: Record<string, number> }>(`/api/approval/permissions/${userId}`),
  },

  // 2.5 策略编排器控制
  strategies: {
    // 策略管理
    list: () =>
      apiClient.get<StrategyConfig[]>('/api/celue/strategies/list'),
    
    deploy: (strategy: Omit<StrategyConfig, 'id' | 'created_at' | 'updated_at'>) =>
      apiClient.post<StrategyConfig>('/api/celue/strategies/deploy', strategy),
    
    updateConfig: (strategyId: string, config: Partial<StrategyConfig>) =>
      apiClient.put<StrategyConfig>(`/api/celue/strategies/${strategyId}/config`, config),
    
    delete: (strategyId: string) =>
      apiClient.delete(`/api/celue/strategies/${strategyId}`),
    
    // 执行控制
    activate: (strategyId: string) =>
      apiClient.post<{ status: string; activated_at: string }>(`/api/celue/strategies/${strategyId}/activate`),
    
    deactivate: (strategyId: string) =>
      apiClient.post<{ status: string; deactivated_at: string }>(`/api/celue/strategies/${strategyId}/deactivate`),
    
    pause: (strategyId: string) =>
      apiClient.post<{ status: string; paused_at: string }>(`/api/celue/strategies/${strategyId}/pause`),
    
    resume: (strategyId: string) =>
      apiClient.post<{ status: string; resumed_at: string }>(`/api/celue/strategies/${strategyId}/resume`),
    
    // 参数调优
    getParameters: (strategyId: string) =>
      apiClient.get<Record<string, any>>(`/api/celue/strategies/${strategyId}/parameters`),
    
    updateParameters: (strategyId: string, parameters: Record<string, any>) =>
      apiClient.put(`/api/celue/strategies/${strategyId}/parameters`, parameters),
    
    runBacktest: (strategyId: string, config: { start_date: string; end_date: string; initial_capital: number }) =>
      apiClient.post<{ backtest_id: string; estimated_completion_time: string }>(`/api/celue/strategies/${strategyId}/backtest`, config),
    
    getPerformance: (strategyId: string, timeRange?: { start: string; end: string }) =>
      apiClient.get<StrategyPerformance>(`/api/celue/strategies/${strategyId}/performance`, { params: timeRange }),
  },

  // 2.6 风险管理控制
  risk: {
    // 风控规则
    getRules: () =>
      apiClient.get<Array<{ rule_id: string; name: string; type: string; threshold: number; action: string; enabled: boolean }>>('/api/celue/risk/rules'),
    
    createRule: (rule: { name: string; type: string; condition: string; threshold: number; action: string }) =>
      apiClient.post<{ rule_id: string; status: string }>('/api/celue/risk/rules/create', rule),
    
    updateRule: (ruleId: string, updates: Record<string, any>) =>
      apiClient.put(`/api/celue/risk/rules/${ruleId}`, updates),
    
    deleteRule: (ruleId: string) =>
      apiClient.delete(`/api/celue/risk/rules/${ruleId}`),
    
    // 实时风控
    getCurrentPositions: () =>
      apiClient.get<Array<{ exchange: string; symbol: string; position: number; market_value_usd: number; pnl_usd: number; risk_score: number }>>('/api/celue/risk/positions'),
    
    updateRiskLimits: (limits: RiskLimits) =>
      apiClient.post('/api/celue/risk/limits/update', limits),
    
    getCurrentExposure: () =>
      apiClient.get<{ total_exposure_usd: number; by_exchange: Record<string, number>; by_asset: Record<string, number>; concentration_risk: number }>('/api/celue/risk/exposure'),
    
    executeHedge: (config: { positions: Array<{ symbol: string; quantity: number }>; hedge_ratio: number }) =>
      apiClient.post<{ hedge_trades: Array<{ symbol: string; side: string; quantity: number; expected_price: number }> }>('/api/celue/risk/hedge', config),
    
    // 紧急控制
    emergencyStop: (reason: string) =>
      apiClient.post('/api/celue/risk/emergency-stop', { reason }),
    
    closeAllPositions: (confirm: boolean = false) =>
      apiClient.post('/api/celue/risk/close-all-positions', { confirm }),
    
    getCircuitBreakerStatus: () =>
      apiClient.get<{ active: boolean; triggered_by: string; triggered_at: string; auto_reset_time: string; manual_reset_required: boolean }>('/api/celue/risk/circuit-breaker/status'),
    
    resetCircuitBreaker: (adminPassword: string) =>
      apiClient.post('/api/celue/risk/circuit-breaker/reset', { admin_password: adminPassword }),
  },

  // 2.7 订单执行管理
  orders: {
    // 订单管理
    create: (order: { exchange: string; symbol: string; side: string; type: string; quantity: number; price?: number }) =>
      apiClient.post<OrderInfo>('/api/celue/orders/create', order),
    
    list: (filters?: { exchange?: string; status?: string; symbol?: string }) =>
      apiClient.get<OrderInfo[]>('/api/celue/orders/list', { params: filters }),
    
    modify: (orderId: string, modifications: { quantity?: number; price?: number }) =>
      apiClient.put<OrderInfo>(`/api/celue/orders/${orderId}/modify`, modifications),
    
    cancel: (orderId: string) =>
      apiClient.delete(`/api/celue/orders/${orderId}/cancel`),
    
    // 执行监控
    getOrderStatus: (orderId: string) =>
      apiClient.get<OrderInfo>(`/api/celue/orders/${orderId}/status`),
    
    getOrderFills: (orderId: string) =>
      apiClient.get<Array<{ fill_id: string; quantity: number; price: number; fees: number; timestamp: string; counterparty?: string }>>(`/api/celue/orders/${orderId}/fills`),
    
    getExecutionReport: (timeRange?: { start: string; end: string }) =>
      apiClient.get<{
        total_orders: number;
        filled_orders: number;
        partially_filled_orders: number;
        cancelled_orders: number;
        avg_fill_time_ms: number;
        total_volume_usd: number;
        total_fees_usd: number;
      }>('/api/celue/orders/execution-report', { params: timeRange }),
    
    // 智能路由
    getRoutingConfig: () =>
      apiClient.get<{ default_exchange: string; routing_rules: Array<{ condition: string; target_exchange: string; priority: number }> }>('/api/celue/routing/config'),
    
    updateRoutingRules: (rules: Array<{ condition: string; target_exchange: string; priority: number }>) =>
      apiClient.put('/api/celue/routing/rules', { rules }),
    
    optimizeRouting: (objective: 'latency' | 'fees' | 'liquidity') =>
      apiClient.post<{ optimization_applied: string[]; estimated_improvement: Record<string, number> }>('/api/celue/routing/optimize', { objective }),
  },
};