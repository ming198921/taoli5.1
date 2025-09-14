import React, { useState, useEffect } from 'react';
import {
  Card,
  Row,
  Col,
  Statistic,
  Button,
  Form,
  InputNumber,
  Slider,
  Alert,
  Space,
  Divider,
  message,
  Spin,
  Typography
} from 'antd';
import {
  WarningOutlined,
  CheckCircleOutlined,
  StopOutlined,
  ReloadOutlined,
  ThunderboltOutlined,
  SettingOutlined
} from '@ant-design/icons';
import axios from 'axios';
import { Line, Gauge } from '@ant-design/charts';

const { Title } = Typography;

interface RiskConfig {
  max_daily_loss_usd: number;
  max_single_loss_pct: number;
  position_limits: Record<string, number>;
  emergency_stop: {
    consecutive_failures: number;
    error_rate_threshold_pct: number;
    latency_threshold_ms: number;
    drawdown_threshold_bps: number;
  };
  risk_weights: {
    volatility_weight: number;
    liquidity_weight: number;
    correlation_weight: number;
    technical_weight: number;
  };
  monitoring: {
    check_interval_ms: number;
    log_level: string;
    alert_thresholds: {
      fund_utilization_warning_pct: number;
      latency_warning_ms: number;
      success_rate_warning_pct: number;
    };
  };
}

interface RiskStatus {
  daily_pnl: number;
  risk_score: number;
  consecutive_failures: number;
  max_daily_loss: number;
  max_consecutive_failures: number;
  is_healthy: boolean;
  active_positions: number;
  fund_utilization_pct: number;
  avg_latency_ms: number;
  last_check: string;
}

const RiskManagementPage: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const [riskConfig, setRiskConfig] = useState<RiskConfig | null>(null);
  const [riskStatus, setRiskStatus] = useState<RiskStatus | null>(null);
  const [riskHistory, setRiskHistory] = useState<any[]>([]);
  const [form] = Form.useForm();
  const [weightsForm] = Form.useForm();

  // 获取风险配置
  const fetchRiskConfig = async () => {
    try {
      const response = await axios.get('/api/risk/config');
      if (response.data.success) {
        setRiskConfig(response.data.data);
        form.setFieldsValue(response.data.data);
        weightsForm.setFieldsValue(response.data.data.risk_weights);
      }
    } catch (error) {
      message.error('获取风险配置失败');
    }
  };

  // 获取风险状态
  const fetchRiskStatus = async () => {
    try {
      const response = await axios.get('/api/risk/status');
      if (response.data.success) {
        setRiskStatus(response.data.data);
      }
    } catch (error) {
      console.error('获取风险状态失败:', error);
    }
  };

  // 获取风险历史
  const fetchRiskHistory = async () => {
    try {
      const response = await axios.get('/api/risk/history');
      if (response.data.success) {
        setRiskHistory(response.data.data);
      }
    } catch (error) {
      console.error('获取风险历史失败:', error);
    }
  };

  useEffect(() => {
    const loadData = async () => {
      setLoading(true);
      try {
        await Promise.all([
          fetchRiskConfig(),
          fetchRiskStatus(),
          fetchRiskHistory()
        ]);
      } catch (error) {
        console.error('数据加载失败:', error);
      } finally {
        setLoading(false);
      }
    };

    loadData();

    // 每3秒刷新状态
    const interval = setInterval(() => {
      fetchRiskStatus();
      fetchRiskHistory();
    }, 3000);

    return () => clearInterval(interval);
  }, []);

  // 更新风险配置
  const updateRiskConfig = async (values: any) => {
    setLoading(true);
    try {
      const response = await axios.post('/api/risk/config', values);
      if (response.data.success) {
        message.success('风险配置更新成功');
        fetchRiskConfig();
      }
    } catch (error) {
      message.error('更新风险配置失败');
    } finally {
      setLoading(false);
    }
  };

  // 更新风险权重
  const updateRiskWeights = async (values: any) => {
    setLoading(true);
    try {
      const response = await axios.post('/api/risk/weights', values);
      if (response.data.success) {
        message.success('风险权重更新成功');
        fetchRiskConfig();
      }
    } catch (error) {
      message.error('更新风险权重失败');
    } finally {
      setLoading(false);
    }
  };

  // 触发紧急停机
  const triggerEmergencyStop = async () => {
    setLoading(true);
    try {
      const response = await axios.post('/api/risk/emergency-stop', { 
        reason: '手动触发紧急停机' 
      });
      if (response.data.success) {
        message.warning('紧急停机已触发');
        fetchRiskStatus();
      }
    } catch (error) {
      message.error('触发紧急停机失败');
    } finally {
      setLoading(false);
    }
  };

  // 重置失败计数
  const resetFailureCount = async () => {
    setLoading(true);
    try {
      const response = await axios.post('/api/risk/reset-failures');
      if (response.data.success) {
        message.success('失败计数已重置');
        fetchRiskStatus();
      }
    } catch (error) {
      message.error('重置失败计数失败');
    } finally {
      setLoading(false);
    }
  };

  // 风险分数仪表盘配置
  const gaugeConfig = {
    percent: riskStatus?.risk_score || 0,
    range: {
      color: ['#30BF78', '#FAAD14', '#F4664A'],
      width: 12,
    },
    indicator: {
      pointer: { style: { stroke: '#D0D0D0' } },
      pin: { style: { stroke: '#D0D0D0' } },
    },
    statistic: {
      title: {
        formatter: () => '风险评分',
        style: { fontSize: '20px' },
      },
      content: {
        formatter: (datum: any) => `${(datum.percent * 100).toFixed(1)}%`,
        style: { fontSize: '24px' },
      },
    },
    gaugeStyle: {
      lineCap: 'round',
    },
  };

  // 历史数据图表配置
  const lineConfig = {
    data: riskHistory.map(item => ({
      time: new Date(item.timestamp).toLocaleTimeString(),
      value: item.risk_score,
      category: '风险分数'
    })).concat(riskHistory.map(item => ({
      time: new Date(item.timestamp).toLocaleTimeString(),
      value: item.fund_utilization_pct / 100,
      category: '资金使用率'
    }))),
    xField: 'time',
    yField: 'value',
    seriesField: 'category',
    smooth: true,
    animation: {
      appear: {
        animation: 'path-in',
        duration: 1000,
      },
    },
    yAxis: {
      label: {
        formatter: (v: string) => `${(parseFloat(v) * 100).toFixed(0)}%`,
      },
    },
  };

  return (
    <div style={{ padding: 24 }}>
      <Title level={2}>
        <ThunderboltOutlined /> AI风险控制中心
      </Title>

      {/* 风险状态概览 */}
      <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="日盈亏"
              value={riskStatus?.daily_pnl || 0}
              precision={2}
              prefix="$"
              valueStyle={{ 
                color: (riskStatus?.daily_pnl || 0) >= 0 ? '#3f8600' : '#cf1322' 
              }}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="连续失败次数"
              value={riskStatus?.consecutive_failures || 0}
              suffix={`/ ${riskStatus?.max_consecutive_failures || 5}`}
              valueStyle={{ 
                color: (riskStatus?.consecutive_failures || 0) > 3 ? '#cf1322' : '#000' 
              }}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="资金使用率"
              value={riskStatus?.fund_utilization_pct || 0}
              precision={1}
              suffix="%"
              valueStyle={{ 
                color: (riskStatus?.fund_utilization_pct || 0) > 80 ? '#faad14' : '#000' 
              }}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="系统状态"
              value={riskStatus?.is_healthy ? '健康' : '警告'}
              valueStyle={{ 
                color: riskStatus?.is_healthy ? '#52c41a' : '#ff4d4f' 
              }}
              prefix={riskStatus?.is_healthy ? 
                <CheckCircleOutlined /> : <WarningOutlined />
              }
            />
          </Card>
        </Col>
      </Row>

      {/* 风险仪表盘和历史图表 */}
      <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
        <Col xs={24} lg={8}>
          <Card title="风险评分仪表盘">
            <Gauge {...gaugeConfig} height={250} />
          </Card>
        </Col>
        <Col xs={24} lg={16}>
          <Card title="风险指标历史">
            {riskHistory.length > 0 ? (
              <Line {...lineConfig} height={250} />
            ) : (
              <div style={{ height: 250, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                <Spin />
              </div>
            )}
          </Card>
        </Col>
      </Row>

      {/* 风险控制操作 */}
      <Card title="紧急控制" style={{ marginBottom: 16 }}>
        <Space size="large">
          <Button 
            danger 
            icon={<StopOutlined />}
            onClick={triggerEmergencyStop}
            loading={loading}
          >
            紧急停机
          </Button>
          <Button 
            type="primary"
            icon={<ReloadOutlined />}
            onClick={resetFailureCount}
            loading={loading}
          >
            重置失败计数
          </Button>
          <Button
            icon={<SettingOutlined />}
            onClick={() => {
              fetchRiskConfig();
              fetchRiskStatus();
            }}
          >
            刷新配置
          </Button>
        </Space>
      </Card>

      {/* 风险参数配置 */}
      <Row gutter={[16, 16]}>
        <Col xs={24} lg={12}>
          <Card title="风险限制配置" loading={!riskConfig}>
            <Form
              form={form}
              layout="vertical"
              onFinish={updateRiskConfig}
            >
              <Form.Item
                label="最大日亏损 (USD)"
                name="max_daily_loss_usd"
                rules={[{ required: true }]}
              >
                <InputNumber
                  min={1000}
                  max={1000000}
                  step={1000}
                  style={{ width: '100%' }}
                  prefix="$"
                />
              </Form.Item>

              <Form.Item
                label="最大单笔亏损比例 (%)"
                name="max_single_loss_pct"
                rules={[{ required: true }]}
              >
                <Slider
                  min={0.1}
                  max={10}
                  step={0.1}
                  marks={{ 1: '1%', 2: '2%', 5: '5%', 10: '10%' }}
                />
              </Form.Item>

              <Divider />

              <Title level={5}>紧急停机条件</Title>
              
              <Form.Item
                label="连续失败次数阈值"
                name={['emergency_stop', 'consecutive_failures']}
              >
                <InputNumber min={1} max={20} style={{ width: '100%' }} />
              </Form.Item>

              <Form.Item
                label="延迟阈值 (ms)"
                name={['emergency_stop', 'latency_threshold_ms']}
              >
                <InputNumber min={10} max={5000} style={{ width: '100%' }} />
              </Form.Item>

              <Form.Item>
                <Button type="primary" htmlType="submit" loading={loading}>
                  更新配置
                </Button>
              </Form.Item>
            </Form>
          </Card>
        </Col>

        <Col xs={24} lg={12}>
          <Card title="风险权重配置" loading={!riskConfig}>
            <Form
              form={weightsForm}
              layout="vertical"
              onFinish={updateRiskWeights}
            >
              <Form.Item
                label="波动性权重"
                name="volatility_weight"
              >
                <Slider
                  min={0}
                  max={1}
                  step={0.05}
                  marks={{ 0: '0', 0.5: '0.5', 1: '1' }}
                />
              </Form.Item>

              <Form.Item
                label="流动性权重"
                name="liquidity_weight"
              >
                <Slider
                  min={0}
                  max={1}
                  step={0.05}
                  marks={{ 0: '0', 0.5: '0.5', 1: '1' }}
                />
              </Form.Item>

              <Form.Item
                label="相关性权重"
                name="correlation_weight"
              >
                <Slider
                  min={0}
                  max={1}
                  step={0.05}
                  marks={{ 0: '0', 0.5: '0.5', 1: '1' }}
                />
              </Form.Item>

              <Form.Item
                label="技术指标权重"
                name="technical_weight"
              >
                <Slider
                  min={0}
                  max={1}
                  step={0.05}
                  marks={{ 0: '0', 0.5: '0.5', 1: '1' }}
                />
              </Form.Item>

              <Alert 
                message="注意：所有权重之和必须等于1.0" 
                type="warning" 
                showIcon 
                style={{ marginBottom: 16 }}
              />

              <Form.Item>
                <Button type="primary" htmlType="submit" loading={loading}>
                  更新权重
                </Button>
              </Form.Item>
            </Form>
          </Card>
        </Col>
      </Row>
    </div>
  );
};

export default RiskManagementPage;