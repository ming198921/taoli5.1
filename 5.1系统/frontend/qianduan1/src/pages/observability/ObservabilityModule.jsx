/**
 * 可观测性模块 - 5.1套利系统日志聚合、链路追踪和告警监控
 */

import { useState, useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import Navigation from '../../components/Navigation.jsx';
import apiService from '../../services/api.js';
import { 
  REFRESH_INTERVALS, 
  SUCCESS_MESSAGES, 
  ERROR_MESSAGES 
} from '../../utils/constants.js';
import { 
  formatTime, 
  formatNumber, 
  formatPercent, 
  formatLatency,
  getStatusColor,
  cn
} from '../../utils/helpers.js';

// 日志聚合卡片
const LogsAggregationCard = ({ logsData }) => {
  const logs = logsData || [];
  
  const logLevels = ['ERROR', 'WARN', 'INFO', 'DEBUG'];
  const logLevelColors = {
    ERROR: '#ff4d4f',
    WARN: '#faad14', 
    INFO: '#52c41a',
    DEBUG: '#1890ff'
  };

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900">日志聚合</h3>
        <div className="flex gap-2">
          <select className="text-sm border border-gray-300 rounded-lg px-3 py-1">
            <option value="all">所有模块</option>
            <option value="arbitrage">套利引擎</option>
            <option value="collector">数据收集器</option>
            <option value="risk">风控引擎</option>
          </select>
          <select className="text-sm border border-gray-300 rounded-lg px-3 py-1">
            <option value="1h">近1小时</option>
            <option value="6h">近6小时</option>
            <option value="1d">近1天</option>
          </select>
        </div>
      </div>
      
      {/* 日志级别统计 */}
      <div className="grid grid-cols-4 gap-3 mb-4">
        {logLevels.map(level => {
          const count = logs.filter(log => log.level === level).length;
          return (
            <div key={level} className="text-center p-3 bg-gray-50 rounded-lg">
              <p className="text-lg font-bold" style={{ color: logLevelColors[level] }}>
                {count}
              </p>
              <p className="text-xs text-gray-600">{level}</p>
            </div>
          );
        })}
      </div>
      
      {/* 日志列表 */}
      <div className="bg-gray-50 rounded-lg p-4 h-64 overflow-y-auto">
        <div className="space-y-2 font-mono text-xs">
          {logs.length > 0 ? (
            logs.map((log, index) => (
              <div key={index} className="flex gap-3 py-1">
                <span className="text-gray-500 w-20 flex-shrink-0">
                  {formatTime(log.timestamp, 'HH:mm:ss')}
                </span>
                <span 
                  className="w-12 flex-shrink-0 font-medium text-center"
                  style={{ color: logLevelColors[log.level] }}
                >
                  {log.level}
                </span>
                <span className="text-blue-600 w-16 flex-shrink-0 truncate">
                  {log.module}
                </span>
                <span className="text-gray-700 flex-1">
                  {log.message}
                </span>
              </div>
            ))
          ) : (
            <div className="text-gray-500 text-center py-8">
              暂无日志数据
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

// 链路追踪卡片
const TracingCard = ({ tracesData }) => {
  const traces = tracesData || [];

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">链路追踪</h3>
      
      <div className="space-y-3">
        {traces.map((trace) => {
          const statusColor = getStatusColor(
            trace.status === 'success' ? 'running' : 'error'
          );
          
          return (
            <div key={trace.traceId} className="border border-gray-200 rounded-lg p-4">
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center gap-3">
                  <div 
                    className="w-3 h-3 rounded-full"
                    style={{ backgroundColor: statusColor }}
                  />
                  <span className="font-medium text-gray-900">{trace.operation}</span>
                  <span className="text-xs text-gray-500 font-mono">
                    {trace.traceId.substring(0, 8)}...
                  </span>
                </div>
                <div className="text-right">
                  <p className="text-sm font-medium text-gray-900">
                    {formatLatency(trace.duration)}
                  </p>
                  <p className="text-xs text-gray-500">
                    {formatTime(trace.timestamp)}
                  </p>
                </div>
              </div>
              
              {/* 服务调用链 */}
              <div className="flex flex-wrap gap-2 mt-2">
                {trace.spans?.map((span, index) => (
                  <div key={index} className="flex items-center gap-1">
                    <span className="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded">
                      {span.serviceName}
                    </span>
                    <span className="text-xs text-gray-400">
                      {formatLatency(span.duration)}
                    </span>
                    {index < trace.spans.length - 1 && (
                      <span className="text-gray-400">→</span>
                    )}
                  </div>
                ))}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

// 告警规则卡片
const AlertRulesCard = ({ alertsData }) => {
  const alerts = alertsData || [];

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900">告警规则</h3>
        <button className="px-3 py-1 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700">
          新增规则
        </button>
      </div>
      
      <div className="space-y-3">
        {alerts.map((alert) => {
          const severityColor = {
            critical: '#ff4d4f',
            high: '#faad14',
            medium: '#52c41a',
            low: '#1890ff'
          }[alert.severity] || '#d9d9d9';
          
          return (
            <div key={alert.id} className="flex items-center justify-between p-3 border border-gray-200 rounded-lg">
              <div className="flex items-center gap-3">
                <div 
                  className="w-3 h-3 rounded-full"
                  style={{ backgroundColor: severityColor }}
                />
                <div>
                  <p className="font-medium text-gray-900">{alert.name}</p>
                  <p className="text-sm text-gray-500">{alert.condition}</p>
                </div>
              </div>
              
              <div className="flex items-center gap-3">
                <span 
                  className="text-xs px-2 py-1 rounded-full font-medium"
                  style={{ 
                    backgroundColor: severityColor + '20',
                    color: severityColor
                  }}
                >
                  {alert.severity.toUpperCase()}
                </span>
                <span className={cn(
                  'text-xs px-2 py-1 rounded-full',
                  alert.enabled 
                    ? 'bg-green-100 text-green-800' 
                    : 'bg-gray-100 text-gray-600'
                )}>
                  {alert.enabled ? '已启用' : '已禁用'}
                </span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

// 指标仪表盘卡片
const MetricsDashboardCard = ({ metricsData }) => {
  const metrics = metricsData || {};
  
  const metricItems = [
    {
      label: '请求总数',
      value: formatNumber(metrics.totalRequests || 0),
      change: '+12.5%',
      color: '#1890ff'
    },
    {
      label: '错误率',
      value: formatPercent(metrics.errorRate || 0),
      change: '-0.8%',
      color: '#ff4d4f'
    },
    {
      label: '平均响应时间',
      value: formatLatency(metrics.avgResponseTime || 0),
      change: '-15ms',
      color: '#52c41a'
    },
    {
      label: '吞吐量',
      value: formatNumber(metrics.throughput || 0) + '/min',
      change: '+8.3%',
      color: '#faad14'
    }
  ];

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">指标仪表盘</h3>
      
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        {metricItems.map((item) => (
          <div key={item.label} className="text-center p-4 bg-gray-50 rounded-lg">
            <p className="text-2xl font-bold" style={{ color: item.color }}>
              {item.value}
            </p>
            <p className="text-sm text-gray-600 mt-1">{item.label}</p>
            <p className="text-xs text-green-600 mt-1">{item.change}</p>
          </div>
        ))}
      </div>
      
      {/* 可视化图表占位符 */}
      <div className="mt-6 h-48 bg-gray-50 rounded-lg flex items-center justify-center">
        <div className="text-center text-gray-500">
          <div className="text-4xl mb-2">📈</div>
          <p>指标趋势图表组件 - 待集成</p>
        </div>
      </div>
    </div>
  );
};

// 主可观测性组件
const ObservabilityModule = () => {
  const queryClient = useQueryClient();
  const [notification, setNotification] = useState(null);
  const [selectedTimeRange, setSelectedTimeRange] = useState('1h');

  // 获取日志数据
  const { data: logsData, isLoading: logsLoading } = useQuery({
    queryKey: ['observabilityLogs'],
    queryFn: () => apiService.observability.getLogs({ lines: 50 }),
    refetchInterval: REFRESH_INTERVALS.FAST,
    select: (response) => response.data?.logs || []
  });

  // 获取链路追踪数据
  const { data: tracesData, isLoading: tracesLoading } = useQuery({
    queryKey: ['observabilityTraces'],
    queryFn: () => apiService.observability.getTraces({}),
    refetchInterval: REFRESH_INTERVALS.NORMAL,
    select: (response) => response.data?.traces || []
  });

  // 获取告警规则
  const { data: alertsData, isLoading: alertsLoading } = useQuery({
    queryKey: ['observabilityAlerts'],
    queryFn: () => apiService.observability.getAlerts(),
    refetchInterval: REFRESH_INTERVALS.NORMAL,
    select: (response) => response.data?.alerts || []
  });

  // 获取指标数据
  const { data: metricsData, isLoading: metricsLoading } = useQuery({
    queryKey: ['observabilityMetrics', selectedTimeRange],
    queryFn: () => apiService.observability.getMetrics('system', selectedTimeRange),
    refetchInterval: REFRESH_INTERVALS.FAST,
    select: (response) => response.data?.metrics || {}
  });

  // 显示通知
  const showNotification = (message, type = 'info') => {
    setNotification({ message, type });
    setTimeout(() => setNotification(null), 4000);
  };

  const isLoading = logsLoading || tracesLoading || alertsLoading || metricsLoading;

  return (
    <div className="min-h-screen bg-gray-50 p-6">
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

      {/* 导航栏 */}
      <Navigation />
      
      <div className="max-w-7xl mx-auto">
        {/* 页面标题 */}
        <div className="mb-6">
          <h1 className="text-3xl font-bold text-gray-900">可观测性</h1>
          <p className="text-gray-600 mt-1">
            5.1套利系统可观测性 - 日志聚合、链路追踪和告警监控
          </p>
        </div>
        
        {isLoading ? (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* 加载占位符 */}
            {[1, 2, 3, 4].map((i) => (
              <div key={i} className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
                <div className="animate-pulse">
                  <div className="h-4 bg-gray-200 rounded w-3/4 mb-4"></div>
                  <div className="space-y-3">
                    <div className="h-3 bg-gray-200 rounded"></div>
                    <div className="h-3 bg-gray-200 rounded w-5/6"></div>
                    <div className="h-3 bg-gray-200 rounded w-4/5"></div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="space-y-6">
            {/* 指标仪表盘 */}
            <MetricsDashboardCard metricsData={metricsData} />
            
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              {/* 日志聚合 */}
              <LogsAggregationCard logsData={logsData} />
              
              {/* 链路追踪 */}
              <TracingCard tracesData={tracesData} />
            </div>
            
            {/* 告警规则 */}
            <AlertRulesCard alertsData={alertsData} />
          </div>
        )}
        
        {/* 页面底部信息 */}
        <div className="mt-8 text-center text-sm text-gray-500">
          最后更新: {formatTime(new Date())} | 
          自动刷新间隔: {REFRESH_INTERVALS.FAST / 1000}秒
        </div>
      </div>
    </div>
  );
};

export default ObservabilityModule;