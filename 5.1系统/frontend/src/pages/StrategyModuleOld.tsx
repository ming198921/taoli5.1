import React, { useState, useEffect, useRef } from 'react';
import { Card, Row, Col, Button, Table, Tabs, Tag, Badge, Progress, Alert, Modal, Form, Input, Switch, Select, Statistic, notification } from 'antd';
import { 
  PlayCircleOutlined, 
  PauseCircleOutlined, 
  ReloadOutlined,
  SettingOutlined,
  BugOutlined,
  ThunderboltOutlined,
  StopOutlined,
  EditOutlined,
  EyeOutlined,
  BarChartOutlined
} from '@ant-design/icons';
import { Line, Area } from '@ant-design/charts';
import { strategyService } from '../services';
import { wsManager } from '../api/apiClient';

const { TabPane } = Tabs;

export default function StrategyModule() {
  const [loading, setLoading] = useState(false);
  const [strategies, setStrategies] = useState<any[]>([]);
  const [realtimeStatus, setRealtimeStatus] = useState<any>({});
  const [debugSessions, setDebugSessions] = useState<any[]>([]);
  const [selectedStrategy, setSelectedStrategy] = useState<any>(null);
  const [configModalVisible, setConfigModalVisible] = useState(false);
  const [logModalVisible, setLogModalVisible] = useState(false);
  const [metricsModalVisible, setMetricsModalVisible] = useState(false);
  const [hotReloadHistory, setHotReloadHistory] = useState<any[]>([]);
  const [strategyLogs, setStrategyLogs] = useState<any[]>([]);
  const [strategyMetrics, setStrategyMetrics] = useState<any>({});
  const wsConnectionRef = useRef<WebSocket | null>(null);
  const [form] = Form.useForm();

  // 获取策略列表
  const fetchStrategies = async () => {
    setLoading(true);
    try {
      const strategyList = await strategyService.listStrategies();
      // 确保strategyList是数组
      if (Array.isArray(strategyList)) {
        setStrategies(strategyList);
      } else {
        console.warn('strategyService.listStrategies() returned non-array:', strategyList);
        setStrategies([]);
      }
    } catch (error) {
      console.error('Failed to fetch strategies:', error);
      setStrategies([]);
    } finally {
      setLoading(false);
    }
  };

  // 获取实时状态
  const fetchRealtimeStatus = async () => {
    try {
      const status = await strategyService.getRealtimeStatus();
      setRealtimeStatus(status);
    } catch (error) {
      console.error('Failed to fetch realtime status:', error);
    }
  };

  // 启动策略
  const handleStartStrategy = async (id: string) => {
    try {
      await strategyService.startStrategy(id);
      fetchStrategies();
    } catch (error) {
      console.error('Failed to start strategy:', error);
    }
  };

  // 停止策略
  const handleStopStrategy = async (id: string) => {
    try {
      await strategyService.stopStrategy(id);
      fetchStrategies();
    } catch (error) {
      console.error('Failed to stop strategy:', error);
    }
  };

  // 热重载策略
  const handleHotReload = async (id: string) => {
    try {
      await strategyService.reloadStrategy(id);
      notification.success({ message: '热重载成功', description: `策略 ${id} 已成功重载` });
      fetchStrategies();
    } catch (error) {
      console.error('Failed to hot reload strategy:', error);
      notification.error({ message: '热重载失败', description: error.message });
    }
  };

  // 重启策略
  const handleRestartStrategy = async (id: string) => {
    try {
      await strategyService.restartStrategy(id);
      notification.success({ message: '重启成功', description: `策略 ${id} 已成功重启` });
      fetchStrategies();
    } catch (error) {
      console.error('Failed to restart strategy:', error);
      notification.error({ message: '重启失败', description: error.message });
    }
  };

  // 暂停策略
  const handlePauseStrategy = async (id: string) => {
    try {
      await strategyService.pauseStrategy(id);
      notification.success({ message: '暂停成功', description: `策略 ${id} 已暂停` });
      fetchStrategies();
    } catch (error) {
      console.error('Failed to pause strategy:', error);
      notification.error({ message: '暂停失败', description: error.message });
    }
  };

  // 恢复策略
  const handleResumeStrategy = async (id: string) => {
    try {
      await strategyService.resumeStrategy(id);
      notification.success({ message: '恢复成功', description: `策略 ${id} 已恢复运行` });
      fetchStrategies();
    } catch (error) {
      console.error('Failed to resume strategy:', error);
      notification.error({ message: '恢复失败', description: error.message });
    }
  };

  // 查看策略配置
  const handleViewConfig = async (strategy: any) => {
    try {
      const config = await strategyService.getStrategyConfig(strategy.id);
      setSelectedStrategy({ ...strategy, config });
      form.setFieldsValue(config);
      setConfigModalVisible(true);
    } catch (error) {
      console.error('Failed to fetch strategy config:', error);
      notification.error({ message: '获取配置失败', description: error.message });
    }
  };

  // 查看策略日志
  const handleViewLogs = async (strategy: any) => {
    try {
      const logs = await strategyService.getStrategyLogs(strategy.id);
      setStrategyLogs(logs);
      setSelectedStrategy(strategy);
      setLogModalVisible(true);
    } catch (error) {
      console.error('Failed to fetch strategy logs:', error);
      notification.error({ message: '获取日志失败', description: error.message });
    }
  };

  // 查看策略指标
  const handleViewMetrics = async (strategy: any) => {
    try {
      const metrics = await strategyService.getStrategyMetrics(strategy.id);
      setStrategyMetrics(metrics);
      setSelectedStrategy(strategy);
      setMetricsModalVisible(true);
    } catch (error) {
      console.error('Failed to fetch strategy metrics:', error);
      notification.error({ message: '获取指标失败', description: error.message });
    }
  };

  // 更新策略配置
  const handleUpdateConfig = async () => {
    try {
      const values = await form.validateFields();
      await strategyService.updateStrategyConfig(selectedStrategy.id, values);
      notification.success({ message: '配置更新成功', description: `策略 ${selectedStrategy.id} 配置已更新` });
      setConfigModalVisible(false);
      fetchStrategies();
    } catch (error) {
      console.error('Failed to update strategy config:', error);
      notification.error({ message: '配置更新失败', description: error.message });
    }
  };

  // 获取热重载历史
  const fetchHotReloadHistory = async () => {
    try {
      const history = await strategyService.getHotReloadHistory();
      setHotReloadHistory(history);
    } catch (error) {
      console.error('Failed to fetch hot reload history:', error);
    }
  };

  useEffect(() => {
    fetchStrategies();
    fetchRealtimeStatus();
    fetchHotReloadHistory();
    
    // 建立WebSocket连接实时更新
    wsConnectionRef.current = wsManager.connect(
      '/strategies/realtime',
      (data) => {
        if (data.type === 'strategy_update') {
          setStrategies(prev => prev.map(s => 
            s.id === data.strategy_id ? { ...s, ...data.updates } : s
          ));
        } else if (data.type === 'realtime_status') {
          setRealtimeStatus(data.status);
        }
      },
      (error) => {
        console.error('WebSocket connection error:', error);
        notification.warning({
          message: 'WebSocket连接断开',
          description: '实时数据更新已暂停，将使用定期刷新'
        });
      }
    );
    
    // 定时刷新实时状态（WebSocket备用方案）
    const interval = setInterval(fetchRealtimeStatus, 10000);
    
    return () => {
      clearInterval(interval);
      if (wsConnectionRef.current) {
        wsManager.disconnect('/strategies/realtime');
      }
    };
  }, []);

  // 策略表格列
  const strategyColumns = [
    {
      title: '策略名称',
      dataIndex: 'name',
      key: 'name'
    },
    {
      title: '类型',
      dataIndex: 'type',
      key: 'type',
      render: (type: string) => {
        const colors = {
          triangular: 'blue',
          arbitrage: 'green',
          grid: 'orange',
          market_making: 'purple'
        };
        return <Tag color={colors[type as keyof typeof colors]}>{type}</Tag>;
      }
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
      title: '盈利',
      dataIndex: 'performance',
      key: 'profit',
      render: (performance: any) => (
        <span style={{ color: performance?.profit > 0 ? '#3f8600' : '#cf1322' }}>
          {performance?.profit?.toFixed(2) || '0.00'}
        </span>
      )
    },
    {
      title: '成功率',
      dataIndex: 'performance',
      key: 'success_rate',
      render: (performance: any) => (
        <Progress 
          percent={Math.round((performance?.success_rate || 0) * 100)} 
          size="small" 
          style={{ width: 100 }}
        />
      )
    },
    {
      title: '操作',
      key: 'actions',
      width: 280,
      render: (_, record) => (
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '4px' }}>
          {/* 主要操作 */}
          {record.status === 'running' ? (
            <Button 
              size="small" 
              danger
              icon={<StopOutlined />}
              onClick={() => handleStopStrategy(record.id)}
            >
              停止
            </Button>
          ) : record.status === 'paused' ? (
            <Button 
              size="small" 
              type="primary"
              icon={<PlayCircleOutlined />}
              onClick={() => handleResumeStrategy(record.id)}
            >
              恢复
            </Button>
          ) : (
            <Button 
              size="small" 
              type="primary"
              icon={<PlayCircleOutlined />}
              onClick={() => handleStartStrategy(record.id)}
            >
              启动
            </Button>
          )}
          
          {/* 暂停按钮 */}
          {record.status === 'running' && (
            <Button 
              size="small" 
              icon={<PauseCircleOutlined />}
              onClick={() => handlePauseStrategy(record.id)}
            >
              暂停
            </Button>
          )}
          
          {/* 重启按钮 */}
          <Button 
            size="small" 
            icon={<ReloadOutlined />}
            onClick={() => handleRestartStrategy(record.id)}
          >
            重启
          </Button>
          
          {/* 热重载 */}
          <Button 
            size="small" 
            icon={<ThunderboltOutlined />}
            onClick={() => handleHotReload(record.id)}
          >
            热重载
          </Button>
          
          {/* 配置 */}
          <Button 
            size="small" 
            icon={<SettingOutlined />}
            onClick={() => handleViewConfig(record)}
          >
            配置
          </Button>
          
          {/* 日志 */}
          <Button 
            size="small" 
            icon={<EyeOutlined />}
            onClick={() => handleViewLogs(record)}
          >
            日志
          </Button>
          
          {/* 指标 */}
          <Button 
            size="small" 
            icon={<BarChartOutlined />}
            onClick={() => handleViewMetrics(record)}
          >
            指标
          </Button>
        </div>
      )
    }
  ];

  return (
    <div style={{ padding: '24px' }}>
      {/* 页面标题 */}
      <div style={{ marginBottom: '24px' }}>
        <h1 style={{ margin: 0, fontSize: '24px', fontWeight: 'bold' }}>
          策略服务管理 - 38个API接口
        </h1>
        <p style={{ margin: '8px 0 0 0', color: '#666' }}>
          端口: 4003 | 策略生命周期管理、实时监控、热重载
        </p>
      </div>

      {/* 实时状态概览 */}
      <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ textAlign: 'center' }}>
              <div style={{ fontSize: '24px', fontWeight: 'bold', color: '#52c41a' }}>
                {strategies.filter(s => s.status === 'running').length}
              </div>
              <div style={{ color: '#666' }}>运行中策略</div>
            </div>
          </Card>
        </Col>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ textAlign: 'center' }}>
              <div style={{ fontSize: '24px', fontWeight: 'bold' }}>
                {strategies.length}
              </div>
              <div style={{ color: '#666' }}>总策略数</div>
            </div>
          </Card>
        </Col>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ textAlign: 'center' }}>
              <div style={{ fontSize: '24px', fontWeight: 'bold', color: '#1890ff' }}>
                {realtimeStatus.cpu_usage || 0}%
              </div>
              <div style={{ color: '#666' }}>CPU使用率</div>
            </div>
          </Card>
        </Col>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ textAlign: 'center' }}>
              <div style={{ fontSize: '24px', fontWeight: 'bold', color: '#fa8c16' }}>
                {realtimeStatus.memory_usage || 0}MB
              </div>
              <div style={{ color: '#666' }}>内存使用</div>
            </div>
          </Card>
        </Col>
      </Row>

      {/* 主要内容标签页 */}
      <Tabs defaultActiveKey="strategies" size="large">
        {/* 策略管理 */}
        <TabPane tab={`策略列表 (${strategies.length})`} key="strategies">
          <Card 
            title="策略生命周期管理"
            extra={
              <Button icon={<ReloadOutlined />} onClick={fetchStrategies} loading={loading}>
                刷新
              </Button>
            }
          >
            <Table
              dataSource={strategies}
              columns={strategyColumns}
              rowKey="id"
              loading={loading}
              pagination={{
                pageSize: 10,
                showTotal: (total) => `共 ${total} 个策略`
              }}
            />
          </Card>
        </TabPane>

        {/* 实时监控 */}
        <TabPane tab="实时监控" key="monitoring">
          <Row gutter={[16, 16]}>
            <Col xs={24} md={12}>
              <Card title="系统健康状态" size="small">
                <div style={{ lineHeight: '2.5' }}>
                  <div>CPU使用率: <Progress percent={realtimeStatus.cpu_usage || 0} size="small" style={{ width: 200 }} /></div>
                  <div>内存使用: <Progress percent={realtimeStatus.memory_usage || 0} size="small" style={{ width: 200 }} /></div>
                  <div>网络I/O: <Progress percent={realtimeStatus.network_io || 0} size="small" style={{ width: 200 }} /></div>
                </div>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="性能指标" size="small">
                <div style={{ lineHeight: '2.5' }}>
                  <div>每秒交易数: {realtimeStatus.trades_per_second || 0}</div>
                  <div>平均延迟: {realtimeStatus.avg_latency || 0}ms</div>
                  <div>错误率: {realtimeStatus.error_rate || 0}%</div>
                </div>
              </Card>
            </Col>
            <Col xs={24}>
              <Card title="活跃告警" size="small">
                {realtimeStatus.alerts?.length > 0 ? (
                  realtimeStatus.alerts.map((alert: any, index: number) => (
                    <Alert
                      key={index}
                      message={alert.message}
                      type={alert.type}
                      showIcon
                      style={{ marginBottom: 8 }}
                    />
                  ))
                ) : (
                  <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                    暂无活跃告警
                  </div>
                )}
              </Card>
            </Col>
          </Row>
        </TabPane>

        {/* 调试工具 */}
        <TabPane tab="调试工具" key="debug">
          <Card 
            title="调试会话管理"
            extra={
              <Button icon={<BugOutlined />} type="primary">
                新建调试会话
              </Button>
            }
          >
            <div style={{ textAlign: 'center', padding: '40px', color: '#666' }}>
              调试工具开发中...
              <br />
              将提供断点设置、变量查看、调用栈分析等功能
            </div>
          </Card>
        </TabPane>

        {/* 热重载管理 */}
        <TabPane tab="热重载" key="hotreload">
          <Row gutter={[16, 16]}>
            <Col xs={24} md={12}>
              <Card title="热重载状态" size="small" extra={
                <Button size="small" onClick={fetchHotReloadHistory}>刷新</Button>
              }>
                <div style={{ lineHeight: '2.5' }}>
                  <div>热重载功能: <Badge status="success" text="已启用" /></div>
                  <div>最后重载: {hotReloadHistory[0]?.timestamp || '未知'}</div>
                  <div>重载次数: {hotReloadHistory.length}</div>
                  <div>成功率: {hotReloadHistory.length > 0 ? 
                    Math.round(hotReloadHistory.filter(h => h.status === 'success').length / hotReloadHistory.length * 100) : 0}%
                  </div>
                </div>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="重载操作" size="small">
                <div style={{ display: 'flex', gap: '8px', flexDirection: 'column' }}>
                  <Button 
                    type="primary" 
                    icon={<ThunderboltOutlined />}
                    onClick={() => {
                      strategies.forEach(s => handleHotReload(s.id));
                    }}
                  >
                    全量重载所有策略
                  </Button>
                  <Button 
                    icon={<ReloadOutlined />}
                    onClick={fetchHotReloadHistory}
                  >
                    刷新重载历史
                  </Button>
                </div>
              </Card>
            </Col>
            <Col xs={24}>
              <Card title="重载历史记录" size="small">
                <Table
                  size="small"
                  dataSource={hotReloadHistory.slice(0, 10)}
                  pagination={false}
                  columns={[
                    {
                      title: '时间',
                      dataIndex: 'timestamp',
                      key: 'timestamp',
                      width: 200
                    },
                    {
                      title: '策略ID',
                      dataIndex: 'strategy_id',
                      key: 'strategy_id',
                      width: 150
                    },
                    {
                      title: '状态',
                      dataIndex: 'status',
                      key: 'status',
                      width: 100,
                      render: (status: string) => (
                        <Badge 
                          status={status === 'success' ? 'success' : 'error'} 
                          text={status === 'success' ? '成功' : '失败'} 
                        />
                      )
                    },
                    {
                      title: '耗时',
                      dataIndex: 'duration',
                      key: 'duration',
                      width: 100,
                      render: (duration: number) => `${duration}ms`
                    },
                    {
                      title: '信息',
                      dataIndex: 'message',
                      key: 'message',
                      ellipsis: true
                    }
                  ]}
                  scroll={{ y: 300 }}
                />
              </Card>
            </Col>
          </Row>
        </TabPane>
      </Tabs>

      {/* 策略配置弹窗 */}
      <Modal
        title={`配置策略: ${selectedStrategy?.name}`}
        open={configModalVisible}
        onOk={handleUpdateConfig}
        onCancel={() => setConfigModalVisible(false)}
        width={800}
        destroyOnClose
      >
        <Form form={form} layout="vertical">
          <Row gutter={16}>
            <Col span={12}>
              <Form.Item label="策略类型" name="type">
                <Select>
                  <Select.Option value="triangular">三角套利</Select.Option>
                  <Select.Option value="arbitrage">简单套利</Select.Option>
                  <Select.Option value="grid">网格交易</Select.Option>
                  <Select.Option value="market_making">做市</Select.Option>
                </Select>
              </Form.Item>
            </Col>
            <Col span={12}>
              <Form.Item label="最大仓位" name="max_position">
                <Input type="number" suffix="%" />
              </Form.Item>
            </Col>
          </Row>
          <Row gutter={16}>
            <Col span={12}>
              <Form.Item label="风险阈值" name="risk_threshold">
                <Input type="number" suffix="%" />
              </Form.Item>
            </Col>
            <Col span={12}>
              <Form.Item label="启用热重载" name="hot_reload_enabled" valuePropName="checked">
                <Switch />
              </Form.Item>
            </Col>
          </Row>
          <Form.Item label="交易对" name="trading_pairs">
            <Select mode="multiple" placeholder="选择交易对">
              <Select.Option value="BTC/USDT">BTC/USDT</Select.Option>
              <Select.Option value="ETH/USDT">ETH/USDT</Select.Option>
              <Select.Option value="BNB/USDT">BNB/USDT</Select.Option>
            </Select>
          </Form.Item>
          <Form.Item label="策略参数" name="parameters">
            <Input.TextArea rows={4} placeholder="JSON格式的策略参数" />
          </Form.Item>
        </Form>
      </Modal>

      {/* 策略日志弹窗 */}
      <Modal
        title={`策略日志: ${selectedStrategy?.name}`}
        open={logModalVisible}
        onCancel={() => setLogModalVisible(false)}
        footer={null}
        width={900}
        destroyOnClose
      >
        <div style={{ maxHeight: '400px', overflowY: 'auto' }}>
          {strategyLogs.map((log, index) => (
            <div key={index} style={{ 
              padding: '8px', 
              borderBottom: '1px solid #f0f0f0',
              fontFamily: 'monospace'
            }}>
              <span style={{ color: '#666' }}>[{log.timestamp}]</span>{' '}
              <span style={{ 
                color: log.level === 'error' ? '#ff4d4f' : 
                       log.level === 'warn' ? '#faad14' : '#52c41a' 
              }}>
                {log.level.toUpperCase()}
              </span>{' '}
              {log.message}
            </div>
          ))}
        </div>
      </Modal>

      {/* 策略指标弹窗 */}
      <Modal
        title={`策略指标: ${selectedStrategy?.name}`}
        open={metricsModalVisible}
        onCancel={() => setMetricsModalVisible(false)}
        footer={null}
        width={800}
        destroyOnClose
      >
        <Row gutter={[16, 16]}>
          <Col span={8}>
            <Statistic 
              title="CPU使用率" 
              value={strategyMetrics.cpu_usage || 0} 
              precision={1} 
              suffix="%" 
            />
          </Col>
          <Col span={8}>
            <Statistic 
              title="内存使用" 
              value={strategyMetrics.memory_usage || 0} 
              precision={0} 
              suffix="MB" 
            />
          </Col>
          <Col span={8}>
            <Statistic 
              title="网络使用" 
              value={strategyMetrics.network_usage || 0} 
              precision={2} 
              suffix="Mb/s" 
            />
          </Col>
          <Col span={8}>
            <Statistic 
              title="每秒交易数" 
              value={strategyMetrics.trades_per_second || 0} 
              precision={0} 
            />
          </Col>
          <Col span={8}>
            <Statistic 
              title="平均延迟" 
              value={strategyMetrics.latency || 0} 
              precision={0} 
              suffix="ms" 
            />
          </Col>
          <Col span={8}>
            <Statistic 
              title="错误率" 
              value={strategyMetrics.error_rate || 0} 
              precision={2} 
              suffix="%" 
            />
          </Col>
        </Row>
        
        <Card title="历史性能图表" style={{ marginTop: 16 }}>
          <div style={{ textAlign: 'center', padding: '40px', color: '#666' }}>
            图表组件开发中...
            <br />
            将显示CPU、内存、延迟等历史数据图表
          </div>
        </Card>
      </Modal>
    </div>
  );
} 