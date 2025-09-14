import React, { useEffect, useState } from 'react';
import { Row, Col, Card, Statistic, Alert, Tabs, Space, Button, Badge, Typography, Progress, message } from 'antd';
import {
  DashboardOutlined,
  ApiOutlined,
  DatabaseOutlined,
  MonitorOutlined,
  SafetyOutlined,
  ArrowUpOutlined,
  ArrowDownOutlined,
  ReloadOutlined,
  WarningOutlined,
  CheckCircleOutlined,
  ThunderboltOutlined,
  PlayCircleOutlined,
  PauseCircleOutlined,
  ControlOutlined,
} from '@ant-design/icons';
import axios from 'axios';
import { useNavigate } from 'react-router-dom';
import { useAppSelector, useAppDispatch } from '@/store/hooks';
import { checkSystemHealth } from '@/store/slices/appSlice';
import wsManager from '@/api/websocket';
import SystemControlPanel from '@/components/SystemControlPanel';

const { Title, Text } = Typography;
const { TabPane } = Tabs;

export const DashboardPage: React.FC = () => {
  const dispatch = useAppDispatch();
  const navigate = useNavigate();
  const [loading, setLoading] = useState(false);
  const [realtimeMarketData, setRealtimeMarketData] = useState<any>(null);
  const [performanceData, setPerformanceData] = useState<any>(null);
  const [wsConnected, setWsConnected] = useState(false);
  const [riskStatus, setRiskStatus] = useState<any>(null);
  const [systemStatus, setSystemStatus] = useState<any>(null);
  const [dataCollectors, setDataCollectors] = useState<any[]>([]);
  const [backendStatus, setBackendStatus] = useState<'running' | 'stopped' | 'starting' | 'stopping' | 'unknown'>('unknown');
  const [systemLogs, setSystemLogs] = useState<string[]>([]);
  const [backendLoading, setBackendLoading] = useState(false);
  
  // 获取各模块状态 - 使用真实的后端数据
  const { systemHealth, connectionStatus, performance } = useAppSelector(state => state.app);
  
  // 暂时使用基于连接状态的模拟数据，直到有真实的模块状态API
  const getModuleStatus = (connectionStatus: string) => {
    return connectionStatus === 'connected' ? 'healthy' : 'error';
  };
  
  const qingxiStatus = getModuleStatus(connectionStatus.api);
  const celueStatus = getModuleStatus(connectionStatus.api);
  const architectureStatus = getModuleStatus(connectionStatus.api);
  const observabilityStatus = getModuleStatus(connectionStatus.api);
  
  // 获取告警信息 - 使用安全的默认值
  const qingxiAlerts = useAppSelector(state => state.qingxi?.activeAlerts || []);
  const celueAlerts = useAppSelector(state => state.celue?.activeAlerts || []);
  const architectureAlerts = useAppSelector(state => state.architecture?.activeAlerts || []);
  
  const totalAlerts = qingxiAlerts.length + celueAlerts.length + architectureAlerts.length;
  const criticalAlerts = [...qingxiAlerts, ...celueAlerts, ...architectureAlerts]
    .filter(alert => alert.level === 'critical').length;

  // 刷新系统健康状态
  const handleRefresh = async () => {
    setLoading(true);
    try {
      await dispatch(checkSystemHealth());
    } finally {
      setLoading(false);
    }
  };

  // 获取真实数据的函数
  const fetchRealData = async () => {
    try {
      // 检查后端状态
      const healthCheck = await axios.get('http://localhost:8080/api/health', { timeout: 3000 });
      if (healthCheck.status === 200) {
        setBackendStatus('running');
      }

      // 获取风险状态
      const riskResponse = await axios.get('http://localhost:8080/api/risk/status');
      if (riskResponse.data.success) {
        setRiskStatus(riskResponse.data.data);
      }

      // 获取系统状态  
      const systemResponse = await axios.get('http://localhost:8080/api/system/status');
      if (systemResponse.data.success) {
        setSystemStatus(systemResponse.data.data);
      }

      // 获取数据收集器状态
      const collectorsResponse = await axios.get('http://localhost:8080/api/qingxi/collectors/list');
      if (collectorsResponse.data) {
        setDataCollectors(collectorsResponse.data);
      }

      // 获取系统日志
      const logsResponse = await axios.get('http://localhost:8080/api/system/logs');
      if (logsResponse.data && logsResponse.data.logs) {
        setSystemLogs(logsResponse.data.logs.slice(-50)); // 只保留最近50条日志
      }

    } catch (error) {
      console.error('获取真实数据失败:', error);
      setBackendStatus('stopped');
    }
  };

  // 启动后端系统 (实际上只是检查状态，因为后端独立运行)
  const startBackend = async () => {
    setBackendLoading(true);
    setBackendStatus('starting');
    try {
      // 首先检查后端是否已经在运行
      try {
        const healthCheck = await axios.get('http://localhost:8080/health', { timeout: 3000 });
        if (healthCheck.status === 200) {
          setBackendStatus('running');
          message.success('后端系统检测为已在运行状态');
          await fetchRealData();
          return;
        }
      } catch {
        // 后端未运行，继续启动流程
      }

      // 调用后端启动API
      const response = await axios.post('http://localhost:8080/api/system/start', {}, { timeout: 15000 });
      
      if (response.data.success) {
        const data = response.data.data;
        if (data.status === 'started' || data.status === 'already_running') {
          setBackendStatus('running');
          message.success(data.message);
          // 等待系统完全启动
          setTimeout(async () => {
            await fetchRealData();
          }, 2000);
        } else if (data.status === 'failed') {
          setBackendStatus('stopped');
          message.error(`启动失败: ${data.message}`);
        }
      }
    } catch (error: any) {
      console.error('启动后端失败:', error);
      setBackendStatus('stopped');
      if (error.response?.data?.message) {
        message.error(`后端启动失败: ${error.response.data.message}`);
      } else {
        message.error('无法启动后端系统，请检查系统状态');
      }
    } finally {
      setBackendLoading(false);
    }
  };

  // 停止后端系统
  const stopBackend = async () => {
    setBackendLoading(true);
    setBackendStatus('stopping');
    
    try {
      // 调用后端停止API
      const response = await axios.post('http://localhost:8080/api/system/stop', {}, { timeout: 10000 });
      
      if (response.data.success) {
        const data = response.data.data;
        if (data.status === 'stopped' || data.status === 'not_running') {
          setBackendStatus('stopped');
          message.success(data.message);
        } else if (data.status === 'error') {
          message.error(`停止失败: ${data.message}`);
        }
      }
    } catch (error: any) {
      console.error('停止后端失败:', error);
      if (error.response?.data?.message) {
        message.error(`后端停止失败: ${error.response.data.message}`);
      } else {
        message.error('无法停止后端系统，请手动停止进程');
      }
    } finally {
      setBackendLoading(false);
    }
  };

  // 页面加载时检查系统健康状态并初始化WebSocket
  useEffect(() => {
    console.log('🚀 DashboardPage: 开始初始化');
    console.log('🔍 当前系统健康状态:', systemHealth);
    console.log('🔍 当前连接状态:', connectionStatus);
    
    handleRefresh();
    fetchRealData();
    
    // 初始化WebSocket连接
    const initWebSocket = async () => {
      try {
        await wsManager.connect();
        setWsConnected(true);
        console.log('✅ WebSocket连接成功');
        
        // 订阅实时市场数据
        wsManager.subscribeRealtimeMarketData((data) => {
          console.log('📡 收到实时市场数据:', data);
          setRealtimeMarketData(data);
        });
        
        // 订阅系统性能数据
        wsManager.subscribePerformanceData((data) => {
          console.log('📊 收到系统性能数据:', data);
          setPerformanceData(data);
        });
        
      } catch (error) {
        console.error('❌ WebSocket连接失败:', error);
        setWsConnected(false);
      }
    };
    
    initWebSocket();
    
    // 每30秒自动刷新真实数据
    const interval = setInterval(() => {
      console.log('🔄 自动刷新系统健康状态和真实数据');
      dispatch(checkSystemHealth());
      fetchRealData();
    }, 30000);
    
    return () => {
      clearInterval(interval);
      // 断开WebSocket连接
      wsManager.disconnect();
    };
  }, [dispatch]);
  
  // 监听状态变化
  useEffect(() => {
    console.log('📊 系统健康状态更新:', systemHealth);
    console.log('📊 状态详细信息:', JSON.stringify(systemHealth, null, 2));
  }, [systemHealth]);
  
  useEffect(() => {
    console.log('🔌 连接状态更新:', connectionStatus);
    console.log('🔌 连接详细信息:', JSON.stringify(connectionStatus, null, 2));
  }, [connectionStatus]);
  
  useEffect(() => {
    console.log('⚡ 性能指标更新:', performance);
  }, [performance]);

  // 获取状态颜色
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'healthy':
        return '#52c41a';
      case 'warning':
        return '#faad14';
      case 'critical':
      case 'error':
        return '#ff4d4f';
      default:
        return '#d9d9d9';
    }
  };

  // 获取状态文本
  const getStatusText = (status: string) => {
    switch (status) {
      case 'healthy':
        return '健康';
      case 'warning':
        return '警告';
      case 'critical':
        return '严重';
      case 'error':
        return '错误';
      case 'maintenance':
        return '维护中';
      default:
        return '未知';
    }
  };

  // 模块状态卡片
  const ModuleStatusCard: React.FC<{
    title: string;
    status: string;
    icon: React.ReactNode;
    description: string;
    alerts: number;
    path?: string;
  }> = ({ title, status, icon, description, alerts, path }) => (
    <Card 
      size="small" 
      className="h-full cursor-pointer hover:shadow-md transition-shadow"
      onClick={() => path && navigate(path)}
    >
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center space-x-2">
          {icon}
          <Text strong>{title}</Text>
        </div>
        {alerts > 0 && (
          <Badge count={alerts} size="small" />
        )}
      </div>
      
      <div className="flex items-center space-x-2 mb-2">
        <div 
          className="w-3 h-3 rounded-full"
          style={{ backgroundColor: getStatusColor(status) }}
        />
        <Text className="text-sm">{getStatusText(status)}</Text>
      </div>
      
      <Text type="secondary" className="text-xs">
        {description}
      </Text>
      
      {path && (
        <div className="mt-2 pt-2 border-t border-gray-100">
          <Text type="secondary" className="text-xs">
            点击查看详情 →
          </Text>
        </div>
      )}
    </Card>
  );

  return (
    <div className="p-6">
      {/* 页头 */}
      <div className="mb-6">
        <div className="flex items-center justify-between">
          <div>
            <Title level={2} className="mb-1">
              <DashboardOutlined className="mr-2" />
              系统概览
            </Title>
            <Text type="secondary">
              5.1高频套利系统运行状态监控面板
            </Text>
          </div>
          
          <Space>
            <Button
              icon={<ReloadOutlined />}
              loading={loading}
              onClick={handleRefresh}
            >
              刷新状态
            </Button>
            
            {/* 后端控制按钮 */}
            {backendStatus === 'running' ? (
              <Button
                danger
                loading={backendLoading}
                onClick={stopBackend}
                icon={<PauseCircleOutlined />}
              >
                停止后端
              </Button>
            ) : (
              <Button
                type="primary"
                loading={backendLoading}
                onClick={startBackend}
                icon={<PlayCircleOutlined />}
              >
                启动后端
              </Button>
            )}
            
            <Badge 
              status={
                backendStatus === 'running' ? 'success' :
                backendStatus === 'starting' || backendStatus === 'stopping' ? 'processing' :
                backendStatus === 'stopped' ? 'error' : 'default'
              } 
              text={
                backendStatus === 'running' ? '后端运行中' :
                backendStatus === 'starting' ? '正在启动...' :
                backendStatus === 'stopping' ? '正在停止...' :
                backendStatus === 'stopped' ? '后端已停止' : '状态未知'
              }
            />
          </Space>
        </div>
      </div>

      {/* 告警信息 */}
      {criticalAlerts > 0 && (
        <Alert
          message="系统存在严重告警"
          description={`检测到 ${criticalAlerts} 个严重告警，请立即处理`}
          type="error"
          showIcon
          icon={<WarningOutlined />}
          className="mb-4"
          action={
            <Button size="small" danger>
              查看详情
            </Button>
          }
        />
      )}

      {totalAlerts > 0 && criticalAlerts === 0 && (
        <Alert
          message={`系统存在 ${totalAlerts} 个告警`}
          type="warning"
          showIcon
          className="mb-4"
          action={
            <Button size="small">
              查看详情
            </Button>
          }
        />
      )}

      {/* 系统总体状态 */}
      <Row gutter={[16, 16]} className="mb-6">
        <Col xs={24} sm={12} md={6}>
          <Card>
            <Statistic
              title="系统总体状态"
              value={getStatusText(systemHealth.overall)}
              valueStyle={{ color: getStatusColor(systemHealth.overall) }}
              prefix={
                systemHealth.overall === 'healthy' ? 
                  <CheckCircleOutlined /> : 
                  <WarningOutlined />
              }
            />
            {systemHealth.lastCheck && (
              <Text type="secondary" className="text-xs">
                最后检查: {new Date(systemHealth.lastCheck).toLocaleTimeString()}
              </Text>
            )}
          </Card>
        </Col>
        
        <Col xs={24} sm={12} md={6}>
          <Card>
            <Statistic
              title="活跃告警"
              value={totalAlerts}
              valueStyle={{ color: totalAlerts > 0 ? '#ff4d4f' : '#52c41a' }}
              prefix={<WarningOutlined />}
              suffix={criticalAlerts > 0 && `(${criticalAlerts}严重)`}
            />
          </Card>
        </Col>
        
        <Col xs={24} sm={12} md={6}>
          <Card>
            <Statistic
              title="系统运行时间"
              value={systemHealth.uptime || 0}
              valueStyle={{ color: '#1890ff' }}
              suffix="小时"
              prefix={<ArrowUpOutlined />}
            />
          </Card>
        </Col>
        
        <Col xs={24} sm={12} md={6}>
          <Card>
            <Statistic
              title="API连接状态"
              value={connectionStatus.api === 'connected' ? '已连接' : '未连接'}
              valueStyle={{ 
                color: connectionStatus.api === 'connected' ? '#52c41a' : '#ff4d4f' 
              }}
              prefix={<ApiOutlined />}
            />
            {connectionStatus.websocket !== 'disconnected' && (
              <Text type="secondary" className="text-xs">
                WebSocket: {connectionStatus.websocket === 'connected' ? '已连接' : '未连接'}
              </Text>
            )}
          </Card>
        </Col>
      </Row>

      {/* 模块状态 */}
      <Row gutter={[16, 16]} className="mb-6">
        <Col xs={24} lg={12} xl={6}>
          <ModuleStatusCard
            title="数据处理模块"
            status={qingxiStatus}
            icon={<DatabaseOutlined />}
            description="负责市场数据收集、处理和存储"
            alerts={qingxiAlerts.length}
            path="/qingxi/collector"
          />
        </Col>
        
        <Col xs={24} lg={12} xl={6}>
          <ModuleStatusCard
            title="策略执行模块"
            status={celueStatus}
            icon={<ApiOutlined />}
            description="AI/ML模型训练和策略执行"
            alerts={celueAlerts.length}
            path="/celue/strategies"
          />
        </Col>
        
        <Col xs={24} lg={12} xl={6}>
          <ModuleStatusCard
            title="系统架构模块"
            status={architectureStatus}
            icon={<SafetyOutlined />}
            description="系统限制、健康检查和故障恢复"
            alerts={architectureAlerts.length}
            path="/architecture/overview"
          />
        </Col>
        
        <Col xs={24} lg={12} xl={6}>
          <ModuleStatusCard
            title="可观测性模块"
            status={observabilityStatus}
            icon={<MonitorOutlined />}
            description="分布式追踪、指标收集和可视化"
            alerts={0}
            path="/observability/metrics"
          />
        </Col>
      </Row>

      {/* 性能指标 - 基于真实后端数据 */}
      <Row gutter={[16, 16]} className="mb-6">
        <Col xs={24} lg={8}>
          <Card title="CPU使用率" size="small">
            {systemStatus?.cpu_usage !== undefined ? (
              <Progress
                percent={Math.round(systemStatus.cpu_usage)}
                status={systemStatus.cpu_usage > 80 ? 'exception' : 'active'}
                strokeColor={{
                  '0%': '#87d068',
                  '50%': '#ffe58f',
                  '100%': '#ffccc7',
                }}
              />
            ) : (
              <div className="text-center py-4">
                <Text type="secondary">暂无CPU使用率数据</Text>
                <br />
                <Text type="secondary" className="text-xs">等待后端系统提供数据...</Text>
              </div>
            )}
          </Card>
        </Col>
        
        <Col xs={24} lg={8}>
          <Card title="内存使用率" size="small">
            {systemStatus?.memory_usage !== undefined ? (
              <Progress
                percent={Math.round(systemStatus.memory_usage)}
                status={systemStatus.memory_usage > 80 ? 'exception' : 'active'}
                strokeColor={{
                  '0%': '#87d068',
                  '50%': '#ffe58f',
                  '100%': '#ffccc7',
                }}
              />
            ) : (
              <div className="text-center py-4">
                <Text type="secondary">暂无内存使用率数据</Text>
                <br />
                <Text type="secondary" className="text-xs">等待后端系统提供数据...</Text>
              </div>
            )}
          </Card>
        </Col>
        
        <Col xs={24} lg={8}>
          <Card title="网络延迟" size="small">
            {systemStatus?.network_latency !== undefined ? (
              <Statistic
                value={systemStatus.network_latency}
                suffix="ms"
                precision={0}
                valueStyle={{
                  color: systemStatus.network_latency > 100 ? '#ff4d4f' : '#52c41a'
                }}
              />
            ) : (
              <div className="text-center py-4">
                <Text type="secondary">暂无网络延迟数据</Text>
                <br />
                <Text type="secondary" className="text-xs">等待后端系统提供数据...</Text>
              </div>
            )}
          </Card>
        </Col>
      </Row>

      {/* 详细信息选项卡 */}
      <Card>
        <Tabs defaultActiveKey="overview">
          <TabPane tab="系统概览" key="overview">
            <Row gutter={[16, 16]}>
              <Col xs={24} md={12}>
                <div className="space-y-2">
                  <Text strong>系统信息</Text>
                  <div className="text-sm text-gray-600">
                    <div>版本: {systemHealth.version || 'Unknown'}</div>
                    <div>运行时间: {systemHealth.uptime || 0} 小时</div>
                    <div>最后检查: {systemHealth.lastCheck ? 
                      new Date(systemHealth.lastCheck).toLocaleString() : 
                      'Never'
                    }</div>
                  </div>
                </div>
              </Col>
              
              <Col xs={24} md={12}>
                <div className="space-y-2">
                  <Text strong>连接状态</Text>
                  <div className="text-sm text-gray-600">
                    <div>API连接: {connectionStatus.api}</div>
                    <div>WebSocket连接: {connectionStatus.websocket}</div>
                  </div>
                </div>
              </Col>
            </Row>
          </TabPane>
          
          <TabPane tab="模块状态" key="modules">
            <Row gutter={[16, 16]}>
              <Col xs={24} md={8}>
                <Card title="数据收集器" size="small">
                  {dataCollectors.length > 0 ? (
                    <div className="space-y-2">
                      {dataCollectors.map((collector, index) => (
                        <div key={index} className="flex items-center justify-between">
                          <span>{collector.name || collector.id}</span>
                          <Badge 
                            status={collector.status === 'running' ? 'success' : 'error'} 
                            text={collector.status === 'running' ? '运行中' : '已停止'}
                          />
                        </div>
                      ))}
                    </div>
                  ) : (
                    <Text type="secondary">暂无数据收集器信息</Text>
                  )}
                </Card>
              </Col>
              
              <Col xs={24} md={8}>
                <Card title="系统状态" size="small">
                  {systemStatus ? (
                    <div className="space-y-2">
                      <div className="flex items-center justify-between">
                        <span>CPU使用率</span>
                        <Text>{systemStatus.cpu_usage || 0}%</Text>
                      </div>
                      <div className="flex items-center justify-between">
                        <span>内存使用率</span>
                        <Text>{systemStatus.memory_usage || 0}%</Text>
                      </div>
                      <div className="flex items-center justify-between">
                        <span>网络延迟</span>
                        <Text>{systemStatus.network_latency || 0}ms</Text>
                      </div>
                    </div>
                  ) : (
                    <Text type="secondary">暂无系统状态信息</Text>
                  )}
                </Card>
              </Col>
              
              <Col xs={24} md={8}>
                <Card title="风险状态" size="small">
                  {riskStatus ? (
                    <div className="space-y-2">
                      <div className="flex items-center justify-between">
                        <span>风险级别</span>
                        <Badge 
                          status={riskStatus.level === 'low' ? 'success' : riskStatus.level === 'medium' ? 'warning' : 'error'} 
                          text={riskStatus.level || '未知'}
                        />
                      </div>
                      <div className="flex items-center justify-between">
                        <span>资金安全</span>
                        <Text type={riskStatus.fund_safety ? 'success' : 'danger'}>
                          {riskStatus.fund_safety ? '安全' : '警告'}
                        </Text>
                      </div>
                    </div>
                  ) : (
                    <Text type="secondary">暂无风险状态信息</Text>
                  )}
                </Card>
              </Col>
            </Row>
          </TabPane>
          
          <TabPane tab={
            <span>
              <ControlOutlined />
              系统控制
            </span>
          } key="control">
            <SystemControlPanel />
          </TabPane>
          
          <TabPane tab="系统日志" key="logs">
            <Card title="实时系统日志" size="small">
              <div style={{ 
                height: '400px', 
                overflowY: 'auto', 
                backgroundColor: '#001529', 
                padding: '12px', 
                borderRadius: '6px',
                fontFamily: 'Monaco, Consolas, "Lucida Console", monospace'
              }}>
                {systemLogs.length > 0 ? (
                  systemLogs.map((log, index) => (
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
                    <small>日志将在后端系统启动后显示</small>
                  </div>
                )}
              </div>
            </Card>
          </TabPane>
        </Tabs>
      </Card>
    </div>
  );
};