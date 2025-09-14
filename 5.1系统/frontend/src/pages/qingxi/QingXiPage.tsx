import React from 'react';
import { Routes, Route } from 'react-router-dom';
import { Card, Typography } from 'antd';
import { DatabaseOutlined } from '@ant-design/icons';
import { DataCollectorManager } from './components/DataCollectorManager';

const { Title, Text } = Typography;

// 占位符组件
const PlaceholderComponent: React.FC<{ title: string; description: string }> = ({ title, description }) => (
  <div className="p-6">
    <Card>
      <div className="text-center py-12">
        <DatabaseOutlined className="text-6xl text-gray-300 mb-4" />
        <Title level={3} type="secondary">{title}</Title>
        <Text type="secondary">{description}</Text>
        <div className="mt-4">
          <Text type="secondary" className="text-sm">
            该模块正在开发中，将提供完整的{title.toLowerCase()}功能
          </Text>
        </div>
      </div>
    </Card>
  </div>
);

export const QingXiPage: React.FC = () => {
  return (
    <Routes>
      <Route 
        path="/collector" 
        element={<DataCollectorManager />} 
      />
      <Route 
        path="/processor" 
        element={
          <PlaceholderComponent 
            title="批处理器控制" 
            description="监控和控制批处理任务的执行、队列管理和资源分配"
          />
        } 
      />
      <Route 
        path="/ccxt" 
        element={
          <PlaceholderComponent 
            title="CCXT适配器管理" 
            description="管理交易所适配器的连接状态、API配置和数据同步"
          />
        } 
      />
      <Route 
        path="/time" 
        element={
          <PlaceholderComponent 
            title="时间管理器" 
            description="系统时间同步、NTP配置和交易时段管理"
          />
        } 
      />
      <Route 
        path="/memory" 
        element={
          <PlaceholderComponent 
            title="内存管理器" 
            description="内存使用监控、垃圾回收优化和资源限制管理"
          />
        } 
      />
      <Route 
        path="/sources" 
        element={
          <PlaceholderComponent 
            title="第三方数据源" 
            description="管理外部数据提供商的API连接和数据质量监控"
          />
        } 
      />
      <Route 
        path="/nats" 
        element={
          <PlaceholderComponent 
            title="NATS消息队列" 
            description="消息队列状态监控、性能指标和连接管理"
          />
        } 
      />
      <Route 
        path="*" 
        element={
          <PlaceholderComponent 
            title="QingXi数据处理模块" 
            description="请从左侧菜单选择具体的功能模块"
          />
        } 
      />
    </Routes>
  );
};