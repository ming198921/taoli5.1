/**
 * 增强版系统控制页面 - 使用完整API网关实现100%控制后端
 */

import { useState, useEffect, useCallback } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import Navigation from '../../components/Navigation.jsx';
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

// 系统控制卡片组件 - 增强版
const EnhancedSystemControlCard = ({ systemData, onStart, onStop, loading }) => {
  const { isRunning, lastStarted, uptime, version } = systemData || {};
  const [currentUptime, setCurrentUptime] = useState(0);
  const [realTimeMetrics, setRealTimeMetrics] = useState(null);
  
  // 实时指标订阅
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
      
      <div className="flex gap-3 mb-4">
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
          onClick={onStop}
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
        </button>
      </div>

      {/* API网关状态指示器 */}
      <div className="border-t pt-4">
        <div className="flex items-center justify-between text-sm">
          <span className="text-gray-600">API网关状态:</span>
          <div className="flex items-center gap-2">
            <div className={cn(
              'w-2 h-2 rounded-full',
              arbitrageSDK.getStatus().initialized ? 'bg-green-500' : 'bg-red-500'
            )} />
            <span>
              {arbitrageSDK.getStatus().initialized ? '已连接' : '未连接'}
            </span>
          </div>
        </div>
        <div className="flex items-center justify-between text-sm mt-1">
          <span className="text-gray-600">WebSocket:</span>
          <div className="flex items-center gap-2">
            <div className={cn(
              'w-2 h-2 rounded-full',
              arbitrageSDK.getWebSocketStatus().connected ? 'bg-green-500' : 'bg-gray-400'
            )} />
            <span>
              {arbitrageSDK.getWebSocketStatus().connected ? '已连接' : '未连接'}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
};

// 实时数据统计卡片
const RealTimeStatsCard = () => {
  const [marketData, setMarketData] = useState([]);
  const [opportunities, setOpportunities] = useState([]);
  const [alerts, setAlerts] = useState([]);

  useEffect(() => {
    let marketDataSub = null;
    let opportunitiesSub = null;
    let alertsSub = null;

    if (arbitrageSDK.getWebSocketStatus().connected) {
      marketDataSub = arbitrageSDK.subscribeMarketData((data) => {
        setMarketData(prev => [data, ...prev.slice(0, 4)]);
      });

      opportunitiesSub = arbitrageSDK.subscribeArbitrageOpportunities((opportunity) => {
        setOpportunities(prev => [opportunity, ...prev.slice(0, 2)]);
      });

      alertsSub = arbitrageSDK.subscribeAlerts((alert) => {
        setAlerts(prev => [alert, ...prev.slice(0, 2)]);
      });
    }

    return () => {
      if (marketDataSub) marketDataSub.unsubscribe();
      if (opportunitiesSub) opportunitiesSub.unsubscribe();
      if (alertsSub) alertsSub.unsubscribe();
    };
  }, []);

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">实时数据流</h3>
      
      <div className="space-y-4">
        <div>
          <h4 className="text-sm font-medium text-gray-700 mb-2">市场数据更新</h4>
          {marketData.length > 0 ? (
            <div className="space-y-1">
              {marketData.map((data, index) => (
                <div key={index} className="text-xs bg-blue-50 p-2 rounded">
                  {data.symbol}@{data.exchange}: ${data.price}
                </div>
              ))}
            </div>
          ) : (
            <div className="text-xs text-gray-500">等待市场数据...</div>
          )}
        </div>

        <div>
          <h4 className="text-sm font-medium text-gray-700 mb-2">套利机会</h4>
          {opportunities.length > 0 ? (
            <div className="space-y-1">
              {opportunities.map((opp, index) => (
                <div key={index} className="text-xs bg-green-50 p-2 rounded">
                  {opp.symbol}: {opp.profit_percentage.toFixed(2)}% 利润
                </div>
              ))}
            </div>
          ) : (
            <div className="text-xs text-gray-500">等待套利机会...</div>
          )}
        </div>

        <div>
          <h4 className="text-sm font-medium text-gray-700 mb-2">系统警报</h4>
          {alerts.length > 0 ? (
            <div className="space-y-1">
              {alerts.map((alert, index) => (
                <div key={index} className={cn(
                  "text-xs p-2 rounded",
                  alert.type === 'error' ? 'bg-red-50' : 
                  alert.type === 'warning' ? 'bg-yellow-50' : 'bg-blue-50'
                )}>
                  {alert.title}
                </div>
              ))}
            </div>
          ) : (
            <div className="text-xs text-gray-500">无活跃警报</div>
          )}
        </div>
      </div>
    </div>
  );
};

// API网关控制面板
const ApiGatewayControlPanel = () => {
  const [authStatus, setAuthStatus] = useState(null);
  const [connecting, setConnecting] = useState(false);

  const connectWebSocket = async () => {
    setConnecting(true);
    try {
      await arbitrageSDK.connectWebSocket();
    } catch (error) {
      console.error('WebSocket连接失败:', error);
    } finally {
      setConnecting(false);
    }
  };

  const disconnectWebSocket = () => {
    arbitrageSDK.disconnectWebSocket();
  };

  useEffect(() => {
    setAuthStatus(arbitrageSDK.getStatus());
    const interval = setInterval(() => {
      setAuthStatus(arbitrageSDK.getStatus());
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">API网关控制面板</h3>
      
      <div className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div>
            <p className="text-sm text-gray-500">SDK状态</p>
            <p className={cn(
              "text-sm font-medium",
              authStatus?.initialized ? 'text-green-600' : 'text-red-600'
            )}>
              {authStatus?.initialized ? '已初始化' : '未初始化'}
            </p>
          </div>
          <div>
            <p className="text-sm text-gray-500">HTTP连接</p>
            <p className={cn(
              "text-sm font-medium",
              authStatus?.httpConnected ? 'text-green-600' : 'text-red-600'
            )}>
              {authStatus?.httpConnected ? '已连接' : '未连接'}
            </p>
          </div>
        </div>

        <div>
          <div className="flex items-center justify-between mb-2">
            <p className="text-sm text-gray-500">WebSocket连接</p>
            <div className="flex gap-2">
              <button
                onClick={connectWebSocket}
                disabled={connecting || authStatus?.wsConnected}
                className="text-xs px-3 py-1 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
              >
                {connecting ? '连接中...' : '连接'}
              </button>
              <button
                onClick={disconnectWebSocket}
                disabled={!authStatus?.wsConnected}
                className="text-xs px-3 py-1 bg-red-600 text-white rounded hover:bg-red-700 disabled:opacity-50"
              >
                断开
              </button>
            </div>
          </div>
          <p className={cn(
            "text-sm font-medium",
            authStatus?.wsConnected ? 'text-green-600' : 'text-gray-600'
          )}>
            {authStatus?.wsConnecting ? '连接中...' : 
             authStatus?.wsConnected ? '已连接' : '未连接'}
          </p>
        </div>

        <div className="border-t pt-4">
          <p className="text-xs text-gray-500 mb-2">API端点</p>
          <div className="space-y-1 text-xs">
            <div>HTTP: {arbitrageSDK.config.baseUrl}</div>
            <div>WebSocket: {arbitrageSDK.config.wsUrl}</div>
          </div>
        </div>
      </div>
    </div>
  );
};

// 增强版主系统控制页面
const SystemControlEnhanced = () => {
  const queryClient = useQueryClient();
  const [notification, setNotification] = useState(null);
  const [sdkStatus, setSdkStatus] = useState(null);
  
  // 获取系统状态 - 使用新的SDK
  const { data: systemData, isLoading, error } = useQuery({
    queryKey: ['systemStatus'],
    queryFn: async () => {
      try {
        return await arbitrageSDK.system.getSystemStatus();
      } catch (error) {
        console.error('系统状态获取失败:', error);
        return {
          isRunning: false,
          uptime: 0,
          version: '5.1.0',
          modules: {},
          cpu_usage: 0,
          memory_usage: 0,
          network_latency: 0
        };
      }
    },
    refetchInterval: REFRESH_INTERVALS.FAST,
    retry: 3,
    retryDelay: 1000,
  });

  // 监控SDK状态
  useEffect(() => {
    const updateSdkStatus = () => {
      setSdkStatus(arbitrageSDK.getStatus());
    };

    updateSdkStatus();
    const interval = setInterval(updateSdkStatus, 1000);

    return () => clearInterval(interval);
  }, []);
  
  // 显示通知
  const showNotification = useCallback((message, type = 'info') => {
    setNotification({ message, type });
    setTimeout(() => setNotification(null), 4000);
  }, []);
  
  // 启动系统 - 使用新的SDK
  const startSystemMutation = useMutation({
    mutationFn: async () => {
      return await arbitrageSDK.system.startSystem();
    },
    onSuccess: (response) => {
      queryClient.invalidateQueries(['systemStatus']);
      showNotification(response.message || SUCCESS_MESSAGES.SYSTEM_STARTED, 'success');
    },
    onError: (error) => {
      showNotification(error.message || ERROR_MESSAGES.SYSTEM_START_FAILED, 'error');
    },
  });
  
  // 停止系统 - 使用新的SDK
  const stopSystemMutation = useMutation({
    mutationFn: async () => {
      return await arbitrageSDK.system.stopSystem();
    },
    onSuccess: (response) => {
      queryClient.invalidateQueries(['systemStatus']);
      showNotification(response.message || SUCCESS_MESSAGES.SYSTEM_STOPPED, 'success');
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
            <p className="text-xs text-gray-500 mt-4">
              如果问题持续存在，请检查API网关是否正常运行在 {arbitrageSDK.config.baseUrl}
            </p>
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
            5.1套利系统核心控制面板 - 使用完整的生产级API网关实现100%前端控制后端
          </p>
          {sdkStatus && (
            <div className="mt-2 flex items-center gap-4 text-sm">
              <span className={cn(
                "px-2 py-1 rounded",
                sdkStatus.initialized ? "bg-green-100 text-green-800" : "bg-red-100 text-red-800"
              )}>
                SDK: {sdkStatus.initialized ? '已初始化' : '未初始化'}
              </span>
              <span className={cn(
                "px-2 py-1 rounded",
                sdkStatus.wsConnected ? "bg-green-100 text-green-800" : "bg-gray-100 text-gray-800"
              )}>
                WebSocket: {sdkStatus.wsConnected ? '已连接' : '未连接'}
              </span>
            </div>
          )}
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
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* 增强版系统控制卡片 */}
            <div className="lg:col-span-2">
              <EnhancedSystemControlCard
                systemData={systemData}
                onStart={() => startSystemMutation.mutate()}
                onStop={() => stopSystemMutation.mutate()}
                loading={isOperating}
              />
            </div>
            
            {/* API网关控制面板 */}
            <ApiGatewayControlPanel />
            
            {/* 实时数据统计卡片 */}
            <RealTimeStatsCard />
          </div>
        )}
        
        {/* 页面底部信息 */}
        <div className="mt-8 text-center text-sm text-gray-500">
          最后更新: {formatTime(new Date())} | 
          刷新间隔: {REFRESH_INTERVALS.FAST / 1000}秒 |
          API网关: {arbitrageSDK.config.baseUrl}
        </div>
      </div>
    </div>
  );
};

export default SystemControlEnhanced;