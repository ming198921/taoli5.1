/**
 * 策略模块页面 - 套利策略配置和监控
 */

import { useState, useEffect, useCallback } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import apiService from '../../services/api.js';
import { 
  STRATEGY_STATUS, 
  REFRESH_INTERVALS, 
  SUCCESS_MESSAGES, 
  ERROR_MESSAGES 
} from '../../utils/constants.js';
import { 
  formatTime, 
  formatCurrency,
  getStatusColor,
  cn
} from '../../utils/helpers.js';
import Navigation from '../../components/Navigation.jsx';

// 策略卡片组件
const StrategyCard = ({ strategy, onStart, onStop, onEdit, loading }) => {
  const { id, name, status, profit, totalTrades, successRate, riskLevel } = strategy;
  const statusColor = getStatusColor(status);
  const isRunning = status === 'running';
  
  const getRiskColor = (risk) => {
    switch (risk) {
      case 'low': return 'text-green-600 bg-green-50';
      case 'medium': return 'text-yellow-600 bg-yellow-50';
      case 'high': return 'text-red-600 bg-red-50';
      default: return 'text-gray-600 bg-gray-50';
    }
  };
  
  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <div 
            className="w-4 h-4 rounded-full"
            style={{ backgroundColor: statusColor }}
          />
          <h3 className="text-lg font-semibold text-gray-900">{name}</h3>
        </div>
        <div className="flex items-center gap-2">
          <span 
            className="px-3 py-1 rounded-full text-sm font-medium"
            style={{ 
              color: statusColor,
              backgroundColor: `${statusColor}20`
            }}
          >
            {isRunning ? '运行中' : '已停止'}
          </span>
          <span className={`px-2 py-1 rounded-full text-xs font-medium ${getRiskColor(riskLevel)}`}>
            {riskLevel === 'low' ? '低风险' : riskLevel === 'medium' ? '中风险' : '高风险'}
          </span>
        </div>
      </div>
      
      {/* 策略指标 */}
      <div className="grid grid-cols-3 gap-4 mb-4">
        <div className="text-center">
          <p className="text-2xl font-bold text-green-600">
            {formatCurrency(profit)}
          </p>
          <p className="text-sm text-gray-500">总收益</p>
        </div>
        <div className="text-center">
          <p className="text-2xl font-bold text-blue-600">
            {totalTrades}
          </p>
          <p className="text-sm text-gray-500">交易次数</p>
        </div>
        <div className="text-center">
          <p className="text-2xl font-bold text-purple-600">
            {successRate}%
          </p>
          <p className="text-sm text-gray-500">成功率</p>
        </div>
      </div>
      
      <div className="mb-4">
        <p className="text-sm text-gray-500 mb-1">策略ID</p>
        <p className="text-gray-900 font-mono text-sm">{id}</p>
      </div>
      
      <div className="flex gap-3">
        <button
          onClick={() => onStart(id)}
          disabled={loading || isRunning}
          className={cn(
            'flex-1 py-2 px-4 rounded-lg font-medium transition-all duration-200',
            'disabled:opacity-50 disabled:cursor-not-allowed',
            !isRunning && !loading
              ? 'bg-green-600 hover:bg-green-700 text-white shadow-sm'
              : 'bg-gray-100 text-gray-400'
          )}
        >
          {loading ? '启动中...' : '启动策略'}
        </button>
        <button
          onClick={() => onStop(id)}
          disabled={loading || !isRunning}
          className={cn(
            'flex-1 py-2 px-4 rounded-lg font-medium transition-all duration-200',
            'disabled:opacity-50 disabled:cursor-not-allowed',
            isRunning && !loading
              ? 'bg-red-600 hover:bg-red-700 text-white shadow-sm'
              : 'bg-gray-100 text-gray-400'
          )}
        >
          {loading ? '停止中...' : '停止策略'}
        </button>
        <button
          onClick={() => onEdit(id)}
          className="px-4 py-2 rounded-lg font-medium border border-gray-300 text-gray-700 hover:bg-gray-50 transition-all duration-200"
        >
          配置
        </button>
      </div>
    </div>
  );
};

// 策略性能监控卡片
const PerformanceCard = () => {
  // 使用真实API获取策略性能数据
  const { data: performanceData, isLoading: performanceLoading } = useQuery({
    queryKey: ['strategyPerformance'],
    queryFn: () => apiService.celue.getPerformance(),
    refetchInterval: REFRESH_INTERVALS.NORMAL,
    select: (response) => response.data,
  });

  if (performanceLoading || !performanceData) {
    return (
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <div className="animate-pulse">
          <div className="h-4 bg-gray-200 rounded w-1/2 mb-4"></div>
          <div className="space-y-3">
            <div className="h-16 bg-gray-200 rounded"></div>
            <div className="h-3 bg-gray-200 rounded w-3/4"></div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">策略性能总览</h3>
      
      {/* 总体收益 */}
      <div className="mb-6 p-4 bg-gradient-to-r from-green-50 to-emerald-50 rounded-lg">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm text-gray-600">总收益</p>
            <p className="text-3xl font-bold text-green-600">
              {formatCurrency(performanceData.totalProfit)}
            </p>
            <p className={`text-sm font-medium ${performanceData.dailyChange >= 0 ? 'text-green-600' : 'text-red-600'}`}>
              今日 {performanceData.dailyChange >= 0 ? '+' : ''}{formatCurrency(performanceData.dailyChange)}
            </p>
          </div>
          <div className="text-right">
            <p className="text-sm text-gray-600">收益率</p>
            <p className="text-2xl font-bold text-blue-600">
              {performanceData.profitRate}%
            </p>
          </div>
        </div>
      </div>

      {/* 关键指标 */}
      <div className="grid grid-cols-2 gap-4 mb-4">
        <div className="text-center p-3 bg-blue-50 rounded-lg">
          <p className="text-2xl font-semibold text-blue-600">
            {performanceData.activeStrategies}
          </p>
          <p className="text-sm text-gray-500">活跃策略</p>
        </div>
        <div className="text-center p-3 bg-purple-50 rounded-lg">
          <p className="text-2xl font-semibold text-purple-600">
            {performanceData.totalTrades}
          </p>
          <p className="text-sm text-gray-500">今日交易</p>
        </div>
        <div className="text-center p-3 bg-yellow-50 rounded-lg">
          <p className="text-2xl font-semibold text-yellow-600">
            {performanceData.avgSuccessRate}%
          </p>
          <p className="text-sm text-gray-500">平均成功率</p>
        </div>
        <div className="text-center p-3 bg-red-50 rounded-lg">
          <p className="text-2xl font-semibold text-red-600">
            {performanceData.maxDrawdown}%
          </p>
          <p className="text-sm text-gray-500">最大回撤</p>
        </div>
      </div>
      
      <div className="mt-4 pt-4 border-t border-gray-200">
        <p className="text-xs text-gray-500">
          最后更新: {formatTime(new Date(performanceData.lastUpdated))}
        </p>
      </div>
    </div>
  );
};

// 交易所对比卡片
const ExchangeComparisonCard = () => {
  // 使用真实API获取交易所对比数据
  const { data: exchangeData, isLoading: exchangeLoading } = useQuery({
    queryKey: ['exchangeComparison'],
    queryFn: () => apiService.celue.getExchangeComparison(),
    refetchInterval: REFRESH_INTERVALS.FAST,
    select: (response) => response.data,
  });

  if (exchangeLoading || !exchangeData) {
    return (
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <div className="animate-pulse">
          <div className="h-4 bg-gray-200 rounded w-1/2 mb-4"></div>
          <div className="space-y-3">
            {[1, 2, 3].map((i) => (
              <div key={i} className="flex justify-between">
                <div className="h-3 bg-gray-200 rounded w-1/4"></div>
                <div className="h-3 bg-gray-200 rounded w-1/4"></div>
              </div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">交易所套利对比</h3>
      
      <div className="space-y-4">
        {exchangeData.pairs.map((pair) => (
          <div key={pair.id} className="p-4 border border-gray-200 rounded-lg">
            <div className="flex items-center justify-between mb-2">
              <h4 className="font-medium text-gray-900">{pair.symbol}</h4>
              <span className={`px-2 py-1 rounded text-sm font-medium ${
                pair.spreadPercent > 0.5 ? 'text-green-600 bg-green-50' : 'text-gray-600 bg-gray-50'
              }`}>
                {pair.spreadPercent > 0 ? '+' : ''}{pair.spreadPercent}%
              </span>
            </div>
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <p className="text-gray-500">{pair.exchange1}</p>
                <p className="font-mono text-gray-900">{formatCurrency(pair.price1)}</p>
              </div>
              <div>
                <p className="text-gray-500">{pair.exchange2}</p>
                <p className="font-mono text-gray-900">{formatCurrency(pair.price2)}</p>
              </div>
            </div>
            <div className="mt-2 text-xs text-gray-500">
              价差: {formatCurrency(Math.abs(pair.price1 - pair.price2))} | 
              更新: {formatTime(new Date(pair.lastUpdate))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

// 主策略模块页面组件
const CelueModule = () => {
  const queryClient = useQueryClient();
  const [notification, setNotification] = useState(null);
  const [operatingStrategy, setOperatingStrategy] = useState(null);
  
  // 获取策略列表
  const { data: strategies = [], isLoading, error } = useQuery({
    queryKey: ['strategies'],
    queryFn: () => apiService.celue.getStrategies(),
    refetchInterval: REFRESH_INTERVALS.NORMAL,
    select: (response) => response.data || [],
    retry: 3,
    retryDelay: 1000,
  });
  
  // 显示通知
  const showNotification = useCallback((message, type = 'info') => {
    setNotification({ message, type });
    setTimeout(() => setNotification(null), 4000);
  }, []);
  
  // 启动策略
  const startStrategyMutation = useMutation({
    mutationFn: (strategyId) => apiService.celue.startStrategy(strategyId),
    onMutate: (strategyId) => {
      setOperatingStrategy(strategyId);
    },
    onSuccess: (response, strategyId) => {
      queryClient.invalidateQueries(['strategies']);
      showNotification(`策略 ${strategyId} 启动成功`, 'success');
    },
    onError: (error, strategyId) => {
      showNotification(`策略 ${strategyId} 启动失败: ${error.message}`, 'error');
    },
    onSettled: () => {
      setOperatingStrategy(null);
    },
  });
  
  // 停止策略
  const stopStrategyMutation = useMutation({
    mutationFn: (strategyId) => apiService.celue.stopStrategy(strategyId),
    onMutate: (strategyId) => {
      setOperatingStrategy(strategyId);
    },
    onSuccess: (response, strategyId) => {
      queryClient.invalidateQueries(['strategies']);
      showNotification(`策略 ${strategyId} 已停止`, 'success');
    },
    onError: (error, strategyId) => {
      showNotification(`策略 ${strategyId} 停止失败: ${error.message}`, 'error');
    },
    onSettled: () => {
      setOperatingStrategy(null);
    },
  });
  
  // 编辑策略配置
  const handleEditStrategy = useCallback((strategyId) => {
    showNotification(`策略 ${strategyId} 配置功能开发中`, 'info');
  }, [showNotification]);
  
  if (error) {
    return (
      <div className="min-h-screen bg-gray-50 p-6">
        <div className="max-w-4xl mx-auto">
          <div className="bg-red-50 border border-red-200 rounded-lg p-6 text-center">
            <p className="text-red-600 text-lg font-medium mb-2">
              策略模块数据加载失败
            </p>
            <p className="text-red-500 text-sm">
              {error.message}
            </p>
            <button
              onClick={() => queryClient.invalidateQueries(['strategies'])}
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
          <h1 className="text-3xl font-bold text-gray-900">策略模块</h1>
          <p className="text-gray-600 mt-1">
            套利策略管理、性能监控和交易所价差分析
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
                    <div className="h-3 bg-gray-200 rounded"></div>
                    <div className="h-3 bg-gray-200 rounded w-5/6"></div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="space-y-6">
            {/* 策略管理区域 */}
            <div>
              <h2 className="text-xl font-semibold text-gray-900 mb-4">套利策略管理</h2>
              <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
                {strategies.map((strategy) => (
                  <StrategyCard
                    key={strategy.id}
                    strategy={strategy}
                    onStart={() => startStrategyMutation.mutate(strategy.id)}
                    onStop={() => stopStrategyMutation.mutate(strategy.id)}
                    onEdit={handleEditStrategy}
                    loading={operatingStrategy === strategy.id}
                  />
                ))}
              </div>
            </div>
            
            {/* 性能监控和交易所对比区域 */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <PerformanceCard />
              <ExchangeComparisonCard />
            </div>
          </div>
        )}
        
        {/* 页面底部信息 */}
        <div className="mt-8 text-center text-sm text-gray-500">
          最后更新: {formatTime(new Date())} | 
          刷新间隔: {REFRESH_INTERVALS.NORMAL / 1000}秒 | 
          活跃策略数量: {strategies.filter(s => s.status === 'running').length}
        </div>
      </div>
      </div>
    </div>
  );
};

export default CelueModule;