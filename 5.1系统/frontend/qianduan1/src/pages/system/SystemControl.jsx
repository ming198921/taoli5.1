/**
 * 系统控制页面 - 5.1套利系统核心控制面板
 */

import { useState, useEffect, useCallback } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import Navigation from '../../components/Navigation.jsx';
import apiService from '../../services/api.js';
import arbitrageSDK from '../../services/sdk.js';
import { 
  SYSTEM_STATUS, 
  REFRESH_INTERVALS, 
  SUCCESS_MESSAGES, 
  ERROR_MESSAGES 
} from '../../utils/constants.js';
import { 
  formatTime, 
  formatUptime, 
  formatPercent, 
  formatLatency,
  getStatusColor,
  cn
} from '../../utils/helpers.js';

// 系统控制卡片组件 - 增强版支持完整API网关
const SystemControlCard = ({ systemData, onStart, onStop, loading }) => {
  const { isRunning, lastStarted, uptime, version } = systemData || {};
  const [currentUptime, setCurrentUptime] = useState(0);
  const [realTimeMetrics, setRealTimeMetrics] = useState(null);
  const [sdkStatus, setSdkStatus] = useState(null);
  
  console.log('SystemControlCard - systemData:', systemData);
  console.log('SystemControlCard - isRunning:', isRunning, 'loading:', loading);
  
  // 实时指标订阅 - 使用完整API网关
  useEffect(() => {
    let metricsSubscription = null;
    
    if (arbitrageSDK.getWebSocketStatus().connected) {
      metricsSubscription = arbitrageSDK.subscribe('system_metrics', (metrics) => {
        setRealTimeMetrics(metrics);
      });
    }
    
    return () => {
      if (metricsSubscription) {
        metricsSubscription.unsubscribe();
      }
    };
  }, []);

  // 监控SDK状态
  useEffect(() => {
    const updateSdkStatus = () => {
      setSdkStatus(arbitrageSDK.getStatus());
    };

    updateSdkStatus();
    const interval = setInterval(updateSdkStatus, 2000);

    return () => clearInterval(interval);
  }, []);
  
  // 计算实时运行时间
  useEffect(() => {
    if (!isRunning || !lastStarted) {
      setCurrentUptime(0);
      return;
    }
    
    const startTime = new Date(lastStarted).getTime();
    const updateUptime = () => {
      const now = Date.now();
      const uptimeInSeconds = Math.floor((now - startTime) / 1000);
      setCurrentUptime(uptimeInSeconds);
    };
    
    updateUptime();
    const interval = setInterval(updateUptime, 1000);
    
    return () => clearInterval(interval);
  }, [isRunning, lastStarted]);
  
  const statusText = isRunning ? '运行中' : '已停止';
  const statusColor = getStatusColor(isRunning ? SYSTEM_STATUS.RUNNING : SYSTEM_STATUS.STOPPED);
  
  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <div 
            className="w-4 h-4 rounded-full animate-pulse"
            style={{ backgroundColor: statusColor }}
          />
          <h2 className="text-xl font-semibold text-gray-900">
            5.1套利系统控制面板 (完整API网关)
          </h2>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-sm text-gray-500">v{version || '5.1.0'}</span>
          {realTimeMetrics && (
            <span className="text-xs px-2 py-1 bg-green-100 text-green-800 rounded">
              实时数据
            </span>
          )}
          {sdkStatus && (
            <div className="flex items-center gap-1">
              <div className={cn(
                'w-2 h-2 rounded-full',
                sdkStatus.initialized ? 'bg-green-500' : 'bg-red-500'
              )} />
              <span className="text-xs text-gray-500">
                {sdkStatus.wsConnected ? 'WS连接' : 'HTTP连接'}
              </span>
            </div>
          )}
        </div>
      </div>
      
      <div className="grid grid-cols-2 gap-4 mb-6">
        <div>
          <p className="text-sm text-gray-500">系统状态</p>
          <p className="text-lg font-medium" style={{ color: statusColor }}>
            {statusText}
          </p>
          {realTimeMetrics && (
            <p className="text-xs text-gray-400">
              活跃连接: {realTimeMetrics.activeConnections || 0}
            </p>
          )}
        </div>
        <div>
          <p className="text-sm text-gray-500">运行时间</p>
          <p className="text-lg font-medium text-gray-900">
            {formatUptime(currentUptime)}
          </p>
          {realTimeMetrics && (
            <p className="text-xs text-gray-400">
              响应时间: {realTimeMetrics.responseTimeMs || 0}ms
            </p>
          )}
        </div>
        <div className="col-span-2">
          <p className="text-sm text-gray-500">最后启动时间</p>
          <p className="text-lg font-medium text-gray-900">
            {formatTime(lastStarted) || '-'}
          </p>
        </div>
      </div>
      
      <div className="flex gap-3">
        <button
          onClick={onStart}
          disabled={loading || isRunning}
          className={cn(
            'flex-1 py-2 px-4 rounded-lg font-medium transition-all duration-200',
            'disabled:opacity-50 disabled:cursor-not-allowed',
            !isRunning && !loading
              ? 'bg-green-600 hover:bg-green-700 text-white shadow-sm'
              : 'bg-gray-100 text-gray-400'
          )}
        >
          {loading ? '启动中...' : '启动系统'}
        </button>
        <button
          onClick={() => {
            console.log('停止按钮被点击, isRunning:', isRunning, 'loading:', loading);
            onStop();
          }}
          disabled={loading || !isRunning}
          className={cn(
            'flex-1 py-2 px-4 rounded-lg font-medium transition-all duration-200',
            'disabled:opacity-50 disabled:cursor-not-allowed',
            isRunning && !loading
              ? 'bg-red-600 hover:bg-red-700 text-white shadow-sm'
              : 'bg-gray-100 text-gray-400'
          )}
        >
          {loading ? '停止中...' : '停止系统'}
          {/* Debug info */}
          <span className="text-xs ml-1">
            {isRunning ? '✅' : '❌'}
          </span>
        </button>
      </div>
    </div>
  );
};

// 性能指标卡片组件
const PerformanceCard = ({ systemData }) => {
  const { cpu_usage = 0, memory_usage = 0, network_latency = 0 } = systemData || {};
  
  const metrics = [
    {
      label: 'CPU使用率',
      value: formatPercent(cpu_usage),
      progress: cpu_usage,
      color: cpu_usage > 80 ? '#ff4d4f' : cpu_usage > 60 ? '#faad14' : '#52c41a'
    },
    {
      label: '内存使用率',
      value: formatPercent(memory_usage),
      progress: memory_usage,
      color: memory_usage > 80 ? '#ff4d4f' : memory_usage > 60 ? '#faad14' : '#52c41a'
    },
    {
      label: '网络延迟',
      value: formatLatency(network_latency),
      progress: Math.min(network_latency / 100 * 100, 100),
      color: network_latency > 50 ? '#ff4d4f' : network_latency > 30 ? '#faad14' : '#52c41a'
    }
  ];
  
  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">性能指标</h3>
      <div className="space-y-4">
        {metrics.map((metric) => (
          <div key={metric.label}>
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-gray-700">
                {metric.label}
              </span>
              <span className="text-sm font-semibold" style={{ color: metric.color }}>
                {metric.value}
              </span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div
                className="h-2 rounded-full transition-all duration-300"
                style={{
                  width: `${metric.progress}%`,
                  backgroundColor: metric.color,
                }}
              />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

// 模块状态卡片组件
const ModulesStatusCard = ({ systemData }) => {
  const { modules = {} } = systemData || {};
  
  const modulesList = Object.entries(modules).map(([key, value]) => ({
    name: key,
    ...value
  }));
  
  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">模块状态</h3>
      <div className="space-y-3">
        {modulesList.map((module) => {
          const statusColor = getStatusColor(module.status === 'running' ? SYSTEM_STATUS.RUNNING : SYSTEM_STATUS.STOPPED);
          
          return (
            <div key={module.name} className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div 
                  className="w-3 h-3 rounded-full"
                  style={{ backgroundColor: statusColor }}
                />
                <span className="font-medium text-gray-900 capitalize">
                  {module.name}
                </span>
              </div>
              <div className="text-right">
                <span 
                  className="text-sm font-medium"
                  style={{ color: statusColor }}
                >
                  {module.status === 'running' ? '运行中' : '已停止'}
                </span>
                <p className="text-xs text-gray-500">
                  {module.health === 'healthy' ? '健康' : 
                   module.health === 'unknown' ? '未知' : '异常'}
                </p>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

// WebSocket连接控制组件
const WebSocketControlCard = () => {
  const [wsStatus, setWsStatus] = useState(null);
  const [connectionLogs, setConnectionLogs] = useState([]);
  
  useEffect(() => {
    const updateStatus = () => {
      const status = arbitrageSDK.getWebSocketStatus();
      setWsStatus(status);
    };
    
    updateStatus();
    const interval = setInterval(updateStatus, 1000);
    
    // 监听连接状态变化
    const handleStatusChange = (status) => {
      const timestamp = new Date().toLocaleTimeString();
      setConnectionLogs(prev => [
        `[${timestamp}] ${status.connected ? 'WebSocket已连接' : 'WebSocket已断开'}`,
        ...prev.slice(0, 9) // 保留最新10条
      ]);
    };
    
    // 如果SDK支持状态变化监听
    if (arbitrageSDK.onStatusChange) {
      arbitrageSDK.onStatusChange(handleStatusChange);
    }
    
    return () => {
      clearInterval(interval);
      if (arbitrageSDK.offStatusChange) {
        arbitrageSDK.offStatusChange(handleStatusChange);
      }
    };
  }, []);
  
  const handleConnect = async () => {
    try {
      console.log('尝试连接WebSocket...');
      // 尝试不需要token的连接
      if (arbitrageSDK.wsClient && arbitrageSDK.wsClient.connect) {
        await arbitrageSDK.wsClient.connect('guest_token');
      } else if (arbitrageSDK.connectWebSocket) {
        await arbitrageSDK.connectWebSocket('guest_token');
      } else {
        console.warn('SDK没有WebSocket连接方法');
      }
    } catch (error) {
      console.error('WebSocket连接失败:', error);
    }
  };
  
  const handleDisconnect = () => {
    try {
      console.log('断开WebSocket连接...');
      if (arbitrageSDK.wsClient && arbitrageSDK.wsClient.disconnect) {
        arbitrageSDK.wsClient.disconnect();
      } else if (arbitrageSDK.disconnectWebSocket) {
        arbitrageSDK.disconnectWebSocket();
      } else {
        console.warn('SDK没有WebSocket断开方法');
      }
    } catch (error) {
      console.error('WebSocket断开失败:', error);
    }
  };
  
  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900">WebSocket连接</h3>
        <div className="flex items-center gap-2">
          <div className={cn(
            'w-3 h-3 rounded-full',
            wsStatus?.connected ? 'bg-green-500 animate-pulse' : 'bg-red-500'
          )} />
          <span className="text-sm font-medium">
            {wsStatus?.connected ? '已连接' : '未连接'}
          </span>
        </div>
      </div>
      
      {wsStatus && (
        <div className="grid grid-cols-2 gap-4 mb-4">
          <div>
            <p className="text-sm text-gray-500">连接URL</p>
            <p className="text-xs font-mono text-gray-900 truncate">
              {wsStatus.url || 'ws://localhost:8080/ws'}
            </p>
          </div>
          <div>
            <p className="text-sm text-gray-500">重连次数</p>
            <p className="text-sm font-medium text-gray-900">
              {wsStatus.reconnectAttempts || 0}
            </p>
          </div>
        </div>
      )}
      
      <div className="flex gap-2 mb-4">
        <button
          onClick={handleConnect}
          disabled={wsStatus?.connected}
          className={cn(
            'px-3 py-1 text-sm font-medium rounded transition-colors',
            wsStatus?.connected 
              ? 'bg-gray-100 text-gray-400 cursor-not-allowed'
              : 'bg-green-600 hover:bg-green-700 text-white'
          )}
        >
          连接
        </button>
        <button
          onClick={handleDisconnect}
          disabled={!wsStatus?.connected}
          className={cn(
            'px-3 py-1 text-sm font-medium rounded transition-colors',
            !wsStatus?.connected
              ? 'bg-gray-100 text-gray-400 cursor-not-allowed'
              : 'bg-red-600 hover:bg-red-700 text-white'
          )}
        >
          断开
        </button>
      </div>
      
      <div className="bg-gray-50 rounded p-3 h-32 overflow-y-auto">
        <div className="space-y-1 font-mono text-xs">
          {connectionLogs.length > 0 ? (
            connectionLogs.map((log, index) => (
              <div key={index} className="text-gray-700">
                {log}
              </div>
            ))
          ) : (
            <div className="text-gray-500 text-center py-4">
              暂无连接日志
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

// 实时数据订阅组件
const RealTimeDataCard = () => {
  const [subscriptions, setSubscriptions] = useState(new Map());
  const [activeChannels, setActiveChannels] = useState([]);
  
  const availableChannels = [
    { id: 'market_data', name: '市场数据', description: '实时价格和交易量' },
    { id: 'arbitrage_opportunities', name: '套利机会', description: '实时套利信号' },
    { id: 'system_alerts', name: '系统告警', description: '系统异常和告警' },
    { id: 'trade_execution', name: '交易执行', description: '交易执行状态' }
  ];
  
  const handleSubscribe = (channelId) => {
    if (arbitrageSDK.subscribe && !subscriptions.has(channelId)) {
      const subscription = arbitrageSDK.subscribe(channelId, (data) => {
        console.log(`[${channelId}] 收到数据:`, data);
      });
      
      if (subscription) {
        setSubscriptions(prev => new Map(prev.set(channelId, subscription)));
        setActiveChannels(prev => [...prev, channelId]);
      }
    }
  };
  
  const handleUnsubscribe = (channelId) => {
    const subscription = subscriptions.get(channelId);
    if (subscription && subscription.unsubscribe) {
      subscription.unsubscribe();
      setSubscriptions(prev => {
        const newMap = new Map(prev);
        newMap.delete(channelId);
        return newMap;
      });
      setActiveChannels(prev => prev.filter(id => id !== channelId));
    }
  };
  
  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">实时数据订阅</h3>
      <div className="space-y-3">
        {availableChannels.map((channel) => {
          const isActive = activeChannels.includes(channel.id);
          
          return (
            <div key={channel.id} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
              <div className="flex-1">
                <div className="flex items-center gap-2">
                  <div className={cn(
                    'w-2 h-2 rounded-full',
                    isActive ? 'bg-green-500' : 'bg-gray-300'
                  )} />
                  <span className="font-medium text-gray-900">{channel.name}</span>
                </div>
                <p className="text-sm text-gray-500 mt-1">{channel.description}</p>
              </div>
              <button
                onClick={() => isActive ? handleUnsubscribe(channel.id) : handleSubscribe(channel.id)}
                className={cn(
                  'px-3 py-1 text-sm font-medium rounded transition-colors',
                  isActive
                    ? 'bg-red-100 text-red-700 hover:bg-red-200'
                    : 'bg-blue-100 text-blue-700 hover:bg-blue-200'
                )}
              >
                {isActive ? '取消订阅' : '订阅'}
              </button>
            </div>
          );
        })}
      </div>
    </div>
  );
};

// 日志显示组件
const SystemLogsCard = () => {
  const { data: logsData } = useQuery({
    queryKey: ['systemLogs'],
    queryFn: () => apiService.system.getLogs(20),
    refetchInterval: REFRESH_INTERVALS.NORMAL,
    select: (response) => response.data?.data?.logs || response.data?.logs || []
  });
  
  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">系统日志</h3>
      <div className="bg-gray-50 rounded-lg p-4 h-64 overflow-y-auto">
        <div className="space-y-1 font-mono text-sm">
          {logsData && logsData.length > 0 ? (
            logsData.map((log, index) => (
              <div key={index} className="text-gray-700 leading-relaxed">
                {log}
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

// API网关状态总览组件
const ApiGatewayStatusCard = () => {
  const [gatewayStatus, setGatewayStatus] = useState(null);
  const [apiMetrics, setApiMetrics] = useState(null);
  
  useEffect(() => {
    const updateGatewayStatus = async () => {
      try {
        const status = arbitrageSDK.getStatus();
        setGatewayStatus(status);
        
        // 获取API指标
        if (status.initialized && arbitrageSDK.api?.system?.getMetrics) {
          const metrics = await arbitrageSDK.api.system.getMetrics();
          setApiMetrics(metrics);
        }
      } catch (error) {
        console.error('获取API网关状态失败:', error);
      }
    };
    
    updateGatewayStatus();
    const interval = setInterval(updateGatewayStatus, 3000);
    
    return () => clearInterval(interval);
  }, []);
  
  const statusItems = [
    {
      label: 'SDK状态',
      value: gatewayStatus?.initialized ? '已初始化' : '未初始化',
      color: gatewayStatus?.initialized ? '#52c41a' : '#ff4d4f'
    },
    {
      label: 'HTTP连接',
      value: gatewayStatus?.httpConnected ? '已连接' : '未连接',
      color: gatewayStatus?.httpConnected ? '#52c41a' : '#ff4d4f'
    },
    {
      label: 'WebSocket',
      value: gatewayStatus?.wsConnected ? '已连接' : '未连接',
      color: gatewayStatus?.wsConnected ? '#52c41a' : '#faad14'
    }
  ];
  
  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900">API网关状态</h3>
        <div className="flex items-center gap-1">
          <div className={cn(
            'w-3 h-3 rounded-full animate-pulse',
            gatewayStatus?.initialized ? 'bg-green-500' : 'bg-red-500'
          )} />
          <span className="text-sm font-medium text-gray-600">
            完整API网关 v5.1
          </span>
        </div>
      </div>
      
      <div className="grid grid-cols-3 gap-4 mb-4">
        {statusItems.map((item) => (
          <div key={item.label} className="text-center">
            <p className="text-xs text-gray-500">{item.label}</p>
            <p className="text-sm font-medium" style={{ color: item.color }}>
              {item.value}
            </p>
          </div>
        ))}
      </div>
      
      {apiMetrics && (
        <div className="grid grid-cols-2 gap-4 pt-4 border-t border-gray-200">
          <div>
            <p className="text-xs text-gray-500">请求总数</p>
            <p className="text-lg font-bold text-blue-600">
              {apiMetrics.totalRequests || 0}
            </p>
          </div>
          <div>
            <p className="text-xs text-gray-500">平均响应时间</p>
            <p className="text-lg font-bold text-green-600">
              {apiMetrics.avgResponseTime || 0}ms
            </p>
          </div>
        </div>
      )}
      
      <div className="mt-4 flex justify-between items-center text-xs text-gray-500">
        <span>完整API网关集成</span>
        <span>实时同步: {gatewayStatus?.wsConnected ? '开启' : '关闭'}</span>
      </div>
    </div>
  );
};

// 主系统控制页面组件
const SystemControl = () => {
  const queryClient = useQueryClient();
  const [notification, setNotification] = useState(null);
  
  // 获取系统状态
  const { data: systemData, isLoading, error } = useQuery({
    queryKey: ['systemStatus'],
    queryFn: () => apiService.system.getStatus(),
    refetchInterval: REFRESH_INTERVALS.FAST,
    select: (response) => {
      const data = response.data?.data || {};
      console.log('系统状态数据:', data);
      console.log('isRunning:', data.isRunning, typeof data.isRunning);
      return data;
    },
    retry: 3,
    retryDelay: 1000,
  });
  
  // 显示通知
  const showNotification = useCallback((message, type = 'info') => {
    setNotification({ message, type });
    setTimeout(() => setNotification(null), 4000);
  }, []);
  
  // 启动系统
  const startSystemMutation = useMutation({
    mutationFn: () => apiService.system.start(),
    onSuccess: (response) => {
      console.log('系统启动成功:', response);
      queryClient.invalidateQueries(['systemStatus']);
      showNotification(response.data?.message || response.message || SUCCESS_MESSAGES.SYSTEM_STARTED, 'success');
    },
    onError: (error) => {
      console.error('系统启动失败:', error);
      const errorMsg = error.message || ERROR_MESSAGES.SYSTEM_START_FAILED;
      showNotification(`系统启动失败: ${errorMsg}`, 'error');
    },
  });
  
  // 停止系统
  const stopSystemMutation = useMutation({
    mutationFn: () => apiService.system.stop(),
    onSuccess: (response) => {
      queryClient.invalidateQueries(['systemStatus']);
      showNotification(response.data?.message || SUCCESS_MESSAGES.SYSTEM_STOPPED, 'success');
    },
    onError: (error) => {
      showNotification(error.message || ERROR_MESSAGES.SYSTEM_STOP_FAILED, 'error');
    },
  });
  
  const isOperating = startSystemMutation.isLoading || stopSystemMutation.isLoading;
  
  if (error) {
    return (
      <div className="min-h-screen bg-gray-50 p-6">
        <div className="max-w-4xl mx-auto">
          <div className="bg-red-50 border border-red-200 rounded-lg p-6 text-center">
            <p className="text-red-600 text-lg font-medium mb-2">
              系统状态加载失败
            </p>
            <p className="text-red-500 text-sm">
              {error.message}
            </p>
            <button
              onClick={() => queryClient.invalidateQueries(['systemStatus'])}
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
      
      <div className="max-w-6xl mx-auto">
        {/* 页面标题 */}
        <div className="mb-6">
          <h1 className="text-3xl font-bold text-gray-900">系统控制 - 完整API网关</h1>
          <p className="text-gray-600 mt-1">
            5.1套利系统核心控制面板 - 实时监控系统状态、WebSocket连接管理、数据订阅控制
          </p>
          <div className="flex items-center gap-2 mt-2">
            <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
              完整API网关集成
            </span>
            <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
              实时WebSocket支持
            </span>
            <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-purple-100 text-purple-800">
              前端100%控制后端
            </span>
            <button 
              onClick={async () => {
                try {
                  const response = await fetch('http://localhost:8080/api/system/start', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' }
                  });
                  const data = await response.json();
                  console.log('直接API测试结果:', data);
                  alert('直接API调用成功: ' + data.message);
                } catch (error) {
                  console.error('直接API测试失败:', error);
                  alert('直接API调用失败: ' + error.message);
                }
              }}
              className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800 hover:bg-yellow-200"
            >
              🔧 测试直接API
            </button>
          </div>
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
                  </div>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="space-y-6">
            {/* 第零行：API网关状态总览 */}
            <ApiGatewayStatusCard />
            
            {/* 第一行：系统控制和性能指标 */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <SystemControlCard
                systemData={systemData}
                onStart={() => startSystemMutation.mutate()}
                onStop={() => stopSystemMutation.mutate()}
                loading={isOperating}
              />
              <PerformanceCard systemData={systemData} />
            </div>
            
            {/* 第二行：WebSocket控制和实时数据订阅 */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <WebSocketControlCard />
              <RealTimeDataCard />
            </div>
            
            {/* 第三行：模块状态和系统日志 */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <ModulesStatusCard systemData={systemData} />
              <SystemLogsCard />
            </div>
          </div>
        )}
        
        {/* 页面底部信息 */}
        <div className="mt-8 text-center text-sm text-gray-500">
          最后更新: {formatTime(new Date())} | 
          刷新间隔: {REFRESH_INTERVALS.FAST / 1000}秒
        </div>
      </div>
    </div>
  );
};

export default SystemControl;