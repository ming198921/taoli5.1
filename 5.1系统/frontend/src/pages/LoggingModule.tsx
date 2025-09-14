import React, { useState, useEffect } from 'react';
import { Card, Row, Col, Button, Table, Tabs, Input, Select, Tag, Badge, Alert, Switch } from 'antd';
import { 
  SearchOutlined, 
  ReloadOutlined, 
  DownloadOutlined,
  PlayCircleOutlined,
  PauseCircleOutlined,
  SettingOutlined
} from '@ant-design/icons';
import { loggingService } from '../services';

const { TabPane } = Tabs;
const { Search } = Input;
const { Option } = Select;

interface LogEntry {
  id: string;
  timestamp: string;
  level: string;
  service: string;
  module: string;
  message: string;
}

export default function LoggingModule() {
  const [loading, setLoading] = useState(false);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [realtimeLogs, setRealtimeLogs] = useState<LogEntry[]>([]);
  const [logConfig, setLogConfig] = useState<any>({});
  const [streamStatus, setStreamStatus] = useState<'running' | 'paused'>('running');
  const [wsConnection, setWsConnection] = useState<WebSocket | null>(null);

  // 日志级别选项
  const logLevels = ['debug', 'info', 'warn', 'error'];
  
  // 服务列表
  const services = [
    'logging-service', 'cleaning-service', 'strategy-service', 
    'performance-service', 'trading-service', 'ai-model-service', 'config-service'
  ];

  // 获取日志数据
  const fetchLogs = async () => {
    setLoading(true);
    try {
      const logData = await loggingService.getRealtimeLogStream();
      setLogs(logData);
    } catch (error) {
      console.error('Failed to fetch logs:', error);
    } finally {
      setLoading(false);
    }
  };

  // 获取日志配置
  const fetchLogConfig = async () => {
    try {
      const config = await loggingService.getLogLevels();
      setLogConfig(config);
    } catch (error) {
      console.error('Failed to fetch log config:', error);
    }
  };

  // 连接实时日志流
  const connectRealtimeStream = () => {
    const ws = loggingService.connectRealtimeLogs(
      (logData) => {
        setRealtimeLogs(prev => [logData, ...prev.slice(0, 99)]); // 保留最新100条
      },
      (error) => {
        console.error('WebSocket error:', error);
      }
    );
    setWsConnection(ws);
  };

  // 断开实时日志流
  const disconnectRealtimeStream = () => {
    if (wsConnection) {
      wsConnection.close();
      setWsConnection(null);
    }
  };

  // 暂停/恢复日志流
  const toggleLogStream = async () => {
    try {
      if (streamStatus === 'running') {
        await loggingService.pauseLogStream();
        setStreamStatus('paused');
      } else {
        await loggingService.resumeLogStream();
        setStreamStatus('running');
      }
    } catch (error) {
      console.error('Failed to toggle log stream:', error);
    }
  };

  // 搜索日志
  const handleSearch = async (query: string) => {
    if (!query.trim()) {
      fetchLogs();
      return;
    }
    
    try {
      setLoading(true);
      const results = await loggingService.searchLogs(query);
      setLogs(results);
    } catch (error) {
      console.error('Failed to search logs:', error);
    } finally {
      setLoading(false);
    }
  };

  // 按服务过滤日志
  const handleFilterByService = async (service: string) => {
    try {
      setLoading(true);
      const results = await loggingService.getLogsByService(service);
      setLogs(results);
    } catch (error) {
      console.error('Failed to filter logs by service:', error);
    } finally {
      setLoading(false);
    }
  };

  // 按级别过滤日志
  const handleFilterByLevel = async (level: string) => {
    try {
      setLoading(true);
      const results = await loggingService.getLogsByLevel(level);
      setLogs(results);
    } catch (error) {
      console.error('Failed to filter logs by level:', error);
    } finally {
      setLoading(false);
    }
  };

  // 导出日志
  const handleExportLogs = async () => {
    try {
      const exportData = await loggingService.exportLogs('json');
      // 创建下载链接
      const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `logs-${new Date().toISOString().split('T')[0]}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Failed to export logs:', error);
    }
  };

  // 设置日志级别
  const handleSetLogLevel = async (service: string, level: string) => {
    try {
      await loggingService.setServiceLogLevel(service, level);
      fetchLogConfig();
    } catch (error) {
      console.error('Failed to set log level:', error);
    }
  };

  useEffect(() => {
    fetchLogs();
    fetchLogConfig();
    connectRealtimeStream();
    
    return () => {
      disconnectRealtimeStream();
    };
  }, []);

  // 日志表格列定义
  const logColumns = [
    {
      title: '时间',
      dataIndex: 'timestamp',
      key: 'timestamp',
      width: 180,
      render: (timestamp: string) => new Date(timestamp).toLocaleString()
    },
    {
      title: '级别',
      dataIndex: 'level',
      key: 'level',
      width: 80,
      render: (level: string) => {
        const color = {
          debug: 'default',
          info: 'blue',
          warn: 'orange',
          error: 'red'
        }[level] || 'default';
        return <Tag color={color}>{level.toUpperCase()}</Tag>;
      }
    },
    {
      title: '服务',
      dataIndex: 'service',
      key: 'service',
      width: 150,
      render: (service: string) => <Badge text={service} />
    },
    {
      title: '模块',
      dataIndex: 'module',
      key: 'module',
      width: 120
    },
    {
      title: '消息',
      dataIndex: 'message',
      key: 'message',
      ellipsis: true
    }
  ];

  return (
    <div style={{ padding: '24px' }}>
      {/* 页面标题 */}
      <div style={{ marginBottom: '24px' }}>
        <h1 style={{ margin: 0, fontSize: '24px', fontWeight: 'bold' }}>
          日志服务管理 - 45个API接口
        </h1>
        <p style={{ margin: '8px 0 0 0', color: '#666' }}>
          端口: 4001 | 实时日志流、配置管理、分析工具
        </p>
      </div>

      {/* 操作工具栏 */}
      <Card style={{ marginBottom: '16px' }}>
        <Row gutter={[16, 16]} align="middle">
          <Col flex="auto">
            <Search
              placeholder="搜索日志内容..."
              onSearch={handleSearch}
              style={{ maxWidth: 400 }}
              enterButton={<SearchOutlined />}
            />
          </Col>
          <Col>
            <Select
              placeholder="按服务过滤"
              style={{ width: 150 }}
              allowClear
              onChange={handleFilterByService}
            >
              {services.map(service => (
                <Option key={service} value={service}>{service}</Option>
              ))}
            </Select>
          </Col>
          <Col>
            <Select
              placeholder="按级别过滤"
              style={{ width: 120 }}
              allowClear
              onChange={handleFilterByLevel}
            >
              {logLevels.map(level => (
                <Option key={level} value={level}>{level.toUpperCase()}</Option>
              ))}
            </Select>
          </Col>
          <Col>
            <Button
              icon={streamStatus === 'running' ? <PauseCircleOutlined /> : <PlayCircleOutlined />}
              onClick={toggleLogStream}
              type={streamStatus === 'running' ? 'primary' : 'default'}
            >
              {streamStatus === 'running' ? '暂停' : '恢复'}流
            </Button>
          </Col>
          <Col>
            <Button icon={<ReloadOutlined />} onClick={fetchLogs} loading={loading}>
              刷新
            </Button>
          </Col>
          <Col>
            <Button icon={<DownloadOutlined />} onClick={handleExportLogs}>
              导出
            </Button>
          </Col>
        </Row>
      </Card>

      {/* 实时状态提示 */}
      {wsConnection && (
        <Alert
          message="实时日志流已连接"
          description={`WebSocket连接正常，实时接收日志数据。当前缓存 ${realtimeLogs.length} 条实时日志。`}
          type="success"
          showIcon
          closable
          style={{ marginBottom: '16px' }}
        />
      )}

      {/* 主要内容标签页 */}
      <Tabs defaultActiveKey="realtime" size="large">
        {/* 实时日志 */}
        <TabPane tab={`实时日志 (${realtimeLogs.length})`} key="realtime">
          <Table
            dataSource={realtimeLogs}
            columns={logColumns}
            rowKey="id"
            loading={loading}
            size="small"
            pagination={{
              pageSize: 50,
              showSizeChanger: true,
              showQuickJumper: true,
              showTotal: (total) => `共 ${total} 条日志`
            }}
            scroll={{ y: 600 }}
          />
        </TabPane>

        {/* 历史日志 */}
        <TabPane tab={`历史日志 (${logs.length})`} key="history">
          <Table
            dataSource={logs}
            columns={logColumns}
            rowKey="id"
            loading={loading}
            size="small"
            pagination={{
              pageSize: 50,
              showSizeChanger: true,
              showQuickJumper: true,
              showTotal: (total) => `共 ${total} 条日志`
            }}
            scroll={{ y: 600 }}
          />
        </TabPane>

        {/* 日志配置 */}
        <TabPane tab="配置管理" key="config">
          <Card title="日志级别配置" extra={<Button icon={<SettingOutlined />}>高级设置</Button>}>
            <Row gutter={[16, 16]}>
              {services.map(service => (
                <Col xs={24} sm={12} md={8} key={service}>
                  <Card size="small" title={service}>
                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                      <span>日志级别:</span>
                      <Select
                        value={logConfig[service] || 'info'}
                        style={{ width: 100 }}
                        onChange={(level) => handleSetLogLevel(service, level)}
                      >
                        {logLevels.map(level => (
                          <Option key={level} value={level}>{level.toUpperCase()}</Option>
                        ))}
                      </Select>
                    </div>
                  </Card>
                </Col>
              ))}
            </Row>
          </Card>
        </TabPane>

        {/* 日志分析 */}
        <TabPane tab="日志分析" key="analysis">
          <Row gutter={[16, 16]}>
            <Col xs={24} md={12}>
              <Card title="错误统计" size="small">
                <p>正在开发中... 将提供错误日志分析功能</p>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="性能分析" size="small">
                <p>正在开发中... 将提供性能日志分析功能</p>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="趋势分析" size="small">
                <p>正在开发中... 将提供日志趋势分析功能</p>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="异常检测" size="small">
                <p>正在开发中... 将提供异常检测功能</p>
              </Card>
            </Col>
          </Row>
        </TabPane>
      </Tabs>
    </div>
  );
} 
import { Card, Row, Col, Button, Table, Tabs, Input, Select, Tag, Badge, Alert, Switch } from 'antd';
import { 
  SearchOutlined, 
  ReloadOutlined, 
  DownloadOutlined,
  PlayCircleOutlined,
  PauseCircleOutlined,
  SettingOutlined
} from '@ant-design/icons';
import { loggingService } from '../services';

const { TabPane } = Tabs;
const { Search } = Input;
const { Option } = Select;

interface LogEntry {
  id: string;
  timestamp: string;
  level: string;
  service: string;
  module: string;
  message: string;
}

export default function LoggingModule() {
  const [loading, setLoading] = useState(false);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [realtimeLogs, setRealtimeLogs] = useState<LogEntry[]>([]);
  const [logConfig, setLogConfig] = useState<any>({});
  const [streamStatus, setStreamStatus] = useState<'running' | 'paused'>('running');
  const [wsConnection, setWsConnection] = useState<WebSocket | null>(null);

  // 日志级别选项
  const logLevels = ['debug', 'info', 'warn', 'error'];
  
  // 服务列表
  const services = [
    'logging-service', 'cleaning-service', 'strategy-service', 
    'performance-service', 'trading-service', 'ai-model-service', 'config-service'
  ];

  // 获取日志数据
  const fetchLogs = async () => {
    setLoading(true);
    try {
      const logData = await loggingService.getRealtimeLogStream();
      setLogs(logData);
    } catch (error) {
      console.error('Failed to fetch logs:', error);
    } finally {
      setLoading(false);
    }
  };

  // 获取日志配置
  const fetchLogConfig = async () => {
    try {
      const config = await loggingService.getLogLevels();
      setLogConfig(config);
    } catch (error) {
      console.error('Failed to fetch log config:', error);
    }
  };

  // 连接实时日志流
  const connectRealtimeStream = () => {
    const ws = loggingService.connectRealtimeLogs(
      (logData) => {
        setRealtimeLogs(prev => [logData, ...prev.slice(0, 99)]); // 保留最新100条
      },
      (error) => {
        console.error('WebSocket error:', error);
      }
    );
    setWsConnection(ws);
  };

  // 断开实时日志流
  const disconnectRealtimeStream = () => {
    if (wsConnection) {
      wsConnection.close();
      setWsConnection(null);
    }
  };

  // 暂停/恢复日志流
  const toggleLogStream = async () => {
    try {
      if (streamStatus === 'running') {
        await loggingService.pauseLogStream();
        setStreamStatus('paused');
      } else {
        await loggingService.resumeLogStream();
        setStreamStatus('running');
      }
    } catch (error) {
      console.error('Failed to toggle log stream:', error);
    }
  };

  // 搜索日志
  const handleSearch = async (query: string) => {
    if (!query.trim()) {
      fetchLogs();
      return;
    }
    
    try {
      setLoading(true);
      const results = await loggingService.searchLogs(query);
      setLogs(results);
    } catch (error) {
      console.error('Failed to search logs:', error);
    } finally {
      setLoading(false);
    }
  };

  // 按服务过滤日志
  const handleFilterByService = async (service: string) => {
    try {
      setLoading(true);
      const results = await loggingService.getLogsByService(service);
      setLogs(results);
    } catch (error) {
      console.error('Failed to filter logs by service:', error);
    } finally {
      setLoading(false);
    }
  };

  // 按级别过滤日志
  const handleFilterByLevel = async (level: string) => {
    try {
      setLoading(true);
      const results = await loggingService.getLogsByLevel(level);
      setLogs(results);
    } catch (error) {
      console.error('Failed to filter logs by level:', error);
    } finally {
      setLoading(false);
    }
  };

  // 导出日志
  const handleExportLogs = async () => {
    try {
      const exportData = await loggingService.exportLogs('json');
      // 创建下载链接
      const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `logs-${new Date().toISOString().split('T')[0]}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Failed to export logs:', error);
    }
  };

  // 设置日志级别
  const handleSetLogLevel = async (service: string, level: string) => {
    try {
      await loggingService.setServiceLogLevel(service, level);
      fetchLogConfig();
    } catch (error) {
      console.error('Failed to set log level:', error);
    }
  };

  useEffect(() => {
    fetchLogs();
    fetchLogConfig();
    connectRealtimeStream();
    
    return () => {
      disconnectRealtimeStream();
    };
  }, []);

  // 日志表格列定义
  const logColumns = [
    {
      title: '时间',
      dataIndex: 'timestamp',
      key: 'timestamp',
      width: 180,
      render: (timestamp: string) => new Date(timestamp).toLocaleString()
    },
    {
      title: '级别',
      dataIndex: 'level',
      key: 'level',
      width: 80,
      render: (level: string) => {
        const color = {
          debug: 'default',
          info: 'blue',
          warn: 'orange',
          error: 'red'
        }[level] || 'default';
        return <Tag color={color}>{level.toUpperCase()}</Tag>;
      }
    },
    {
      title: '服务',
      dataIndex: 'service',
      key: 'service',
      width: 150,
      render: (service: string) => <Badge text={service} />
    },
    {
      title: '模块',
      dataIndex: 'module',
      key: 'module',
      width: 120
    },
    {
      title: '消息',
      dataIndex: 'message',
      key: 'message',
      ellipsis: true
    }
  ];

  return (
    <div style={{ padding: '24px' }}>
      {/* 页面标题 */}
      <div style={{ marginBottom: '24px' }}>
        <h1 style={{ margin: 0, fontSize: '24px', fontWeight: 'bold' }}>
          日志服务管理 - 45个API接口
        </h1>
        <p style={{ margin: '8px 0 0 0', color: '#666' }}>
          端口: 4001 | 实时日志流、配置管理、分析工具
        </p>
      </div>

      {/* 操作工具栏 */}
      <Card style={{ marginBottom: '16px' }}>
        <Row gutter={[16, 16]} align="middle">
          <Col flex="auto">
            <Search
              placeholder="搜索日志内容..."
              onSearch={handleSearch}
              style={{ maxWidth: 400 }}
              enterButton={<SearchOutlined />}
            />
          </Col>
          <Col>
            <Select
              placeholder="按服务过滤"
              style={{ width: 150 }}
              allowClear
              onChange={handleFilterByService}
            >
              {services.map(service => (
                <Option key={service} value={service}>{service}</Option>
              ))}
            </Select>
          </Col>
          <Col>
            <Select
              placeholder="按级别过滤"
              style={{ width: 120 }}
              allowClear
              onChange={handleFilterByLevel}
            >
              {logLevels.map(level => (
                <Option key={level} value={level}>{level.toUpperCase()}</Option>
              ))}
            </Select>
          </Col>
          <Col>
            <Button
              icon={streamStatus === 'running' ? <PauseCircleOutlined /> : <PlayCircleOutlined />}
              onClick={toggleLogStream}
              type={streamStatus === 'running' ? 'primary' : 'default'}
            >
              {streamStatus === 'running' ? '暂停' : '恢复'}流
            </Button>
          </Col>
          <Col>
            <Button icon={<ReloadOutlined />} onClick={fetchLogs} loading={loading}>
              刷新
            </Button>
          </Col>
          <Col>
            <Button icon={<DownloadOutlined />} onClick={handleExportLogs}>
              导出
            </Button>
          </Col>
        </Row>
      </Card>

      {/* 实时状态提示 */}
      {wsConnection && (
        <Alert
          message="实时日志流已连接"
          description={`WebSocket连接正常，实时接收日志数据。当前缓存 ${realtimeLogs.length} 条实时日志。`}
          type="success"
          showIcon
          closable
          style={{ marginBottom: '16px' }}
        />
      )}

      {/* 主要内容标签页 */}
      <Tabs defaultActiveKey="realtime" size="large">
        {/* 实时日志 */}
        <TabPane tab={`实时日志 (${realtimeLogs.length})`} key="realtime">
          <Table
            dataSource={realtimeLogs}
            columns={logColumns}
            rowKey="id"
            loading={loading}
            size="small"
            pagination={{
              pageSize: 50,
              showSizeChanger: true,
              showQuickJumper: true,
              showTotal: (total) => `共 ${total} 条日志`
            }}
            scroll={{ y: 600 }}
          />
        </TabPane>

        {/* 历史日志 */}
        <TabPane tab={`历史日志 (${logs.length})`} key="history">
          <Table
            dataSource={logs}
            columns={logColumns}
            rowKey="id"
            loading={loading}
            size="small"
            pagination={{
              pageSize: 50,
              showSizeChanger: true,
              showQuickJumper: true,
              showTotal: (total) => `共 ${total} 条日志`
            }}
            scroll={{ y: 600 }}
          />
        </TabPane>

        {/* 日志配置 */}
        <TabPane tab="配置管理" key="config">
          <Card title="日志级别配置" extra={<Button icon={<SettingOutlined />}>高级设置</Button>}>
            <Row gutter={[16, 16]}>
              {services.map(service => (
                <Col xs={24} sm={12} md={8} key={service}>
                  <Card size="small" title={service}>
                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                      <span>日志级别:</span>
                      <Select
                        value={logConfig[service] || 'info'}
                        style={{ width: 100 }}
                        onChange={(level) => handleSetLogLevel(service, level)}
                      >
                        {logLevels.map(level => (
                          <Option key={level} value={level}>{level.toUpperCase()}</Option>
                        ))}
                      </Select>
                    </div>
                  </Card>
                </Col>
              ))}
            </Row>
          </Card>
        </TabPane>

        {/* 日志分析 */}
        <TabPane tab="日志分析" key="analysis">
          <Row gutter={[16, 16]}>
            <Col xs={24} md={12}>
              <Card title="错误统计" size="small">
                <p>正在开发中... 将提供错误日志分析功能</p>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="性能分析" size="small">
                <p>正在开发中... 将提供性能日志分析功能</p>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="趋势分析" size="small">
                <p>正在开发中... 将提供日志趋势分析功能</p>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="异常检测" size="small">
                <p>正在开发中... 将提供异常检测功能</p>
              </Card>
            </Col>
          </Row>
        </TabPane>
      </Tabs>
    </div>
  );
} 