import React, { useState, useEffect } from 'react';
import { 
  Card, 
  Table, 
  Button, 
  Tag, 
  Space, 
  Typography, 
  Statistic, 
  Progress,
  Modal,
  Form,
  Input,
  Select,
  message,
  Popconfirm
} from 'antd';
import {
  PlayCircleOutlined,
  PauseCircleOutlined,
  SettingOutlined,
  ReloadOutlined,
  EyeOutlined,
  ApiOutlined
} from '@ant-design/icons';
import { apiClient } from '@/api/client';
import { qingxiAPI } from '@/api/qingxi';

const { Title, Text } = Typography;
const { Option } = Select;

interface DataCollector {
  id: string;
  name: string;
  exchange: string;
  status: 'running' | 'stopped' | 'error';
  symbols: string[];
  last_update: string;
  data_quality: number;
  latency_ms: number;
  error_rate: number;
}

interface CleaningPerformance {
  overall_stats: {
    fastest_ms: number;
    slowest_ms: number;
    average_ms: number;
    total_count: number;
    last_update: number | null;
  };
  per_currency_stats: Record<string, {
    fastest_ms: number;
    slowest_ms: number;
    average_ms: number;
    total_count: number;
    last_update: number | null;
  }>;
  per_collector_stats?: Record<string, {
    collector_name: string;
    exchange: string;
    overall_performance: {
      fastest_ms: number;
      slowest_ms: number;
      average_ms: number;
      total_count: number;
    };
    slowest_pairs: Array<{
      pair: string;
      average_ms: number;
      slowest_ms: number;
    }>;
  }>;
  system_info: {
    v3_optimizations_enabled: boolean;
    target_range_ms: string;
    performance_status: string;
  };
}

export const DataCollectorManager: React.FC = () => {
  const [collectors, setCollectors] = useState<DataCollector[]>([]);
  const [loading, setLoading] = useState(false);
  const [configModalVisible, setConfigModalVisible] = useState(false);
  const [selectedCollector, setSelectedCollector] = useState<DataCollector | null>(null);
  const [cleaningPerformance, setCleaningPerformance] = useState<CleaningPerformance | null>(null);
  const [form] = Form.useForm();

  // 获取数据收集器列表
  const fetchCollectors = async () => {
    setLoading(true);
    try {
      // 直接使用完整的后端URL，不依赖apiClient的可能错误配置
      const response = await fetch('http://localhost:8080/api/qingxi/collectors/list', {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      
      if (response.ok) {
        const data = await response.json();
        console.log('收集器数据响应:', data);
        // 确保只设置真实数据，如果没有数据就设置空数组
        setCollectors(data || []);
      } else {
        console.warn('后端API返回错误状态:', response.status);
        setCollectors([]); // 后端不可用时显示空状态
        message.warning('后端服务暂时不可用，显示空状态');
      }
    } catch (error) {
      console.error('Failed to fetch collectors:', error);
      setCollectors([]); // 网络错误时显示空状态
      message.warning('无法连接到后端服务');
    } finally {
      setLoading(false);
    }
  };

  // 获取清洗性能数据
  const fetchCleaningPerformance = async () => {
    try {
      const response = await fetch('http://localhost:8080/api/qingxi/cleaning/performance', {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      
      if (response.ok) {
        const performance = await response.json();
        console.log('清洗性能数据响应:', performance);
        setCleaningPerformance(performance);
      } else {
        console.warn('清洗性能API不可用:', response.status);
        setCleaningPerformance(null);
      }
    } catch (error) {
      console.error('Failed to fetch cleaning performance:', error);
      setCleaningPerformance(null);
    }
  };

  // 启动收集器
  const startCollector = async (collectorId: string) => {
    const collector = collectors.find(c => c.id === collectorId);
    if (!collector) {
      message.error('未找到指定的收集器');
      return;
    }

    // 防止重复点击 - 如果已经是运行状态就不处理
    if (collector.status === 'running') {
      console.log('收集器已经是运行状态，忽略重复操作');
      return;
    }

    try {
      console.log(`正在启动收集器: ${collector.name} (ID: ${collectorId})`);
      
      // 调用启动API - 不要预先修改UI状态，避免抖动
      const response = await fetch(`http://localhost:8080/api/qingxi/collectors/${collectorId}/start`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      console.log('启动收集器响应:', response);
      
      message.success(`收集器 ${collector.name} 启动命令已发送`);
      
      // 立即刷新状态获取真实后端状态
      await fetchCollectors();
      
    } catch (error: any) {
      console.error('启动收集器失败:', error);
      message.error(`启动收集器 ${collector.name} 失败: ${error?.message || '网络错误'}`);
      
      // 发生错误时也要刷新状态，确保UI与后端一致
      await fetchCollectors();
    }
  };

  // 停止收集器
  const stopCollector = async (collectorId: string) => {
    const collector = collectors.find(c => c.id === collectorId);
    if (!collector) {
      message.error('未找到指定的收集器');
      return;
    }

    // 防止重复点击 - 如果已经是停止状态就不处理
    if (collector.status === 'stopped') {
      console.log('收集器已经是停止状态，忽略重复操作');
      return;
    }

    try {
      console.log(`正在停止收集器: ${collector.name} (ID: ${collectorId})`);
      
      // 调用停止API - 不要预先修改UI状态，避免抖动
      const response = await fetch(`http://localhost:8080/api/qingxi/collectors/${collectorId}/stop`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      console.log('停止收集器响应:', response);
      
      message.success(`收集器 ${collector.name} 停止命令已发送`);
      
      // 立即刷新状态获取真实后端状态
      await fetchCollectors();
      
    } catch (error: any) {
      console.error('停止收集器失败:', error);
      message.error(`停止收集器 ${collector.name} 失败: ${error?.message || '网络错误'}`);
      
      // 发生错误时也要刷新状态，确保UI与后端一致
      await fetchCollectors();
    }
  };

  // 打开配置弹窗
  const openConfigModal = async (collector: DataCollector) => {
    setSelectedCollector(collector);
    try {
      const config = await apiClient.get(`/api/qingxi/collectors/${collector.id}/config`);
      form.setFieldsValue(config);
      setConfigModalVisible(true);
    } catch (error) {
      message.error('获取配置失败');
    }
  };

  // 保存配置
  const saveConfig = async () => {
    if (!selectedCollector) return;
    
    try {
      const values = await form.validateFields();
      await apiClient.post(`/api/qingxi/collectors/${selectedCollector.id}/config`, values);
      message.success('配置更新成功');
      setConfigModalVisible(false);
      fetchCollectors();
    } catch (error) {
      message.error('配置更新失败');
    }
  };

  // 获取状态颜色
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running': return 'green';
      case 'stopped': return 'default';
      case 'error': return 'red';
      default: return 'default';
    }
  };

  // 获取状态文本
  const getStatusText = (status: string) => {
    switch (status) {
      case 'running': return '运行中';
      case 'stopped': return '已停止';
      case 'error': return '错误';
      default: return '未知';
    }
  };

  // 表格列定义
  const columns = [
    {
      title: '名称',
      dataIndex: 'name',
      key: 'name',
      render: (text: string, record: DataCollector) => (
        <Space>
          <ApiOutlined />
          <div>
            <div className="font-medium">{text}</div>
            <Text type="secondary" className="text-xs">{record.exchange}</Text>
          </div>
        </Space>
      ),
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => (
        <Tag color={getStatusColor(status)}>
          {getStatusText(status)}
        </Tag>
      ),
    },
    {
      title: '交易对',
      dataIndex: 'symbols',
      key: 'symbols',
      render: (symbols: string[]) => (
        <div>
          {symbols.slice(0, 3).map((symbol, index) => (
            <Tag key={index}>{symbol}</Tag>
          ))}
          {symbols.length > 3 && (
            <Tag>+{symbols.length - 3}个</Tag>
          )}
        </div>
      ),
    },
    {
      title: '数据质量',
      dataIndex: 'data_quality',
      key: 'data_quality',
      render: (quality: number) => (
        <Progress 
          percent={quality} 
          size="small" 
          status={quality > 95 ? 'success' : quality > 90 ? 'normal' : 'exception'}
          format={percent => `${percent?.toFixed(1)}%`}
        />
      ),
    },
    {
      title: '延迟',
      dataIndex: 'latency_ms',
      key: 'latency_ms',
      render: (latency: number) => (
        <Statistic 
          value={latency} 
          suffix="ms" 
          precision={0}
          valueStyle={{ fontSize: '14px', color: latency < 50 ? '#52c41a' : latency < 100 ? '#faad14' : '#ff4d4f' }}
        />
      ),
    },
    {
      title: '错误率',
      dataIndex: 'error_rate',
      key: 'error_rate',
      render: (rate: number) => (
        <Text style={{ color: rate < 0.01 ? '#52c41a' : rate < 0.05 ? '#faad14' : '#ff4d4f' }}>
          {(rate * 100).toFixed(2)}%
        </Text>
      ),
    },
    {
      title: '操作',
      key: 'actions',
      render: (_, record: DataCollector) => (
        <Space>
          {record.status === 'running' ? (
            <Popconfirm
              title="确定要停止此数据收集器吗？"
              onConfirm={() => stopCollector(record.id)}
              okText="确定"
              cancelText="取消"
            >
              <Button 
                type="text" 
                icon={<PauseCircleOutlined />} 
                size="small"
                danger
              >
                停止
              </Button>
            </Popconfirm>
          ) : (
            <Button 
              type="text" 
              icon={<PlayCircleOutlined />} 
              size="small"
              onClick={() => startCollector(record.id)}
            >
              启动
            </Button>
          )}
          <Button 
            type="text" 
            icon={<SettingOutlined />} 
            size="small"
            onClick={() => openConfigModal(record)}
          >
            配置
          </Button>
          <Button 
            type="text" 
            icon={<EyeOutlined />} 
            size="small"
          >
            详情
          </Button>
        </Space>
      ),
    },
  ];

  // 统计数据 - 基于真实后端数据
  const runningCount = collectors.filter(c => c.status === 'running').length;
  const stoppedCount = collectors.filter(c => c.status === 'stopped').length;
  const errorCount = collectors.filter(c => c.status === 'error').length;
  
  // 只有当有数据时才计算平均值，否则显示为0或无数据状态
  const avgLatency = collectors.length > 0 
    ? collectors.reduce((sum, c) => sum + (c.latency_ms || 0), 0) / collectors.length
    : 0;
  
  const avgQuality = collectors.length > 0 
    ? collectors.reduce((sum, c) => sum + (c.data_quality || 0), 0) / collectors.length
    : 0;

  useEffect(() => {
    fetchCollectors();
    fetchCleaningPerformance();
    
    // 每30秒刷新数据收集器列表
    const collectorsInterval = setInterval(() => {
      fetchCollectors();
    }, 30000);
    
    // 每30秒刷新清洗性能数据（符合用户要求）
    const performanceInterval = setInterval(() => {
      fetchCleaningPerformance();
    }, 30000);
    
    return () => {
      clearInterval(collectorsInterval);
      clearInterval(performanceInterval);
    };
  }, []);

  return (
    <div className="p-6">
      {/* 页头 */}
      <div className="mb-6">
        <div className="flex items-center justify-between">
          <div>
            <Title level={2} className="mb-1">
              <ApiOutlined className="mr-2" />
              数据收集器管理
            </Title>
            <Text type="secondary">
              管理市场数据收集器的配置、状态监控和性能优化
            </Text>
          </div>
          <Button 
            icon={<ReloadOutlined />} 
            loading={loading}
            onClick={fetchCollectors}
          >
            刷新
          </Button>
        </div>
      </div>

      {/* 统计卡片 */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
        <Card>
          <Statistic
            title="运行中"
            value={runningCount}
            suffix={`/ ${collectors.length}`}
            valueStyle={{ color: '#52c41a' }}
            prefix={<PlayCircleOutlined />}
          />
        </Card>
        <Card>
          <Statistic
            title="已停止"
            value={stoppedCount}
            suffix={`/ ${collectors.length}`}
            valueStyle={{ color: '#d9d9d9' }}
            prefix={<PauseCircleOutlined />}
          />
        </Card>
        <Card>
          <Statistic
            title="平均延迟"
            value={collectors.length > 0 ? avgLatency : 0}
            precision={1}
            suffix={collectors.length > 0 ? "ms" : ""}
            valueStyle={{ 
              color: collectors.length === 0 ? '#d9d9d9' : 
                     avgLatency < 50 ? '#52c41a' : 
                     avgLatency < 100 ? '#faad14' : '#ff4d4f' 
            }}
            formatter={(value) => 
              collectors.length === 0 ? '暂无数据' : `${value}`
            }
          />
        </Card>
        <Card>
          <Statistic
            title="平均数据质量"
            value={collectors.length > 0 ? avgQuality : 0}
            precision={1}
            suffix={collectors.length > 0 ? "%" : ""}
            valueStyle={{ 
              color: collectors.length === 0 ? '#d9d9d9' : '#1890ff'
            }}
            formatter={(value) => 
              collectors.length === 0 ? '暂无数据' : `${value}`
            }
          />
        </Card>
      </div>

      {/* 清洗性能统计卡片 */}
      <div className="mb-6">
        <Title level={4} className="mb-4">V3+O1 数据清洗性能监控</Title>
        {cleaningPerformance ? (
          <>
            <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
              <Card>
                <Statistic
                  title="最快清洗速度"
                  value={cleaningPerformance.overall_stats.fastest_ms}
                  precision={3}
                  suffix="ms"
                  valueStyle={{ 
                    color: cleaningPerformance.overall_stats.fastest_ms < 0.1 ? '#52c41a' : '#1890ff'
                  }}
                />
              </Card>
              <Card>
                <Statistic
                  title="最慢清洗速度"
                  value={cleaningPerformance.overall_stats.slowest_ms}
                  precision={3}
                  suffix="ms"
                  valueStyle={{ 
                    color: cleaningPerformance.overall_stats.slowest_ms > 0.3 ? '#ff4d4f' : '#52c41a'
                  }}
                />
              </Card>
              <Card>
                <Statistic
                  title="平均清洗速度"
                  value={cleaningPerformance.overall_stats.average_ms}
                  precision={3}
                  suffix="ms"
                  valueStyle={{ 
                    color: (() => {
                      const avg = cleaningPerformance.overall_stats.average_ms;
                      if (avg < 0.1) return '#52c41a';
                      if (avg < 0.3) return '#faad14';
                      return '#ff4d4f';
                    })()
                  }}
                />
              </Card>
              <Card>
                <Statistic
                  title="处理总数"
                  value={cleaningPerformance.overall_stats.total_count}
                  valueStyle={{ color: '#1890ff' }}
                />
              </Card>
            </div>
            
            {/* 性能状态指示器 */}
            <Card className="mt-4">
              <div className="flex items-center justify-between">
                <div>
                  <Text strong>系统性能状态: </Text>
                  <Tag color={
                    cleaningPerformance.system_info.performance_status === 'excellent' ? 'green' :
                    cleaningPerformance.system_info.performance_status === 'good' ? 'blue' :
                    cleaningPerformance.system_info.performance_status === 'normal' ? 'orange' : 'red'
                  }>
                    {cleaningPerformance.system_info.performance_status === 'excellent' ? '优秀' :
                     cleaningPerformance.system_info.performance_status === 'good' ? '良好' :
                     cleaningPerformance.system_info.performance_status === 'normal' ? '正常' : '需要优化'}
                  </Tag>
                </div>
                <div>
                  <Text type="secondary">
                    目标范围: {cleaningPerformance.system_info.target_range_ms} ms |{' '}
                    V3优化: {cleaningPerformance.system_info.v3_optimizations_enabled ? '已启用' : '已禁用'}
                  </Text>
                </div>
              </div>
            </Card>
          </>
        ) : (
          <Card>
            <div className="text-center py-8">
              <Text type="secondary" className="text-lg">
                ⚠️ 清洗性能监控数据未就绪
              </Text>
              <br />
              <Text type="secondary" className="text-sm">
                等待后端API响应中，请确保后端系统正常运行...
              </Text>
            </div>
          </Card>
        )}
        
        {/* 按币种清洗性能表格 */}
        {cleaningPerformance && Object.keys(cleaningPerformance.per_currency_stats).length > 0 && (
          <Card className="mt-4" title="按交易对清洗性能统计">
            <Table
              size="small"
              pagination={false}
              scroll={{ x: true }}
              dataSource={Object.entries(cleaningPerformance.per_currency_stats).map(([currency, stats]) => ({
                key: currency,
                currency,
                ...stats
              }))}
              columns={[
                {
                  title: '交易对',
                  dataIndex: 'currency',
                  key: 'currency',
                  fixed: 'left',
                  width: 120,
                  render: (currency: string) => (
                    <Tag color="blue">{currency}</Tag>
                  )
                },
                {
                  title: '最快 (ms)',
                  dataIndex: 'fastest_ms',
                  key: 'fastest_ms',
                  width: 100,
                  render: (value: number) => (
                    <Text style={{ color: value < 0.1 ? '#52c41a' : '#1890ff' }}>
                      {value.toFixed(3)}
                    </Text>
                  ),
                  sorter: (a: any, b: any) => a.fastest_ms - b.fastest_ms,
                },
                {
                  title: '最慢 (ms)',
                  dataIndex: 'slowest_ms',
                  key: 'slowest_ms',
                  width: 100,
                  render: (value: number) => (
                    <Text style={{ color: value > 0.3 ? '#ff4d4f' : '#52c41a' }}>
                      {value.toFixed(3)}
                    </Text>
                  ),
                  sorter: (a: any, b: any) => a.slowest_ms - b.slowest_ms,
                },
                {
                  title: '平均 (ms)',
                  dataIndex: 'average_ms',
                  key: 'average_ms',
                  width: 100,
                  render: (value: number) => (
                    <Text style={{ 
                      color: value < 0.1 ? '#52c41a' : value < 0.3 ? '#faad14' : '#ff4d4f' 
                    }}>
                      {value.toFixed(3)}
                    </Text>
                  ),
                  sorter: (a: any, b: any) => a.average_ms - b.average_ms,
                },
                {
                  title: '处理数量',
                  dataIndex: 'total_count',
                  key: 'total_count',
                  width: 100,
                  render: (value: number) => (
                    <Text>{value.toLocaleString()}</Text>
                  ),
                  sorter: (a: any, b: any) => a.total_count - b.total_count,
                },
                {
                  title: '性能状态',
                  key: 'performance',
                  width: 100,
                  render: (_: any, record: any) => {
                    const avg = record.average_ms;
                    if (avg < 0.1) return <Tag color="green">优秀</Tag>;
                    if (avg < 0.2) return <Tag color="blue">良好</Tag>;
                    if (avg < 0.3) return <Tag color="orange">正常</Tag>;
                    return <Tag color="red">需优化</Tag>;
                  }
                }
              ]}
            />
          </Card>
        )}

        {/* 按数据收集器清洗性能监控 */}
        {cleaningPerformance && cleaningPerformance.per_collector_stats && Object.keys(cleaningPerformance.per_collector_stats).length > 0 && (
          <div className="mt-6">
            <Title level={5} className="mb-4">按数据收集器清洗性能详细监控</Title>
            <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
              {Object.entries(cleaningPerformance.per_collector_stats).map(([collectorId, stats]) => (
                <Card 
                  key={collectorId}
                  title={
                    <div className="flex items-center justify-between">
                      <div>
                        <Tag color="blue" className="mr-2">{stats.exchange.toUpperCase()}</Tag>
                        <Text strong>{stats.collector_name}</Text>
                      </div>
                      <Tag color={
                        stats.overall_performance.average_ms < 0.1 ? 'green' :
                        stats.overall_performance.average_ms < 0.2 ? 'blue' :
                        stats.overall_performance.average_ms < 0.3 ? 'orange' : 'red'
                      }>
                        {stats.overall_performance.average_ms < 0.1 ? '优秀' :
                         stats.overall_performance.average_ms < 0.2 ? '良好' :
                         stats.overall_performance.average_ms < 0.3 ? '正常' : '需优化'}
                      </Tag>
                    </div>
                  }
                  size="small"
                >
                  {/* 收集器性能统计 */}
                  <div className="grid grid-cols-2 gap-2 mb-4">
                    <Statistic
                      title="最快"
                      value={stats.overall_performance.fastest_ms}
                      precision={3}
                      suffix="ms"
                      valueStyle={{ 
                        fontSize: '12px',
                        color: stats.overall_performance.fastest_ms < 0.1 ? '#52c41a' : '#1890ff'
                      }}
                    />
                    <Statistic
                      title="最慢"
                      value={stats.overall_performance.slowest_ms}
                      precision={3}
                      suffix="ms"
                      valueStyle={{ 
                        fontSize: '12px',
                        color: stats.overall_performance.slowest_ms > 0.3 ? '#ff4d4f' : '#52c41a'
                      }}
                    />
                    <Statistic
                      title="平均"
                      value={stats.overall_performance.average_ms}
                      precision={3}
                      suffix="ms"
                      valueStyle={{ 
                        fontSize: '12px',
                        color: (() => {
                          const avg = stats.overall_performance.average_ms;
                          if (avg < 0.1) return '#52c41a';
                          if (avg < 0.3) return '#faad14';
                          return '#ff4d4f';
                        })()
                      }}
                    />
                    <Statistic
                      title="处理数"
                      value={stats.overall_performance.total_count}
                      valueStyle={{ fontSize: '12px', color: '#1890ff' }}
                    />
                  </div>
                  
                  {/* 最慢清洗交易对列表 */}
                  {stats.slowest_pairs.length > 0 && (
                    <div>
                      <Text strong className="block mb-2">🐌 最慢清洗交易对 (Top {stats.slowest_pairs.length}):</Text>
                      <div className="space-y-1">
                        {stats.slowest_pairs.slice(0, 5).map((pair, index) => (
                          <div key={pair.pair} className="flex items-center justify-between text-xs">
                            <div className="flex items-center">
                              <Text className="w-4 text-center font-mono text-gray-500">#{index + 1}</Text>
                              <Tag size="small" color="orange" className="ml-1">{pair.pair}</Tag>
                            </div>
                            <div className="flex items-center space-x-2">
                              <Text style={{ 
                                color: pair.average_ms > 0.3 ? '#ff4d4f' : pair.average_ms > 0.2 ? '#faad14' : '#52c41a',
                                fontSize: '11px'
                              }}>
                                均: {pair.average_ms.toFixed(3)}ms
                              </Text>
                              <Text style={{ 
                                color: pair.slowest_ms > 0.5 ? '#ff4d4f' : '#faad14',
                                fontSize: '11px'
                              }}>
                                慢: {pair.slowest_ms.toFixed(3)}ms
                              </Text>
                            </div>
                          </div>
                        ))}
                        {stats.slowest_pairs.length > 5 && (
                          <Text type="secondary" className="text-xs">
                            还有 {stats.slowest_pairs.length - 5} 个交易对...
                          </Text>
                        )}
                      </div>
                    </div>
                  )}
                </Card>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* 数据收集器表格 */}
      <Card>
        <Table
          columns={columns}
          dataSource={collectors}
          rowKey="id"
          loading={loading}
          pagination={{
            pageSize: 10,
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total) => `共 ${total} 个收集器`,
          }}
        />
      </Card>

      {/* 配置弹窗 */}
      <Modal
        title={`配置收集器: ${selectedCollector?.name}`}
        open={configModalVisible}
        onOk={saveConfig}
        onCancel={() => setConfigModalVisible(false)}
        width={600}
      >
        <Form
          form={form}
          layout="vertical"
        >
          <Form.Item
            name="symbols"
            label="交易对"
            rules={[{ required: true, message: '请输入交易对' }]}
          >
            <Select
              mode="tags"
              placeholder="输入交易对，如 BTC/USDT"
              tokenSeparators={[',']}
            >
              <Option value="BTC/USDT">BTC/USDT</Option>
              <Option value="ETH/USDT">ETH/USDT</Option>
              <Option value="BNB/USDT">BNB/USDT</Option>
            </Select>
          </Form.Item>
          
          <Form.Item
            name="update_interval"
            label="更新间隔 (毫秒)"
            rules={[{ required: true, message: '请输入更新间隔' }]}
          >
            <Input type="number" placeholder="100" />
          </Form.Item>
          
          <Form.Item
            name="batch_size"
            label="批处理大小"
            rules={[{ required: true, message: '请输入批处理大小' }]}
          >
            <Input type="number" placeholder="1000" />
          </Form.Item>
          
          <Form.Item
            name="retry_attempts"
            label="重试次数"
            rules={[{ required: true, message: '请输入重试次数' }]}
          >
            <Input type="number" placeholder="3" />
          </Form.Item>
          
          <Form.Item
            name="timeout_seconds"
            label="超时时间 (秒)"
            rules={[{ required: true, message: '请输入超时时间' }]}
          >
            <Input type="number" placeholder="10" />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
};