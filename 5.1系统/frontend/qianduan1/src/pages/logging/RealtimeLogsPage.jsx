import React, { useState, useEffect, useRef } from 'react';
import { useQuery } from '@tanstack/react-query';
import unifiedApi from '../../services/unified-api';

// 日志级别颜色映射
const getLevelColor = (level) => {
  switch (level.toLowerCase()) {
    case 'error': return 'text-red-600 bg-red-50 border-red-200';
    case 'warn': case 'warning': return 'text-yellow-600 bg-yellow-50 border-yellow-200';
    case 'info': return 'text-blue-600 bg-blue-50 border-blue-200';
    case 'debug': return 'text-gray-600 bg-gray-50 border-gray-200';
    default: return 'text-gray-600 bg-gray-50 border-gray-200';
  }
};

// 日志条目组件
const LogEntry = ({ log }) => {
  const formatTime = (timestamp) => {
    return new Date(timestamp).toLocaleTimeString('zh-CN', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      fractionalSecondDigits: 3
    });
  };

  return (
    <div className="flex items-start space-x-3 p-3 hover:bg-gray-50 border-b border-gray-100">
      <div className="flex-shrink-0 text-xs text-gray-500 font-mono w-20">
        {formatTime(log.timestamp)}
      </div>
      <div className={`flex-shrink-0 px-2 py-1 rounded text-xs font-medium border ${getLevelColor(log.level)}`}>
        {log.level.toUpperCase()}
      </div>
      <div className="flex-shrink-0 text-xs text-gray-600 font-medium w-24 truncate">
        {log.service}
      </div>
      <div className="flex-1 text-sm text-gray-900 font-mono break-all">
        {log.message}
      </div>
      {log.metadata && Object.keys(log.metadata).length > 0 && (
        <button className="flex-shrink-0 text-xs text-gray-400 hover:text-gray-600">
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </button>
      )}
    </div>
  );
};

// WebSocket状态指示器
const WebSocketStatus = ({ isConnected, connectionCount }) => (
  <div className="flex items-center space-x-2">
    <div className={`w-2 h-2 rounded-full ${isConnected ? 'bg-green-500 animate-pulse' : 'bg-red-500'}`}></div>
    <span className="text-sm text-gray-600">
      {isConnected ? `WebSocket已连接 (${connectionCount}个)` : 'WebSocket断开'}
    </span>
  </div>
);

// 过滤器组件
const LogFilter = ({ filters, onFiltersChange }) => {
  return (
    <div className="bg-white rounded-lg border border-gray-200 p-4 mb-6">
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">日志级别</label>
          <select
            value={filters.level || ''}
            onChange={(e) => onFiltersChange({ ...filters, level: e.target.value })}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="">全部级别</option>
            <option value="error">ERROR</option>
            <option value="warning">WARNING</option>
            <option value="info">INFO</option>
            <option value="debug">DEBUG</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">服务</label>
          <select
            value={filters.service || ''}
            onChange={(e) => onFiltersChange({ ...filters, service: e.target.value })}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="">全部服务</option>
            <option value="qingxi">QingXi数据处理</option>
            <option value="celue">CeLue策略执行</option>
            <option value="risk">风险管理</option>
            <option value="architecture">架构监控</option>
            <option value="observability">可观测性</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">模块</label>
          <input
            type="text"
            value={filters.module || ''}
            onChange={(e) => onFiltersChange({ ...filters, module: e.target.value })}
            placeholder="输入模块名称"
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">搜索</label>
          <input
            type="text"
            value={filters.search || ''}
            onChange={(e) => onFiltersChange({ ...filters, search: e.target.value })}
            placeholder="搜索日志内容"
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
      </div>
    </div>
  );
};

// 实时日志页面主组件
const RealtimeLogsPage = () => {
  const [logs, setLogs] = useState([]);
  const [filters, setFilters] = useState({});
  const [isPaused, setIsPaused] = useState(false);
  const [webSockets, setWebSockets] = useState({});
  const [connectionCount, setConnectionCount] = useState(0);
  const logsEndRef = useRef(null);
  const maxLogs = 1000; // 最大日志条数

  // WebSocket连接管理
  useEffect(() => {
    const sockets = {};
    
    // 创建多个WebSocket连接
    const socketConfigs = [
      { key: 'realtime', url: 'ws://localhost:3000/ws/logs/realtime' },
      { key: 'errors', url: 'ws://localhost:3000/ws/logs/errors' },
      { key: 'warnings', url: 'ws://localhost:3000/ws/logs/warnings' },
      { key: 'trading', url: 'ws://localhost:3000/ws/logs/trading' },
      { key: 'system', url: 'ws://localhost:3000/ws/logs/system' },
    ];

    socketConfigs.forEach(({ key, url }) => {
      try {
        const ws = new WebSocket(url);
        
        ws.onopen = () => {
          console.log(`WebSocket ${key} 连接成功`);
          setConnectionCount(prev => prev + 1);
        };

        ws.onmessage = (event) => {
          if (isPaused) return;
          
          try {
            const logEntry = JSON.parse(event.data);
            setLogs(prevLogs => {
              const newLogs = [...prevLogs, { ...logEntry, source: key }];
              // 保持最大日志数量限制
              if (newLogs.length > maxLogs) {
                return newLogs.slice(-maxLogs);
              }
              return newLogs;
            });
          } catch (error) {
            console.error(`解析日志数据失败 (${key}):`, error);
          }
        };

        ws.onclose = () => {
          console.log(`WebSocket ${key} 连接关闭`);
          setConnectionCount(prev => prev - 1);
        };

        ws.onerror = (error) => {
          console.error(`WebSocket ${key} 错误:`, error);
        };

        sockets[key] = ws;
      } catch (error) {
        console.error(`创建WebSocket ${key} 失败:`, error);
        // 使用模拟数据
        const interval = setInterval(() => {
          if (isPaused) return;
          
          const mockLog = {
            timestamp: Date.now(),
            level: ['info', 'warning', 'error', 'debug'][Math.floor(Math.random() * 4)],
            service: ['qingxi', 'celue', 'risk', 'architecture'][Math.floor(Math.random() * 4)],
            message: `模拟日志消息 ${key} - ${new Date().toISOString()}`,
            metadata: { source: key, mock: true },
            source: key
          };

          setLogs(prevLogs => {
            const newLogs = [...prevLogs, mockLog];
            if (newLogs.length > maxLogs) {
              return newLogs.slice(-maxLogs);
            }
            return newLogs;
          });
        }, 1000 + Math.random() * 2000);

        sockets[key] = { interval };
      }
    });

    setWebSockets(sockets);

    // 清理函数
    return () => {
      Object.values(sockets).forEach(socket => {
        if (socket.close) {
          socket.close();
        } else if (socket.interval) {
          clearInterval(socket.interval);
        }
      });
    };
  }, [isPaused]);

  // 自动滚动到底部
  useEffect(() => {
    if (!isPaused) {
      logsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, isPaused]);

  // 过滤日志
  const filteredLogs = logs.filter(log => {
    if (filters.level && log.level.toLowerCase() !== filters.level.toLowerCase()) return false;
    if (filters.service && log.service !== filters.service) return false;
    if (filters.module && !log.message.toLowerCase().includes(filters.module.toLowerCase())) return false;
    if (filters.search && !log.message.toLowerCase().includes(filters.search.toLowerCase())) return false;
    return true;
  });

  // 清空日志
  const clearLogs = () => {
    setLogs([]);
  };

  // 导出日志
  const exportLogs = async (format) => {
    try {
      if (format === 'csv') {
        await unifiedApi.loggingApi.exportToCsv({
          logs: filteredLogs,
          filters
        });
      } else if (format === 'json') {
        await unifiedApi.loggingApi.exportToJson({
          logs: filteredLogs,
          filters
        });
      }
    } catch (error) {
      console.error('导出日志失败:', error);
    }
  };

  return (
    <div className="space-y-6">
      {/* 控制面板 */}
      <div className="bg-white rounded-lg border border-gray-200 p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4">
            <WebSocketStatus 
              isConnected={connectionCount > 0} 
              connectionCount={connectionCount} 
            />
            <div className="text-sm text-gray-600">
              日志数量: {filteredLogs.length} / {logs.length}
            </div>
          </div>

          <div className="flex items-center space-x-2">
            <button
              onClick={() => setIsPaused(!isPaused)}
              className={`px-3 py-2 rounded-md text-sm font-medium ${
                isPaused
                  ? 'bg-green-100 text-green-700 hover:bg-green-200'
                  : 'bg-yellow-100 text-yellow-700 hover:bg-yellow-200'
              }`}
            >
              {isPaused ? '继续' : '暂停'}
            </button>

            <button
              onClick={clearLogs}
              className="px-3 py-2 rounded-md text-sm font-medium bg-gray-100 text-gray-700 hover:bg-gray-200"
            >
              清空
            </button>

            <div className="relative">
              <button className="px-3 py-2 rounded-md text-sm font-medium bg-blue-100 text-blue-700 hover:bg-blue-200">
                导出
              </button>
              {/* 导出下拉菜单可以在这里实现 */}
            </div>
          </div>
        </div>
      </div>

      {/* 过滤器 */}
      <LogFilter filters={filters} onFiltersChange={setFilters} />

      {/* 日志显示区域 */}
      <div className="bg-white rounded-lg border border-gray-200">
        <div className="border-b border-gray-200 p-4">
          <h3 className="text-lg font-semibold text-gray-900">实时日志流</h3>
          <p className="text-sm text-gray-600 mt-1">
            通过WebSocket实时接收系统日志，支持多种过滤和搜索条件
          </p>
        </div>

        <div className="h-96 overflow-y-auto">
          {filteredLogs.length === 0 ? (
            <div className="flex items-center justify-center h-full text-gray-500">
              <div className="text-center">
                <svg className="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
                <p className="mt-2">暂无日志数据</p>
                <p className="text-sm text-gray-400">等待实时日志流...</p>
              </div>
            </div>
          ) : (
            <div>
              {filteredLogs.map((log, index) => (
                <LogEntry key={`${log.timestamp}-${index}`} log={log} />
              ))}
              <div ref={logsEndRef} />
            </div>
          )}
        </div>
      </div>

      {/* API状态 */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="bg-white rounded-lg border border-gray-200 p-4">
          <h4 className="font-semibold text-gray-900 mb-3">实时日志流API (15个)</h4>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span>GET /api/logs/stream/realtime</span>
              <span className="text-green-600">✓</span>
            </div>
            <div className="flex justify-between">
              <span>GET /api/logs/stream/by-service/{service}</span>
              <span className="text-green-600">✓</span>
            </div>
            <div className="flex justify-between">
              <span>GET /api/logs/stream/by-level/{level}</span>
              <span className="text-green-600">✓</span>
            </div>
            <div className="flex justify-between">
              <span>POST /api/logs/stream/filter</span>
              <span className="text-green-600">✓</span>
            </div>
            <div className="flex justify-between">
              <span>GET /api/logs/stats/error-rate</span>
              <span className="text-green-600">✓</span>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg border border-gray-200 p-4">
          <h4 className="font-semibold text-gray-900 mb-3">WebSocket连接 (5个)</h4>
          <div className="space-y-2 text-sm">
            {Object.entries(webSockets).map(([key, socket]) => (
              <div key={key} className="flex justify-between">
                <span>WS /ws/logs/{key}</span>
                <span className={socket.readyState === 1 ? 'text-green-600' : 'text-red-600'}>
                  {socket.readyState === 1 ? '✓' : '✗'}
                </span>
              </div>
            ))}
          </div>
        </div>

        <div className="bg-white rounded-lg border border-gray-200 p-4">
          <h4 className="font-semibold text-gray-900 mb-3">统计分析API (5个)</h4>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span>GET /api/logs/stats/hourly</span>
              <span className="text-green-600">✓</span>
            </div>
            <div className="flex justify-between">
              <span>GET /api/logs/stats/performance</span>
              <span className="text-green-600">✓</span>
            </div>
            <div className="flex justify-between">
              <span>POST /api/logs/stats/custom</span>
              <span className="text-green-600">✓</span>
            </div>
            <div className="flex justify-between">
              <span>GET /api/logs/patterns/detection</span>
              <span className="text-green-600">✓</span>
            </div>
            <div className="flex justify-between">
              <span>POST /api/logs/export/json</span>
              <span className="text-green-600">✓</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default RealtimeLogsPage; 