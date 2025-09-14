import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import unifiedApi from '../services/unified-api';

// 图标组件
const ModuleIcon = ({ children, className = "" }) => (
  <div className={`w-12 h-12 rounded-lg flex items-center justify-center text-white font-bold text-xl ${className}`}>
    {children}
  </div>
);

// 状态指示器组件
const StatusIndicator = ({ status, count }) => {
  const getStatusColor = (status) => {
    switch (status) {
      case 'healthy': return 'bg-green-500';
      case 'warning': return 'bg-yellow-500';
      case 'error': return 'bg-red-500';
      case 'loading': return 'bg-blue-500';
      default: return 'bg-gray-500';
    }
  };

  return (
    <div className="flex items-center space-x-2">
      <div className={`w-3 h-3 rounded-full ${getStatusColor(status)}`}></div>
      <span className="text-sm font-medium">{count}</span>
    </div>
  );
};

// API模块卡片组件
const ApiModuleCard = ({ module, isLoading = false }) => {
  const { name, path, icon, iconBg, apiCount, status, description, apis } = module;
  
  return (
    <Link 
      to={path} 
      className="group block p-6 bg-white rounded-xl shadow-sm border border-gray-200 hover:shadow-lg hover:border-blue-300 transition-all duration-200"
    >
      <div className="flex items-start justify-between">
        <div className="flex items-start space-x-4">
          <ModuleIcon className={iconBg}>
            {icon}
          </ModuleIcon>
          <div className="flex-1">
            <h3 className="text-lg font-semibold text-gray-900 group-hover:text-blue-600 transition-colors">
              {name}
            </h3>
            <p className="text-sm text-gray-600 mt-1 line-clamp-2">
              {description}
            </p>
            <div className="flex items-center space-x-4 mt-3">
              <div className="flex items-center space-x-1">
                <span className="text-sm font-medium text-gray-700">API:</span>
                <span className="text-sm font-bold text-blue-600">{apiCount}</span>
              </div>
              <StatusIndicator 
                status={isLoading ? 'loading' : status.overall} 
                count={`${status.healthy}/${apiCount}`}
              />
            </div>
          </div>
        </div>
        <div className="flex items-center space-x-2">
          <div className="text-right text-xs text-gray-500">
            <div className="flex space-x-3">
              <StatusIndicator status="healthy" count={status.healthy} />
              <StatusIndicator status="warning" count={status.warning} />
              <StatusIndicator status="error" count={status.error} />
            </div>
          </div>
          <svg className="w-5 h-5 text-gray-400 group-hover:text-blue-600 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
          </svg>
        </div>
      </div>
    </Link>
  );
};

// 系统总览统计组件
const SystemOverview = ({ stats, isLoading }) => {
  if (isLoading) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
        {[...Array(4)].map((_, i) => (
          <div key={i} className="bg-white p-6 rounded-xl shadow-sm border border-gray-200 animate-pulse">
            <div className="h-4 bg-gray-200 rounded w-3/4 mb-2"></div>
            <div className="h-8 bg-gray-200 rounded w-1/2"></div>
          </div>
        ))}
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
      <div className="bg-white p-6 rounded-xl shadow-sm border border-gray-200">
        <div className="flex items-center">
          <div className="flex-shrink-0">
            <div className="w-8 h-8 bg-blue-100 rounded-lg flex items-center justify-center">
              <svg className="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
            </div>
          </div>
          <div className="ml-4">
            <p className="text-sm font-medium text-gray-600">总API接口</p>
            <p className="text-2xl font-bold text-gray-900">{stats.totalApis}</p>
          </div>
        </div>
      </div>

      <div className="bg-white p-6 rounded-xl shadow-sm border border-gray-200">
        <div className="flex items-center">
          <div className="flex-shrink-0">
            <div className="w-8 h-8 bg-green-100 rounded-lg flex items-center justify-center">
              <svg className="w-5 h-5 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
          </div>
          <div className="ml-4">
            <p className="text-sm font-medium text-gray-600">健康接口</p>
            <p className="text-2xl font-bold text-green-600">{stats.healthyApis}</p>
          </div>
        </div>
      </div>

      <div className="bg-white p-6 rounded-xl shadow-sm border border-gray-200">
        <div className="flex items-center">
          <div className="flex-shrink-0">
            <div className="w-8 h-8 bg-yellow-100 rounded-lg flex items-center justify-center">
              <svg className="w-5 h-5 text-yellow-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
              </svg>
            </div>
          </div>
          <div className="ml-4">
            <p className="text-sm font-medium text-gray-600">警告接口</p>
            <p className="text-2xl font-bold text-yellow-600">{stats.warningApis}</p>
          </div>
        </div>
      </div>

      <div className="bg-white p-6 rounded-xl shadow-sm border border-gray-200">
        <div className="flex items-center">
          <div className="flex-shrink-0">
            <div className="w-8 h-8 bg-red-100 rounded-lg flex items-center justify-center">
              <svg className="w-5 h-5 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
          </div>
          <div className="ml-4">
            <p className="text-sm font-medium text-gray-600">错误接口</p>
            <p className="text-2xl font-bold text-red-600">{stats.errorApis}</p>
          </div>
        </div>
      </div>
    </div>
  );
};

// 主仪表板组件
const Dashboard = () => {
  const [selectedModule, setSelectedModule] = useState(null);

  // 387个API模块配置
  const apiModules = [
    {
      name: '日志监控系统',
      path: '/logging',
      icon: '📊',
      iconBg: 'bg-blue-500',
      apiCount: 45,
      description: '实时日志流、配置管理、分析工具 - 45个专业API接口',
      status: { overall: 'healthy', healthy: 42, warning: 2, error: 1 },
      apis: ['实时日志流(15个)', '配置管理(18个)', '分析工具(12个)']
    },
    {
      name: '清洗配置系统',
      path: '/cleaning',
      icon: '🧹',
      iconBg: 'bg-green-500',
      apiCount: 52,
      description: '清洗规则引擎、交易所配置、SIMD优化 - 52个专业API接口',
      status: { overall: 'healthy', healthy: 50, warning: 2, error: 0 },
      apis: ['清洗规则(20个)', '交易所配置(18个)', '性能监控(14个)']
    },
    {
      name: '策略监控系统',
      path: '/strategy-monitoring',
      icon: '🎯',
      iconBg: 'bg-purple-500',
      apiCount: 38,
      description: '策略状态监控、生命周期管理、调试工具 - 38个专业API接口',
      status: { overall: 'warning', healthy: 35, warning: 3, error: 0 },
      apis: ['实时监控(15个)', '生命周期(12个)', '调试工具(11个)']
    },
    {
      name: '性能调优系统',
      path: '/performance',
      icon: '⚡',
      iconBg: 'bg-yellow-500',
      apiCount: 67,
      description: 'CPU/内存/网络/存储全方位性能调优 - 67个专业API接口',
      status: { overall: 'healthy', healthy: 64, warning: 3, error: 0 },
      apis: ['CPU调优(18个)', '内存调优(16个)', '网络调优(17个)', '存储调优(16个)']
    },
    {
      name: '交易监控系统',
      path: '/trading',
      icon: '💹',
      iconBg: 'bg-red-500',
      apiCount: 41,
      description: '实时交易监控、订单管理、风险控制 - 41个专业API接口',
      status: { overall: 'healthy', healthy: 39, warning: 2, error: 0 },
      apis: ['实时监控(15个)', '订单管理(14个)', '连接监控(12个)']
    },
    {
      name: 'AI模型管理',
      path: '/ai-model',
      icon: '🤖',
      iconBg: 'bg-indigo-500',
      apiCount: 48,
      description: '模型训练、部署、监控、解释性分析 - 48个专业API接口',
      status: { overall: 'healthy', healthy: 46, warning: 2, error: 0 },
      apis: ['训练管理(18个)', '性能监控(15个)', '配置管理(15个)']
    },
    {
      name: '配置管理系统',
      path: '/config',
      icon: '⚙️',
      iconBg: 'bg-gray-600',
      apiCount: 96,
      description: '配置热更新、版本管理、监控告警 - 96个专业API接口',
      status: { overall: 'healthy', healthy: 92, warning: 4, error: 0 },
      apis: ['系统配置(24个)', '业务配置(25个)', '管理工具(23个)', '监控告警(24个)']
    }
  ];

  // 现有模块
  const existingModules = [
    {
      name: '系统控制',
      path: '/system',
      icon: '🖥️',
      iconBg: 'bg-gray-500',
      apiCount: 15,
      description: '系统启停、状态监控、基础控制功能',
      status: { overall: 'healthy', healthy: 15, warning: 0, error: 0 }
    },
    {
      name: 'QingXi数据',
      path: '/qingxi',
      icon: '🔄',
      iconBg: 'bg-cyan-500',
      apiCount: 12,
      description: '数据清洗、处理、同步功能',
      status: { overall: 'healthy', healthy: 12, warning: 0, error: 0 }
    },
    {
      name: 'CeLue策略',
      path: '/celue',
      icon: '📈',
      iconBg: 'bg-green-600',
      apiCount: 18,
      description: '策略执行、回测、优化功能',
      status: { overall: 'healthy', healthy: 18, warning: 0, error: 0 }
    },
    {
      name: '风险管理',
      path: '/risk',
      icon: '🛡️',
      iconBg: 'bg-red-600',
      apiCount: 10,
      description: '风险控制、监控、告警功能',
      status: { overall: 'healthy', healthy: 10, warning: 0, error: 0 }
    },
    {
      name: '架构监控',
      path: '/architecture',
      icon: '🏗️',
      iconBg: 'bg-orange-500',
      apiCount: 8,
      description: '架构健康、性能、稳定性监控',
      status: { overall: 'healthy', healthy: 8, warning: 0, error: 0 }
    },
    {
      name: '可观测性',
      path: '/observability',
      icon: '👁️',
      iconBg: 'bg-teal-500',
      apiCount: 14,
      description: '分布式追踪、指标收集、可视化',
      status: { overall: 'healthy', healthy: 14, warning: 0, error: 0 }
    }
  ];

  // 计算总体统计
  const calculateStats = () => {
    const allModules = [...apiModules, ...existingModules];
    const totalApis = allModules.reduce((sum, module) => sum + module.apiCount, 0);
    const healthyApis = allModules.reduce((sum, module) => sum + module.status.healthy, 0);
    const warningApis = allModules.reduce((sum, module) => sum + module.status.warning, 0);
    const errorApis = allModules.reduce((sum, module) => sum + module.status.error, 0);

    return { totalApis, healthyApis, warningApis, errorApis };
  };

  const stats = calculateStats();

  // 模拟API健康检查
  const { data: healthData, isLoading: healthLoading } = useQuery({
    queryKey: ['system-health'],
    queryFn: async () => {
      // 这里可以调用实际的健康检查API
      // return unifiedApi.apiClient.get('/health');
      return { success: true, timestamp: Date.now() };
    },
    refetchInterval: 30000, // 30秒刷新一次
  });

  return (
    <div className="min-h-screen bg-gray-50">
      {/* 页面头部 */}
      <div className="bg-white shadow-sm border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-6">
            <div>
              <h1 className="text-3xl font-bold text-gray-900">5.1套利系统控制台</h1>
              <p className="mt-1 text-sm text-gray-600">
                387个API接口统一管理平台 - 专业量化交易系统
              </p>
            </div>
            <div className="flex items-center space-x-4">
              <div className="flex items-center space-x-2">
                <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                <span className="text-sm text-gray-600">系统运行中</span>
              </div>
              <div className="text-sm text-gray-500">
                最后更新: {new Date().toLocaleTimeString()}
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* 系统总览统计 */}
        <SystemOverview stats={stats} isLoading={healthLoading} />

        {/* 387个新增API模块 */}
        <div className="mb-12">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h2 className="text-2xl font-bold text-gray-900">新增API模块</h2>
              <p className="text-sm text-gray-600 mt-1">387个专业API接口，7个功能模块</p>
            </div>
            <div className="flex items-center space-x-2">
              <div className="w-3 h-3 bg-blue-500 rounded-full"></div>
              <span className="text-sm font-medium text-gray-700">新增功能</span>
            </div>
          </div>
          
          <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
            {apiModules.map((module, index) => (
              <ApiModuleCard 
                key={index} 
                module={module} 
                isLoading={healthLoading}
              />
            ))}
          </div>
        </div>

        {/* 现有模块 */}
        <div className="mb-12">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h2 className="text-2xl font-bold text-gray-900">现有系统模块</h2>
              <p className="text-sm text-gray-600 mt-1">77个基础API接口，6个核心模块</p>
            </div>
            <div className="flex items-center space-x-2">
              <div className="w-3 h-3 bg-green-500 rounded-full"></div>
              <span className="text-sm font-medium text-gray-700">运行稳定</span>
            </div>
          </div>
          
          <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
            {existingModules.map((module, index) => (
              <ApiModuleCard 
                key={index} 
                module={module} 
                isLoading={healthLoading}
              />
            ))}
          </div>
        </div>

        {/* 快速操作面板 */}
        <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">快速操作</h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <button className="flex items-center justify-center space-x-2 p-3 bg-blue-50 hover:bg-blue-100 rounded-lg transition-colors">
              <svg className="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              <span className="text-sm font-medium text-blue-600">刷新状态</span>
            </button>
            
            <Link to="/logging" className="flex items-center justify-center space-x-2 p-3 bg-green-50 hover:bg-green-100 rounded-lg transition-colors">
              <svg className="w-5 h-5 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v4a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
              </svg>
              <span className="text-sm font-medium text-green-600">查看日志</span>
            </Link>
            
            <Link to="/performance" className="flex items-center justify-center space-x-2 p-3 bg-yellow-50 hover:bg-yellow-100 rounded-lg transition-colors">
              <svg className="w-5 h-5 text-yellow-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
              <span className="text-sm font-medium text-yellow-600">性能调优</span>
            </Link>
            
            <Link to="/config" className="flex items-center justify-center space-x-2 p-3 bg-gray-50 hover:bg-gray-100 rounded-lg transition-colors">
              <svg className="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
              <span className="text-sm font-medium text-gray-600">系统配置</span>
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Dashboard;