/**
 * 架构监控模块 - 5.1套利系统服务架构监控
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
  formatUptime, 
  formatPercent, 
  formatLatency,
  formatBytes,
  getStatusColor,
  cn
} from '../../utils/helpers.js';

// 服务状态卡片组件
const ServiceStatusCard = ({ servicesData }) => {
  const services = servicesData || [];
  
  const getServiceIcon = (serviceType) => {
    const icons = {
      'api': '🚀',
      'database': '🗄️',
      'redis': '💾',
      'message-queue': '📨',
      'monitor': '📊',
      'web': '🌐'
    };
    return icons[serviceType] || '⚙️';
  };

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">服务状态监控</h3>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {services.map((service) => {
          const statusColor = getStatusColor(service.status === 'healthy' ? 'running' : 'error');
          
          return (
            <div key={service.name} className="border border-gray-200 rounded-lg p-4">
              <div className="flex items-center gap-3 mb-3">
                <span className="text-2xl">{getServiceIcon(service.type)}</span>
                <div>
                  <h4 className="font-medium text-gray-900">{service.name}</h4>
                  <p className="text-sm text-gray-500">{service.description}</p>
                </div>
              </div>
              
              <div className="space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-sm text-gray-600">状态</span>
                  <div className="flex items-center gap-2">
                    <div 
                      className="w-2 h-2 rounded-full"
                      style={{ backgroundColor: statusColor }}
                    />
                    <span 
                      className="text-sm font-medium"
                      style={{ color: statusColor }}
                    >
                      {service.status === 'healthy' ? '健康' : 
                       service.status === 'warning' ? '警告' : '异常'}
                    </span>
                  </div>
                </div>
                
                <div className="flex justify-between items-center">
                  <span className="text-sm text-gray-600">响应时间</span>
                  <span className="text-sm font-mono text-gray-900">
                    {formatLatency(service.response_time)}
                  </span>
                </div>
                
                <div className="flex justify-between items-center">
                  <span className="text-sm text-gray-600">运行时间</span>
                  <span className="text-sm font-mono text-gray-900">
                    {formatUptime(service.uptime)}
                  </span>
                </div>
                
                {service.version && (
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-gray-600">版本</span>
                    <span className="text-sm font-mono text-gray-900">
                      {service.version}
                    </span>
                  </div>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

// 组件健康检查卡片
const HealthCheckCard = ({ healthData }) => {
  const checks = healthData?.checks || [];
  const overallStatus = healthData?.status || 'unknown';
  
  const statusColor = getStatusColor(
    overallStatus === 'healthy' ? 'running' : 
    overallStatus === 'warning' ? 'warning' : 'error'
  );

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900">健康检查</h3>
        <div className="flex items-center gap-2">
          <div 
            className="w-3 h-3 rounded-full"
            style={{ backgroundColor: statusColor }}
          />
          <span 
            className="text-sm font-medium"
            style={{ color: statusColor }}
          >
            {overallStatus === 'healthy' ? '全部正常' : 
             overallStatus === 'warning' ? '部分异常' : '系统异常'}
          </span>
        </div>
      </div>
      
      <div className="space-y-3">
        {checks.map((check) => {
          const checkStatusColor = getStatusColor(
            check.status === 'pass' ? 'running' : 'error'
          );
          
          return (
            <div key={check.name} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
              <div className="flex items-center gap-3">
                <div 
                  className="w-2 h-2 rounded-full"
                  style={{ backgroundColor: checkStatusColor }}
                />
                <div>
                  <p className="font-medium text-gray-900">{check.name}</p>
                  <p className="text-sm text-gray-500">{check.description}</p>
                </div>
              </div>
              
              <div className="text-right">
                <p 
                  className="text-sm font-medium"
                  style={{ color: checkStatusColor }}
                >
                  {check.status === 'pass' ? '通过' : '失败'}
                </p>
                <p className="text-xs text-gray-500">
                  {formatTime(check.last_check)}
                </p>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

// 性能指标卡片
const PerformanceMetricsCard = ({ metricsData }) => {
  const metrics = metricsData || {};
  
  const performanceItems = [
    {
      label: 'CPU使用率',
      value: formatPercent(metrics.cpu_usage || 0),
      progress: metrics.cpu_usage || 0,
      color: (metrics.cpu_usage || 0) > 80 ? '#ff4d4f' : 
             (metrics.cpu_usage || 0) > 60 ? '#faad14' : '#52c41a'
    },
    {
      label: '内存使用率',
      value: formatPercent(metrics.memory_usage || 0),
      progress: metrics.memory_usage || 0,
      color: (metrics.memory_usage || 0) > 80 ? '#ff4d4f' : 
             (metrics.memory_usage || 0) > 60 ? '#faad14' : '#52c41a'
    },
    {
      label: '磁盘使用率',
      value: formatPercent(metrics.disk_usage || 0),
      progress: metrics.disk_usage || 0,
      color: (metrics.disk_usage || 0) > 80 ? '#ff4d4f' : 
             (metrics.disk_usage || 0) > 60 ? '#faad14' : '#52c41a'
    },
    {
      label: '网络吞吐量',
      value: formatBytes(metrics.network_throughput || 0) + '/s',
      progress: Math.min((metrics.network_throughput || 0) / 1024 / 1024 * 10, 100),
      color: '#1890ff'
    }
  ];

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">性能指标</h3>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {performanceItems.map((item) => (
          <div key={item.label}>
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-gray-700">
                {item.label}
              </span>
              <span 
                className="text-sm font-semibold"
                style={{ color: item.color }}
              >
                {item.value}
              </span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div
                className="h-2 rounded-full transition-all duration-300"
                style={{
                  width: `${Math.min(item.progress, 100)}%`,
                  backgroundColor: item.color,
                }}
              />
            </div>
          </div>
        ))}
      </div>
      
      <div className="mt-6 grid grid-cols-2 md:grid-cols-4 gap-4 text-center">
        <div className="p-3 bg-blue-50 rounded-lg">
          <p className="text-2xl font-bold text-blue-600">
            {metrics.active_connections || 0}
          </p>
          <p className="text-sm text-blue-600">活跃连接</p>
        </div>
        <div className="p-3 bg-green-50 rounded-lg">
          <p className="text-2xl font-bold text-green-600">
            {formatLatency(metrics.avg_response_time || 0)}
          </p>
          <p className="text-sm text-green-600">平均响应</p>
        </div>
        <div className="p-3 bg-yellow-50 rounded-lg">
          <p className="text-2xl font-bold text-yellow-600">
            {metrics.requests_per_minute || 0}
          </p>
          <p className="text-sm text-yellow-600">每分钟请求</p>
        </div>
        <div className="p-3 bg-purple-50 rounded-lg">
          <p className="text-2xl font-bold text-purple-600">
            {formatPercent(metrics.success_rate || 0)}
          </p>
          <p className="text-sm text-purple-600">成功率</p>
        </div>
      </div>
    </div>
  );
};

// 系统拓扑图卡片
const SystemTopologyCard = ({ topologyData }) => {
  const components = topologyData?.components || [];
  const connections = topologyData?.connections || [];

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">系统拓扑图</h3>
      
      <div className="relative bg-gray-50 rounded-lg p-6 h-80 overflow-hidden">
        <svg className="w-full h-full" viewBox="0 0 800 300">
          {/* 绘制连接线 */}
          {connections.map((conn, index) => (
            <line
              key={index}
              x1={conn.from.x}
              y1={conn.from.y}
              x2={conn.to.x}
              y2={conn.to.y}
              stroke="#d1d5db"
              strokeWidth="2"
              strokeDasharray={conn.type === 'async' ? '5,5' : ''}
            />
          ))}
          
          {/* 绘制组件节点 */}
          {components.map((component) => {
            const statusColor = getStatusColor(
              component.status === 'healthy' ? 'running' : 'error'
            );
            
            return (
              <g key={component.id}>
                <rect
                  x={component.x - 60}
                  y={component.y - 25}
                  width="120"
                  height="50"
                  rx="8"
                  fill="white"
                  stroke={statusColor}
                  strokeWidth="2"
                />
                <text
                  x={component.x}
                  y={component.y - 5}
                  textAnchor="middle"
                  className="text-xs font-medium"
                  fill="#374151"
                >
                  {component.name}
                </text>
                <text
                  x={component.x}
                  y={component.y + 10}
                  textAnchor="middle"
                  className="text-xs"
                  fill="#6b7280"
                >
                  {component.type}
                </text>
                <circle
                  cx={component.x + 45}
                  cy={component.y - 15}
                  r="3"
                  fill={statusColor}
                />
              </g>
            );
          })}
        </svg>
        
        <div className="absolute bottom-4 right-4 bg-white border border-gray-200 rounded-lg p-3">
          <p className="text-xs text-gray-600 mb-2">图例</p>
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <div className="w-2 h-2 rounded-full bg-green-500"></div>
              <span className="text-xs text-gray-600">健康</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-2 h-2 rounded-full bg-red-500"></div>
              <span className="text-xs text-gray-600">异常</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-4 h-0.5 bg-gray-400"></div>
              <span className="text-xs text-gray-600">同步连接</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-4 h-0.5 bg-gray-400" style={{ backgroundImage: 'repeating-linear-gradient(to right, transparent, transparent 2px, #9ca3af 2px, #9ca3af 4px)' }}></div>
              <span className="text-xs text-gray-600">异步连接</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

// 依赖关系卡片
const DependenciesCard = ({ dependenciesData }) => {
  const dependencies = dependenciesData || [];

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">依赖关系监控</h3>
      
      <div className="space-y-3">
        {dependencies.map((dep) => {
          const statusColor = getStatusColor(
            dep.status === 'available' ? 'running' : 'error'
          );
          
          return (
            <div key={dep.name} className="flex items-center justify-between p-4 border border-gray-200 rounded-lg">
              <div className="flex items-center gap-3">
                <div 
                  className="w-3 h-3 rounded-full"
                  style={{ backgroundColor: statusColor }}
                />
                <div>
                  <p className="font-medium text-gray-900">{dep.name}</p>
                  <p className="text-sm text-gray-500">{dep.type} • {dep.version}</p>
                </div>
              </div>
              
              <div className="text-right">
                <p 
                  className="text-sm font-medium"
                  style={{ color: statusColor }}
                >
                  {dep.status === 'available' ? '可用' : '不可用'}
                </p>
                <p className="text-xs text-gray-500">
                  延迟: {formatLatency(dep.latency)}
                </p>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

// 主架构监控组件
const ArchitectureModule = () => {
  const queryClient = useQueryClient();
  const [notification, setNotification] = useState(null);

  // 获取服务状态数据
  const { data: servicesData, isLoading: servicesLoading } = useQuery({
    queryKey: ['architectureServices'],
    queryFn: () => apiService.architecture.getServices(),
    refetchInterval: REFRESH_INTERVALS.NORMAL,
    select: (response) => response.data?.services || []
  });

  // 获取健康检查数据
  const { data: healthData, isLoading: healthLoading } = useQuery({
    queryKey: ['architectureHealth'],
    queryFn: () => apiService.architecture.getHealthCheck(),
    refetchInterval: REFRESH_INTERVALS.NORMAL,
    select: (response) => response.data || {}
  });

  // 获取性能指标数据
  const { data: metricsData, isLoading: metricsLoading } = useQuery({
    queryKey: ['architectureMetrics'],
    queryFn: () => apiService.architecture.getMetrics(),
    refetchInterval: REFRESH_INTERVALS.FAST,
    select: (response) => response.data?.metrics || {}
  });

  // 获取系统拓扑数据
  const { data: topologyData, isLoading: topologyLoading } = useQuery({
    queryKey: ['architectureTopology'],
    queryFn: () => apiService.architecture.getTopology(),
    refetchInterval: REFRESH_INTERVALS.SLOW,
    select: (response) => response.data || {}
  });

  // 获取依赖关系数据
  const { data: dependenciesData, isLoading: dependenciesLoading } = useQuery({
    queryKey: ['architectureDependencies'],
    queryFn: () => apiService.architecture.getDependencies(),
    refetchInterval: REFRESH_INTERVALS.NORMAL,
    select: (response) => response.data?.dependencies || []
  });

  // 显示通知
  const showNotification = (message, type = 'info') => {
    setNotification({ message, type });
    setTimeout(() => setNotification(null), 4000);
  };

  // 重启服务
  const restartServiceMutation = useMutation({
    mutationFn: (serviceName) => apiService.architecture.restartService(serviceName),
    onSuccess: (response, serviceName) => {
      queryClient.invalidateQueries(['architectureServices']);
      queryClient.invalidateQueries(['architectureHealth']);
      showNotification(`服务 ${serviceName} 重启成功`, 'success');
    },
    onError: (error, serviceName) => {
      showNotification(`服务 ${serviceName} 重启失败: ${error.message}`, 'error');
    }
  });

  const isLoading = servicesLoading || healthLoading || metricsLoading || topologyLoading || dependenciesLoading;


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
          <h1 className="text-3xl font-bold text-gray-900">架构监控</h1>
          <p className="text-gray-600 mt-1">
            5.1套利系统架构监控 - 实时监控服务状态、性能指标和系统拓扑
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
            {/* 服务状态监控 */}
            <ServiceStatusCard servicesData={servicesData} />
            
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              {/* 健康检查 */}
              <HealthCheckCard healthData={healthData} />
              
              {/* 依赖关系监控 */}
              <DependenciesCard dependenciesData={dependenciesData} />
            </div>
            
            {/* 性能指标 */}
            <PerformanceMetricsCard metricsData={metricsData} />
            
            {/* 系统拓扑图 */}
            <SystemTopologyCard topologyData={topologyData} />
          </div>
        )}
        
        {/* 页面底部信息 */}
        <div className="mt-8 text-center text-sm text-gray-500">
          最后更新: {formatTime(new Date())} | 
          自动刷新间隔: {REFRESH_INTERVALS.NORMAL / 1000}秒
        </div>
      </div>
    </div>
  );
};

export default ArchitectureModule;