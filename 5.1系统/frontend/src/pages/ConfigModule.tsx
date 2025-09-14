import React, { useState, useEffect } from 'react';
import { Card, Row, Col, Button, Table, Tabs, Tag, Badge, Tree, Input, Select } from 'antd';
import { ReloadOutlined, SettingOutlined, BranchesOutlined, ThunderboltOutlined, GlobalOutlined } from '@ant-design/icons';
import { configService } from '../services';

const { TabPane } = Tabs;
const { Search } = Input;
const { Option } = Select;

export default function ConfigModule() {
  const [loading, setLoading] = useState(false);
  const [configs, setConfigs] = useState<any[]>([]);
  const [versions, setVersions] = useState<any[]>([]);
  const [environments, setEnvironments] = useState<any[]>([]);
  const [hotReloadStatus, setHotReloadStatus] = useState<any>({});

  const fetchConfigData = async () => {
    setLoading(true);
    try {
      const [configData, versionData, envData, reloadStatus] = await Promise.all([
        configService.listConfigs(),
        configService.listVersions(),
        configService.listEnvironments(),
        configService.getHotReloadStatus()
      ]);
      
      // 确保所有数据都是数组格式
      setConfigs(Array.isArray(configData) ? configData : []);
      setVersions(Array.isArray(versionData) ? versionData : []);
      setEnvironments(Array.isArray(envData) ? envData : []);
      setHotReloadStatus(reloadStatus || {});
      
      if (!Array.isArray(configData)) console.warn('configService.listConfigs() returned non-array:', configData);
      if (!Array.isArray(versionData)) console.warn('configService.listVersions() returned non-array:', versionData);
      if (!Array.isArray(envData)) console.warn('configService.listEnvironments() returned non-array:', envData);
    } catch (error) {
      console.error('Failed to fetch config data:', error);
      // 设置默认空数组
      setConfigs([]);
      setVersions([]);
      setEnvironments([]);
      setHotReloadStatus({});
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchConfigData();
  }, []);

  const configColumns = [
    { title: '配置键', dataIndex: 'key', key: 'key' },
    { title: '类型', dataIndex: 'type', key: 'type', render: (type: string) => <Tag>{type}</Tag> },
    { title: '描述', dataIndex: 'description', key: 'description', ellipsis: true },
    { title: '更新时间', dataIndex: 'updated_at', key: 'updated_at', render: (time: string) => new Date(time).toLocaleDateString() }
  ];

  const versionColumns = [
    { title: '版本', dataIndex: 'version', key: 'version' },
    { title: '名称', dataIndex: 'name', key: 'name' },
    { title: '状态', dataIndex: 'deployed', key: 'deployed', render: (deployed: boolean) => <Badge status={deployed ? 'success' : 'default'} text={deployed ? '已部署' : '未部署'} /> },
    { title: '创建时间', dataIndex: 'created_at', key: 'created_at', render: (time: string) => new Date(time).toLocaleDateString() }
  ];

  const envColumns = [
    { title: '环境名称', dataIndex: 'name', key: 'name' },
    { title: '状态', dataIndex: 'status', key: 'status', render: (status: string) => <Badge status={status === 'active' ? 'success' : 'default'} text={status} /> },
    { title: '描述', dataIndex: 'description', key: 'description', ellipsis: true }
  ];

  return (
    <div style={{ padding: '24px' }}>
      <div style={{ marginBottom: '24px' }}>
        <h1 style={{ margin: 0, fontSize: '24px', fontWeight: 'bold' }}>
          配置服务管理 - 96个API接口
        </h1>
        <p style={{ margin: '8px 0 0 0', color: '#666' }}>
          端口: 4007 | 配置管理、版本控制、热重载、环境管理
        </p>
      </div>

      <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <SettingOutlined style={{ fontSize: '24px', color: '#1890ff', marginRight: '12px' }} />
              <div>
                <div style={{ fontSize: '24px', fontWeight: 'bold' }}>{configs.length}</div>
                <div style={{ color: '#666' }}>配置项总数</div>
              </div>
            </div>
          </Card>
        </Col>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <BranchesOutlined style={{ fontSize: '24px', color: '#52c41a', marginRight: '12px' }} />
              <div>
                <div style={{ fontSize: '24px', fontWeight: 'bold' }}>{versions.length}</div>
                <div style={{ color: '#666' }}>配置版本</div>
              </div>
            </div>
          </Card>
        </Col>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <GlobalOutlined style={{ fontSize: '24px', color: '#fa8c16', marginRight: '12px' }} />
              <div>
                <div style={{ fontSize: '24px', fontWeight: 'bold' }}>{environments.length}</div>
                <div style={{ color: '#666' }}>环境数量</div>
              </div>
            </div>
          </Card>
        </Col>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <ThunderboltOutlined style={{ fontSize: '24px', color: hotReloadStatus.enabled ? '#52c41a' : '#d9d9d9', marginRight: '12px' }} />
              <div>
                <div style={{ fontSize: '16px', fontWeight: 'bold' }}>
                  <Badge status={hotReloadStatus.enabled ? 'success' : 'default'} text={hotReloadStatus.enabled ? '已启用' : '已禁用'} />
                </div>
                <div style={{ color: '#666' }}>热重载状态</div>
              </div>
            </div>
          </Card>
        </Col>
      </Row>

      <Tabs defaultActiveKey="configs" size="large">
        <TabPane tab={`基础配置 (${configs.length})`} key="configs">
          <Card 
            title="配置管理"
            extra={
              <div>
                <Search placeholder="搜索配置..." style={{ width: 200, marginRight: 8 }} />
                <Button icon={<ReloadOutlined />} onClick={fetchConfigData} loading={loading}>刷新</Button>
              </div>
            }
          >
            <Table
              dataSource={configs}
              columns={configColumns}
              rowKey="key"
              loading={loading}
              pagination={{ pageSize: 20 }}
            />
          </Card>
        </TabPane>

        <TabPane tab={`版本控制 (${versions.length})`} key="versions">
          <Card 
            title="版本管理"
            extra={<Button type="primary">创建新版本</Button>}
          >
            <Table
              dataSource={versions}
              columns={versionColumns}
              rowKey="version"
              loading={loading}
              pagination={{ pageSize: 10 }}
            />
          </Card>
        </TabPane>

        <TabPane tab="热重载管理" key="hotreload">
          <Row gutter={[16, 16]}>
            <Col xs={24} md={12}>
              <Card title="热重载状态" size="small">
                <div style={{ lineHeight: '2.5' }}>
                  <div>状态: <Badge status={hotReloadStatus.enabled ? 'success' : 'default'} text={hotReloadStatus.enabled ? '已启用' : '已禁用'} /></div>
                  <div>最后重载: {hotReloadStatus.last_reload || 'N/A'}</div>
                  <div>重载状态: <Tag color={hotReloadStatus.status === 'success' ? 'green' : 'orange'}>{hotReloadStatus.status || 'idle'}</Tag></div>
                </div>
                <div style={{ marginTop: '16px' }}>
                  <Button type="primary" style={{ marginRight: 8 }}>
                    {hotReloadStatus.enabled ? '禁用热重载' : '启用热重载'}
                  </Button>
                  <Button>触发重载</Button>
                </div>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="服务重载状态" size="small">
                <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                  服务重载状态监控开发中...
                </div>
              </Card>
            </Col>
            <Col xs={24}>
              <Card title="重载历史" size="small">
                <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                  重载历史记录开发中...
                </div>
              </Card>
            </Col>
          </Row>
        </TabPane>

        <TabPane tab={`环境管理 (${environments.length})`} key="environments">
          <Card 
            title="环境配置"
            extra={<Button type="primary">创建新环境</Button>}
          >
            <Table
              dataSource={environments}
              columns={envColumns}
              rowKey="name"
              loading={loading}
              pagination={{ pageSize: 10 }}
            />
          </Card>
        </TabPane>

        <TabPane tab="配置树" key="tree">
          <Card title="配置层次结构">
            <div style={{ color: '#666', textAlign: 'center', padding: '40px' }}>
              配置树视图开发中...
            </div>
          </Card>
        </TabPane>

        <TabPane tab="安全管理" key="security">
          <Row gutter={[16, 16]}>
            <Col xs={24} md={12}>
              <Card title="权限控制" size="small">
                <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                  权限控制管理开发中...
                </div>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="审计日志" size="small">
                <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                  审计日志查看开发中...
                </div>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="密钥管理" size="small">
                <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                  密钥管理功能开发中...
                </div>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="备份恢复" size="small">
                <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                  备份恢复功能开发中...
                </div>
              </Card>
            </Col>
          </Row>
        </TabPane>
      </Tabs>
    </div>
  );
} 
import { Card, Row, Col, Button, Table, Tabs, Tag, Badge, Tree, Input, Select } from 'antd';
import { ReloadOutlined, SettingOutlined, BranchesOutlined, ThunderboltOutlined, GlobalOutlined } from '@ant-design/icons';
import { configService } from '../services';

const { TabPane } = Tabs;
const { Search } = Input;
const { Option } = Select;

export default function ConfigModule() {
  const [loading, setLoading] = useState(false);
  const [configs, setConfigs] = useState<any[]>([]);
  const [versions, setVersions] = useState<any[]>([]);
  const [environments, setEnvironments] = useState<any[]>([]);
  const [hotReloadStatus, setHotReloadStatus] = useState<any>({});

  const fetchConfigData = async () => {
    setLoading(true);
    try {
      const [configData, versionData, envData, reloadStatus] = await Promise.all([
        configService.listConfigs(),
        configService.listVersions(),
        configService.listEnvironments(),
        configService.getHotReloadStatus()
      ]);
      
      // 确保所有数据都是数组格式
      setConfigs(Array.isArray(configData) ? configData : []);
      setVersions(Array.isArray(versionData) ? versionData : []);
      setEnvironments(Array.isArray(envData) ? envData : []);
      setHotReloadStatus(reloadStatus || {});
      
      if (!Array.isArray(configData)) console.warn('configService.listConfigs() returned non-array:', configData);
      if (!Array.isArray(versionData)) console.warn('configService.listVersions() returned non-array:', versionData);
      if (!Array.isArray(envData)) console.warn('configService.listEnvironments() returned non-array:', envData);
    } catch (error) {
      console.error('Failed to fetch config data:', error);
      // 设置默认空数组
      setConfigs([]);
      setVersions([]);
      setEnvironments([]);
      setHotReloadStatus({});
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchConfigData();
  }, []);

  const configColumns = [
    { title: '配置键', dataIndex: 'key', key: 'key' },
    { title: '类型', dataIndex: 'type', key: 'type', render: (type: string) => <Tag>{type}</Tag> },
    { title: '描述', dataIndex: 'description', key: 'description', ellipsis: true },
    { title: '更新时间', dataIndex: 'updated_at', key: 'updated_at', render: (time: string) => new Date(time).toLocaleDateString() }
  ];

  const versionColumns = [
    { title: '版本', dataIndex: 'version', key: 'version' },
    { title: '名称', dataIndex: 'name', key: 'name' },
    { title: '状态', dataIndex: 'deployed', key: 'deployed', render: (deployed: boolean) => <Badge status={deployed ? 'success' : 'default'} text={deployed ? '已部署' : '未部署'} /> },
    { title: '创建时间', dataIndex: 'created_at', key: 'created_at', render: (time: string) => new Date(time).toLocaleDateString() }
  ];

  const envColumns = [
    { title: '环境名称', dataIndex: 'name', key: 'name' },
    { title: '状态', dataIndex: 'status', key: 'status', render: (status: string) => <Badge status={status === 'active' ? 'success' : 'default'} text={status} /> },
    { title: '描述', dataIndex: 'description', key: 'description', ellipsis: true }
  ];

  return (
    <div style={{ padding: '24px' }}>
      <div style={{ marginBottom: '24px' }}>
        <h1 style={{ margin: 0, fontSize: '24px', fontWeight: 'bold' }}>
          配置服务管理 - 96个API接口
        </h1>
        <p style={{ margin: '8px 0 0 0', color: '#666' }}>
          端口: 4007 | 配置管理、版本控制、热重载、环境管理
        </p>
      </div>

      <Row gutter={[16, 16]} style={{ marginBottom: '24px' }}>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <SettingOutlined style={{ fontSize: '24px', color: '#1890ff', marginRight: '12px' }} />
              <div>
                <div style={{ fontSize: '24px', fontWeight: 'bold' }}>{configs.length}</div>
                <div style={{ color: '#666' }}>配置项总数</div>
              </div>
            </div>
          </Card>
        </Col>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <BranchesOutlined style={{ fontSize: '24px', color: '#52c41a', marginRight: '12px' }} />
              <div>
                <div style={{ fontSize: '24px', fontWeight: 'bold' }}>{versions.length}</div>
                <div style={{ color: '#666' }}>配置版本</div>
              </div>
            </div>
          </Card>
        </Col>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <GlobalOutlined style={{ fontSize: '24px', color: '#fa8c16', marginRight: '12px' }} />
              <div>
                <div style={{ fontSize: '24px', fontWeight: 'bold' }}>{environments.length}</div>
                <div style={{ color: '#666' }}>环境数量</div>
              </div>
            </div>
          </Card>
        </Col>
        <Col xs={24} sm={6}>
          <Card>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <ThunderboltOutlined style={{ fontSize: '24px', color: hotReloadStatus.enabled ? '#52c41a' : '#d9d9d9', marginRight: '12px' }} />
              <div>
                <div style={{ fontSize: '16px', fontWeight: 'bold' }}>
                  <Badge status={hotReloadStatus.enabled ? 'success' : 'default'} text={hotReloadStatus.enabled ? '已启用' : '已禁用'} />
                </div>
                <div style={{ color: '#666' }}>热重载状态</div>
              </div>
            </div>
          </Card>
        </Col>
      </Row>

      <Tabs defaultActiveKey="configs" size="large">
        <TabPane tab={`基础配置 (${configs.length})`} key="configs">
          <Card 
            title="配置管理"
            extra={
              <div>
                <Search placeholder="搜索配置..." style={{ width: 200, marginRight: 8 }} />
                <Button icon={<ReloadOutlined />} onClick={fetchConfigData} loading={loading}>刷新</Button>
              </div>
            }
          >
            <Table
              dataSource={configs}
              columns={configColumns}
              rowKey="key"
              loading={loading}
              pagination={{ pageSize: 20 }}
            />
          </Card>
        </TabPane>

        <TabPane tab={`版本控制 (${versions.length})`} key="versions">
          <Card 
            title="版本管理"
            extra={<Button type="primary">创建新版本</Button>}
          >
            <Table
              dataSource={versions}
              columns={versionColumns}
              rowKey="version"
              loading={loading}
              pagination={{ pageSize: 10 }}
            />
          </Card>
        </TabPane>

        <TabPane tab="热重载管理" key="hotreload">
          <Row gutter={[16, 16]}>
            <Col xs={24} md={12}>
              <Card title="热重载状态" size="small">
                <div style={{ lineHeight: '2.5' }}>
                  <div>状态: <Badge status={hotReloadStatus.enabled ? 'success' : 'default'} text={hotReloadStatus.enabled ? '已启用' : '已禁用'} /></div>
                  <div>最后重载: {hotReloadStatus.last_reload || 'N/A'}</div>
                  <div>重载状态: <Tag color={hotReloadStatus.status === 'success' ? 'green' : 'orange'}>{hotReloadStatus.status || 'idle'}</Tag></div>
                </div>
                <div style={{ marginTop: '16px' }}>
                  <Button type="primary" style={{ marginRight: 8 }}>
                    {hotReloadStatus.enabled ? '禁用热重载' : '启用热重载'}
                  </Button>
                  <Button>触发重载</Button>
                </div>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="服务重载状态" size="small">
                <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                  服务重载状态监控开发中...
                </div>
              </Card>
            </Col>
            <Col xs={24}>
              <Card title="重载历史" size="small">
                <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                  重载历史记录开发中...
                </div>
              </Card>
            </Col>
          </Row>
        </TabPane>

        <TabPane tab={`环境管理 (${environments.length})`} key="environments">
          <Card 
            title="环境配置"
            extra={<Button type="primary">创建新环境</Button>}
          >
            <Table
              dataSource={environments}
              columns={envColumns}
              rowKey="name"
              loading={loading}
              pagination={{ pageSize: 10 }}
            />
          </Card>
        </TabPane>

        <TabPane tab="配置树" key="tree">
          <Card title="配置层次结构">
            <div style={{ color: '#666', textAlign: 'center', padding: '40px' }}>
              配置树视图开发中...
            </div>
          </Card>
        </TabPane>

        <TabPane tab="安全管理" key="security">
          <Row gutter={[16, 16]}>
            <Col xs={24} md={12}>
              <Card title="权限控制" size="small">
                <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                  权限控制管理开发中...
                </div>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="审计日志" size="small">
                <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                  审计日志查看开发中...
                </div>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="密钥管理" size="small">
                <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                  密钥管理功能开发中...
                </div>
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="备份恢复" size="small">
                <div style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
                  备份恢复功能开发中...
                </div>
              </Card>
            </Col>
          </Row>
        </TabPane>
      </Tabs>
    </div>
  );
} 