import React, { useState, useEffect } from 'react';
import { 
  Card, Row, Col, Button, Table, Modal, Form, Input, Select, Switch, 
  Tabs, Progress, Statistic, Badge, Alert, message, notification,
  Upload, DatePicker, Tag, Tooltip, Popconfirm, Space, Drawer
} from 'antd';
import {
  PlusOutlined, DeleteOutlined, EditOutlined, PlayCircleOutlined,
  PauseCircleOutlined, UploadOutlined, DownloadOutlined, ExperimentOutlined,
  SearchOutlined, HistoryOutlined, SettingOutlined, CheckCircleOutlined,
  ExclamationCircleOutlined, ReloadOutlined, FileTextOutlined,
  LinkOutlined, MonitorOutlined, BarChartOutlined
} from '@ant-design/icons';

const { Option } = Select;
const { TextArea } = Input;
const { RangePicker } = DatePicker;

// 数据类型定义
interface CleaningRule {
  id: string;
  name: string;
  description: string;
  type: 'filter' | 'transform' | 'validate' | 'normalize';
  enabled: boolean;
  config: any;
  priority: number;
  created_at: string;
  updated_at: string;
  usage_count: number;
  success_rate: number;
}

interface Exchange {
  id: string;
  name: string;
  code: string;
  status: 'active' | 'inactive' | 'maintenance';
  enabled: boolean;
  symbols_count: number;
  rules_count: number;
  config: any;
  metrics: {
    latency: number;
    success_rate: number;
    data_quality: number;
  };
}

interface QualityMetric {
  component: string;
  score: number;
  issues_count: number;
  trend: 'up' | 'down' | 'stable';
  last_updated: string;
}

interface QualityIssue {
  id: string;
  type: 'missing_data' | 'invalid_format' | 'outlier' | 'duplicate';
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  source: string;
  created_at: string;
  status: 'open' | 'resolved' | 'ignored';
}

export default function CleaningModule() {
  const [loading, setLoading] = useState(false);
  const [activeTab, setActiveTab] = useState('overview');
  
  // 清洗规则状态
  const [cleaningRules, setCleaningRules] = useState<CleaningRule[]>([]);
  const [ruleModalVisible, setRuleModalVisible] = useState(false);
  const [editingRule, setEditingRule] = useState<CleaningRule | null>(null);
  const [ruleForm] = Form.useForm();
  
  // 交易所配置状态
  const [exchanges, setExchanges] = useState<Exchange[]>([]);
  const [exchangeModalVisible, setExchangeModalVisible] = useState(false);
  const [editingExchange, setEditingExchange] = useState<Exchange | null>(null);
  const [exchangeForm] = Form.useForm();
  
  // 数据质量状态
  const [qualityMetrics, setQualityMetrics] = useState<QualityMetric[]>([]);
  const [qualityIssues, setQualityIssues] = useState<QualityIssue[]>([]);
  const [qualityScore, setQualityScore] = useState(0);
  
  // 概览数据
  const [overviewStats, setOverviewStats] = useState({
    total_rules: 0,
    active_rules: 0,
    total_exchanges: 0,
    active_exchanges: 0,
    data_quality_score: 0,
    pending_issues: 0,
    processed_records: 0,
    error_rate: 0
  });

  // 初始化数据
  const initializeData = async () => {
    setLoading(true);
    try {
      // 生成模拟清洗规则数据
      const mockRules: CleaningRule[] = [
        {
          id: 'rule_001',
          name: '价格异常值过滤',
          description: '过滤价格波动超过5%的异常数据',
          type: 'filter',
          enabled: true,
          config: { threshold: 0.05, action: 'filter' },
          priority: 1,
          created_at: new Date(Date.now() - 86400000 * 7).toISOString(),
          updated_at: new Date(Date.now() - 3600000).toISOString(),
          usage_count: 1245,
          success_rate: 98.5
        },
        {
          id: 'rule_002',
          name: '交易量标准化',
          description: '将不同交易所的交易量数据标准化',
          type: 'normalize',
          enabled: true,
          config: { base_unit: 'BTC', precision: 8 },
          priority: 2,
          created_at: new Date(Date.now() - 86400000 * 5).toISOString(),
          updated_at: new Date(Date.now() - 7200000).toISOString(),
          usage_count: 892,
          success_rate: 99.2
        },
        {
          id: 'rule_003',
          name: '数据格式验证',
          description: '验证价格和时间戳格式的正确性',
          type: 'validate',
          enabled: false,
          config: { price_format: 'decimal', timestamp_format: 'unix' },
          priority: 3,
          created_at: new Date(Date.now() - 86400000 * 3).toISOString(),
          updated_at: new Date(Date.now() - 1800000).toISOString(),
          usage_count: 567,
          success_rate: 97.8
        },
        {
          id: 'rule_004',
          name: '重复数据清理',
          description: '清理相同时间戳的重复交易数据',
          type: 'transform',
          enabled: true,
          config: { dedup_fields: ['timestamp', 'symbol', 'exchange'] },
          priority: 4,
          created_at: new Date(Date.now() - 86400000 * 2).toISOString(),
          updated_at: new Date(Date.now() - 900000).toISOString(),
          usage_count: 1034,
          success_rate: 99.6
        },
        {
          id: 'rule_005',
          name: '价格精度统一',
          description: '统一不同交易所的价格精度',
          type: 'transform',
          enabled: true,
          config: { decimal_places: 8, rounding_mode: 'half_up' },
          priority: 5,
          created_at: new Date(Date.now() - 86400000).toISOString(),
          updated_at: new Date(Date.now() - 300000).toISOString(),
          usage_count: 2156,
          success_rate: 99.9
        }
      ];

      // 生成模拟交易所数据
      const mockExchanges: Exchange[] = [
        {
          id: 'binance',
          name: 'Binance',
          code: 'BINANCE',
          status: 'active',
          enabled: true,
          symbols_count: 1247,
          rules_count: 5,
          config: {
            api_endpoint: 'https://api.binance.com',
            rate_limit: 1200,
            data_format: 'json'
          },
          metrics: {
            latency: 45,
            success_rate: 99.8,
            data_quality: 95.2
          }
        },
        {
          id: 'okx',
          name: 'OKX',
          code: 'OKX',
          status: 'active',
          enabled: true,
          symbols_count: 892,
          rules_count: 4,
          config: {
            api_endpoint: 'https://www.okx.com/api',
            rate_limit: 600,
            data_format: 'json'
          },
          metrics: {
            latency: 52,
            success_rate: 99.6,
            data_quality: 94.7
          }
        },
        {
          id: 'bybit',
          name: 'Bybit',
          code: 'BYBIT',
          status: 'maintenance',
          enabled: false,
          symbols_count: 456,
          rules_count: 3,
          config: {
            api_endpoint: 'https://api.bybit.com',
            rate_limit: 800,
            data_format: 'json'
          },
          metrics: {
            latency: 68,
            success_rate: 98.9,
            data_quality: 92.1
          }
        },
        {
          id: 'gate',
          name: 'Gate.io',
          code: 'GATE',
          status: 'active',
          enabled: true,
          symbols_count: 634,
          rules_count: 4,
          config: {
            api_endpoint: 'https://api.gateio.ws',
            rate_limit: 400,
            data_format: 'json'
          },
          metrics: {
            latency: 73,
            success_rate: 99.1,
            data_quality: 93.5
          }
        }
      ];

      // 生成数据质量指标
      const mockQualityMetrics: QualityMetric[] = [
        {
          component: '价格数据',
          score: 96.8,
          issues_count: 12,
          trend: 'up',
          last_updated: new Date().toISOString()
        },
        {
          component: '交易量数据',
          score: 94.2,
          issues_count: 18,
          trend: 'stable',
          last_updated: new Date().toISOString()
        },
        {
          component: '时间戳数据',
          score: 99.1,
          issues_count: 3,
          trend: 'up',
          last_updated: new Date().toISOString()
        },
        {
          component: '交易对数据',
          score: 92.5,
          issues_count: 24,
          trend: 'down',
          last_updated: new Date().toISOString()
        }
      ];

      // 生成质量问题
      const mockQualityIssues: QualityIssue[] = [
        {
          id: 'issue_001',
          type: 'missing_data',
          severity: 'medium',
          description: 'Binance BTCUSDT 15:30-15:35 缺失交易数据',
          source: 'binance',
          created_at: new Date(Date.now() - 1800000).toISOString(),
          status: 'open'
        },
        {
          id: 'issue_002',
          type: 'outlier',
          severity: 'high',
          description: 'OKX ETHUSDT 价格出现异常波动',
          source: 'okx',
          created_at: new Date(Date.now() - 3600000).toISOString(),
          status: 'open'
        },
        {
          id: 'issue_003',
          type: 'duplicate',
          severity: 'low',
          description: 'Gate.io 检测到重复的交易记录',
          source: 'gate',
          created_at: new Date(Date.now() - 7200000).toISOString(),
          status: 'resolved'
        }
      ];

      // 计算概览统计
      const activeRules = mockRules.filter(r => r.enabled).length;
      const activeExchanges = mockExchanges.filter(e => e.enabled).length;
      const avgQualityScore = mockQualityMetrics.reduce((sum, m) => sum + m.score, 0) / mockQualityMetrics.length;
      const openIssues = mockQualityIssues.filter(i => i.status === 'open').length;

      setCleaningRules(mockRules);
      setExchanges(mockExchanges);
      setQualityMetrics(mockQualityMetrics);
      setQualityIssues(mockQualityIssues);
      setQualityScore(avgQualityScore);
      
      setOverviewStats({
        total_rules: mockRules.length,
        active_rules: activeRules,
        total_exchanges: mockExchanges.length,
        active_exchanges: activeExchanges,
        data_quality_score: Math.round(avgQualityScore),
        pending_issues: openIssues,
        processed_records: 2847561,
        error_rate: 0.5
      });

    } catch (error) {
      console.error('初始化清洗模块数据失败:', error);
      message.error('数据加载失败');
    } finally {
      setLoading(false);
    }
  };

  // 清洗规则操作
  const handleRuleAction = async (action: string, rule?: CleaningRule) => {
    try {
      switch (action) {
        case 'create':
          setEditingRule(null);
          setRuleModalVisible(true);
          ruleForm.resetFields();
          break;
        
        case 'edit':
          setEditingRule(rule!);
          setRuleModalVisible(true);
          ruleForm.setFieldsValue(rule);
          break;
        
        case 'delete':
          setCleaningRules(prev => prev.filter(r => r.id !== rule!.id));
          message.success('规则删除成功');
          break;
        
        case 'toggle':
          setCleaningRules(prev => prev.map(r => 
            r.id === rule!.id ? { ...r, enabled: !r.enabled } : r
          ));
          message.success(`规则已${rule!.enabled ? '禁用' : '启用'}`);
          break;
        
        case 'test':
          message.loading('正在测试规则...', 2);
          setTimeout(() => {
            message.success('规则测试通过');
          }, 2000);
          break;
      }
    } catch (error) {
      message.error(`操作失败: ${error.message}`);
    }
  };

  // 交易所操作
  const handleExchangeAction = async (action: string, exchange?: Exchange) => {
    try {
      switch (action) {
        case 'edit':
          setEditingExchange(exchange!);
          setExchangeModalVisible(true);
          exchangeForm.setFieldsValue(exchange);
          break;
        
        case 'toggle':
          setExchanges(prev => prev.map(e => 
            e.id === exchange!.id ? { ...e, enabled: !e.enabled, status: e.enabled ? 'inactive' : 'active' } : e
          ));
          message.success(`${exchange!.name}已${exchange!.enabled ? '禁用' : '启用'}`);
          break;
        
        case 'test':
          message.loading(`正在测试${exchange!.name}连接...`, 2);
          setTimeout(() => {
            message.success('连接测试成功');
          }, 2000);
          break;
        
        case 'reset':
          Modal.confirm({
            title: `确认重置${exchange!.name}配置？`,
            content: '重置后将恢复默认配置',
            onOk: () => {
              message.success('配置重置成功');
            }
          });
          break;
      }
    } catch (error) {
      message.error(`操作失败: ${error.message}`);
    }
  };

  // 数据质量操作
  const handleQualityAction = async (action: string, issue?: QualityIssue) => {
    try {
      switch (action) {
        case 'resolve':
          setQualityIssues(prev => prev.map(i => 
            i.id === issue!.id ? { ...i, status: 'resolved' } : i
          ));
          message.success('问题已解决');
          break;
        
        case 'analyze':
          message.loading('正在分析数据质量...', 3);
          setTimeout(() => {
            notification.success({
              message: '质量分析完成',
              description: '发现3个新的质量问题，总体质量分数提升2.1分'
            });
          }, 3000);
          break;
        
        case 'generate_report':
          message.loading('正在生成质量报告...', 2);
          setTimeout(() => {
            message.success('质量报告生成成功');
          }, 2000);
          break;
      }
    } catch (error) {
      message.error(`操作失败: ${error.message}`);
    }
  };

  // 保存规则
  const handleSaveRule = async (values: any) => {
    try {
      const ruleData = {
        ...values,
        id: editingRule?.id || `rule_${Date.now()}`,
        created_at: editingRule?.created_at || new Date().toISOString(),
        updated_at: new Date().toISOString(),
        usage_count: editingRule?.usage_count || 0,
        success_rate: editingRule?.success_rate || 0
      };

      if (editingRule) {
        setCleaningRules(prev => prev.map(r => r.id === editingRule.id ? ruleData : r));
        message.success('规则更新成功');
      } else {
        setCleaningRules(prev => [...prev, ruleData]);
        message.success('规则创建成功');
      }

      setRuleModalVisible(false);
      setEditingRule(null);
    } catch (error) {
      message.error('保存失败');
    }
  };

  useEffect(() => {
    initializeData();
  }, []);

  // 表格列定义
  const ruleColumns = [
    { title: '规则名称', dataIndex: 'name', key: 'name' },
    { title: '类型', dataIndex: 'type', key: 'type', 
      render: (type: string) => <Tag color={type === 'filter' ? 'blue' : type === 'transform' ? 'green' : type === 'validate' ? 'orange' : 'purple'}>{type}</Tag> 
    },
    { title: '优先级', dataIndex: 'priority', key: 'priority' },
    { title: '使用次数', dataIndex: 'usage_count', key: 'usage_count' },
    { title: '成功率', dataIndex: 'success_rate', key: 'success_rate', render: (rate: number) => `${rate}%` },
    { 
      title: '状态', 
      dataIndex: 'enabled', 
      key: 'enabled',
      render: (enabled: boolean) => <Badge status={enabled ? 'success' : 'default'} text={enabled ? '启用' : '禁用'} />
    },
    {
      title: '操作',
      key: 'actions',
      render: (_, record: CleaningRule) => (
        <Space>
          <Button size="small" icon={<EditOutlined />} onClick={() => handleRuleAction('edit', record)}>编辑</Button>
          <Button size="small" icon={<ExperimentOutlined />} onClick={() => handleRuleAction('test', record)}>测试</Button>
          <Button size="small" icon={record.enabled ? <PauseCircleOutlined /> : <PlayCircleOutlined />} 
                  onClick={() => handleRuleAction('toggle', record)}>
            {record.enabled ? '禁用' : '启用'}
          </Button>
          <Popconfirm title="确认删除？" onConfirm={() => handleRuleAction('delete', record)}>
            <Button size="small" icon={<DeleteOutlined />} danger>删除</Button>
          </Popconfirm>
        </Space>
      )
    }
  ];

  const exchangeColumns = [
    { title: '交易所', dataIndex: 'name', key: 'name' },
    { title: '代码', dataIndex: 'code', key: 'code' },
    { 
      title: '状态', 
      dataIndex: 'status', 
      key: 'status',
      render: (status: string) => (
        <Badge status={status === 'active' ? 'success' : status === 'maintenance' ? 'warning' : 'default'} 
               text={status === 'active' ? '正常' : status === 'maintenance' ? '维护中' : '停用'} />
      )
    },
    { title: '交易对', dataIndex: 'symbols_count', key: 'symbols_count' },
    { title: '规则数', dataIndex: 'rules_count', key: 'rules_count' },
    { title: '延迟(ms)', dataIndex: ['metrics', 'latency'], key: 'latency' },
    { title: '成功率', dataIndex: ['metrics', 'success_rate'], key: 'success_rate', render: (rate: number) => `${rate}%` },
    { title: '质量分', dataIndex: ['metrics', 'data_quality'], key: 'data_quality', render: (score: number) => `${score}%` },
    {
      title: '操作',
      key: 'actions',
      render: (_, record: Exchange) => (
        <Space>
          <Button size="small" icon={<SettingOutlined />} onClick={() => handleExchangeAction('edit', record)}>配置</Button>
          <Button size="small" icon={<LinkOutlined />} onClick={() => handleExchangeAction('test', record)}>测试</Button>
          <Button size="small" icon={record.enabled ? <PauseCircleOutlined /> : <PlayCircleOutlined />} 
                  onClick={() => handleExchangeAction('toggle', record)}>
            {record.enabled ? '禁用' : '启用'}
          </Button>
          <Button size="small" onClick={() => handleExchangeAction('reset', record)}>重置</Button>
        </Space>
      )
    }
  ];

  const issueColumns = [
    { title: '类型', dataIndex: 'type', key: 'type',
      render: (type: string) => <Tag color={type === 'missing_data' ? 'red' : type === 'outlier' ? 'orange' : type === 'duplicate' ? 'blue' : 'purple'}>{type}</Tag>
    },
    { title: '严重程度', dataIndex: 'severity', key: 'severity',
      render: (severity: string) => <Tag color={severity === 'critical' ? 'red' : severity === 'high' ? 'orange' : severity === 'medium' ? 'yellow' : 'green'}>{severity}</Tag>
    },
    { title: '描述', dataIndex: 'description', key: 'description' },
    { title: '来源', dataIndex: 'source', key: 'source' },
    { title: '状态', dataIndex: 'status', key: 'status',
      render: (status: string) => <Badge status={status === 'resolved' ? 'success' : status === 'open' ? 'error' : 'default'} text={status} />
    },
    {
      title: '操作',
      key: 'actions',
      render: (_, record: QualityIssue) => (
        <Space>
          {record.status === 'open' && (
            <Button size="small" type="primary" onClick={() => handleQualityAction('resolve', record)}>解决</Button>
          )}
          <Button size="small">详情</Button>
        </Space>
      )
    }
  ];

  return (
    <div style={{ padding: '24px' }}>
      <div style={{ marginBottom: '24px' }}>
        <h1>数据清洗模块</h1>
        <p style={{ color: '#666' }}>清洗规则管理、交易所配置、数据质量监控</p>
      </div>

      <Tabs 
        activeKey={activeTab} 
        onChange={setActiveTab} 
        size="large"
        items={[
          {
            key: 'overview',
            label: '概览',
            children: (
              <>
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic title="清洗规则" value={overviewStats.total_rules} 
                                suffix={`/ ${overviewStats.active_rules}个启用`} />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic title="交易所" value={overviewStats.total_exchanges} 
                                suffix={`/ ${overviewStats.active_exchanges}个活跃`} />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic title="数据质量分数" value={overviewStats.data_quality_score} 
                                suffix="/ 100" valueStyle={{ color: '#3f8600' }} />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic title="待处理问题" value={overviewStats.pending_issues} 
                                valueStyle={{ color: overviewStats.pending_issues > 0 ? '#cf1322' : '#3f8600' }} />
                    </Card>
                  </Col>
                </Row>

                <Row gutter={[16, 16]}>
                  <Col xs={24} md={12}>
                    <Card title="处理统计" size="small">
                      <Statistic title="已处理记录" value={overviewStats.processed_records} />
                      <div style={{ marginTop: '16px' }}>
                        <div>错误率</div>
                        <Progress percent={overviewStats.error_rate} size="small" status={overviewStats.error_rate > 1 ? 'exception' : 'success'} />
                      </div>
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card title="快速操作" size="small">
                      <Space direction="vertical" style={{ width: '100%' }}>
                        <Button type="primary" block onClick={() => handleRuleAction('create')}>创建清洗规则</Button>
                        <Button block onClick={() => handleQualityAction('analyze')}>运行质量分析</Button>
                        <Button block onClick={() => handleQualityAction('generate_report')}>生成质量报告</Button>
                      </Space>
                    </Card>
                  </Col>
                </Row>
              </>
            )
          },
          {
            key: 'rules',
            label: `清洗规则 (${cleaningRules.length})`,
            children: (
              <Card 
                title="清洗规则管理"
                extra={
                  <Space>
                    <Button icon={<UploadOutlined />}>导入</Button>
                    <Button icon={<DownloadOutlined />}>导出</Button>
                    <Button type="primary" icon={<PlusOutlined />} onClick={() => handleRuleAction('create')}>新建规则</Button>
                  </Space>
                }
              >
                <Table
                  dataSource={cleaningRules}
                  columns={ruleColumns}
                  rowKey="id"
                  loading={loading}
                  pagination={{ pageSize: 10 }}
                />
              </Card>
            )
          },
          {
            key: 'exchanges',
            label: `交易所配置 (${exchanges.length})`,
            children: (
              <Card title="交易所配置管理">
                <Table
                  dataSource={exchanges}
                  columns={exchangeColumns}
                  rowKey="id"
                  loading={loading}
                  pagination={false}
                />
              </Card>
            )
          },
          {
            key: 'quality',
            label: '数据质量',
            children: (
              <>
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} md={12}>
                    <Card title="质量指标" size="small">
                      {qualityMetrics.map((metric, index) => (
                        <div key={index} style={{ marginBottom: '16px' }}>
                          <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                            <span>{metric.component}</span>
                            <span>{metric.score.toFixed(1)}%</span>
                          </div>
                          <Progress percent={metric.score} size="small" 
                                   status={metric.score >= 95 ? 'success' : metric.score >= 85 ? 'normal' : 'exception'} />
                        </div>
                      ))}
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card title="质量趋势" size="small">
                      <Statistic title="整体质量分数" value={qualityScore.toFixed(1)} suffix="/ 100" 
                                valueStyle={{ color: '#3f8600' }} />
                      <div style={{ marginTop: '16px', color: '#666' }}>
                        过去7天质量分数稳步提升，数据清洗效果良好
                      </div>
                    </Card>
                  </Col>
                </Row>

                <Card title="质量问题" extra={<Button onClick={() => handleQualityAction('analyze')}>重新分析</Button>}>
                  <Table
                    dataSource={qualityIssues}
                    columns={issueColumns}
                    rowKey="id"
                    loading={loading}
                    pagination={{ pageSize: 10 }}
                  />
                </Card>
              </>
            )
          }
        ]}
      />

      {/* 规则编辑模态框 */}
      <Modal
        title={editingRule ? '编辑清洗规则' : '创建清洗规则'}
        open={ruleModalVisible}
        onCancel={() => setRuleModalVisible(false)}
        onOk={() => ruleForm.submit()}
        width={600}
      >
        <Form form={ruleForm} onFinish={handleSaveRule} layout="vertical">
          <Form.Item name="name" label="规则名称" rules={[{ required: true }]}>
            <Input placeholder="输入规则名称" />
          </Form.Item>
          <Form.Item name="description" label="规则描述">
            <TextArea placeholder="输入规则描述" rows={3} />
          </Form.Item>
          <Row gutter={16}>
            <Col span={12}>
              <Form.Item name="type" label="规则类型" rules={[{ required: true }]}>
                <Select placeholder="选择规则类型">
                  <Option value="filter">过滤</Option>
                  <Option value="transform">转换</Option>
                  <Option value="validate">验证</Option>
                  <Option value="normalize">标准化</Option>
                </Select>
              </Form.Item>
            </Col>
            <Col span={12}>
              <Form.Item name="priority" label="优先级" rules={[{ required: true }]}>
                <Input type="number" placeholder="输入优先级(1-10)" />
              </Form.Item>
            </Col>
          </Row>
          <Form.Item name="enabled" label="启用规则" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item name="config" label="规则配置">
            <TextArea placeholder="输入JSON格式的规则配置" rows={4} />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
}
import { 
  Card, Row, Col, Button, Table, Modal, Form, Input, Select, Switch, 
  Tabs, Progress, Statistic, Badge, Alert, message, notification,
  Upload, DatePicker, Tag, Tooltip, Popconfirm, Space, Drawer
} from 'antd';
import {
  PlusOutlined, DeleteOutlined, EditOutlined, PlayCircleOutlined,
  PauseCircleOutlined, UploadOutlined, DownloadOutlined, ExperimentOutlined,
  SearchOutlined, HistoryOutlined, SettingOutlined, CheckCircleOutlined,
  ExclamationCircleOutlined, ReloadOutlined, FileTextOutlined,
  LinkOutlined, MonitorOutlined, BarChartOutlined
} from '@ant-design/icons';

const { Option } = Select;
const { TextArea } = Input;
const { RangePicker } = DatePicker;

// 数据类型定义
interface CleaningRule {
  id: string;
  name: string;
  description: string;
  type: 'filter' | 'transform' | 'validate' | 'normalize';
  enabled: boolean;
  config: any;
  priority: number;
  created_at: string;
  updated_at: string;
  usage_count: number;
  success_rate: number;
}

interface Exchange {
  id: string;
  name: string;
  code: string;
  status: 'active' | 'inactive' | 'maintenance';
  enabled: boolean;
  symbols_count: number;
  rules_count: number;
  config: any;
  metrics: {
    latency: number;
    success_rate: number;
    data_quality: number;
  };
}

interface QualityMetric {
  component: string;
  score: number;
  issues_count: number;
  trend: 'up' | 'down' | 'stable';
  last_updated: string;
}

interface QualityIssue {
  id: string;
  type: 'missing_data' | 'invalid_format' | 'outlier' | 'duplicate';
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  source: string;
  created_at: string;
  status: 'open' | 'resolved' | 'ignored';
}

export default function CleaningModule() {
  const [loading, setLoading] = useState(false);
  const [activeTab, setActiveTab] = useState('overview');
  
  // 清洗规则状态
  const [cleaningRules, setCleaningRules] = useState<CleaningRule[]>([]);
  const [ruleModalVisible, setRuleModalVisible] = useState(false);
  const [editingRule, setEditingRule] = useState<CleaningRule | null>(null);
  const [ruleForm] = Form.useForm();
  
  // 交易所配置状态
  const [exchanges, setExchanges] = useState<Exchange[]>([]);
  const [exchangeModalVisible, setExchangeModalVisible] = useState(false);
  const [editingExchange, setEditingExchange] = useState<Exchange | null>(null);
  const [exchangeForm] = Form.useForm();
  
  // 数据质量状态
  const [qualityMetrics, setQualityMetrics] = useState<QualityMetric[]>([]);
  const [qualityIssues, setQualityIssues] = useState<QualityIssue[]>([]);
  const [qualityScore, setQualityScore] = useState(0);
  
  // 概览数据
  const [overviewStats, setOverviewStats] = useState({
    total_rules: 0,
    active_rules: 0,
    total_exchanges: 0,
    active_exchanges: 0,
    data_quality_score: 0,
    pending_issues: 0,
    processed_records: 0,
    error_rate: 0
  });

  // 初始化数据
  const initializeData = async () => {
    setLoading(true);
    try {
      // 生成模拟清洗规则数据
      const mockRules: CleaningRule[] = [
        {
          id: 'rule_001',
          name: '价格异常值过滤',
          description: '过滤价格波动超过5%的异常数据',
          type: 'filter',
          enabled: true,
          config: { threshold: 0.05, action: 'filter' },
          priority: 1,
          created_at: new Date(Date.now() - 86400000 * 7).toISOString(),
          updated_at: new Date(Date.now() - 3600000).toISOString(),
          usage_count: 1245,
          success_rate: 98.5
        },
        {
          id: 'rule_002',
          name: '交易量标准化',
          description: '将不同交易所的交易量数据标准化',
          type: 'normalize',
          enabled: true,
          config: { base_unit: 'BTC', precision: 8 },
          priority: 2,
          created_at: new Date(Date.now() - 86400000 * 5).toISOString(),
          updated_at: new Date(Date.now() - 7200000).toISOString(),
          usage_count: 892,
          success_rate: 99.2
        },
        {
          id: 'rule_003',
          name: '数据格式验证',
          description: '验证价格和时间戳格式的正确性',
          type: 'validate',
          enabled: false,
          config: { price_format: 'decimal', timestamp_format: 'unix' },
          priority: 3,
          created_at: new Date(Date.now() - 86400000 * 3).toISOString(),
          updated_at: new Date(Date.now() - 1800000).toISOString(),
          usage_count: 567,
          success_rate: 97.8
        },
        {
          id: 'rule_004',
          name: '重复数据清理',
          description: '清理相同时间戳的重复交易数据',
          type: 'transform',
          enabled: true,
          config: { dedup_fields: ['timestamp', 'symbol', 'exchange'] },
          priority: 4,
          created_at: new Date(Date.now() - 86400000 * 2).toISOString(),
          updated_at: new Date(Date.now() - 900000).toISOString(),
          usage_count: 1034,
          success_rate: 99.6
        },
        {
          id: 'rule_005',
          name: '价格精度统一',
          description: '统一不同交易所的价格精度',
          type: 'transform',
          enabled: true,
          config: { decimal_places: 8, rounding_mode: 'half_up' },
          priority: 5,
          created_at: new Date(Date.now() - 86400000).toISOString(),
          updated_at: new Date(Date.now() - 300000).toISOString(),
          usage_count: 2156,
          success_rate: 99.9
        }
      ];

      // 生成模拟交易所数据
      const mockExchanges: Exchange[] = [
        {
          id: 'binance',
          name: 'Binance',
          code: 'BINANCE',
          status: 'active',
          enabled: true,
          symbols_count: 1247,
          rules_count: 5,
          config: {
            api_endpoint: 'https://api.binance.com',
            rate_limit: 1200,
            data_format: 'json'
          },
          metrics: {
            latency: 45,
            success_rate: 99.8,
            data_quality: 95.2
          }
        },
        {
          id: 'okx',
          name: 'OKX',
          code: 'OKX',
          status: 'active',
          enabled: true,
          symbols_count: 892,
          rules_count: 4,
          config: {
            api_endpoint: 'https://www.okx.com/api',
            rate_limit: 600,
            data_format: 'json'
          },
          metrics: {
            latency: 52,
            success_rate: 99.6,
            data_quality: 94.7
          }
        },
        {
          id: 'bybit',
          name: 'Bybit',
          code: 'BYBIT',
          status: 'maintenance',
          enabled: false,
          symbols_count: 456,
          rules_count: 3,
          config: {
            api_endpoint: 'https://api.bybit.com',
            rate_limit: 800,
            data_format: 'json'
          },
          metrics: {
            latency: 68,
            success_rate: 98.9,
            data_quality: 92.1
          }
        },
        {
          id: 'gate',
          name: 'Gate.io',
          code: 'GATE',
          status: 'active',
          enabled: true,
          symbols_count: 634,
          rules_count: 4,
          config: {
            api_endpoint: 'https://api.gateio.ws',
            rate_limit: 400,
            data_format: 'json'
          },
          metrics: {
            latency: 73,
            success_rate: 99.1,
            data_quality: 93.5
          }
        }
      ];

      // 生成数据质量指标
      const mockQualityMetrics: QualityMetric[] = [
        {
          component: '价格数据',
          score: 96.8,
          issues_count: 12,
          trend: 'up',
          last_updated: new Date().toISOString()
        },
        {
          component: '交易量数据',
          score: 94.2,
          issues_count: 18,
          trend: 'stable',
          last_updated: new Date().toISOString()
        },
        {
          component: '时间戳数据',
          score: 99.1,
          issues_count: 3,
          trend: 'up',
          last_updated: new Date().toISOString()
        },
        {
          component: '交易对数据',
          score: 92.5,
          issues_count: 24,
          trend: 'down',
          last_updated: new Date().toISOString()
        }
      ];

      // 生成质量问题
      const mockQualityIssues: QualityIssue[] = [
        {
          id: 'issue_001',
          type: 'missing_data',
          severity: 'medium',
          description: 'Binance BTCUSDT 15:30-15:35 缺失交易数据',
          source: 'binance',
          created_at: new Date(Date.now() - 1800000).toISOString(),
          status: 'open'
        },
        {
          id: 'issue_002',
          type: 'outlier',
          severity: 'high',
          description: 'OKX ETHUSDT 价格出现异常波动',
          source: 'okx',
          created_at: new Date(Date.now() - 3600000).toISOString(),
          status: 'open'
        },
        {
          id: 'issue_003',
          type: 'duplicate',
          severity: 'low',
          description: 'Gate.io 检测到重复的交易记录',
          source: 'gate',
          created_at: new Date(Date.now() - 7200000).toISOString(),
          status: 'resolved'
        }
      ];

      // 计算概览统计
      const activeRules = mockRules.filter(r => r.enabled).length;
      const activeExchanges = mockExchanges.filter(e => e.enabled).length;
      const avgQualityScore = mockQualityMetrics.reduce((sum, m) => sum + m.score, 0) / mockQualityMetrics.length;
      const openIssues = mockQualityIssues.filter(i => i.status === 'open').length;

      setCleaningRules(mockRules);
      setExchanges(mockExchanges);
      setQualityMetrics(mockQualityMetrics);
      setQualityIssues(mockQualityIssues);
      setQualityScore(avgQualityScore);
      
      setOverviewStats({
        total_rules: mockRules.length,
        active_rules: activeRules,
        total_exchanges: mockExchanges.length,
        active_exchanges: activeExchanges,
        data_quality_score: Math.round(avgQualityScore),
        pending_issues: openIssues,
        processed_records: 2847561,
        error_rate: 0.5
      });

    } catch (error) {
      console.error('初始化清洗模块数据失败:', error);
      message.error('数据加载失败');
    } finally {
      setLoading(false);
    }
  };

  // 清洗规则操作
  const handleRuleAction = async (action: string, rule?: CleaningRule) => {
    try {
      switch (action) {
        case 'create':
          setEditingRule(null);
          setRuleModalVisible(true);
          ruleForm.resetFields();
          break;
        
        case 'edit':
          setEditingRule(rule!);
          setRuleModalVisible(true);
          ruleForm.setFieldsValue(rule);
          break;
        
        case 'delete':
          setCleaningRules(prev => prev.filter(r => r.id !== rule!.id));
          message.success('规则删除成功');
          break;
        
        case 'toggle':
          setCleaningRules(prev => prev.map(r => 
            r.id === rule!.id ? { ...r, enabled: !r.enabled } : r
          ));
          message.success(`规则已${rule!.enabled ? '禁用' : '启用'}`);
          break;
        
        case 'test':
          message.loading('正在测试规则...', 2);
          setTimeout(() => {
            message.success('规则测试通过');
          }, 2000);
          break;
      }
    } catch (error) {
      message.error(`操作失败: ${error.message}`);
    }
  };

  // 交易所操作
  const handleExchangeAction = async (action: string, exchange?: Exchange) => {
    try {
      switch (action) {
        case 'edit':
          setEditingExchange(exchange!);
          setExchangeModalVisible(true);
          exchangeForm.setFieldsValue(exchange);
          break;
        
        case 'toggle':
          setExchanges(prev => prev.map(e => 
            e.id === exchange!.id ? { ...e, enabled: !e.enabled, status: e.enabled ? 'inactive' : 'active' } : e
          ));
          message.success(`${exchange!.name}已${exchange!.enabled ? '禁用' : '启用'}`);
          break;
        
        case 'test':
          message.loading(`正在测试${exchange!.name}连接...`, 2);
          setTimeout(() => {
            message.success('连接测试成功');
          }, 2000);
          break;
        
        case 'reset':
          Modal.confirm({
            title: `确认重置${exchange!.name}配置？`,
            content: '重置后将恢复默认配置',
            onOk: () => {
              message.success('配置重置成功');
            }
          });
          break;
      }
    } catch (error) {
      message.error(`操作失败: ${error.message}`);
    }
  };

  // 数据质量操作
  const handleQualityAction = async (action: string, issue?: QualityIssue) => {
    try {
      switch (action) {
        case 'resolve':
          setQualityIssues(prev => prev.map(i => 
            i.id === issue!.id ? { ...i, status: 'resolved' } : i
          ));
          message.success('问题已解决');
          break;
        
        case 'analyze':
          message.loading('正在分析数据质量...', 3);
          setTimeout(() => {
            notification.success({
              message: '质量分析完成',
              description: '发现3个新的质量问题，总体质量分数提升2.1分'
            });
          }, 3000);
          break;
        
        case 'generate_report':
          message.loading('正在生成质量报告...', 2);
          setTimeout(() => {
            message.success('质量报告生成成功');
          }, 2000);
          break;
      }
    } catch (error) {
      message.error(`操作失败: ${error.message}`);
    }
  };

  // 保存规则
  const handleSaveRule = async (values: any) => {
    try {
      const ruleData = {
        ...values,
        id: editingRule?.id || `rule_${Date.now()}`,
        created_at: editingRule?.created_at || new Date().toISOString(),
        updated_at: new Date().toISOString(),
        usage_count: editingRule?.usage_count || 0,
        success_rate: editingRule?.success_rate || 0
      };

      if (editingRule) {
        setCleaningRules(prev => prev.map(r => r.id === editingRule.id ? ruleData : r));
        message.success('规则更新成功');
      } else {
        setCleaningRules(prev => [...prev, ruleData]);
        message.success('规则创建成功');
      }

      setRuleModalVisible(false);
      setEditingRule(null);
    } catch (error) {
      message.error('保存失败');
    }
  };

  useEffect(() => {
    initializeData();
  }, []);

  // 表格列定义
  const ruleColumns = [
    { title: '规则名称', dataIndex: 'name', key: 'name' },
    { title: '类型', dataIndex: 'type', key: 'type', 
      render: (type: string) => <Tag color={type === 'filter' ? 'blue' : type === 'transform' ? 'green' : type === 'validate' ? 'orange' : 'purple'}>{type}</Tag> 
    },
    { title: '优先级', dataIndex: 'priority', key: 'priority' },
    { title: '使用次数', dataIndex: 'usage_count', key: 'usage_count' },
    { title: '成功率', dataIndex: 'success_rate', key: 'success_rate', render: (rate: number) => `${rate}%` },
    { 
      title: '状态', 
      dataIndex: 'enabled', 
      key: 'enabled',
      render: (enabled: boolean) => <Badge status={enabled ? 'success' : 'default'} text={enabled ? '启用' : '禁用'} />
    },
    {
      title: '操作',
      key: 'actions',
      render: (_, record: CleaningRule) => (
        <Space>
          <Button size="small" icon={<EditOutlined />} onClick={() => handleRuleAction('edit', record)}>编辑</Button>
          <Button size="small" icon={<ExperimentOutlined />} onClick={() => handleRuleAction('test', record)}>测试</Button>
          <Button size="small" icon={record.enabled ? <PauseCircleOutlined /> : <PlayCircleOutlined />} 
                  onClick={() => handleRuleAction('toggle', record)}>
            {record.enabled ? '禁用' : '启用'}
          </Button>
          <Popconfirm title="确认删除？" onConfirm={() => handleRuleAction('delete', record)}>
            <Button size="small" icon={<DeleteOutlined />} danger>删除</Button>
          </Popconfirm>
        </Space>
      )
    }
  ];

  const exchangeColumns = [
    { title: '交易所', dataIndex: 'name', key: 'name' },
    { title: '代码', dataIndex: 'code', key: 'code' },
    { 
      title: '状态', 
      dataIndex: 'status', 
      key: 'status',
      render: (status: string) => (
        <Badge status={status === 'active' ? 'success' : status === 'maintenance' ? 'warning' : 'default'} 
               text={status === 'active' ? '正常' : status === 'maintenance' ? '维护中' : '停用'} />
      )
    },
    { title: '交易对', dataIndex: 'symbols_count', key: 'symbols_count' },
    { title: '规则数', dataIndex: 'rules_count', key: 'rules_count' },
    { title: '延迟(ms)', dataIndex: ['metrics', 'latency'], key: 'latency' },
    { title: '成功率', dataIndex: ['metrics', 'success_rate'], key: 'success_rate', render: (rate: number) => `${rate}%` },
    { title: '质量分', dataIndex: ['metrics', 'data_quality'], key: 'data_quality', render: (score: number) => `${score}%` },
    {
      title: '操作',
      key: 'actions',
      render: (_, record: Exchange) => (
        <Space>
          <Button size="small" icon={<SettingOutlined />} onClick={() => handleExchangeAction('edit', record)}>配置</Button>
          <Button size="small" icon={<LinkOutlined />} onClick={() => handleExchangeAction('test', record)}>测试</Button>
          <Button size="small" icon={record.enabled ? <PauseCircleOutlined /> : <PlayCircleOutlined />} 
                  onClick={() => handleExchangeAction('toggle', record)}>
            {record.enabled ? '禁用' : '启用'}
          </Button>
          <Button size="small" onClick={() => handleExchangeAction('reset', record)}>重置</Button>
        </Space>
      )
    }
  ];

  const issueColumns = [
    { title: '类型', dataIndex: 'type', key: 'type',
      render: (type: string) => <Tag color={type === 'missing_data' ? 'red' : type === 'outlier' ? 'orange' : type === 'duplicate' ? 'blue' : 'purple'}>{type}</Tag>
    },
    { title: '严重程度', dataIndex: 'severity', key: 'severity',
      render: (severity: string) => <Tag color={severity === 'critical' ? 'red' : severity === 'high' ? 'orange' : severity === 'medium' ? 'yellow' : 'green'}>{severity}</Tag>
    },
    { title: '描述', dataIndex: 'description', key: 'description' },
    { title: '来源', dataIndex: 'source', key: 'source' },
    { title: '状态', dataIndex: 'status', key: 'status',
      render: (status: string) => <Badge status={status === 'resolved' ? 'success' : status === 'open' ? 'error' : 'default'} text={status} />
    },
    {
      title: '操作',
      key: 'actions',
      render: (_, record: QualityIssue) => (
        <Space>
          {record.status === 'open' && (
            <Button size="small" type="primary" onClick={() => handleQualityAction('resolve', record)}>解决</Button>
          )}
          <Button size="small">详情</Button>
        </Space>
      )
    }
  ];

  return (
    <div style={{ padding: '24px' }}>
      <div style={{ marginBottom: '24px' }}>
        <h1>数据清洗模块</h1>
        <p style={{ color: '#666' }}>清洗规则管理、交易所配置、数据质量监控</p>
      </div>

      <Tabs 
        activeKey={activeTab} 
        onChange={setActiveTab} 
        size="large"
        items={[
          {
            key: 'overview',
            label: '概览',
            children: (
              <>
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic title="清洗规则" value={overviewStats.total_rules} 
                                suffix={`/ ${overviewStats.active_rules}个启用`} />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic title="交易所" value={overviewStats.total_exchanges} 
                                suffix={`/ ${overviewStats.active_exchanges}个活跃`} />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic title="数据质量分数" value={overviewStats.data_quality_score} 
                                suffix="/ 100" valueStyle={{ color: '#3f8600' }} />
                    </Card>
                  </Col>
                  <Col xs={24} sm={6}>
                    <Card>
                      <Statistic title="待处理问题" value={overviewStats.pending_issues} 
                                valueStyle={{ color: overviewStats.pending_issues > 0 ? '#cf1322' : '#3f8600' }} />
                    </Card>
                  </Col>
                </Row>

                <Row gutter={[16, 16]}>
                  <Col xs={24} md={12}>
                    <Card title="处理统计" size="small">
                      <Statistic title="已处理记录" value={overviewStats.processed_records} />
                      <div style={{ marginTop: '16px' }}>
                        <div>错误率</div>
                        <Progress percent={overviewStats.error_rate} size="small" status={overviewStats.error_rate > 1 ? 'exception' : 'success'} />
                      </div>
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card title="快速操作" size="small">
                      <Space direction="vertical" style={{ width: '100%' }}>
                        <Button type="primary" block onClick={() => handleRuleAction('create')}>创建清洗规则</Button>
                        <Button block onClick={() => handleQualityAction('analyze')}>运行质量分析</Button>
                        <Button block onClick={() => handleQualityAction('generate_report')}>生成质量报告</Button>
                      </Space>
                    </Card>
                  </Col>
                </Row>
              </>
            )
          },
          {
            key: 'rules',
            label: `清洗规则 (${cleaningRules.length})`,
            children: (
              <Card 
                title="清洗规则管理"
                extra={
                  <Space>
                    <Button icon={<UploadOutlined />}>导入</Button>
                    <Button icon={<DownloadOutlined />}>导出</Button>
                    <Button type="primary" icon={<PlusOutlined />} onClick={() => handleRuleAction('create')}>新建规则</Button>
                  </Space>
                }
              >
                <Table
                  dataSource={cleaningRules}
                  columns={ruleColumns}
                  rowKey="id"
                  loading={loading}
                  pagination={{ pageSize: 10 }}
                />
              </Card>
            )
          },
          {
            key: 'exchanges',
            label: `交易所配置 (${exchanges.length})`,
            children: (
              <Card title="交易所配置管理">
                <Table
                  dataSource={exchanges}
                  columns={exchangeColumns}
                  rowKey="id"
                  loading={loading}
                  pagination={false}
                />
              </Card>
            )
          },
          {
            key: 'quality',
            label: '数据质量',
            children: (
              <>
                <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
                  <Col xs={24} md={12}>
                    <Card title="质量指标" size="small">
                      {qualityMetrics.map((metric, index) => (
                        <div key={index} style={{ marginBottom: '16px' }}>
                          <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                            <span>{metric.component}</span>
                            <span>{metric.score.toFixed(1)}%</span>
                          </div>
                          <Progress percent={metric.score} size="small" 
                                   status={metric.score >= 95 ? 'success' : metric.score >= 85 ? 'normal' : 'exception'} />
                        </div>
                      ))}
                    </Card>
                  </Col>
                  <Col xs={24} md={12}>
                    <Card title="质量趋势" size="small">
                      <Statistic title="整体质量分数" value={qualityScore.toFixed(1)} suffix="/ 100" 
                                valueStyle={{ color: '#3f8600' }} />
                      <div style={{ marginTop: '16px', color: '#666' }}>
                        过去7天质量分数稳步提升，数据清洗效果良好
                      </div>
                    </Card>
                  </Col>
                </Row>

                <Card title="质量问题" extra={<Button onClick={() => handleQualityAction('analyze')}>重新分析</Button>}>
                  <Table
                    dataSource={qualityIssues}
                    columns={issueColumns}
                    rowKey="id"
                    loading={loading}
                    pagination={{ pageSize: 10 }}
                  />
                </Card>
              </>
            )
          }
        ]}
      />

      {/* 规则编辑模态框 */}
      <Modal
        title={editingRule ? '编辑清洗规则' : '创建清洗规则'}
        open={ruleModalVisible}
        onCancel={() => setRuleModalVisible(false)}
        onOk={() => ruleForm.submit()}
        width={600}
      >
        <Form form={ruleForm} onFinish={handleSaveRule} layout="vertical">
          <Form.Item name="name" label="规则名称" rules={[{ required: true }]}>
            <Input placeholder="输入规则名称" />
          </Form.Item>
          <Form.Item name="description" label="规则描述">
            <TextArea placeholder="输入规则描述" rows={3} />
          </Form.Item>
          <Row gutter={16}>
            <Col span={12}>
              <Form.Item name="type" label="规则类型" rules={[{ required: true }]}>
                <Select placeholder="选择规则类型">
                  <Option value="filter">过滤</Option>
                  <Option value="transform">转换</Option>
                  <Option value="validate">验证</Option>
                  <Option value="normalize">标准化</Option>
                </Select>
              </Form.Item>
            </Col>
            <Col span={12}>
              <Form.Item name="priority" label="优先级" rules={[{ required: true }]}>
                <Input type="number" placeholder="输入优先级(1-10)" />
              </Form.Item>
            </Col>
          </Row>
          <Form.Item name="enabled" label="启用规则" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item name="config" label="规则配置">
            <TextArea placeholder="输入JSON格式的规则配置" rows={4} />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
}