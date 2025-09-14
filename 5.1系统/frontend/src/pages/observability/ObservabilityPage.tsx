import React from 'react';
import { Routes, Route } from 'react-router-dom';
import { Card, Typography } from 'antd';
import { MonitorOutlined } from '@ant-design/icons';

const { Title, Text } = Typography;

// 占位符组件
const PlaceholderComponent: React.FC<{ title: string; description: string }> = ({ title, description }) => (
  <div className="p-6">
    <Card>
      <div className="text-center py-12">
        <MonitorOutlined className="text-6xl text-gray-300 mb-4" />
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

export const ObservabilityPage: React.FC = () => {
  return (
    <Routes>
      <Route 
        path="/tracing" 
        element={
          <PlaceholderComponent 
            title="分布式追踪控制" 
            description="服务调用链追踪、性能分析和异常检测"
          />
        } 
      />
      <Route 
        path="/metrics" 
        element={
          <PlaceholderComponent 
            title="指标收集控制" 
            description="自定义指标定义、数据收集和查询分析"
          />
        } 
      />
      <Route 
        path="/alerts" 
        element={
          <PlaceholderComponent 
            title="告警规则控制" 
            description="告警规则管理、通知配置和历史记录查询"
          />
        } 
      />
      <Route 
        path="/logs" 
        element={
          <PlaceholderComponent 
            title="日志聚合控制" 
            description="日志收集、搜索分析和导出管理"
          />
        } 
      />
      <Route 
        path="/dashboards" 
        element={
          <PlaceholderComponent 
            title="可视化管理" 
            description="仪表板创建、图表配置和数据展示管理"
          />
        } 
      />
      <Route 
        path="*" 
        element={
          <PlaceholderComponent 
            title="Observability可观测性模块" 
            description="请从左侧菜单选择具体的功能模块"
          />
        } 
      />
    </Routes>
  );
};