/**
 * 风险管理模块页面 - 风险指标监控和预警系统
 */

import { useState, useEffect, useCallback } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import apiService from '../../services/api.js';
import { 
  RISK_LEVELS, 
  REFRESH_INTERVALS, 
  SUCCESS_MESSAGES, 
  ERROR_MESSAGES 
} from '../../utils/constants.js';
import { 
  formatTime, 
  formatCurrency,
  formatPercentage,
  getStatusColor,
  cn
} from '../../utils/helpers.js';
import Navigation from '../../components/Navigation.jsx';

// 风险指标卡片组件
const RiskMetricCard = ({ title, value, threshold, status, trend, icon, unit = '%' }) => {
  const getStatusInfo = (status) => {
    switch (status) {
      case 'safe':
        return { color: 'text-green-600', bgColor: 'bg-green-50', borderColor: 'border-green-200' };
      case 'warning':
        return { color: 'text-yellow-600', bgColor: 'bg-yellow-50', borderColor: 'border-yellow-200' };
      case 'danger':
        return { color: 'text-red-600', bgColor: 'bg-red-50', borderColor: 'border-red-200' };
      default:
        return { color: 'text-gray-600', bgColor: 'bg-gray-50', borderColor: 'border-gray-200' };
    }
  };

  const statusInfo = getStatusInfo(status);
  
  return (
    <div className={`bg-white rounded-lg shadow-sm border-2 ${statusInfo.borderColor} p-6`}>
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900">{title}</h3>
        <div className={`p-2 rounded-full ${statusInfo.bgColor}`}>
          <span className={`text-xl ${statusInfo.color}`}>{icon}</span>
        </div>
      </div>
      
      <div className="mb-4">
        <div className="flex items-baseline gap-2">
          <span className={`text-3xl font-bold ${statusInfo.color}`}>
            {typeof value === 'number' ? value.toFixed(2) : value}{unit}
          </span>
          {trend && (
            <span className={`text-sm font-medium ${
              trend > 0 ? 'text-red-500' : trend < 0 ? 'text-green-500' : 'text-gray-500'
            }`}>
              {trend > 0 ? '↗' : trend < 0 ? '↘' : '→'} {Math.abs(trend).toFixed(1)}%
            </span>
          )}
        </div>
        <p className="text-sm text-gray-500 mt-1">
          阈值: {threshold}{unit}
        </p>
      </div>
      
      <div className="w-full bg-gray-200 rounded-full h-2">
        <div
          className={`h-2 rounded-full transition-all duration-300 ${
            status === 'safe' ? 'bg-green-500' : 
            status === 'warning' ? 'bg-yellow-500' : 'bg-red-500'
          }`}
          style={{ width: `${Math.min(100, (value / threshold) * 100)}%` }}
        />
      </div>
      
      <div className="mt-3">
        <span className={`px-2 py-1 rounded-full text-xs font-medium ${statusInfo.bgColor} ${statusInfo.color}`}>
          {status === 'safe' ? '安全' : status === 'warning' ? '预警' : '危险'}
        </span>
      </div>
    </div>
  );
};

// 预警列表卡片
const AlertListCard = () => {
  // 使用真实API获取预警数据
  const { data: alerts = [], isLoading: alertsLoading } = useQuery({
    queryKey: ['riskAlerts'],
    queryFn: () => apiService.risk.getAlerts(),
    refetchInterval: REFRESH_INTERVALS.FAST,
    select: (response) => response.data || [],
  });

  if (alertsLoading) {
    return (
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <div className="animate-pulse">
          <div className="h-4 bg-gray-200 rounded w-1/2 mb-4"></div>
          <div className="space-y-3">
            {[1, 2, 3].map((i) => (
              <div key={i} className="flex justify-between">
                <div className="h-3 bg-gray-200 rounded w-1/3"></div>
                <div className="h-3 bg-gray-200 rounded w-1/4"></div>
              </div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  const getSeverityColor = (severity) => {
    switch (severity) {
      case 'high': return 'text-red-600 bg-red-50 border-red-200';
      case 'medium': return 'text-yellow-600 bg-yellow-50 border-yellow-200';
      case 'low': return 'text-blue-600 bg-blue-50 border-blue-200';
      default: return 'text-gray-600 bg-gray-50 border-gray-200';
    }
  };

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900">实时预警</h3>
        <span className="text-sm text-gray-500">{alerts.length} 条预警</span>
      </div>
      
      {alerts.length === 0 ? (
        <div className="text-center py-8">
          <div className="text-green-500 text-4xl mb-2">✓</div>
          <p className="text-gray-500">暂无预警信息</p>
        </div>
      ) : (
        <div className="space-y-3 max-h-96 overflow-y-auto">
          {alerts.map((alert) => (
            <div key={alert.id} className={`p-4 rounded-lg border ${getSeverityColor(alert.severity)}`}>
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <h4 className="font-medium text-gray-900 mb-1">{alert.title}</h4>
                  <p className="text-sm text-gray-600 mb-2">{alert.message}</p>
                  <div className="flex items-center gap-4 text-xs text-gray-500">
                    <span>触发时间: {formatTime(new Date(alert.timestamp))}</span>
                    <span>指标: {alert.metric}</span>
                    <span>当前值: {alert.currentValue}</span>
                  </div>
                </div>
                <span className={`px-2 py-1 rounded-full text-xs font-medium ${getSeverityColor(alert.severity)}`}>
                  {alert.severity === 'high' ? '高危' : alert.severity === 'medium' ? '中危' : '低危'}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

// 资金安全监控卡片
const FundSafetyCard = () => {
  // 使用真实API获取资金安全数据
  const { data: fundData, isLoading: fundLoading } = useQuery({
    queryKey: ['fundSafety'],
    queryFn: () => apiService.risk.getFundSafety(),
    refetchInterval: REFRESH_INTERVALS.NORMAL,
    select: (response) => response.data,
  });

  if (fundLoading || !fundData) {
    return (
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <div className="animate-pulse">
          <div className="h-4 bg-gray-200 rounded w-1/2 mb-4"></div>
          <div className="space-y-4">
            <div className="h-16 bg-gray-200 rounded"></div>
            <div className="h-12 bg-gray-200 rounded"></div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">资金安全监控</h3>
      
      {/* 总资金状态 */}
      <div className="mb-6 p-4 bg-gradient-to-r from-blue-50 to-indigo-50 rounded-lg">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm text-gray-600">总资金</p>
            <p className="text-3xl font-bold text-blue-600">
              {formatCurrency(fundData.totalFunds)}
            </p>
            <p className={`text-sm font-medium ${fundData.fundChange >= 0 ? 'text-green-600' : 'text-red-600'}`}>
              24小时变化: {fundData.fundChange >= 0 ? '+' : ''}{formatCurrency(fundData.fundChange)}
            </p>
          </div>
          <div className="text-right">
            <p className="text-sm text-gray-600">安全评级</p>
            <p className={`text-2xl font-bold ${
              fundData.safetyRating === 'excellent' ? 'text-green-600' :
              fundData.safetyRating === 'good' ? 'text-blue-600' :
              fundData.safetyRating === 'warning' ? 'text-yellow-600' : 'text-red-600'
            }`}>
              {fundData.safetyRating === 'excellent' ? 'A+' :
               fundData.safetyRating === 'good' ? 'A' :
               fundData.safetyRating === 'warning' ? 'B' : 'C'}
            </p>
          </div>
        </div>
      </div>

      {/* 各交易所资金分布 */}
      <div className="space-y-4">
        <h4 className="text-md font-medium text-gray-900">交易所资金分布</h4>
        {fundData.exchangeFunds.map((exchange) => (
          <div key={exchange.name} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
            <div className="flex items-center gap-3">
              <span className="font-medium text-gray-700">{exchange.name}</span>
              <span className={`px-2 py-1 rounded text-xs ${
                exchange.status === 'normal' ? 'bg-green-100 text-green-700' :
                exchange.status === 'warning' ? 'bg-yellow-100 text-yellow-700' :
                'bg-red-100 text-red-700'
              }`}>
                {exchange.status === 'normal' ? '正常' :
                 exchange.status === 'warning' ? '警告' : '异常'}
              </span>
            </div>
            <div className="text-right">
              <p className="font-medium text-gray-900">{formatCurrency(exchange.amount)}</p>
              <p className="text-sm text-gray-500">{exchange.percentage}%</p>
            </div>
          </div>
        ))}
      </div>
      
      <div className="mt-4 pt-4 border-t border-gray-200">
        <p className="text-xs text-gray-500">
          最后更新: {formatTime(new Date(fundData.lastUpdated))}
        </p>
      </div>
    </div>
  );
};

// 风险配置卡片
const RiskConfigCard = () => {
  const [isEditing, setIsEditing] = useState(false);
  
  // 使用真实API获取风险配置
  const { data: config, isLoading: configLoading } = useQuery({
    queryKey: ['riskConfig'],
    queryFn: () => apiService.risk.getConfig(),
    select: (response) => response.data,
  });

  if (configLoading || !config) {
    return (
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <div className="animate-pulse">
          <div className="h-4 bg-gray-200 rounded w-1/2 mb-4"></div>
          <div className="space-y-3">
            {[1, 2, 3, 4].map((i) => (
              <div key={i} className="h-3 bg-gray-200 rounded"></div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900">风险配置</h3>
        <button
          onClick={() => setIsEditing(!isEditing)}
          className="px-3 py-1 text-sm font-medium text-blue-600 hover:text-blue-700 border border-blue-300 rounded-lg hover:bg-blue-50"
        >
          {isEditing ? '保存' : '编辑'}
        </button>
      </div>
      
      <div className="space-y-4">
        {config.settings.map((setting) => (
          <div key={setting.key} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
            <div>
              <p className="font-medium text-gray-900">{setting.label}</p>
              <p className="text-sm text-gray-500">{setting.description}</p>
            </div>
            <div className="text-right">
              {isEditing ? (
                <input
                  type="number"
                  defaultValue={setting.value}
                  className="w-20 px-2 py-1 text-sm border border-gray-300 rounded text-right"
                  step="0.01"
                />
              ) : (
                <span className="font-medium text-gray-900">
                  {setting.value}{setting.unit}
                </span>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

// 主风险管理模块页面组件
const RiskModule = () => {
  const queryClient = useQueryClient();
  const [notification, setNotification] = useState(null);
  
  // 获取风险指标数据
  const { data: riskMetrics, isLoading, error } = useQuery({
    queryKey: ['riskMetrics'],
    queryFn: () => apiService.risk.getMetrics(),
    refetchInterval: REFRESH_INTERVALS.NORMAL,
    select: (response) => response.data,
    retry: 3,
    retryDelay: 1000,
  });
  
  // 显示通知
  const showNotification = useCallback((message, type = 'info') => {
    setNotification({ message, type });
    setTimeout(() => setNotification(null), 4000);
  }, []);
  
  if (error) {
    return (
      <div className="min-h-screen bg-gray-50 p-6">
        <div className="max-w-4xl mx-auto">
          <div className="bg-red-50 border border-red-200 rounded-lg p-6 text-center">
            <p className="text-red-600 text-lg font-medium mb-2">
              风险管理模块数据加载失败
            </p>
            <p className="text-red-500 text-sm">
              {error.message}
            </p>
            <button
              onClick={() => queryClient.invalidateQueries(['riskMetrics'])}
              className="mt-4 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700"
            >
              重试
            </button>
          </div>
        </div>
      </div>
    );
  }
  
  return (
    <div className="min-h-screen bg-gray-50">
      <Navigation />
      <div className="p-6">
      {/* 通知组件 */}
      {notification && (
        <div className={cn(
          'fixed top-4 right-4 z-50 p-4 rounded-lg shadow-lg max-w-sm',
          notification.type === 'success' && 'bg-green-50 border border-green-200 text-green-800',
          notification.type === 'error' && 'bg-red-50 border border-red-200 text-red-800',
          notification.type === 'info' && 'bg-blue-50 border border-blue-200 text-blue-800'
        )}>
          {notification.message}
        </div>
      )}
      
      <div className="max-w-7xl mx-auto">
        {/* 页面标题 */}
        <div className="mb-6">
          <h1 className="text-3xl font-bold text-gray-900">风险管理</h1>
          <p className="text-gray-600 mt-1">
            实时风险监控、预警管理和资金安全保障
          </p>
        </div>
        
        {isLoading ? (
          <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
            {/* 加载占位符 */}
            {[1, 2, 3, 4, 5, 6].map((i) => (
              <div key={i} className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
                <div className="animate-pulse">
                  <div className="h-4 bg-gray-200 rounded w-3/4 mb-4"></div>
                  <div className="space-y-3">
                    <div className="h-8 bg-gray-200 rounded"></div>
                    <div className="h-3 bg-gray-200 rounded w-5/6"></div>
                    <div className="h-2 bg-gray-200 rounded"></div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="space-y-6">
            {/* 核心风险指标 */}
            {riskMetrics && (
              <div>
                <h2 className="text-xl font-semibold text-gray-900 mb-4">核心风险指标</h2>
                <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-4 gap-6">
                  <RiskMetricCard
                    title="最大回撤率"
                    value={riskMetrics.maxDrawdown}
                    threshold={riskMetrics.maxDrawdownThreshold}
                    status={riskMetrics.drawdownStatus}
                    trend={riskMetrics.drawdownTrend}
                    icon="📉"
                    unit="%"
                  />
                  <RiskMetricCard
                    title="资金使用率"
                    value={riskMetrics.capitalUtilization}
                    threshold={riskMetrics.capitalThreshold}
                    status={riskMetrics.capitalStatus}
                    trend={riskMetrics.capitalTrend}
                    icon="💰"
                    unit="%"
                  />
                  <RiskMetricCard
                    title="波动风险"
                    value={riskMetrics.volatilityRisk}
                    threshold={riskMetrics.volatilityThreshold}
                    status={riskMetrics.volatilityStatus}
                    trend={riskMetrics.volatilityTrend}
                    icon="📊"
                    unit="%"
                  />
                  <RiskMetricCard
                    title="流动性风险"
                    value={riskMetrics.liquidityRisk}
                    threshold={riskMetrics.liquidityThreshold}
                    status={riskMetrics.liquidityStatus}
                    trend={riskMetrics.liquidityTrend}
                    icon="🌊"
                    unit="%"
                  />
                </div>
              </div>
            )}
            
            {/* 预警和资金监控 */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <AlertListCard />
              <FundSafetyCard />
            </div>
            
            {/* 风险配置 */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <RiskConfigCard />
              <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">风险统计</h3>
                <div className="grid grid-cols-2 gap-4">
                  <div className="text-center p-4 bg-red-50 rounded-lg">
                    <p className="text-2xl font-bold text-red-600">
                      {riskMetrics ? riskMetrics.totalAlerts : 0}
                    </p>
                    <p className="text-sm text-gray-500">总预警数</p>
                  </div>
                  <div className="text-center p-4 bg-green-50 rounded-lg">
                    <p className="text-2xl font-bold text-green-600">
                      {riskMetrics ? riskMetrics.resolvedAlerts : 0}
                    </p>
                    <p className="text-sm text-gray-500">已处理</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}
        
        {/* 页面底部信息 */}
        <div className="mt-8 text-center text-sm text-gray-500">
          最后更新: {formatTime(new Date())} | 
          刷新间隔: {REFRESH_INTERVALS.NORMAL / 1000}秒 | 
          风险监控状态: 正常
        </div>
      </div>
      </div>
    </div>
  );
};

export default RiskModule;