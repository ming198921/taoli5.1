// 统一系统控制面板
// 支持多种部署环境的模块控制

import React, { useState, useEffect } from 'react';
import { Card, Row, Col, Button, Badge, message, Tabs, Alert, Typography } from 'antd';
import { 
  PlayCircleOutlined, 
  PauseCircleOutlined, 
  ReloadOutlined,
  SettingOutlined,
  CodeOutlined,
  InfoCircleOutlined
} from '@ant-design/icons';
import systemControl, { 
  SystemModule, 
  ControlResponse, 
  DeploymentType 
} from '@/services/systemControl';

const { Title, Text } = Typography;
const { TabPane } = Tabs;

interface ModuleConfig {
  name: string;
  displayName: string;
  description: string;
  color: string;
  icon: React.ReactNode;
}

const SYSTEM_MODULES: ModuleConfig[] = [
  {
    name: 'system',
    displayName: '核心系统',
    description: '主要的套利系统核心',
    color: '#1890ff',
    icon: <PlayCircleOutlined />
  },
  {
    name: 'qingxi',
    displayName: '数据处理',
    description: '市场数据收集和清洗',
    color: '#52c41a',
    icon: <CodeOutlined />
  },
  {
    name: 'celue',
    displayName: '策略执行',
    description: 'AI策略引擎',
    color: '#fa8c16',
    icon: <SettingOutlined />
  },
  {
    name: 'risk',
    displayName: '风险控制',
    description: '动态风险管理',
    color: '#f5222d',
    icon: <InfoCircleOutlined />
  }
];

export const SystemControlPanel: React.FC = () => {
  const [modules, setModules] = useState<Map<string, SystemModule>>(new Map());
  const [loading, setLoading] = useState<Map<string, boolean>>(new Map());
  const [refreshing, setRefreshing] = useState(false);
  const [logs, setLogs] = useState<Map<string, string[]>>(new Map());
  const [deploymentType, setDeploymentType] = useState<DeploymentType>(DeploymentType.DIRECT);

  // 初始化
  useEffect(() => {
    setDeploymentType(systemControl.getDeploymentType());
    refreshAllModules();
    
    // 定期刷新状态
    const interval = setInterval(refreshAllModules, 10000);
    return () => clearInterval(interval);
  }, []);

  // 刷新所有模块状态
  const refreshAllModules = async () => {
    setRefreshing(true);
    try {
      const moduleNames = SYSTEM_MODULES.map(m => m.name);
      const statuses = await systemControl.getAllModuleStatuses(moduleNames);
      
      const newModules = new Map<string, SystemModule>();
      statuses.forEach(status => {
        newModules.set(status.name, status);
      });
      
      setModules(newModules);
    } catch (error) {
      message.error('刷新模块状态失败');
      console.error('Refresh failed:', error);
    } finally {
      setRefreshing(false);
    }
  };

  // 控制模块
  const controlModule = async (
    moduleName: string, 
    action: 'start' | 'stop' | 'restart'
  ) => {
    setLoading(prev => new Map(prev.set(moduleName, true)));
    
    try {
      let result: ControlResponse;
      
      switch (action) {
        case 'start':
          result = await systemControl.startModule(moduleName);
          break;
        case 'stop':
          result = await systemControl.stopModule(moduleName);
          break;
        case 'restart':
          result = await systemControl.restartModule(moduleName);
          break;
      }
      
      if (result.success) {
        message.success(result.message);
        // 延迟刷新状态以等待操作完成
        setTimeout(refreshAllModules, 2000);
      } else {
        message.error(result.message);
      }
    } catch (error) {
      message.error(`${action} 操作失败`);
      console.error(`${action} failed:`, error);
    } finally {
      setLoading(prev => new Map(prev.set(moduleName, false)));
    }
  };

  // 获取日志
  const fetchLogs = async (moduleName: string) => {
    try {
      const logLines = await systemControl.getModuleLogs(moduleName, 50);
      setLogs(prev => new Map(prev.set(moduleName, logLines)));
    } catch (error) {
      message.error('获取日志失败');
      console.error('Fetch logs failed:', error);
    }
  };

  // 获取状态颜色
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running': return 'success';
      case 'stopped': return 'default';
      case 'starting': return 'processing';
      case 'stopping': return 'processing';
      case 'error': return 'error';
      default: return 'default';
    }
  };

  // 获取健康状态颜色
  const getHealthColor = (health?: string) => {
    switch (health) {
      case 'healthy': return '#52c41a';
      case 'unhealthy': return '#f5222d';
      case 'unknown': return '#d9d9d9';
      default: return '#d9d9d9';
    }
  };

  // 模块卡片组件
  const ModuleCard: React.FC<{ config: ModuleConfig }> = ({ config }) => {
    const module = modules.get(config.name);
    const isLoading = loading.get(config.name) || false;
    const canStart = module?.status === 'stopped' || module?.status === 'error';
    const canStop = module?.status === 'running';

    return (
      <Card 
        title={
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              {config.icon}
              <span>{config.displayName}</span>
            </div>
            <div className="flex items-center space-x-2">
              <div 
                className="w-3 h-3 rounded-full"
                style={{ backgroundColor: getHealthColor(module?.health) }}
              />
              <Badge 
                status={getStatusColor(module?.status || 'error')} 
                text={module?.status || 'unknown'}
              />
            </div>
          </div>
        }
        size="small"
        className="h-full"
        loading={isLoading}
        extra={
          <Button 
            size="small" 
            icon={<ReloadOutlined />}
            onClick={() => fetchLogs(config.name)}
          >
            日志
          </Button>
        }
      >
        <div className="space-y-3">
          <Text type="secondary" className="text-sm">
            {config.description}
          </Text>
          
          {/* 指标 */}
          {module?.metrics && (
            <div className="grid grid-cols-3 gap-2 text-xs">
              <div className="text-center">
                <div className="font-medium">{module.metrics.cpu.toFixed(1)}%</div>
                <div className="text-gray-500">CPU</div>
              </div>
              <div className="text-center">
                <div className="font-medium">{module.metrics.memory.toFixed(1)}%</div>
                <div className="text-gray-500">内存</div>
              </div>
              <div className="text-center">
                <div className="font-medium">{module.metrics.requests}</div>
                <div className="text-gray-500">请求</div>
              </div>
            </div>
          )}
          
          {/* 控制按钮 */}
          <div className="flex space-x-2">
            <Button
              type="primary"
              size="small"
              icon={<PlayCircleOutlined />}
              disabled={!canStart || isLoading}
              onClick={() => controlModule(config.name, 'start')}
              loading={isLoading}
            >
              启动
            </Button>
            <Button
              danger
              size="small"
              icon={<PauseCircleOutlined />}
              disabled={!canStop || isLoading}
              onClick={() => controlModule(config.name, 'stop')}
              loading={isLoading}
            >
              停止
            </Button>
            <Button
              size="small"
              icon={<ReloadOutlined />}
              disabled={isLoading}
              onClick={() => controlModule(config.name, 'restart')}
              loading={isLoading}
            >
              重启
            </Button>
          </div>
          
          {/* 最后心跳时间 */}
          {module?.lastHeartbeat && (
            <Text type="secondary" className="text-xs">
              最后活动: {new Date(module.lastHeartbeat).toLocaleTimeString()}
            </Text>
          )}
        </div>
      </Card>
    );
  };

  // 批量操作
  const handleBatchOperation = async (action: 'start' | 'stop') => {
    const moduleNames = SYSTEM_MODULES.map(m => m.name);
    
    try {
      if (action === 'start') {
        await systemControl.startAllModules(moduleNames);
        message.success('批量启动完成');
      } else {
        await systemControl.stopAllModules(moduleNames);
        message.success('批量停止完成');
      }
      
      setTimeout(refreshAllModules, 3000);
    } catch (error) {
      message.error(`批量${action === 'start' ? '启动' : '停止'}失败`);
    }
  };

  return (
    <div className="p-6">
      {/* 标题和全局控制 */}
      <div className="mb-6">
        <div className="flex items-center justify-between">
          <div>
            <Title level={2} className="mb-1">
              系统控制面板
            </Title>
            <Text type="secondary">
              当前部署环境: {deploymentType} | 统一控制所有系统模块
            </Text>
          </div>
          
          <div className="flex items-center space-x-3">
            <Button
              icon={<ReloadOutlined />}
              loading={refreshing}
              onClick={refreshAllModules}
            >
              刷新状态
            </Button>
            
            <Button
              type="primary"
              icon={<PlayCircleOutlined />}
              onClick={() => handleBatchOperation('start')}
            >
              全部启动
            </Button>
            
            <Button
              danger
              icon={<PauseCircleOutlined />}
              onClick={() => handleBatchOperation('stop')}
            >
              全部停止
            </Button>
          </div>
        </div>
      </div>

      {/* 部署环境提示 */}
      {deploymentType !== DeploymentType.DIRECT && (
        <Alert
          message={`当前使用 ${deploymentType.toUpperCase()} 部署模式`}
          description="系统将通过对应的容器编排平台进行控制。某些功能可能需要额外的权限配置。"
          type="info"
          showIcon
          className="mb-4"
        />
      )}

      {/* 模块状态卡片 */}
      <Row gutter={[16, 16]} className="mb-6">
        {SYSTEM_MODULES.map(config => (
          <Col xs={24} sm={12} lg={6} key={config.name}>
            <ModuleCard config={config} />
          </Col>
        ))}
      </Row>

      {/* 详细信息选项卡 */}
      <Card>
        <Tabs defaultActiveKey="overview">
          <TabPane tab="系统概览" key="overview">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
              {SYSTEM_MODULES.map(config => {
                const module = modules.get(config.name);
                return (
                  <Card key={config.name} size="small" title={config.displayName}>
                    <div className="space-y-2">
                      <div className="flex justify-between">
                        <span>状态:</span>
                        <Badge 
                          status={getStatusColor(module?.status || 'error')} 
                          text={module?.status || 'unknown'}
                        />
                      </div>
                      <div className="flex justify-between">
                        <span>健康:</span>
                        <span style={{ color: getHealthColor(module?.health) }}>
                          {module?.health || 'unknown'}
                        </span>
                      </div>
                      {module?.metrics && (
                        <>
                          <div className="flex justify-between">
                            <span>CPU:</span>
                            <span>{module.metrics.cpu.toFixed(1)}%</span>
                          </div>
                          <div className="flex justify-between">
                            <span>内存:</span>
                            <span>{module.metrics.memory.toFixed(1)}%</span>
                          </div>
                        </>
                      )}
                    </div>
                  </Card>
                );
              })}
            </div>
          </TabPane>
          
          <TabPane tab="系统日志" key="logs">
            <div className="space-y-4">
              {SYSTEM_MODULES.map(config => {
                const moduleLogs = logs.get(config.name);
                return (
                  <Card 
                    key={config.name} 
                    title={`${config.displayName} 日志`}
                    size="small"
                    extra={
                      <Button 
                        size="small"
                        onClick={() => fetchLogs(config.name)}
                      >
                        刷新
                      </Button>
                    }
                  >
                    <div style={{ 
                      height: '200px', 
                      overflowY: 'auto', 
                      backgroundColor: '#001529', 
                      padding: '12px', 
                      borderRadius: '6px',
                      fontFamily: 'Monaco, Consolas, "Lucida Console", monospace'
                    }}>
                      {moduleLogs && moduleLogs.length > 0 ? (
                        moduleLogs.map((log, index) => (
                          <div key={index} style={{ 
                            color: log.includes('ERROR') || log.includes('❌') ? '#ff4d4f' :
                                   log.includes('WARN') || log.includes('⚠️') ? '#faad14' :
                                   log.includes('INFO') || log.includes('✅') ? '#52c41a' : '#d9d9d9',
                            fontSize: '12px',
                            lineHeight: '1.4',
                            marginBottom: '2px',
                            whiteSpace: 'pre-wrap'
                          }}>
                            {log}
                          </div>
                        ))
                      ) : (
                        <div style={{ color: '#d9d9d9', textAlign: 'center', marginTop: '50px' }}>
                          暂无日志数据...
                          <br />
                          <small>点击"刷新"按钮获取最新日志</small>
                        </div>
                      )}
                    </div>
                  </Card>
                );
              })}
            </div>
          </TabPane>
          
          <TabPane tab="配置管理" key="config">
            <Alert
              message="配置管理"
              description={`当前部署模式: ${deploymentType}。配置更新方式根据部署环境自动适配。`}
              type="info"
              showIcon
            />
            <div className="mt-4">
              <Text>配置管理功能正在开发中...</Text>
            </div>
          </TabPane>
        </Tabs>
      </Card>
    </div>
  );
};

export default SystemControlPanel;