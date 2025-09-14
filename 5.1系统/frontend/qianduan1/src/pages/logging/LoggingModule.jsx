import React, { useState, useEffect } from 'react';
import { Routes, Route, Link, useLocation } from 'react-router-dom';
import { useQuery, useMutation } from '@tanstack/react-query';
import unifiedApi from '../../services/unified-api';

// 子页面组件
import RealtimeLogsPage from './RealtimeLogsPage';
import LogConfigPage from './LogConfigPage';
import LogAnalysisPage from './LogAnalysisPage';

// 导航标签组件
const NavTab = ({ to, children, isActive }) => (
  <Link
    to={to}
    className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
      isActive
        ? 'bg-blue-100 text-blue-700 border border-blue-200'
        : 'text-gray-600 hover:text-gray-900 hover:bg-gray-100'
    }`}
  >
    {children}
  </Link>
);

// 状态卡片组件
const StatusCard = ({ title, value, icon, color, description }) => (
  <div className="bg-white rounded-lg border border-gray-200 p-6">
    <div className="flex items-center">
      <div className={`flex-shrink-0 w-8 h-8 rounded-lg flex items-center justify-center ${color}`}>
        {icon}
      </div>
      <div className="ml-4 flex-1">
        <p className="text-sm font-medium text-gray-600">{title}</p>
        <p className="text-2xl font-bold text-gray-900">{value}</p>
        {description && (
          <p className="text-xs text-gray-500 mt-1">{description}</p>
        )}
      </div>
    </div>
  </div>
);

// API状态指示器
const ApiStatus = ({ api, status }) => {
  const getStatusColor = (status) => {
    switch (status) {
      case 'healthy': return 'text-green-600 bg-green-100';
      case 'warning': return 'text-yellow-600 bg-yellow-100';
      case 'error': return 'text-red-600 bg-red-100';
      default: return 'text-gray-600 bg-gray-100';
    }
  };

  return (
    <div className="flex items-center justify-between p-3 bg-white rounded-lg border border-gray-200">
      <div className="flex items-center space-x-3">
        <div className={`w-2 h-2 rounded-full ${status === 'healthy' ? 'bg-green-500' : status === 'warning' ? 'bg-yellow-500' : 'bg-red-500'}`}></div>
        <span className="text-sm font-medium text-gray-900">{api}</span>
      </div>
      <span className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(status)}`}>
        {status}
      </span>
    </div>
  );
};

// 日志监控主页面
const LoggingOverview = () => {
  const { data: logStats, isLoading } = useQuery({
    queryKey: ['log-stats'],
    queryFn: async () => {
      // 模拟API调用
      return {
        totalLogs: 1250000,
        errorLogs: 1250,
        warningLogs: 8500,
        infoLogs: 1240250,
        logRate: 450, // logs per second
        storageUsed: '2.3 GB',
        retentionDays: 30
      };
    },
    refetchInterval: 5000,
  });

  // 45个日志API的状态
  const apiList = [
    // 实时日志流 (15个API)
    { name: 'GET /api/logs/stream/realtime', status: 'healthy', category: '实时日志流' },
    { name: 'GET /api/logs/stream/by-service/{service}', status: 'healthy', category: '实时日志流' },
    { name: 'GET /api/logs/stream/by-level/{level}', status: 'healthy', category: '实时日志流' },
    { name: 'GET /api/logs/stream/by-module/{module}', status: 'healthy', category: '实时日志流' },
    { name: 'POST /api/logs/stream/filter', status: 'healthy', category: '实时日志流' },
    { name: 'WS /ws/logs/realtime', status: 'warning', category: '实时日志流' },
    { name: 'WS /ws/logs/errors', status: 'healthy', category: '实时日志流' },
    { name: 'WS /ws/logs/warnings', status: 'healthy', category: '实时日志流' },
    { name: 'WS /ws/logs/trading', status: 'healthy', category: '实时日志流' },
    { name: 'WS /ws/logs/system', status: 'healthy', category: '实时日志流' },
    { name: 'GET /api/logs/stats/hourly', status: 'healthy', category: '实时日志流' },
    { name: 'GET /api/logs/stats/error-rate', status: 'healthy', category: '实时日志流' },
    { name: 'GET /api/logs/stats/performance', status: 'healthy', category: '实时日志流' },
    { name: 'POST /api/logs/stats/custom', status: 'healthy', category: '实时日志流' },
    { name: 'GET /api/logs/patterns/detection', status: 'healthy', category: '实时日志流' },

    // 日志配置管理 (18个API)
    { name: 'GET /api/logs/config/levels', status: 'healthy', category: '配置管理' },
    { name: 'PUT /api/logs/config/levels/{module}', status: 'healthy', category: '配置管理' },
    { name: 'POST /api/logs/config/levels/batch', status: 'healthy', category: '配置管理' },
    { name: 'GET /api/logs/config/appenders', status: 'healthy', category: '配置管理' },
    { name: 'PUT /api/logs/config/appenders/{id}', status: 'healthy', category: '配置管理' },
    { name: 'GET /api/logs/config/rotation', status: 'healthy', category: '配置管理' },
    { name: 'PUT /api/logs/config/rotation', status: 'healthy', category: '配置管理' },
    { name: 'POST /api/logs/rotation/trigger', status: 'healthy', category: '配置管理' },
    { name: 'GET /api/logs/files/list', status: 'healthy', category: '配置管理' },
    { name: 'GET /api/logs/files/{filename}/download', status: 'healthy', category: '配置管理' },
    { name: 'GET /api/logs/storage/config', status: 'healthy', category: '配置管理' },
    { name: 'PUT /api/logs/storage/config', status: 'healthy', category: '配置管理' },
    { name: 'GET /api/logs/storage/usage', status: 'warning', category: '配置管理' },
    { name: 'POST /api/logs/storage/cleanup', status: 'healthy', category: '配置管理' },
    { name: 'GET /api/logs/storage/backup', status: 'healthy', category: '配置管理' },
    { name: 'POST /api/logs/storage/backup/create', status: 'healthy', category: '配置管理' },
    { name: 'POST /api/logs/storage/restore', status: 'healthy', category: '配置管理' },
    { name: 'GET /api/logs/retention/policy', status: 'healthy', category: '配置管理' },

    // 日志分析工具 (12个API)
    { name: 'POST /api/logs/analysis/anomaly', status: 'healthy', category: '分析工具' },
    { name: 'POST /api/logs/analysis/correlation', status: 'healthy', category: '分析工具' },
    { name: 'POST /api/logs/analysis/timeline', status: 'healthy', category: '分析工具' },
    { name: 'GET /api/logs/analysis/frequent-errors', status: 'healthy', category: '分析工具' },
    { name: 'POST /api/logs/analysis/root-cause', status: 'healthy', category: '分析工具' },
    { name: 'POST /api/logs/reports/generate', status: 'healthy', category: '分析工具' },
    { name: 'GET /api/logs/reports/templates', status: 'healthy', category: '分析工具' },
    { name: 'GET /api/logs/reports/scheduled', status: 'healthy', category: '分析工具' },
    { name: 'POST /api/logs/reports/schedule', status: 'healthy', category: '分析工具' },
    { name: 'DELETE /api/logs/reports/{id}', status: 'healthy', category: '分析工具' },
    { name: 'POST /api/logs/export/csv', status: 'healthy', category: '分析工具' },
    { name: 'POST /api/logs/export/json', status: 'healthy', category: '分析工具' },
  ];

  // 按分类分组API
  const groupedApis = apiList.reduce((groups, api) => {
    if (!groups[api.category]) {
      groups[api.category] = [];
    }
    groups[api.category].push(api);
    return groups;
  }, {});

  if (isLoading) {
    return (
      <div className="p-6">
        <div className="animate-pulse">
          <div className="h-8 bg-gray-200 rounded w-1/4 mb-6"></div>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
            {[...Array(4)].map((_, i) => (
              <div key={i} className="h-24 bg-gray-200 rounded"></div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* 统计概览 */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <StatusCard
          title="总日志数量"
          value={logStats?.totalLogs.toLocaleString()}
          icon={<svg className="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" /></svg>}
          color="bg-blue-100"
          description="过去24小时"
        />
        <StatusCard
          title="错误日志"
          value={logStats?.errorLogs.toLocaleString()}
          icon={<svg className="w-5 h-5 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>}
          color="bg-red-100"
          description="需要关注"
        />
        <StatusCard
          title="日志速率"
          value={`${logStats?.logRate}/s`}
          icon={<svg className="w-5 h-5 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>}
          color="bg-green-100"
          description="实时写入速度"
        />
        <StatusCard
          title="存储使用"
          value={logStats?.storageUsed}
          icon={<svg className="w-5 h-5 text-yellow-600" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 4V2a1 1 0 011-1h8a1 1 0 011 1v2m-9 0h10a2 2 0 012 2v12a2 2 0 01-2 2H6a2 2 0 01-2-2V6a2 2 0 012-2z" /></svg>}
          color="bg-yellow-100"
          description={`保留${logStats?.retentionDays}天`}
        />
      </div>

      {/* API状态监控 */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <div className="flex items-center justify-between mb-6">
          <h3 className="text-lg font-semibold text-gray-900">API接口状态监控</h3>
          <div className="flex items-center space-x-4 text-sm">
            <div className="flex items-center space-x-1">
              <div className="w-2 h-2 bg-green-500 rounded-full"></div>
              <span className="text-gray-600">健康 ({apiList.filter(api => api.status === 'healthy').length})</span>
            </div>
            <div className="flex items-center space-x-1">
              <div className="w-2 h-2 bg-yellow-500 rounded-full"></div>
              <span className="text-gray-600">警告 ({apiList.filter(api => api.status === 'warning').length})</span>
            </div>
            <div className="flex items-center space-x-1">
              <div className="w-2 h-2 bg-red-500 rounded-full"></div>
              <span className="text-gray-600">错误 ({apiList.filter(api => api.status === 'error').length})</span>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {Object.entries(groupedApis).map(([category, apis]) => (
            <div key={category} className="space-y-3">
              <h4 className="font-medium text-gray-900 border-b border-gray-200 pb-2">
                {category} ({apis.length}个)
              </h4>
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {apis.map((api, index) => (
                  <ApiStatus key={index} api={api.name} status={api.status} />
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* 快速操作 */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Link to="realtime" className="block p-6 bg-white rounded-lg border border-gray-200 hover:shadow-md transition-shadow">
          <div className="flex items-center space-x-4">
            <div className="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center">
              <svg className="w-6 h-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
            </div>
            <div>
              <h3 className="font-semibold text-gray-900">实时日志流</h3>
              <p className="text-sm text-gray-600">查看实时日志流和WebSocket连接</p>
            </div>
          </div>
        </Link>

        <Link to="config" className="block p-6 bg-white rounded-lg border border-gray-200 hover:shadow-md transition-shadow">
          <div className="flex items-center space-x-4">
            <div className="w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center">
              <svg className="w-6 h-6 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
            </div>
            <div>
              <h3 className="font-semibold text-gray-900">配置管理</h3>
              <p className="text-sm text-gray-600">管理日志级别和存储配置</p>
            </div>
          </div>
        </Link>

        <Link to="analysis" className="block p-6 bg-white rounded-lg border border-gray-200 hover:shadow-md transition-shadow">
          <div className="flex items-center space-x-4">
            <div className="w-12 h-12 bg-purple-100 rounded-lg flex items-center justify-center">
              <svg className="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v4a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
              </svg>
            </div>
            <div>
              <h3 className="font-semibold text-gray-900">日志分析</h3>
              <p className="text-sm text-gray-600">异常检测和根因分析工具</p>
            </div>
          </div>
        </Link>
      </div>
    </div>
  );
};

// 日志监控主组件
const LoggingModule = () => {
  const location = useLocation();
  
  // 导航配置
  const tabs = [
    { path: '/logging', label: '总览', exact: true },
    { path: '/logging/realtime', label: '实时日志' },
    { path: '/logging/config', label: '配置管理' },
    { path: '/logging/analysis', label: '日志分析' },
  ];

  return (
    <div className="min-h-screen bg-gray-50">
      {/* 页面头部 */}
      <div className="bg-white shadow-sm border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-6">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">日志监控系统</h1>
              <p className="mt-1 text-sm text-gray-600">
                45个专业API接口 - 实时日志流、配置管理、分析工具
              </p>
            </div>
            <div className="flex items-center space-x-2">
              <div className="w-2 h-2 bg-blue-500 rounded-full animate-pulse"></div>
              <span className="text-sm text-gray-600">实时监控中</span>
            </div>
          </div>
          
          {/* 导航标签 */}
          <div className="flex space-x-2 pb-4">
            {tabs.map((tab) => (
              <NavTab
                key={tab.path}
                to={tab.path}
                isActive={tab.exact ? location.pathname === tab.path : location.pathname.startsWith(tab.path)}
              >
                {tab.label}
              </NavTab>
            ))}
          </div>
        </div>
      </div>

      {/* 页面内容 */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <Routes>
          <Route path="/" element={<LoggingOverview />} />
          <Route path="/realtime" element={<RealtimeLogsPage />} />
          <Route path="/config" element={<LogConfigPage />} />
          <Route path="/analysis" element={<LogAnalysisPage />} />
        </Routes>
      </div>
    </div>
  );
};

export default LoggingModule; 