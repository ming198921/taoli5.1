import React, { useState, useEffect } from 'react';
import { Card, Row, Col, Button, Table, Tabs, Tag, Badge, Progress } from 'antd';
import { ReloadOutlined, RobotOutlined, ExperimentOutlined, ThunderboltOutlined, BarChartOutlined } from '@ant-design/icons';
import { aiModelService } from '../services';


export default function AIModelModule() {
  const [loading, setLoading] = useState(false);
  const [models, setModels] = useState<any[]>([]);
  const [trainingJobs, setTrainingJobs] = useState<any[]>([]);
  const [datasets, setDatasets] = useState<any[]>([]);

  const fetchAIData = async () => {
    setLoading(true);
    try {
      const [modelData, jobData, datasetData] = await Promise.all([
        aiModelService.listModels(),
        aiModelService.listTrainingJobs(),
        aiModelService.listDatasets()
      ]);
      
      // 确保所有数据都是数组格式
      setModels(Array.isArray(modelData) ? modelData : []);
      setTrainingJobs(Array.isArray(jobData) ? jobData : []);
      setDatasets(Array.isArray(datasetData) ? datasetData : []);
      
      if (!Array.isArray(modelData)) console.warn('aiModelService.listModels() returned non-array:', modelData);
      if (!Array.isArray(jobData)) console.warn('aiModelService.listTrainingJobs() returned non-array:', jobData);
      if (!Array.isArray(datasetData)) console.warn('aiModelService.listDatasets() returned non-array:', datasetData);
    } catch (error) {
      console.error('Failed to fetch AI data:', error);
      // 设置默认空数组
      setModels([]);
      setTrainingJobs([]);
      setDatasets([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchAIData();
  }, []);

  const modelColumns = [
    { title: '模型名称', dataIndex: 'name', key: 'name' },
    { 
      title: '类型', 
      dataIndex: 'type', 
      key: 'type',
      render: (type: string) => <Tag color="blue">{type}</Tag>
    },
    { 
      title: '状态', 
      dataIndex: 'status', 
      key: 'status',
      render: (status: string) => {
        const colors = { training: 'processing', deployed: 'success', inactive: 'default', error: 'error' };
        return <Badge status={colors[status as keyof typeof colors]} text={status} />;
      }
    },
    { title: '版本', dataIndex: 'version', key: 'version' },
    { 
      title: '准确率', 
      dataIndex: 'accuracy', 
      key: 'accuracy',
      render: (accuracy: number) => `${(accuracy * 100).toFixed(1)}%`
    },
    { 
      title: '创建时间', 
      dataIndex: 'created_at', 
      key: 'created_at',
      render: (time: string) => new Date(time).toLocaleDateString()
    }
  ];

  const jobColumns = [
    { title: '任务ID', dataIndex: 'id', key: 'id' },
    { title: '模型ID', dataIndex: 'model_id', key: 'model_id' },
    { 
      title: '状态', 
      dataIndex: 'status', 
      key: 'status',
      render: (status: string) => {
        const colors = { running: 'processing', completed: 'success', failed: 'error', cancelled: 'default' };
        return <Badge status={colors[status as keyof typeof colors]} text={status} />;
      }
    },
    { 
      title: '进度', 
      dataIndex: 'progress', 
      key: 'progress',
      render: (progress: number) => <Progress percent={progress} size="small" style={{ width: 100 }} />
    },
    { title: '数据集', dataIndex: 'dataset_id', key: 'dataset_id' },
    { 
      title: '开始时间', 
      dataIndex: 'started_at', 
      key: 'started_at',
      render: (time: string) => new Date(time).toLocaleDateString()
    }
  ];

  return (
    <div style={{ padding: '24px' }}>
      <div style={{ marginBottom: '24px' }}>
        <h1 style={{ margin: 0, fontSize: '24px', fontWeight: 'bold' }}>
          AI模型服务管理 - 48个API接口
        </h1>
        <p style={{ margin: '8px 0 0 0', color: '#666' }}>
          端口: 4006 | 机器学习模型管理、训练、推理、特征工程
        </p>
      </div>

      <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <RobotOutlined style={{ fontSize: '24px', color: '#1890ff', marginRight: '12px' }} />
              <div>
                <div style={{ fontSize: '24px', fontWeight: 'bold' }}>{models.length}</div>
                <div style={{ color: '#666' }}>AI模型总数</div>
              </div>
            </div>
          </Card>
        </Col>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <ExperimentOutlined style={{ fontSize: '24px', color: '#52c41a', marginRight: '12px' }} />
              <div>
                <div style={{ fontSize: '24px', fontWeight: 'bold' }}>
                  {models.filter(m => m.status === 'deployed').length}
                </div>
                <div style={{ color: '#666' }}>已部署模型</div>
              </div>
            </div>
          </Card>
        </Col>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <ThunderboltOutlined style={{ fontSize: '24px', color: '#fa8c16', marginRight: '12px' }} />
              <div>
                <div style={{ fontSize: '24px', fontWeight: 'bold' }}>
                  {trainingJobs.filter(j => j.status === 'running').length}
                </div>
                <div style={{ color: '#666' }}>训练中任务</div>
              </div>
            </div>
          </Card>
        </Col>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <BarChartOutlined style={{ fontSize: '24px', color: '#722ed1', marginRight: '12px' }} />
              <div>
                <div style={{ fontSize: '24px', fontWeight: 'bold' }}>{datasets.length}</div>
                <div style={{ color: '#666' }}>数据集数量</div>
              </div>
            </div>
          </Card>
        </Col>
      </Row>

      <Tabs 
        defaultActiveKey="models" 
        size="large"
        items={[
          {
            key: 'models',
            label: `模型管理 (${models.length})`,
            children: (
              <Card 
                title="AI模型列表"
                extra={
                  <div>
                    <Button type="primary" style={{ marginRight: 8 }}>新建模型</Button>
                    <Button icon={<ReloadOutlined />} onClick={fetchAIData} loading={loading}>刷新</Button>
                  </div>
                }
              >
                <Table
                  dataSource={models}
                  columns={modelColumns}
                  rowKey="id"
                  loading={loading}
                  pagination={{ pageSize: 10 }}
                />
              </Card>
            )
          },
          {
            key: 'training',
            label: `训练任务 (${trainingJobs.length})`,
            children: (
              <Card 
                title="训练任务管理"
                extra={<Button type="primary">创建训练任务</Button>}
              >
                <Table
                  dataSource={trainingJobs}
                  columns={jobColumns}
                  rowKey="id"
                  loading={loading}
                  pagination={{ pageSize: 10 }}
                />
              </Card>
            )
          },
          {
            key: 'inference',
            label: '推理服务',
            children: (
              <Row gutter={[16, 16]}>
                <Col xs={24} md={12}>
                  <Card title="推理统计" size="small">
                    <div style={{ lineHeight: '2.5' }}>
                      <div>今日推理次数: 1,245</div>
                      <div>平均响应时间: 45ms</div>
                      <div>成功率: 99.8%</div>
                      <div>并发请求: 12</div>
                    </div>
                  </Card>
                </Col>
                <Col xs={24} md={12}>
                  <Card title="推理性能" size="small">
                    <div style={{ marginBottom: '16px' }}>
                      <div>CPU使用率</div>
                      <Progress percent={68} />
                    </div>
                    <div style={{ marginBottom: '16px' }}>
                      <div>内存使用</div>
                      <Progress percent={45} />
                    </div>
                    <div>
                      <div>GPU使用率</div>
                      <Progress percent={82} />
                    </div>
                  </Card>
                </Col>
                <Col xs={24}>
                  <Card title="推理队列" size="small">
                    <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                      推理队列监控开发中...
                    </div>
                  </Card>
                </Col>
              </Row>
            )
          },
          {
            key: 'datasets',
            label: `数据集 (${datasets.length})`,
            children: (
              <Card 
                title="数据集管理"
                extra={<Button type="primary">上传数据集</Button>}
              >
                <div style={{ color: '#666', textAlign: 'center', padding: '40px' }}>
                  数据集管理界面开发中...
                </div>
              </Card>
            )
          },
          {
            key: 'features',
            label: '特征工程',
            children: (
              <Row gutter={[16, 16]}>
                <Col xs={24} md={12}>
                  <Card title="特征提取" size="small">
                    <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                      特征提取工具开发中...
                    </div>
                  </Card>
                </Col>
                <Col xs={24} md={12}>
                  <Card title="特征选择" size="small">
                    <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                      特征选择工具开发中...
                    </div>
                  </Card>
                </Col>
                <Col xs={24} md={12}>
                  <Card title="特征重要性" size="small">
                    <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                      特征重要性分析开发中...
                    </div>
                  </Card>
                </Col>
                <Col xs={24} md={12}>
                  <Card title="特征统计" size="small">
                    <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                      特征统计信息开发中...
                    </div>
                  </Card>
                </Col>
              </Row>
            )
          }
        ]}
      />
    </div>
  );
} 
import { Card, Row, Col, Button, Table, Tabs, Tag, Badge, Progress } from 'antd';
import { ReloadOutlined, RobotOutlined, ExperimentOutlined, ThunderboltOutlined, BarChartOutlined } from '@ant-design/icons';
import { aiModelService } from '../services';


export default function AIModelModule() {
  const [loading, setLoading] = useState(false);
  const [models, setModels] = useState<any[]>([]);
  const [trainingJobs, setTrainingJobs] = useState<any[]>([]);
  const [datasets, setDatasets] = useState<any[]>([]);

  const fetchAIData = async () => {
    setLoading(true);
    try {
      const [modelData, jobData, datasetData] = await Promise.all([
        aiModelService.listModels(),
        aiModelService.listTrainingJobs(),
        aiModelService.listDatasets()
      ]);
      
      // 确保所有数据都是数组格式
      setModels(Array.isArray(modelData) ? modelData : []);
      setTrainingJobs(Array.isArray(jobData) ? jobData : []);
      setDatasets(Array.isArray(datasetData) ? datasetData : []);
      
      if (!Array.isArray(modelData)) console.warn('aiModelService.listModels() returned non-array:', modelData);
      if (!Array.isArray(jobData)) console.warn('aiModelService.listTrainingJobs() returned non-array:', jobData);
      if (!Array.isArray(datasetData)) console.warn('aiModelService.listDatasets() returned non-array:', datasetData);
    } catch (error) {
      console.error('Failed to fetch AI data:', error);
      // 设置默认空数组
      setModels([]);
      setTrainingJobs([]);
      setDatasets([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchAIData();
  }, []);

  const modelColumns = [
    { title: '模型名称', dataIndex: 'name', key: 'name' },
    { 
      title: '类型', 
      dataIndex: 'type', 
      key: 'type',
      render: (type: string) => <Tag color="blue">{type}</Tag>
    },
    { 
      title: '状态', 
      dataIndex: 'status', 
      key: 'status',
      render: (status: string) => {
        const colors = { training: 'processing', deployed: 'success', inactive: 'default', error: 'error' };
        return <Badge status={colors[status as keyof typeof colors]} text={status} />;
      }
    },
    { title: '版本', dataIndex: 'version', key: 'version' },
    { 
      title: '准确率', 
      dataIndex: 'accuracy', 
      key: 'accuracy',
      render: (accuracy: number) => `${(accuracy * 100).toFixed(1)}%`
    },
    { 
      title: '创建时间', 
      dataIndex: 'created_at', 
      key: 'created_at',
      render: (time: string) => new Date(time).toLocaleDateString()
    }
  ];

  const jobColumns = [
    { title: '任务ID', dataIndex: 'id', key: 'id' },
    { title: '模型ID', dataIndex: 'model_id', key: 'model_id' },
    { 
      title: '状态', 
      dataIndex: 'status', 
      key: 'status',
      render: (status: string) => {
        const colors = { running: 'processing', completed: 'success', failed: 'error', cancelled: 'default' };
        return <Badge status={colors[status as keyof typeof colors]} text={status} />;
      }
    },
    { 
      title: '进度', 
      dataIndex: 'progress', 
      key: 'progress',
      render: (progress: number) => <Progress percent={progress} size="small" style={{ width: 100 }} />
    },
    { title: '数据集', dataIndex: 'dataset_id', key: 'dataset_id' },
    { 
      title: '开始时间', 
      dataIndex: 'started_at', 
      key: 'started_at',
      render: (time: string) => new Date(time).toLocaleDateString()
    }
  ];

  return (
    <div style={{ padding: '24px' }}>
      <div style={{ marginBottom: '24px' }}>
        <h1 style={{ margin: 0, fontSize: '24px', fontWeight: 'bold' }}>
          AI模型服务管理 - 48个API接口
        </h1>
        <p style={{ margin: '8px 0 0 0', color: '#666' }}>
          端口: 4006 | 机器学习模型管理、训练、推理、特征工程
        </p>
      </div>

      <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <RobotOutlined style={{ fontSize: '24px', color: '#1890ff', marginRight: '12px' }} />
              <div>
                <div style={{ fontSize: '24px', fontWeight: 'bold' }}>{models.length}</div>
                <div style={{ color: '#666' }}>AI模型总数</div>
              </div>
            </div>
          </Card>
        </Col>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <ExperimentOutlined style={{ fontSize: '24px', color: '#52c41a', marginRight: '12px' }} />
              <div>
                <div style={{ fontSize: '24px', fontWeight: 'bold' }}>
                  {models.filter(m => m.status === 'deployed').length}
                </div>
                <div style={{ color: '#666' }}>已部署模型</div>
              </div>
            </div>
          </Card>
        </Col>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <ThunderboltOutlined style={{ fontSize: '24px', color: '#fa8c16', marginRight: '12px' }} />
              <div>
                <div style={{ fontSize: '24px', fontWeight: 'bold' }}>
                  {trainingJobs.filter(j => j.status === 'running').length}
                </div>
                <div style={{ color: '#666' }}>训练中任务</div>
              </div>
            </div>
          </Card>
        </Col>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <BarChartOutlined style={{ fontSize: '24px', color: '#722ed1', marginRight: '12px' }} />
              <div>
                <div style={{ fontSize: '24px', fontWeight: 'bold' }}>{datasets.length}</div>
                <div style={{ color: '#666' }}>数据集数量</div>
              </div>
            </div>
          </Card>
        </Col>
      </Row>

      <Tabs 
        defaultActiveKey="models" 
        size="large"
        items={[
          {
            key: 'models',
            label: `模型管理 (${models.length})`,
            children: (
              <Card 
                title="AI模型列表"
                extra={
                  <div>
                    <Button type="primary" style={{ marginRight: 8 }}>新建模型</Button>
                    <Button icon={<ReloadOutlined />} onClick={fetchAIData} loading={loading}>刷新</Button>
                  </div>
                }
              >
                <Table
                  dataSource={models}
                  columns={modelColumns}
                  rowKey="id"
                  loading={loading}
                  pagination={{ pageSize: 10 }}
                />
              </Card>
            )
          },
          {
            key: 'training',
            label: `训练任务 (${trainingJobs.length})`,
            children: (
              <Card 
                title="训练任务管理"
                extra={<Button type="primary">创建训练任务</Button>}
              >
                <Table
                  dataSource={trainingJobs}
                  columns={jobColumns}
                  rowKey="id"
                  loading={loading}
                  pagination={{ pageSize: 10 }}
                />
              </Card>
            )
          },
          {
            key: 'inference',
            label: '推理服务',
            children: (
              <Row gutter={[16, 16]}>
                <Col xs={24} md={12}>
                  <Card title="推理统计" size="small">
                    <div style={{ lineHeight: '2.5' }}>
                      <div>今日推理次数: 1,245</div>
                      <div>平均响应时间: 45ms</div>
                      <div>成功率: 99.8%</div>
                      <div>并发请求: 12</div>
                    </div>
                  </Card>
                </Col>
                <Col xs={24} md={12}>
                  <Card title="推理性能" size="small">
                    <div style={{ marginBottom: '16px' }}>
                      <div>CPU使用率</div>
                      <Progress percent={68} />
                    </div>
                    <div style={{ marginBottom: '16px' }}>
                      <div>内存使用</div>
                      <Progress percent={45} />
                    </div>
                    <div>
                      <div>GPU使用率</div>
                      <Progress percent={82} />
                    </div>
                  </Card>
                </Col>
                <Col xs={24}>
                  <Card title="推理队列" size="small">
                    <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                      推理队列监控开发中...
                    </div>
                  </Card>
                </Col>
              </Row>
            )
          },
          {
            key: 'datasets',
            label: `数据集 (${datasets.length})`,
            children: (
              <Card 
                title="数据集管理"
                extra={<Button type="primary">上传数据集</Button>}
              >
                <div style={{ color: '#666', textAlign: 'center', padding: '40px' }}>
                  数据集管理界面开发中...
                </div>
              </Card>
            )
          },
          {
            key: 'features',
            label: '特征工程',
            children: (
              <Row gutter={[16, 16]}>
                <Col xs={24} md={12}>
                  <Card title="特征提取" size="small">
                    <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                      特征提取工具开发中...
                    </div>
                  </Card>
                </Col>
                <Col xs={24} md={12}>
                  <Card title="特征选择" size="small">
                    <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                      特征选择工具开发中...
                    </div>
                  </Card>
                </Col>
                <Col xs={24} md={12}>
                  <Card title="特征重要性" size="small">
                    <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                      特征重要性分析开发中...
                    </div>
                  </Card>
                </Col>
                <Col xs={24} md={12}>
                  <Card title="特征统计" size="small">
                    <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                      特征统计信息开发中...
                    </div>
                  </Card>
                </Col>
              </Row>
            )
          }
        ]}
      />
    </div>
  );
} 