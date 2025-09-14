import React, { useState, useEffect } from 'react';
import { 
  Card, Row, Col, Button, Table, Modal, Form, Input, Select, Switch, 
  Tabs, Progress, Statistic, Badge, Alert, message, notification,
  Drawer, Tag, Tooltip, Popconfirm, Space, Timeline, Descriptions,
  Typography, Divider, List, Avatar, Collapse, Tree
} from 'antd';
import {
  PlayCircleOutlined, PauseCircleOutlined, StopOutlined, ReloadOutlined,
  SettingOutlined, BugOutlined, FireOutlined, MonitorOutlined,
  LineChartOutlined, CodeOutlined, HistoryOutlined, FileTextOutlined,
  ThunderboltOutlined, EyeOutlined, DeleteOutlined, EditOutlined,
  PlusOutlined, CheckCircleOutlined, ExclamationCircleOutlined,
  ClockCircleOutlined, SyncOutlined, CaretRightOutlined
} from '@ant-design/icons';

const { Option } = Select;
const { TextArea } = Input;
const { Title, Text } = Typography;
const { Panel } = Collapse;

// 数据类型定义
interface Strategy {
  id: string;
  name: string;
  type: 'arbitrage' | 'market_making' | 'trend' | 'grid';
  status: 'running' | 'stopped' | 'paused' | 'error';
  config: any;
  metrics: {
    uptime: number;
    pnl: number;
    trades: number;
    success_rate: number;
    cpu_usage: number;
    memory_usage: number;
  };
  hot_reload_enabled: boolean;
  debug_enabled: boolean;
  created_at: string;
  last_updated: string;
}

interface RealtimeStatus {
  total_strategies: number;
  running_strategies: number;
  total_pnl: number;
  total_trades: number;
  system_health: 'healthy' | 'warning' | 'critical';
  cpu_usage: number;
  memory_usage: number;
  network_io: number;
  active_alerts: number;
}

interface DebugSession {
  id: string;
  strategy_id: string;
  strategy_name: string;
  status: 'active' | 'paused' | 'stopped';
  breakpoints: number;
  created_at: string;
}

interface HotReloadHistory {
  id: string;
  strategy_id: string;
  strategy_name: string;
  type: 'reload' | 'rollback';
  status: 'success' | 'failed' | 'pending';
  timestamp: string;
  changes: string;
}

interface Alert {
  id: string;
  strategy_id: string;
  type: 'error' | 'warning' | 'info';
  message: string;
  timestamp: string;
  resolved: boolean;
}

export default function StrategyModule() {
  const [loading, setLoading] = useState(false);
  const [activeTab, setActiveTab] = useState('overview');
  
  // 策略管理状态
  const [strategies, setStrategies] = useState<Strategy[]>([]);
  const [selectedStrategy, setSelectedStrategy] = useState<Strategy | null>(null);
  const [configModalVisible, setConfigModalVisible] = useState(false);
  const [logsModalVisible, setLogsModalVisible] = useState(false);
  const [metricsModalVisible, setMetricsModalVisible] = useState(false);
  
  // 实时监控状态
  const [realtimeStatus, setRealtimeStatus] = useState<RealtimeStatus>({
    total_strategies: 0,
    running_strategies: 0,
    total_pnl: 0,
    total_trades: 0,
    system_health: 'healthy',
    cpu_usage: 0,
    memory_usage: 0,
    network_io: 0,
    active_alerts: 0
  });
  
  // 调试工具状态
  const [debugSessions, setDebugSessions] = useState<DebugSession[]>([]);
  const [debugDrawerVisible, setDebugDrawerVisible] = useState(false);
  const [selectedDebugStrategy, setSelectedDebugStrategy] = useState<string | null>(null);
  
  // 热重载状态
  const [hotReloadHistory, setHotReloadHistory] = useState<HotReloadHistory[]>([]);
  const [hotReloadStatus, setHotReloadStatus] = useState<any>({});
  
  // 告警状态
  const [alerts, setAlerts] = useState<Alert[]>([]);
  
  // 表单实例
  const [configForm] = Form.useForm();

  // 初始化数据 - 模拟38个API接口的功能
  const initializeData = async () => {
    setLoading(true);
    try {
      // 1. 策略生命周期管理API (12个接口)
      const mockStrategies: Strategy[] = [
        {
          id: 'strategy_common',
          name: '通用套利策略',
          type: 'arbitrage',
          status: 'running',
          config: {
            max_position: 100,
            risk_threshold: 0.02,
            pair: 'BTC/USDT',
            exchanges: ['binance', 'okx']
          },
          metrics: {
            uptime: 18540,
            pnl: 2847.35,
            trades: 156,
            success_rate: 94.2,
            cpu_usage: 15.6,
            memory_usage: 234.5
          },
          hot_reload_enabled: true,
          debug_enabled: false,
          created_at: new Date(Date.now() - 86400000 * 3).toISOString(),
          last_updated: new Date(Date.now() - 3600000).toISOString()
        },
        {
          id: 'strategy_shadow_trading',
          name: '影子交易策略',
          type: 'market_making',
          status: 'running',
          config: {
            spread: 0.001,
            depth: 5,
            pair: 'ETH/USDT',
            min_profit: 0.02
          },
          metrics: {
            uptime: 25200,
            pnl: 1654.88,
            trades: 89,
            success_rate: 96.7,
            cpu_usage: 22.1,
            memory_usage: 187.3
          },
          hot_reload_enabled: true,
          debug_enabled: true,
          created_at: new Date(Date.now() - 86400000 * 5).toISOString(),
          last_updated: new Date(Date.now() - 1800000).toISOString()
        },
        {
          id: 'strategy_orchestrator',
          name: '策略编排器',
          type: 'grid',
          status: 'paused',
          config: {
            grid_size: 10,
            price_range: 0.05,
            pair: 'ADA/USDT',
            investment: 1000
          },
          metrics: {
            uptime: 7200,
            pnl: -45.23,
            trades: 23,
            success_rate: 78.3,
            cpu_usage: 8.4,
            memory_usage: 156.7
          },
          hot_reload_enabled: false,
          debug_enabled: false,
          created_at: new Date(Date.now() - 86400000 * 2).toISOString(),
          last_updated: new Date(Date.now() - 900000).toISOString()
        },
        {
          id: 'strategy_adapters',
          name: '适配器策略',
          type: 'trend',
          status: 'stopped',
          config: {
            trend_period: 24,
            signal_threshold: 0.03,
            pair: 'BNB/USDT',
            stop_loss: 0.02
          },
          metrics: {
            uptime: 0,
            pnl: 0,
            trades: 0,
            success_rate: 0,
            cpu_usage: 0,
            memory_usage: 0
          },
          hot_reload_enabled: true,
          debug_enabled: false,
          created_at: new Date(Date.now() - 86400000).toISOString(),
          last_updated: new Date(Date.now() - 300000).toISOString()
        },
        {
          id: 'strategy_ml_predictor',
          name: 'ML预测策略',
          type: 'arbitrage',
          status: 'running',
          config: {
            model_type: 'lstm',
            lookback_period: 60,
            confidence_threshold: 0.8,
            pair: 'SOL/USDT'
          },
          metrics: {
            uptime: 14400,
            pnl: 892.44,
            trades: 67,
            success_rate: 89.5,
            cpu_usage: 35.7,
            memory_usage: 412.8
          },
          hot_reload_enabled: true,
          debug_enabled: true,
          created_at: new Date(Date.now() - 86400000 * 4).toISOString(),
          last_updated: new Date(Date.now() - 600000).toISOString()
        }
      ];

      // 2. 实时监控API (8个接口)
      const runningStrategies = mockStrategies.filter(s => s.status === 'running');
      const totalPnl = mockStrategies.reduce((sum, s) => sum + s.metrics.pnl, 0);
      const totalTrades = mockStrategies.reduce((sum, s) => sum + s.metrics.trades, 0);
      const avgCpuUsage = runningStrategies.length > 0 ? 
        runningStrategies.reduce((sum, s) => sum + s.metrics.cpu_usage, 0) / runningStrategies.length : 0;
      const avgMemoryUsage = runningStrategies.length > 0 ? 
        runningStrategies.reduce((sum, s) => sum + s.metrics.memory_usage, 0) / runningStrategies.length : 0;

      const mockRealtimeStatus: RealtimeStatus = {
        total_strategies: mockStrategies.length,
        running_strategies: runningStrategies.length,
        total_pnl: totalPnl,
        total_trades: totalTrades,
        system_health: runningStrategies.length === mockStrategies.length ? 'healthy' : 
                      runningStrategies.length >= mockStrategies.length * 0.7 ? 'warning' : 'critical',
        cpu_usage: Math.round(avgCpuUsage),
        memory_usage: Math.round(avgMemoryUsage),
        network_io: Math.round(Math.random() * 1000 + 500),
        active_alerts: Math.floor(Math.random() * 3)
      };

      // 3. 调试工具API (9个接口)
      const mockDebugSessions: DebugSession[] = mockStrategies
        .filter(s => s.debug_enabled)
        .map(s => ({
          id: `debug_${s.id}`,
          strategy_id: s.id,
          strategy_name: s.name,
          status: s.status === 'running' ? 'active' : 'paused',
          breakpoints: Math.floor(Math.random() * 5) + 1,
          created_at: new Date(Date.now() - Math.random() * 86400000).toISOString()
        }));

      // 4. 热重载API (9个接口)
      const mockHotReloadHistory: HotReloadHistory[] = [
        {
          id: 'hr_001',
          strategy_id: 'strategy_common',
          strategy_name: '通用套利策略',
          type: 'reload',
          status: 'success',
          timestamp: new Date(Date.now() - 3600000).toISOString(),
          changes: '更新风险阈值配置'
        },
        {
          id: 'hr_002',
          strategy_id: 'strategy_shadow_trading',
          strategy_name: '影子交易策略',
          type: 'reload',
          status: 'success',
          timestamp: new Date(Date.now() - 7200000).toISOString(),
          changes: '优化订单深度算法'
        },
        {
          id: 'hr_003',
          strategy_id: 'strategy_ml_predictor',
          strategy_name: 'ML预测策略',
          type: 'rollback',
          status: 'success',
          timestamp: new Date(Date.now() - 10800000).toISOString(),
          changes: '回滚模型参数变更'
        }
      ];

      // 生成告警数据
      const mockAlerts: Alert[] = [
        {
          id: 'alert_001',
          strategy_id: 'strategy_orchestrator',
          type: 'warning',
          message: '策略编排器已暂停，PnL出现负值',
          timestamp: new Date(Date.now() - 1800000).toISOString(),
          resolved: false
        },
        {
          id: 'alert_002',
          strategy_id: 'strategy_ml_predictor',
          type: 'info',
          message: 'ML预测策略CPU使用率较高，建议优化',
          timestamp: new Date(Date.now() - 3600000).toISOString(),
          resolved: false
        }
      ];

      // 更新状态
      setStrategies(mockStrategies);
      setRealtimeStatus(mockRealtimeStatus);
      setDebugSessions(mockDebugSessions);
      setHotReloadHistory(mockHotReloadHistory);
      setAlerts(mockAlerts);
      
    } catch (error) {
      console.error('初始化策略数据失败:', error);
      message.error('数据加载失败');
    } finally {
      setLoading(false);
    }
  };

  // 策略生命周期管理
  const handleStrategyLifecycle = async (strategyId: string, action: string) => {
    try {
      const strategy = strategies.find(s => s.id === strategyId);
      if (!strategy) return;

      message.loading(`正在${action}策略...`, 2);
      
      // 模拟API调用延迟
      await new Promise(resolve => setTimeout(resolve, 1500));
      
      let newStatus: Strategy['status'] = strategy.status;
      let actionMessage = '';

      switch (action) {
        case 'start':
          newStatus = 'running';
          actionMessage = '启动成功';
          break;
        case 'stop':
          newStatus = 'stopped';
          actionMessage = '停止成功';
          break;
        case 'restart':
          newStatus = 'running';
          actionMessage = '重启成功';
          break;
        case 'pause':
          newStatus = 'paused';
          actionMessage = '暂停成功';
          break;
        case 'resume':
          newStatus = 'running';
          actionMessage = '恢复成功';
          break;
      }

      setStrategies(prev => prev.map(s => 
        s.id === strategyId ? { ...s, status: newStatus, last_updated: new Date().toISOString() } : s
      ));

      message.success(actionMessage);
      
      // 刷新实时状态
      setTimeout(initializeData, 1000);
      
    } catch (error) {
      message.error(`操作失败: ${error.message}`);
    }
  };

  // 热重载操作
  const handleHotReload = async (strategyId: string, action: 'reload' | 'rollback' | 'validate') => {
    try {
      const strategy = strategies.find(s => s.id === strategyId);
      if (!strategy) return;

      const actionText = {
        reload: '热重载',
        rollback: '回滚',
        validate: '验证'
      }[action];

      message.loading(`正在${actionText}策略...`, 2);
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      // 添加热重载历史记录
      const newHistoryItem: HotReloadHistory = {
        id: `hr_${Date.now()}`,
        strategy_id: strategyId,
        strategy_name: strategy.name,
        type: action === 'rollback' ? 'rollback' : 'reload',
        status: 'success',
        timestamp: new Date().toISOString(),
        changes: `${actionText}操作 - 配置更新`
      };

      setHotReloadHistory(prev => [newHistoryItem, ...prev]);
      message.success(`${actionText}成功`);
      
    } catch (error) {
      message.error(`${action}失败`);
    }
  };

  // 调试会话管理
  const handleDebugSession = async (action: string, strategyId?: string, sessionId?: string) => {
    try {
      switch (action) {
        case 'create':
          if (!strategyId) return;
          const strategy = strategies.find(s => s.id === strategyId);
          if (!strategy) return;

          const newSession: DebugSession = {
            id: `debug_${Date.now()}`,
            strategy_id: strategyId,
            strategy_name: strategy.name,
            status: 'active',
            breakpoints: 0,
            created_at: new Date().toISOString()
          };

          setDebugSessions(prev => [...prev, newSession]);
          message.success('调试会话创建成功');
          break;
          
        case 'delete':
          if (!sessionId) return;
          setDebugSessions(prev => prev.filter(s => s.id !== sessionId));
          message.success('调试会话删除成功');
          break;
      }
    } catch (error) {
      message.error('调试操作失败');
    }
  };

  // 显示策略配置
  const showStrategyConfig = (strategy: Strategy) => {
    setSelectedStrategy(strategy);
    configForm.setFieldsValue(strategy.config);
    setConfigModalVisible(true);
  };

  // 保存策略配置
  const saveStrategyConfig = async (values: any) => {
    try {
      if (!selectedStrategy) return;
      
      setStrategies(prev => prev.map(s => 
        s.id === selectedStrategy.id ? { ...s, config: values, last_updated: new Date().toISOString() } : s
      ));
      
      setConfigModalVisible(false);
      message.success('配置更新成功');
    } catch (error) {
      message.error('配置更新失败');
    }
  };

  // 显示策略日志
  const showStrategyLogs = (strategy: Strategy) => {
    setSelectedStrategy(strategy);
    setLogsModalVisible(true);
  };

  // 显示策略指标
  const showStrategyMetrics = (strategy: Strategy) => {
    setSelectedStrategy(strategy);
    setMetricsModalVisible(true);
  };

  useEffect(() => {
    initializeData();
    // 每30秒刷新数据
    const interval = setInterval(initializeData, 30000);
    return () => clearInterval(interval);
  }, []);

  // 表格列定义
  const strategyColumns = [
    {
      title: '策略名称',
      dataIndex: 'name',
      key: 'name',
      render: (name: string, record: Strategy) => (
        <div>
          <Text strong>{name}</Text>
          <br />
          <Tag color={record.type === 'arbitrage' ? 'blue' : record.type === 'market_making' ? 'green' : record.type === 'trend' ? 'orange' : 'purple'}>
            {record.type}
          </Tag>
        </div>
      )
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => {
        const colors = {
          running: 'success',
          stopped: 'default',
          paused: 'warning',
          error: 'error'
        };
        return <Badge status={colors[status as keyof typeof colors]} text={status} />;
      }
    },
    {
      title: 'PnL',
      dataIndex: ['metrics', 'pnl'],
      key: 'pnl',
      render: (pnl: number) => (
        <Text style={{ color: pnl >= 0 ? '#3f8600' : '#cf1322' }}>
          {pnl >= 0 ? '+' : ''}{pnl.toFixed(2)}
        </Text>
      )
    },
    {
      title: '交易数',
      dataIndex: ['metrics', 'trades'],
      key: 'trades'
    },
    {
      title: '成功率',
      dataIndex: ['metrics', 'success_rate'],
      key: 'success_rate',
      render: (rate: number) => `${rate.toFixed(1)}%`
    },
    {
      title: '资源使用',
      key: 'resources',
      render: (_, record: Strategy) => (
        <div>
          <div>CPU: {record.metrics.cpu_usage.toFixed(1)}%</div>
          <div>内存: {record.metrics.memory_usage.toFixed(1)}MB</div>
        </div>
      )
    },
    {
      title: '功能',
      key: 'features',
      render: (_, record: Strategy) => (
        <Space>
          {record.hot_reload_enabled && <Tag color="orange">热重载</Tag>}
          {record.debug_enabled && <Tag color="red">调试</Tag>}
        </Space>
      )
    },
    {
      title: '操作',
      key: 'actions',
      render: (_, record: Strategy) => (
        <Space size="small" wrap>
          {record.status === 'stopped' && (
            <Button size="small" type="primary" icon={<PlayCircleOutlined />} 
                    onClick={() => handleStrategyLifecycle(record.id, 'start')}>
              启动
            </Button>
          )}
          {record.status === 'running' && (
            <>
              <Button size="small" icon={<PauseCircleOutlined />} 
                      onClick={() => handleStrategyLifecycle(record.id, 'pause')}>
                暂停
              </Button>
              <Button size="small" icon={<StopOutlined />} 
                      onClick={() => handleStrategyLifecycle(record.id, 'stop')}>
                停止
              </Button>
            </>
          )}
          {record.status === 'paused' && (
            <Button size="small" type="primary" icon={<PlayCircleOutlined />} 
                    onClick={() => handleStrategyLifecycle(record.id, 'resume')}>
              恢复
            </Button>
          )}
          <Button size="small" icon={<ReloadOutlined />} 
                  onClick={() => handleStrategyLifecycle(record.id, 'restart')}>
            重启
          </Button>
          <Button size="small" icon={<SettingOutlined />} 
                  onClick={() => showStrategyConfig(record)}>
            配置
          </Button>
          <Button size="small" icon={<FileTextOutlined />} 
                  onClick={() => showStrategyLogs(record)}>
            日志
          </Button>
          <Button size="small" icon={<LineChartOutlined />} 
                  onClick={() => showStrategyMetrics(record)}>
            指标
          </Button>
          {record.hot_reload_enabled && (
            <Button size="small" icon={<FireOutlined />} 
                    onClick={() => handleHotReload(record.id, 'reload')}>
              热重载
            </Button>
          )}
          {record.debug_enabled && (
            <Button size="small" icon={<BugOutlined />} 
                    onClick={() => {
                      setSelectedDebugStrategy(record.id);
                      setDebugDrawerVisible(true);
                    }}>
              调试
            </Button>
          )}
        </Space>
      )
    }
  ];

  const debugSessionColumns = [
    { title: '会话ID', dataIndex: 'id', key: 'id' },
    { title: '策略名称', dataIndex: 'strategy_name', key: 'strategy_name' },
    { title: '状态', dataIndex: 'status', key: 'status',
      render: (status: string) => <Badge status={status === 'active' ? 'success' : 'warning'} text={status} />
    },
    { title: '断点数', dataIndex: 'breakpoints', key: 'breakpoints' },
    { title: '创建时间', dataIndex: 'created_at', key: 'created_at',
      render: (time: string) => new Date(time).toLocaleString()
    },
    {
      title: '操作',
      key: 'actions',
      render: (_, record: DebugSession) => (
        <Space>
          <Button size="small">查看</Button>
          <Popconfirm title="确认删除?" onConfirm={() => handleDebugSession('delete', undefined, record.id)}>
            <Button size="small" danger>删除</Button>
          </Popconfirm>
        </Space>
      )
    }
  ];

  const hotReloadColumns = [
    { title: '策略名称', dataIndex: 'strategy_name', key: 'strategy_name' },
    { title: '操作类型', dataIndex: 'type', key: 'type',
      render: (type: string) => <Tag color={type === 'reload' ? 'blue' : 'orange'}>{type}</Tag>
    },
    { title: '状态', dataIndex: 'status', key: 'status',
      render: (status: string) => <Badge status={status === 'success' ? 'success' : 'error'} text={status} />
    },
    { title: '变更说明', dataIndex: 'changes', key: 'changes' },
    { title: '时间', dataIndex: 'timestamp', key: 'timestamp',
      render: (time: string) => new Date(time).toLocaleString()
    }
  ];

  return (
    <div style={{ padding: '24px' }}>
      <div style={{ marginBottom: '24px' }}>
        <Title level={2}>策略服务中心</Title>
        <Text type="secondary">策略生命周期管理、实时监控、调试工具、热重载功能</Text>
      </div>

      <Tabs 
        activeKey={activeTab} 
        onChange={setActiveTab} 
        size="large"
        items={[
          {
            key: 'overview',
            label: '系统概览',
            children: (
              <>
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="策略总数"
                        value={realtimeStatus.total_strategies}
                        suffix={`/ ${realtimeStatus.running_strategies}运行中`}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="总盈亏"
                        value={realtimeStatus.total_pnl}
                        precision={2}
                        valueStyle={{ color: realtimeStatus.total_pnl >= 0 ? '#3f8600' : '#cf1322' }}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="交易总数"
                        value={realtimeStatus.total_trades}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="系统健康度"
                        value={realtimeStatus.system_health}
                        valueStyle={{ 
                          color: realtimeStatus.system_health === 'healthy' ? '#3f8600' : 
                                 realtimeStatus.system_health === 'warning' ? '#fa8c16' : '#cf1322'
                        }}
                      />
                    </Card>
                  </Col>
                </Row>

                <Row gutter={[16, 16]}>
                  <Col xs={24} md={12}>
                    <Card title="系统资源监控" size="small">
                      <div style={{ marginBottom: '16px' }}>
                        <div>CPU使用率</div>
                        <Progress percent={realtimeStatus.cpu_usage} />
                      </div>
                      <div style={{ marginBottom: '16px' }}>
                        <div>内存使用率</div>
                        <Progress percent={Math.round(realtimeStatus.memory_usage / 10)} />
                      </div>
                      <div>
                        <div>网络I/O</div>
                        <Progress percent={Math.round(realtimeStatus.network_io / 10)} />
                      </div>
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card title="活跃告警" size="small">
                      {alerts.filter(a => !a.resolved).map(alert => (
                        <Alert
                          key={alert.id}
                          message={alert.message}
                          type={alert.type}
                          showIcon
                          style={{ marginBottom: 8 }}
                          action={
                            <Button size="small" onClick={() => {
                              setAlerts(prev => prev.map(a => a.id === alert.id ? { ...a, resolved: true } : a));
                            }}>
                              解决
                            </Button>
                          }
                        />
                      ))}
                      {alerts.filter(a => !a.resolved).length === 0 && (
                        <Text type="secondary">暂无活跃告警</Text>
                      )}
                    </Card>
                  </Col>
                </Row>
              </>
            )
          },
          {
            key: 'strategies',
            label: `策略管理 (${strategies.length})`,
            children: (
              <Card
                title="策略列表"
                extra={
                  <Button icon={<ReloadOutlined />} onClick={initializeData} loading={loading}>
                    刷新
                  </Button>
                }
              >
                <Table
                  dataSource={strategies}
                  columns={strategyColumns}
                  rowKey="id"
                  loading={loading}
                  pagination={{ pageSize: 10 }}
                  scroll={{ x: 1200 }}
                />
              </Card>
            )
          },
          {
            key: 'monitoring',
            label: '实时监控',
            children: (
              <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                <Col xs={24} md={8}>
                  <Card title="性能概览" size="small">
                    <Statistic title="平均CPU使用率" value={realtimeStatus.cpu_usage} suffix="%" />
                    <Statistic title="平均内存使用" value={realtimeStatus.memory_usage} suffix="MB" />
                    <Statistic title="网络I/O" value={realtimeStatus.network_io} suffix="KB/s" />
                  </Card>
                </Col>
                <Col xs={24} md={8}>
                  <Card title="运行状态" size="small">
                    <div style={{ lineHeight: '2.5' }}>
                      <div>运行中策略: <Text strong>{realtimeStatus.running_strategies}</Text></div>
                      <div>暂停策略: <Text>{strategies.filter(s => s.status === 'paused').length}</Text></div>
                      <div>停止策略: <Text>{strategies.filter(s => s.status === 'stopped').length}</Text></div>
                      <div>异常策略: <Text type="danger">{strategies.filter(s => s.status === 'error').length}</Text></div>
                    </div>
                  </Card>
                </Col>
                <Col xs={24} md={8}>
                  <Card title="交易统计" size="small">
                    <Statistic title="今日交易量" value={realtimeStatus.total_trades} />
                    <Statistic 
                      title="今日盈亏" 
                      value={realtimeStatus.total_pnl} 
                      precision={2}
                      valueStyle={{ color: realtimeStatus.total_pnl >= 0 ? '#3f8600' : '#cf1322' }}
                    />
                  </Card>
                </Col>
              </Row>
            )
          },
          {
            key: 'debug',
            label: `调试工具 (${debugSessions.length})`,
            children: (
              <Card
                title="调试会话"
                extra={
                  <Select
                    placeholder="选择策略创建调试会话"
                    style={{ width: 200 }}
                    onChange={(strategyId) => handleDebugSession('create', strategyId)}
                  >
                    {strategies.filter(s => s.status === 'running').map(s => (
                      <Option key={s.id} value={s.id}>{s.name}</Option>
                    ))}
                  </Select>
                }
              >
                <Table
                  dataSource={debugSessions}
                  columns={debugSessionColumns}
                  rowKey="id"
                  loading={loading}
                  pagination={{ pageSize: 10 }}
                />
              </Card>
            )
          },
          {
            key: 'hotreload',
            label: `热重载 (${hotReloadHistory.length})`,
            children: (
              <Card title="热重载历史">
                <Table
                  dataSource={hotReloadHistory}
                  columns={hotReloadColumns}
                  rowKey="id"
                  loading={loading}
                  pagination={{ pageSize: 10 }}
                />
              </Card>
            )
          }
        ]}
      />

      {/* 策略配置模态框 */}
      <Modal
        title={`配置策略: ${selectedStrategy?.name}`}
        open={configModalVisible}
        onCancel={() => setConfigModalVisible(false)}
        onOk={() => configForm.submit()}
        width={600}
      >
        {selectedStrategy && (
          <Form form={configForm} onFinish={saveStrategyConfig} layout="vertical" initialValues={selectedStrategy?.config}>
            {Object.entries(selectedStrategy.config).map(([key, value]) => (
              <Form.Item key={key} name={key} label={key}>
                <Input />
              </Form.Item>
            ))}
          </Form>
        )}
      </Modal>

      {/* 策略日志模态框 */}
      <Modal
        title={`策略日志: ${selectedStrategy?.name}`}
        open={logsModalVisible}
        onCancel={() => setLogsModalVisible(false)}
        footer={null}
        width={800}
      >
        <div style={{ height: '400px', overflow: 'auto', backgroundColor: '#000', color: '#fff', padding: '16px', fontFamily: 'monospace' }}>
          <div>[2024-09-09 10:30:45] INFO: 策略启动完成</div>
          <div>[2024-09-09 10:30:46] INFO: 连接交易所...</div>
          <div>[2024-09-09 10:30:47] INFO: 开始监控价格差异</div>
          <div>[2024-09-09 10:31:12] INFO: 发现套利机会 BTCUSDT 0.02%</div>
          <div>[2024-09-09 10:31:13] INFO: 执行买入订单...</div>
          <div>[2024-09-09 10:31:15] SUCCESS: 订单执行成功 +12.45 USDT</div>
          <div>[2024-09-09 10:32:01] INFO: 继续监控中...</div>
        </div>
      </Modal>

      {/* 策略指标模态框 */}
      <Modal
        title={`策略指标: ${selectedStrategy?.name}`}
        open={metricsModalVisible}
        onCancel={() => setMetricsModalVisible(false)}
        footer={null}
        width={600}
      >
        {selectedStrategy && (
          <Descriptions bordered column={2}>
            <Descriptions.Item label="运行时间">{Math.floor(selectedStrategy.metrics.uptime / 3600)}小时</Descriptions.Item>
            <Descriptions.Item label="盈亏">
              <Text style={{ color: selectedStrategy.metrics.pnl >= 0 ? '#3f8600' : '#cf1322' }}>
                {selectedStrategy.metrics.pnl.toFixed(2)} USDT
              </Text>
            </Descriptions.Item>
            <Descriptions.Item label="交易数量">{selectedStrategy.metrics.trades}</Descriptions.Item>
            <Descriptions.Item label="成功率">{selectedStrategy.metrics.success_rate.toFixed(1)}%</Descriptions.Item>
            <Descriptions.Item label="CPU使用率">{selectedStrategy.metrics.cpu_usage.toFixed(1)}%</Descriptions.Item>
            <Descriptions.Item label="内存使用">{selectedStrategy.metrics.memory_usage.toFixed(1)}MB</Descriptions.Item>
          </Descriptions>
        )}
      </Modal>

      {/* 调试抽屉 */}
      <Drawer
        title="策略调试工具"
        open={debugDrawerVisible}
        onClose={() => setDebugDrawerVisible(false)}
        width={600}
      >
        <Tabs 
          items={[
            {
              key: 'breakpoints',
              label: '断点管理',
              children: (
                <Space direction="vertical" style={{ width: '100%' }}>
                  <Button type="primary" icon={<PlusOutlined />}>添加断点</Button>
                  <List
                    size="small"
                    dataSource={['第45行: 价格检查', '第78行: 订单执行', '第102行: 风险控制']}
                    renderItem={(item, index) => (
                      <List.Item
                        actions={[
                          <Button size="small" danger icon={<DeleteOutlined />}>删除</Button>
                        ]}
                      >
                        {item}
                      </List.Item>
                    )}
                  />
                </Space>
              )
            },
            {
              key: 'variables',
              label: '变量查看',
              children: (
                <pre style={{ fontSize: '12px' }}>
{`{
  "current_price": 45234.56,
  "target_profit": 0.02,
  "position_size": 0.1,
  "risk_level": "low",
  "last_trade_time": "2024-09-09T10:30:45Z"
}`}
                </pre>
              )
            },
            {
              key: 'stack',
              label: '调用栈',
              children: (
                <Timeline>
                  <Timeline.Item>main() - 策略主循环</Timeline.Item>
                  <Timeline.Item>check_arbitrage() - 检查套利机会</Timeline.Item>
                  <Timeline.Item color="red">execute_trade() - 执行交易 [当前]</Timeline.Item>
                  <Timeline.Item>validate_order() - 验证订单</Timeline.Item>
                </Timeline>
              )
            }
          ]}
        />
      </Drawer>
    </div>
  );
}
import { 
  Card, Row, Col, Button, Table, Modal, Form, Input, Select, Switch, 
  Tabs, Progress, Statistic, Badge, Alert, message, notification,
  Drawer, Tag, Tooltip, Popconfirm, Space, Timeline, Descriptions,
  Typography, Divider, List, Avatar, Collapse, Tree
} from 'antd';
import {
  PlayCircleOutlined, PauseCircleOutlined, StopOutlined, ReloadOutlined,
  SettingOutlined, BugOutlined, FireOutlined, MonitorOutlined,
  LineChartOutlined, CodeOutlined, HistoryOutlined, FileTextOutlined,
  ThunderboltOutlined, EyeOutlined, DeleteOutlined, EditOutlined,
  PlusOutlined, CheckCircleOutlined, ExclamationCircleOutlined,
  ClockCircleOutlined, SyncOutlined, CaretRightOutlined
} from '@ant-design/icons';

const { Option } = Select;
const { TextArea } = Input;
const { Title, Text } = Typography;
const { Panel } = Collapse;

// 数据类型定义
interface Strategy {
  id: string;
  name: string;
  type: 'arbitrage' | 'market_making' | 'trend' | 'grid';
  status: 'running' | 'stopped' | 'paused' | 'error';
  config: any;
  metrics: {
    uptime: number;
    pnl: number;
    trades: number;
    success_rate: number;
    cpu_usage: number;
    memory_usage: number;
  };
  hot_reload_enabled: boolean;
  debug_enabled: boolean;
  created_at: string;
  last_updated: string;
}

interface RealtimeStatus {
  total_strategies: number;
  running_strategies: number;
  total_pnl: number;
  total_trades: number;
  system_health: 'healthy' | 'warning' | 'critical';
  cpu_usage: number;
  memory_usage: number;
  network_io: number;
  active_alerts: number;
}

interface DebugSession {
  id: string;
  strategy_id: string;
  strategy_name: string;
  status: 'active' | 'paused' | 'stopped';
  breakpoints: number;
  created_at: string;
}

interface HotReloadHistory {
  id: string;
  strategy_id: string;
  strategy_name: string;
  type: 'reload' | 'rollback';
  status: 'success' | 'failed' | 'pending';
  timestamp: string;
  changes: string;
}

interface Alert {
  id: string;
  strategy_id: string;
  type: 'error' | 'warning' | 'info';
  message: string;
  timestamp: string;
  resolved: boolean;
}

export default function StrategyModule() {
  const [loading, setLoading] = useState(false);
  const [activeTab, setActiveTab] = useState('overview');
  
  // 策略管理状态
  const [strategies, setStrategies] = useState<Strategy[]>([]);
  const [selectedStrategy, setSelectedStrategy] = useState<Strategy | null>(null);
  const [configModalVisible, setConfigModalVisible] = useState(false);
  const [logsModalVisible, setLogsModalVisible] = useState(false);
  const [metricsModalVisible, setMetricsModalVisible] = useState(false);
  
  // 实时监控状态
  const [realtimeStatus, setRealtimeStatus] = useState<RealtimeStatus>({
    total_strategies: 0,
    running_strategies: 0,
    total_pnl: 0,
    total_trades: 0,
    system_health: 'healthy',
    cpu_usage: 0,
    memory_usage: 0,
    network_io: 0,
    active_alerts: 0
  });
  
  // 调试工具状态
  const [debugSessions, setDebugSessions] = useState<DebugSession[]>([]);
  const [debugDrawerVisible, setDebugDrawerVisible] = useState(false);
  const [selectedDebugStrategy, setSelectedDebugStrategy] = useState<string | null>(null);
  
  // 热重载状态
  const [hotReloadHistory, setHotReloadHistory] = useState<HotReloadHistory[]>([]);
  const [hotReloadStatus, setHotReloadStatus] = useState<any>({});
  
  // 告警状态
  const [alerts, setAlerts] = useState<Alert[]>([]);
  
  // 表单实例
  const [configForm] = Form.useForm();

  // 初始化数据 - 模拟38个API接口的功能
  const initializeData = async () => {
    setLoading(true);
    try {
      // 1. 策略生命周期管理API (12个接口)
      const mockStrategies: Strategy[] = [
        {
          id: 'strategy_common',
          name: '通用套利策略',
          type: 'arbitrage',
          status: 'running',
          config: {
            max_position: 100,
            risk_threshold: 0.02,
            pair: 'BTC/USDT',
            exchanges: ['binance', 'okx']
          },
          metrics: {
            uptime: 18540,
            pnl: 2847.35,
            trades: 156,
            success_rate: 94.2,
            cpu_usage: 15.6,
            memory_usage: 234.5
          },
          hot_reload_enabled: true,
          debug_enabled: false,
          created_at: new Date(Date.now() - 86400000 * 3).toISOString(),
          last_updated: new Date(Date.now() - 3600000).toISOString()
        },
        {
          id: 'strategy_shadow_trading',
          name: '影子交易策略',
          type: 'market_making',
          status: 'running',
          config: {
            spread: 0.001,
            depth: 5,
            pair: 'ETH/USDT',
            min_profit: 0.02
          },
          metrics: {
            uptime: 25200,
            pnl: 1654.88,
            trades: 89,
            success_rate: 96.7,
            cpu_usage: 22.1,
            memory_usage: 187.3
          },
          hot_reload_enabled: true,
          debug_enabled: true,
          created_at: new Date(Date.now() - 86400000 * 5).toISOString(),
          last_updated: new Date(Date.now() - 1800000).toISOString()
        },
        {
          id: 'strategy_orchestrator',
          name: '策略编排器',
          type: 'grid',
          status: 'paused',
          config: {
            grid_size: 10,
            price_range: 0.05,
            pair: 'ADA/USDT',
            investment: 1000
          },
          metrics: {
            uptime: 7200,
            pnl: -45.23,
            trades: 23,
            success_rate: 78.3,
            cpu_usage: 8.4,
            memory_usage: 156.7
          },
          hot_reload_enabled: false,
          debug_enabled: false,
          created_at: new Date(Date.now() - 86400000 * 2).toISOString(),
          last_updated: new Date(Date.now() - 900000).toISOString()
        },
        {
          id: 'strategy_adapters',
          name: '适配器策略',
          type: 'trend',
          status: 'stopped',
          config: {
            trend_period: 24,
            signal_threshold: 0.03,
            pair: 'BNB/USDT',
            stop_loss: 0.02
          },
          metrics: {
            uptime: 0,
            pnl: 0,
            trades: 0,
            success_rate: 0,
            cpu_usage: 0,
            memory_usage: 0
          },
          hot_reload_enabled: true,
          debug_enabled: false,
          created_at: new Date(Date.now() - 86400000).toISOString(),
          last_updated: new Date(Date.now() - 300000).toISOString()
        },
        {
          id: 'strategy_ml_predictor',
          name: 'ML预测策略',
          type: 'arbitrage',
          status: 'running',
          config: {
            model_type: 'lstm',
            lookback_period: 60,
            confidence_threshold: 0.8,
            pair: 'SOL/USDT'
          },
          metrics: {
            uptime: 14400,
            pnl: 892.44,
            trades: 67,
            success_rate: 89.5,
            cpu_usage: 35.7,
            memory_usage: 412.8
          },
          hot_reload_enabled: true,
          debug_enabled: true,
          created_at: new Date(Date.now() - 86400000 * 4).toISOString(),
          last_updated: new Date(Date.now() - 600000).toISOString()
        }
      ];

      // 2. 实时监控API (8个接口)
      const runningStrategies = mockStrategies.filter(s => s.status === 'running');
      const totalPnl = mockStrategies.reduce((sum, s) => sum + s.metrics.pnl, 0);
      const totalTrades = mockStrategies.reduce((sum, s) => sum + s.metrics.trades, 0);
      const avgCpuUsage = runningStrategies.length > 0 ? 
        runningStrategies.reduce((sum, s) => sum + s.metrics.cpu_usage, 0) / runningStrategies.length : 0;
      const avgMemoryUsage = runningStrategies.length > 0 ? 
        runningStrategies.reduce((sum, s) => sum + s.metrics.memory_usage, 0) / runningStrategies.length : 0;

      const mockRealtimeStatus: RealtimeStatus = {
        total_strategies: mockStrategies.length,
        running_strategies: runningStrategies.length,
        total_pnl: totalPnl,
        total_trades: totalTrades,
        system_health: runningStrategies.length === mockStrategies.length ? 'healthy' : 
                      runningStrategies.length >= mockStrategies.length * 0.7 ? 'warning' : 'critical',
        cpu_usage: Math.round(avgCpuUsage),
        memory_usage: Math.round(avgMemoryUsage),
        network_io: Math.round(Math.random() * 1000 + 500),
        active_alerts: Math.floor(Math.random() * 3)
      };

      // 3. 调试工具API (9个接口)
      const mockDebugSessions: DebugSession[] = mockStrategies
        .filter(s => s.debug_enabled)
        .map(s => ({
          id: `debug_${s.id}`,
          strategy_id: s.id,
          strategy_name: s.name,
          status: s.status === 'running' ? 'active' : 'paused',
          breakpoints: Math.floor(Math.random() * 5) + 1,
          created_at: new Date(Date.now() - Math.random() * 86400000).toISOString()
        }));

      // 4. 热重载API (9个接口)
      const mockHotReloadHistory: HotReloadHistory[] = [
        {
          id: 'hr_001',
          strategy_id: 'strategy_common',
          strategy_name: '通用套利策略',
          type: 'reload',
          status: 'success',
          timestamp: new Date(Date.now() - 3600000).toISOString(),
          changes: '更新风险阈值配置'
        },
        {
          id: 'hr_002',
          strategy_id: 'strategy_shadow_trading',
          strategy_name: '影子交易策略',
          type: 'reload',
          status: 'success',
          timestamp: new Date(Date.now() - 7200000).toISOString(),
          changes: '优化订单深度算法'
        },
        {
          id: 'hr_003',
          strategy_id: 'strategy_ml_predictor',
          strategy_name: 'ML预测策略',
          type: 'rollback',
          status: 'success',
          timestamp: new Date(Date.now() - 10800000).toISOString(),
          changes: '回滚模型参数变更'
        }
      ];

      // 生成告警数据
      const mockAlerts: Alert[] = [
        {
          id: 'alert_001',
          strategy_id: 'strategy_orchestrator',
          type: 'warning',
          message: '策略编排器已暂停，PnL出现负值',
          timestamp: new Date(Date.now() - 1800000).toISOString(),
          resolved: false
        },
        {
          id: 'alert_002',
          strategy_id: 'strategy_ml_predictor',
          type: 'info',
          message: 'ML预测策略CPU使用率较高，建议优化',
          timestamp: new Date(Date.now() - 3600000).toISOString(),
          resolved: false
        }
      ];

      // 更新状态
      setStrategies(mockStrategies);
      setRealtimeStatus(mockRealtimeStatus);
      setDebugSessions(mockDebugSessions);
      setHotReloadHistory(mockHotReloadHistory);
      setAlerts(mockAlerts);
      
    } catch (error) {
      console.error('初始化策略数据失败:', error);
      message.error('数据加载失败');
    } finally {
      setLoading(false);
    }
  };

  // 策略生命周期管理
  const handleStrategyLifecycle = async (strategyId: string, action: string) => {
    try {
      const strategy = strategies.find(s => s.id === strategyId);
      if (!strategy) return;

      message.loading(`正在${action}策略...`, 2);
      
      // 模拟API调用延迟
      await new Promise(resolve => setTimeout(resolve, 1500));
      
      let newStatus: Strategy['status'] = strategy.status;
      let actionMessage = '';

      switch (action) {
        case 'start':
          newStatus = 'running';
          actionMessage = '启动成功';
          break;
        case 'stop':
          newStatus = 'stopped';
          actionMessage = '停止成功';
          break;
        case 'restart':
          newStatus = 'running';
          actionMessage = '重启成功';
          break;
        case 'pause':
          newStatus = 'paused';
          actionMessage = '暂停成功';
          break;
        case 'resume':
          newStatus = 'running';
          actionMessage = '恢复成功';
          break;
      }

      setStrategies(prev => prev.map(s => 
        s.id === strategyId ? { ...s, status: newStatus, last_updated: new Date().toISOString() } : s
      ));

      message.success(actionMessage);
      
      // 刷新实时状态
      setTimeout(initializeData, 1000);
      
    } catch (error) {
      message.error(`操作失败: ${error.message}`);
    }
  };

  // 热重载操作
  const handleHotReload = async (strategyId: string, action: 'reload' | 'rollback' | 'validate') => {
    try {
      const strategy = strategies.find(s => s.id === strategyId);
      if (!strategy) return;

      const actionText = {
        reload: '热重载',
        rollback: '回滚',
        validate: '验证'
      }[action];

      message.loading(`正在${actionText}策略...`, 2);
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      // 添加热重载历史记录
      const newHistoryItem: HotReloadHistory = {
        id: `hr_${Date.now()}`,
        strategy_id: strategyId,
        strategy_name: strategy.name,
        type: action === 'rollback' ? 'rollback' : 'reload',
        status: 'success',
        timestamp: new Date().toISOString(),
        changes: `${actionText}操作 - 配置更新`
      };

      setHotReloadHistory(prev => [newHistoryItem, ...prev]);
      message.success(`${actionText}成功`);
      
    } catch (error) {
      message.error(`${action}失败`);
    }
  };

  // 调试会话管理
  const handleDebugSession = async (action: string, strategyId?: string, sessionId?: string) => {
    try {
      switch (action) {
        case 'create':
          if (!strategyId) return;
          const strategy = strategies.find(s => s.id === strategyId);
          if (!strategy) return;

          const newSession: DebugSession = {
            id: `debug_${Date.now()}`,
            strategy_id: strategyId,
            strategy_name: strategy.name,
            status: 'active',
            breakpoints: 0,
            created_at: new Date().toISOString()
          };

          setDebugSessions(prev => [...prev, newSession]);
          message.success('调试会话创建成功');
          break;
          
        case 'delete':
          if (!sessionId) return;
          setDebugSessions(prev => prev.filter(s => s.id !== sessionId));
          message.success('调试会话删除成功');
          break;
      }
    } catch (error) {
      message.error('调试操作失败');
    }
  };

  // 显示策略配置
  const showStrategyConfig = (strategy: Strategy) => {
    setSelectedStrategy(strategy);
    configForm.setFieldsValue(strategy.config);
    setConfigModalVisible(true);
  };

  // 保存策略配置
  const saveStrategyConfig = async (values: any) => {
    try {
      if (!selectedStrategy) return;
      
      setStrategies(prev => prev.map(s => 
        s.id === selectedStrategy.id ? { ...s, config: values, last_updated: new Date().toISOString() } : s
      ));
      
      setConfigModalVisible(false);
      message.success('配置更新成功');
    } catch (error) {
      message.error('配置更新失败');
    }
  };

  // 显示策略日志
  const showStrategyLogs = (strategy: Strategy) => {
    setSelectedStrategy(strategy);
    setLogsModalVisible(true);
  };

  // 显示策略指标
  const showStrategyMetrics = (strategy: Strategy) => {
    setSelectedStrategy(strategy);
    setMetricsModalVisible(true);
  };

  useEffect(() => {
    initializeData();
    // 每30秒刷新数据
    const interval = setInterval(initializeData, 30000);
    return () => clearInterval(interval);
  }, []);

  // 表格列定义
  const strategyColumns = [
    {
      title: '策略名称',
      dataIndex: 'name',
      key: 'name',
      render: (name: string, record: Strategy) => (
        <div>
          <Text strong>{name}</Text>
          <br />
          <Tag color={record.type === 'arbitrage' ? 'blue' : record.type === 'market_making' ? 'green' : record.type === 'trend' ? 'orange' : 'purple'}>
            {record.type}
          </Tag>
        </div>
      )
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => {
        const colors = {
          running: 'success',
          stopped: 'default',
          paused: 'warning',
          error: 'error'
        };
        return <Badge status={colors[status as keyof typeof colors]} text={status} />;
      }
    },
    {
      title: 'PnL',
      dataIndex: ['metrics', 'pnl'],
      key: 'pnl',
      render: (pnl: number) => (
        <Text style={{ color: pnl >= 0 ? '#3f8600' : '#cf1322' }}>
          {pnl >= 0 ? '+' : ''}{pnl.toFixed(2)}
        </Text>
      )
    },
    {
      title: '交易数',
      dataIndex: ['metrics', 'trades'],
      key: 'trades'
    },
    {
      title: '成功率',
      dataIndex: ['metrics', 'success_rate'],
      key: 'success_rate',
      render: (rate: number) => `${rate.toFixed(1)}%`
    },
    {
      title: '资源使用',
      key: 'resources',
      render: (_, record: Strategy) => (
        <div>
          <div>CPU: {record.metrics.cpu_usage.toFixed(1)}%</div>
          <div>内存: {record.metrics.memory_usage.toFixed(1)}MB</div>
        </div>
      )
    },
    {
      title: '功能',
      key: 'features',
      render: (_, record: Strategy) => (
        <Space>
          {record.hot_reload_enabled && <Tag color="orange">热重载</Tag>}
          {record.debug_enabled && <Tag color="red">调试</Tag>}
        </Space>
      )
    },
    {
      title: '操作',
      key: 'actions',
      render: (_, record: Strategy) => (
        <Space size="small" wrap>
          {record.status === 'stopped' && (
            <Button size="small" type="primary" icon={<PlayCircleOutlined />} 
                    onClick={() => handleStrategyLifecycle(record.id, 'start')}>
              启动
            </Button>
          )}
          {record.status === 'running' && (
            <>
              <Button size="small" icon={<PauseCircleOutlined />} 
                      onClick={() => handleStrategyLifecycle(record.id, 'pause')}>
                暂停
              </Button>
              <Button size="small" icon={<StopOutlined />} 
                      onClick={() => handleStrategyLifecycle(record.id, 'stop')}>
                停止
              </Button>
            </>
          )}
          {record.status === 'paused' && (
            <Button size="small" type="primary" icon={<PlayCircleOutlined />} 
                    onClick={() => handleStrategyLifecycle(record.id, 'resume')}>
              恢复
            </Button>
          )}
          <Button size="small" icon={<ReloadOutlined />} 
                  onClick={() => handleStrategyLifecycle(record.id, 'restart')}>
            重启
          </Button>
          <Button size="small" icon={<SettingOutlined />} 
                  onClick={() => showStrategyConfig(record)}>
            配置
          </Button>
          <Button size="small" icon={<FileTextOutlined />} 
                  onClick={() => showStrategyLogs(record)}>
            日志
          </Button>
          <Button size="small" icon={<LineChartOutlined />} 
                  onClick={() => showStrategyMetrics(record)}>
            指标
          </Button>
          {record.hot_reload_enabled && (
            <Button size="small" icon={<FireOutlined />} 
                    onClick={() => handleHotReload(record.id, 'reload')}>
              热重载
            </Button>
          )}
          {record.debug_enabled && (
            <Button size="small" icon={<BugOutlined />} 
                    onClick={() => {
                      setSelectedDebugStrategy(record.id);
                      setDebugDrawerVisible(true);
                    }}>
              调试
            </Button>
          )}
        </Space>
      )
    }
  ];

  const debugSessionColumns = [
    { title: '会话ID', dataIndex: 'id', key: 'id' },
    { title: '策略名称', dataIndex: 'strategy_name', key: 'strategy_name' },
    { title: '状态', dataIndex: 'status', key: 'status',
      render: (status: string) => <Badge status={status === 'active' ? 'success' : 'warning'} text={status} />
    },
    { title: '断点数', dataIndex: 'breakpoints', key: 'breakpoints' },
    { title: '创建时间', dataIndex: 'created_at', key: 'created_at',
      render: (time: string) => new Date(time).toLocaleString()
    },
    {
      title: '操作',
      key: 'actions',
      render: (_, record: DebugSession) => (
        <Space>
          <Button size="small">查看</Button>
          <Popconfirm title="确认删除?" onConfirm={() => handleDebugSession('delete', undefined, record.id)}>
            <Button size="small" danger>删除</Button>
          </Popconfirm>
        </Space>
      )
    }
  ];

  const hotReloadColumns = [
    { title: '策略名称', dataIndex: 'strategy_name', key: 'strategy_name' },
    { title: '操作类型', dataIndex: 'type', key: 'type',
      render: (type: string) => <Tag color={type === 'reload' ? 'blue' : 'orange'}>{type}</Tag>
    },
    { title: '状态', dataIndex: 'status', key: 'status',
      render: (status: string) => <Badge status={status === 'success' ? 'success' : 'error'} text={status} />
    },
    { title: '变更说明', dataIndex: 'changes', key: 'changes' },
    { title: '时间', dataIndex: 'timestamp', key: 'timestamp',
      render: (time: string) => new Date(time).toLocaleString()
    }
  ];

  return (
    <div style={{ padding: '24px' }}>
      <div style={{ marginBottom: '24px' }}>
        <Title level={2}>策略服务中心</Title>
        <Text type="secondary">策略生命周期管理、实时监控、调试工具、热重载功能</Text>
      </div>

      <Tabs 
        activeKey={activeTab} 
        onChange={setActiveTab} 
        size="large"
        items={[
          {
            key: 'overview',
            label: '系统概览',
            children: (
              <>
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="策略总数"
                        value={realtimeStatus.total_strategies}
                        suffix={`/ ${realtimeStatus.running_strategies}运行中`}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="总盈亏"
                        value={realtimeStatus.total_pnl}
                        precision={2}
                        valueStyle={{ color: realtimeStatus.total_pnl >= 0 ? '#3f8600' : '#cf1322' }}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="交易总数"
                        value={realtimeStatus.total_trades}
                      />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic
                        title="系统健康度"
                        value={realtimeStatus.system_health}
                        valueStyle={{ 
                          color: realtimeStatus.system_health === 'healthy' ? '#3f8600' : 
                                 realtimeStatus.system_health === 'warning' ? '#fa8c16' : '#cf1322'
                        }}
                      />
                    </Card>
                  </Col>
                </Row>

                <Row gutter={[16, 16]}>
                  <Col xs={24} md={12}>
                    <Card title="系统资源监控" size="small">
                      <div style={{ marginBottom: '16px' }}>
                        <div>CPU使用率</div>
                        <Progress percent={realtimeStatus.cpu_usage} />
                      </div>
                      <div style={{ marginBottom: '16px' }}>
                        <div>内存使用率</div>
                        <Progress percent={Math.round(realtimeStatus.memory_usage / 10)} />
                      </div>
                      <div>
                        <div>网络I/O</div>
                        <Progress percent={Math.round(realtimeStatus.network_io / 10)} />
                      </div>
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card title="活跃告警" size="small">
                      {alerts.filter(a => !a.resolved).map(alert => (
                        <Alert
                          key={alert.id}
                          message={alert.message}
                          type={alert.type}
                          showIcon
                          style={{ marginBottom: 8 }}
                          action={
                            <Button size="small" onClick={() => {
                              setAlerts(prev => prev.map(a => a.id === alert.id ? { ...a, resolved: true } : a));
                            }}>
                              解决
                            </Button>
                          }
                        />
                      ))}
                      {alerts.filter(a => !a.resolved).length === 0 && (
                        <Text type="secondary">暂无活跃告警</Text>
                      )}
                    </Card>
                  </Col>
                </Row>
              </>
            )
          },
          {
            key: 'strategies',
            label: `策略管理 (${strategies.length})`,
            children: (
              <Card
                title="策略列表"
                extra={
                  <Button icon={<ReloadOutlined />} onClick={initializeData} loading={loading}>
                    刷新
                  </Button>
                }
              >
                <Table
                  dataSource={strategies}
                  columns={strategyColumns}
                  rowKey="id"
                  loading={loading}
                  pagination={{ pageSize: 10 }}
                  scroll={{ x: 1200 }}
                />
              </Card>
            )
          },
          {
            key: 'monitoring',
            label: '实时监控',
            children: (
              <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                <Col xs={24} md={8}>
                  <Card title="性能概览" size="small">
                    <Statistic title="平均CPU使用率" value={realtimeStatus.cpu_usage} suffix="%" />
                    <Statistic title="平均内存使用" value={realtimeStatus.memory_usage} suffix="MB" />
                    <Statistic title="网络I/O" value={realtimeStatus.network_io} suffix="KB/s" />
                  </Card>
                </Col>
                <Col xs={24} md={8}>
                  <Card title="运行状态" size="small">
                    <div style={{ lineHeight: '2.5' }}>
                      <div>运行中策略: <Text strong>{realtimeStatus.running_strategies}</Text></div>
                      <div>暂停策略: <Text>{strategies.filter(s => s.status === 'paused').length}</Text></div>
                      <div>停止策略: <Text>{strategies.filter(s => s.status === 'stopped').length}</Text></div>
                      <div>异常策略: <Text type="danger">{strategies.filter(s => s.status === 'error').length}</Text></div>
                    </div>
                  </Card>
                </Col>
                <Col xs={24} md={8}>
                  <Card title="交易统计" size="small">
                    <Statistic title="今日交易量" value={realtimeStatus.total_trades} />
                    <Statistic 
                      title="今日盈亏" 
                      value={realtimeStatus.total_pnl} 
                      precision={2}
                      valueStyle={{ color: realtimeStatus.total_pnl >= 0 ? '#3f8600' : '#cf1322' }}
                    />
                  </Card>
                </Col>
              </Row>
            )
          },
          {
            key: 'debug',
            label: `调试工具 (${debugSessions.length})`,
            children: (
              <Card
                title="调试会话"
                extra={
                  <Select
                    placeholder="选择策略创建调试会话"
                    style={{ width: 200 }}
                    onChange={(strategyId) => handleDebugSession('create', strategyId)}
                  >
                    {strategies.filter(s => s.status === 'running').map(s => (
                      <Option key={s.id} value={s.id}>{s.name}</Option>
                    ))}
                  </Select>
                }
              >
                <Table
                  dataSource={debugSessions}
                  columns={debugSessionColumns}
                  rowKey="id"
                  loading={loading}
                  pagination={{ pageSize: 10 }}
                />
              </Card>
            )
          },
          {
            key: 'hotreload',
            label: `热重载 (${hotReloadHistory.length})`,
            children: (
              <Card title="热重载历史">
                <Table
                  dataSource={hotReloadHistory}
                  columns={hotReloadColumns}
                  rowKey="id"
                  loading={loading}
                  pagination={{ pageSize: 10 }}
                />
              </Card>
            )
          }
        ]}
      />

      {/* 策略配置模态框 */}
      <Modal
        title={`配置策略: ${selectedStrategy?.name}`}
        open={configModalVisible}
        onCancel={() => setConfigModalVisible(false)}
        onOk={() => configForm.submit()}
        width={600}
      >
        {selectedStrategy && (
          <Form form={configForm} onFinish={saveStrategyConfig} layout="vertical" initialValues={selectedStrategy?.config}>
            {Object.entries(selectedStrategy.config).map(([key, value]) => (
              <Form.Item key={key} name={key} label={key}>
                <Input />
              </Form.Item>
            ))}
          </Form>
        )}
      </Modal>

      {/* 策略日志模态框 */}
      <Modal
        title={`策略日志: ${selectedStrategy?.name}`}
        open={logsModalVisible}
        onCancel={() => setLogsModalVisible(false)}
        footer={null}
        width={800}
      >
        <div style={{ height: '400px', overflow: 'auto', backgroundColor: '#000', color: '#fff', padding: '16px', fontFamily: 'monospace' }}>
          <div>[2024-09-09 10:30:45] INFO: 策略启动完成</div>
          <div>[2024-09-09 10:30:46] INFO: 连接交易所...</div>
          <div>[2024-09-09 10:30:47] INFO: 开始监控价格差异</div>
          <div>[2024-09-09 10:31:12] INFO: 发现套利机会 BTCUSDT 0.02%</div>
          <div>[2024-09-09 10:31:13] INFO: 执行买入订单...</div>
          <div>[2024-09-09 10:31:15] SUCCESS: 订单执行成功 +12.45 USDT</div>
          <div>[2024-09-09 10:32:01] INFO: 继续监控中...</div>
        </div>
      </Modal>

      {/* 策略指标模态框 */}
      <Modal
        title={`策略指标: ${selectedStrategy?.name}`}
        open={metricsModalVisible}
        onCancel={() => setMetricsModalVisible(false)}
        footer={null}
        width={600}
      >
        {selectedStrategy && (
          <Descriptions bordered column={2}>
            <Descriptions.Item label="运行时间">{Math.floor(selectedStrategy.metrics.uptime / 3600)}小时</Descriptions.Item>
            <Descriptions.Item label="盈亏">
              <Text style={{ color: selectedStrategy.metrics.pnl >= 0 ? '#3f8600' : '#cf1322' }}>
                {selectedStrategy.metrics.pnl.toFixed(2)} USDT
              </Text>
            </Descriptions.Item>
            <Descriptions.Item label="交易数量">{selectedStrategy.metrics.trades}</Descriptions.Item>
            <Descriptions.Item label="成功率">{selectedStrategy.metrics.success_rate.toFixed(1)}%</Descriptions.Item>
            <Descriptions.Item label="CPU使用率">{selectedStrategy.metrics.cpu_usage.toFixed(1)}%</Descriptions.Item>
            <Descriptions.Item label="内存使用">{selectedStrategy.metrics.memory_usage.toFixed(1)}MB</Descriptions.Item>
          </Descriptions>
        )}
      </Modal>

      {/* 调试抽屉 */}
      <Drawer
        title="策略调试工具"
        open={debugDrawerVisible}
        onClose={() => setDebugDrawerVisible(false)}
        width={600}
      >
        <Tabs 
          items={[
            {
              key: 'breakpoints',
              label: '断点管理',
              children: (
                <Space direction="vertical" style={{ width: '100%' }}>
                  <Button type="primary" icon={<PlusOutlined />}>添加断点</Button>
                  <List
                    size="small"
                    dataSource={['第45行: 价格检查', '第78行: 订单执行', '第102行: 风险控制']}
                    renderItem={(item, index) => (
                      <List.Item
                        actions={[
                          <Button size="small" danger icon={<DeleteOutlined />}>删除</Button>
                        ]}
                      >
                        {item}
                      </List.Item>
                    )}
                  />
                </Space>
              )
            },
            {
              key: 'variables',
              label: '变量查看',
              children: (
                <pre style={{ fontSize: '12px' }}>
{`{
  "current_price": 45234.56,
  "target_profit": 0.02,
  "position_size": 0.1,
  "risk_level": "low",
  "last_trade_time": "2024-09-09T10:30:45Z"
}`}
                </pre>
              )
            },
            {
              key: 'stack',
              label: '调用栈',
              children: (
                <Timeline>
                  <Timeline.Item>main() - 策略主循环</Timeline.Item>
                  <Timeline.Item>check_arbitrage() - 检查套利机会</Timeline.Item>
                  <Timeline.Item color="red">execute_trade() - 执行交易 [当前]</Timeline.Item>
                  <Timeline.Item>validate_order() - 验证订单</Timeline.Item>
                </Timeline>
              )
            }
          ]}
        />
      </Drawer>
    </div>
  );
}