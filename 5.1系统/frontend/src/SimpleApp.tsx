import React from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { Layout, Card, Button, Typography, Row, Col, Statistic, message } from 'antd';
import { 
  DashboardOutlined, 
  PlayCircleOutlined, 
  PauseCircleOutlined,
  ReloadOutlined
} from '@ant-design/icons';

const { Header, Content } = Layout;
const { Title, Text } = Typography;

// 简单的Dashboard组件
const SimpleDashboard: React.FC = () => {
  const [systemRunning, setSystemRunning] = React.useState(false);
  const [systemData, setSystemData] = React.useState<any>(null);
  const [loading, setLoading] = React.useState(false);

  const startSystem = async () => {
    setLoading(true);
    try {
      const response = await fetch('http://57.183.21.242:8080/api/system/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
      });
      const result = await response.json();
      if (result.success) {
        setSystemRunning(true);
        message.success(result.message);
        await checkSystemStatus();
      } else {
        message.error('启动失败');
      }
    } catch (error) {
      message.error('API调用失败');
      console.error(error);
    } finally {
      setLoading(false);
    }
  };

  const stopSystem = async () => {
    setLoading(true);
    try {
      const response = await fetch('http://57.183.21.242:8080/api/system/stop', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
      });
      const result = await response.json();
      if (result.success) {
        setSystemRunning(false);
        message.success(result.message);
        setSystemData(null);
      } else {
        message.error('停止失败');
      }
    } catch (error) {
      message.error('API调用失败');
      console.error(error);
    } finally {
      setLoading(false);
    }
  };

  const checkSystemStatus = async () => {
    try {
      const response = await fetch('http://57.183.21.242:8080/api/system/status');
      const result = await response.json();
      if (result.success) {
        setSystemData(result.data);
        setSystemRunning(result.data.isRunning);
      }
    } catch (error) {
      console.error('获取状态失败:', error);
    }
  };

  React.useEffect(() => {
    checkSystemStatus();
    const interval = setInterval(checkSystemStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div style={{ padding: '24px' }}>
      <div style={{ marginBottom: '24px' }}>
        <Title level={2}>
          <DashboardOutlined style={{ marginRight: '8px' }} />
          5.1套利系统控制面板
        </Title>
        <Text type="secondary">实时系统状态监控与控制</Text>
      </div>

      <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
        <Col span={6}>
          <Card>
            <Statistic
              title="系统状态"
              value={systemRunning ? '运行中' : '已停止'}
              valueStyle={{ color: systemRunning ? '#3f8600' : '#cf1322' }}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="CPU使用率"
              value={systemData?.cpu_usage || 0}
              suffix="%"
              precision={1}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="内存使用率"
              value={systemData?.memory_usage || 0}
              suffix="%"
              precision={1}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="网络延迟"
              value={systemData?.network_latency || 0}
              suffix="ms"
              precision={0}
            />
          </Card>
        </Col>
      </Row>

      <Card title="系统控制" style={{ marginBottom: '24px' }}>
        <div style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
          {systemRunning ? (
            <Button
              danger
              type="primary"
              icon={<PauseCircleOutlined />}
              loading={loading}
              onClick={stopSystem}
            >
              停止系统
            </Button>
          ) : (
            <Button
              type="primary"
              icon={<PlayCircleOutlined />}
              loading={loading}
              onClick={startSystem}
            >
              启动系统
            </Button>
          )}
          <Button
            icon={<ReloadOutlined />}
            onClick={checkSystemStatus}
          >
            刷新状态
          </Button>
        </div>
      </Card>

      {systemData && (
        <Row gutter={[16, 16]}>
          <Col span={12}>
            <Card title="模块状态" size="small">
              {Object.entries(systemData.modules || {}).map(([name, module]: [string, any]) => (
                <div key={name} style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
                  <span>{name}</span>
                  <span style={{ color: module.status === 'running' ? '#52c41a' : '#ff4d4f' }}>
                    {module.status === 'running' ? '运行中' : '已停止'}
                  </span>
                </div>
              ))}
            </Card>
          </Col>
          <Col span={12}>
            <Card title="系统信息" size="small">
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
                <span>版本</span>
                <span>{systemData.version}</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
                <span>运行时间</span>
                <span>{systemData.uptime ? `${systemData.uptime.toFixed(2)}小时` : '0小时'}</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span>最后启动</span>
                <span>
                  {systemData.lastStarted 
                    ? new Date(systemData.lastStarted).toLocaleString()
                    : '从未启动'
                  }
                </span>
              </div>
            </Card>
          </Col>
        </Row>
      )}
    </div>
  );
};

// 简单的应用组件
function SimpleApp() {
  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Header style={{ 
        display: 'flex', 
        alignItems: 'center', 
        background: '#001529',
        padding: '0 24px'
      }}>
        <div style={{ color: 'white', fontSize: '18px', fontWeight: 'bold' }}>
          5.1高频套利系统
        </div>
      </Header>
      <Content>
        <Routes>
          <Route path="/dashboard" element={<SimpleDashboard />} />
          <Route path="/" element={<Navigate to="/dashboard" replace />} />
          <Route path="*" element={<Navigate to="/dashboard" replace />} />
        </Routes>
      </Content>
    </Layout>
  );
}

export default SimpleApp;