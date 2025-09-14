import React, { useState, useEffect } from 'react';
import { Card, Row, Col, Button, Table, Tabs, Input, Select, Tag, Badge, Progress, Switch } from 'antd';
import { 
  SearchOutlined, 
  ReloadOutlined, 
  PlusOutlined,
  SettingOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined
} from '@ant-design/icons';
import { cleaningService } from '../services';

const { TabPane } = Tabs;
const { Search } = Input;
const { Option } = Select;

export default function CleaningModule() {
  const [loading, setLoading] = useState(false);
  const [cleaningRules, setCleaningRules] = useState<any[]>([]);
  const [exchanges, setExchanges] = useState<any[]>([]);
  const [qualityMetrics, setQualityMetrics] = useState<any>({});

  // 获取清洗规则
  const fetchCleaningRules = async () => {
    setLoading(true);
    try {
      const rules = await cleaningService.listCleaningRules();
      // 确保rules是数组
      setCleaningRules(Array.isArray(rules) ? rules : []);
      if (!Array.isArray(rules)) console.warn('cleaningService.listCleaningRules() returned non-array:', rules);
    } catch (error) {
      console.error('Failed to fetch cleaning rules:', error);
      setCleaningRules([]);
    } finally {
      setLoading(false);
    }
  };

  // 获取交易所配置
  const fetchExchanges = async () => {
    try {
      const exchangeList = await cleaningService.listExchanges();
      // 确保exchangeList是数组
      setExchanges(Array.isArray(exchangeList) ? exchangeList : []);
      if (!Array.isArray(exchangeList)) console.warn('cleaningService.listExchanges() returned non-array:', exchangeList);
    } catch (error) {
      console.error('Failed to fetch exchanges:', error);
      setExchanges([]);
    }
  };

  // 获取质量指标
  const fetchQualityMetrics = async () => {
    try {
      const metrics = await cleaningService.getQualityMetrics();
      setQualityMetrics(metrics);
    } catch (error) {
      console.error('Failed to fetch quality metrics:', error);
    }
  };

  useEffect(() => {
    fetchCleaningRules();
    fetchExchanges();
    fetchQualityMetrics();
  }, []);

  // 清洗规则表格列
  const ruleColumns = [
    {
      title: '规则名称',
      dataIndex: 'name',
      key: 'name'
    },
    {
      title: '状态',
      dataIndex: 'enabled',
      key: 'enabled',
      render: (enabled: boolean) => (
        <Badge 
          status={enabled ? 'success' : 'default'} 
          text={enabled ? '启用' : '禁用'} 
        />
      )
    },
    {
      title: '模式',
      dataIndex: 'pattern',
      key: 'pattern',
      ellipsis: true
    },
    {
      title: '优先级',
      dataIndex: 'priority',
      key: 'priority',
      render: (priority: number) => <Tag color="blue">{priority}</Tag>
    },
    {
      title: '创建时间',
      dataIndex: 'created_at',
      key: 'created_at',
      render: (time: string) => new Date(time).toLocaleDateString()
    }
  ];

  // 交易所表格列
  const exchangeColumns = [
    {
      title: '交易所',
      dataIndex: 'name',
      key: 'name'
    },
    {
      title: '状态',
      dataIndex: 'enabled',
      key: 'enabled',
      render: (enabled: boolean) => (
        <Badge 
          status={enabled ? 'success' : 'error'} 
          text={enabled ? '启用' : '禁用'} 
        />
      )
    },
    {
      title: '交易对数量',
      dataIndex: 'symbols',
      key: 'symbols',
      render: (symbols: string[]) => symbols?.length || 0
    },
    {
      title: '规则数量',
      dataIndex: 'rules',
      key: 'rules',
      render: (rules: any[]) => rules?.length || 0
    }
  ];

  return (
    <div style={{ padding: '24px' }}>
      {/* 页面标题 */}
      <div style={{ marginBottom: '24px' }}>
        <h1 style={{ margin: 0, fontSize: '24px', fontWeight: 'bold' }}>
          清洗服务管理 - 52个API接口
        </h1>
        <p style={{ margin: '8px 0 0 0', color: '#666' }}>
          端口: 4002 | 市场数据清洗、规范化、质量控制
        </p>
      </div>

      {/* 数据质量概览 */}
      <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
        <Col xs={24} sm={8}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <CheckCircleOutlined style={{ fontSize: '24px', color: '#52c41a', marginRight: '12px' }} />
              <div>
                <div style={{ fontSize: '24px', fontWeight: 'bold', color: '#52c41a' }}>
                  {((qualityMetrics.score || 0) * 100).toFixed(1)}%
                </div>
                <div style={{ color: '#666' }}>数据质量分数</div>
              </div>
            </div>
          </Card>
        </Col>
        <Col xs={24} sm={8}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <ExclamationCircleOutlined style={{ fontSize: '24px', color: '#fa8c16', marginRight: '12px' }} />
              <div>
                <div style={{ fontSize: '24px', fontWeight: 'bold' }}>
                  {qualityMetrics.issues?.length || 0}
                </div>
                <div style={{ color: '#666' }}>待处理问题</div>
              </div>
            </div>
          </Card>
        </Col>
        <Col xs={24} sm={8}>
          <Card>
            <div>
              <div style={{ marginBottom: '8px', display: 'flex', justifyContent: 'space-between' }}>
                <span>处理进度</span>
                <span>85%</span>
              </div>
              <Progress percent={85} status="active" />
            </div>
          </Card>
        </Col>
      </Row>

      {/* 主要内容标签页 */}
      <Tabs defaultActiveKey="rules" size="large">
        {/* 清洗规则管理 */}
        <TabPane tab={`清洗规则 (${cleaningRules.length})`} key="rules">
          <Card 
            title="清洗规则管理"
            extra={
              <div>
                <Button icon={<PlusOutlined />} type="primary" style={{ marginRight: 8 }}>
                  新建规则
                </Button>
                <Button icon={<ReloadOutlined />} onClick={fetchCleaningRules} loading={loading}>
                  刷新
                </Button>
              </div>
            }
          >
            <div style={{ marginBottom: '16px' }}>
              <Search
                placeholder="搜索清洗规则..."
                style={{ width: 300, marginRight: 16 }}
                enterButton={<SearchOutlined />}
              />
              <Select placeholder="规则状态" style={{ width: 120 }}>
                <Option value="enabled">启用</Option>
                <Option value="disabled">禁用</Option>
              </Select>
            </div>
            
            <Table
              dataSource={cleaningRules}
              columns={ruleColumns}
              rowKey="id"
              loading={loading}
              pagination={{
                pageSize: 20,
                showSizeChanger: true,
                showTotal: (total) => `共 ${total} 条规则`
              }}
            />
          </Card>
        </TabPane>

        {/* 交易所配置 */}
        <TabPane tab={`交易所配置 (${exchanges.length})`} key="exchanges">
          <Card 
            title="交易所配置管理"
            extra={
              <Button icon={<ReloadOutlined />} onClick={fetchExchanges}>
                刷新
              </Button>
            }
          >
            <Table
              dataSource={exchanges}
              columns={exchangeColumns}
              rowKey="id"
              pagination={{
                pageSize: 10,
                showTotal: (total) => `共 ${total} 个交易所`
              }}
            />
          </Card>
        </TabPane>

        {/* 数据质量监控 */}
        <TabPane tab="质量监控" key="quality">
          <Row gutter={[16, 16]}>
            <Col xs={24} md={12}>
              <Card title="质量指标" size="small">
                <div style={{ lineHeight: '2.5' }}>
                  <div>数据完整性: <Progress percent={92} size="small" style={{ width: 200 }} /></div>
                  <div>数据准确性: <Progress percent={88} size="small" style={{ width: 200 }} /></div>
                  <div>数据一致性: <Progress percent={95} size="small" style={{ width: 200 }} /></div>
                  <div>数据时效性: <Progress percent={90} size="small" style={{ width: 200 }} /></div>
                </div>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="质量趋势" size="small">
                <p>质量趋势图表开发中...</p>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="异常检测" size="small">
                <p>异常检测功能开发中...</p>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="质量报告" size="small">
                <p>质量报告功能开发中...</p>
              </Card>
            </Col>
          </Row>
        </TabPane>

        {/* SIMD优化 */}
        <TabPane tab="SIMD优化" key="simd">
          <Card title="SIMD优化配置">
            <Row gutter={[16, 16]}>
              <Col xs={24} md={8}>
                <Card size="small" title="CPU向量化">
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <span>启用SIMD加速</span>
                    <Switch defaultChecked />
                  </div>
                  <div style={{ marginTop: '12px', fontSize: '12px', color: '#666' }}>
                    使用AVX2指令集加速数据处理
                  </div>
                </Card>
              </Col>
              <Col xs={24} md={8}>
                <Card size="small" title="并行处理">
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <span>多线程处理</span>
                    <Switch defaultChecked />
                  </div>
                  <div style={{ marginTop: '12px', fontSize: '12px', color: '#666' }}>
                    使用线程池并行处理数据
                  </div>
                </Card>
              </Col>
              <Col xs={24} md={8}>
                <Card size="small" title="内存优化">
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <span>零拷贝优化</span>
                    <Switch defaultChecked />
                  </div>
                  <div style={{ marginTop: '12px', fontSize: '12px', color: '#666' }}>
                    减少内存拷贝提升性能
                  </div>
                </Card>
              </Col>
            </Row>
          </Card>
        </TabPane>
      </Tabs>
    </div>
  );
} 