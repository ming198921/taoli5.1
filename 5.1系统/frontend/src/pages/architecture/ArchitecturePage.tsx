import React from 'react';
import { Routes, Route } from 'react-router-dom';
import { Card, Typography } from 'antd';
import { SettingOutlined } from '@ant-design/icons';

const { Title, Text } = Typography;

// 占位符组件
const PlaceholderComponent: React.FC<{ title: string; description: string }> = ({ title, description }) => (
  <div className="p-6">
    <Card>
      <div className="text-center py-12">
        <SettingOutlined className="text-6xl text-gray-300 mb-4" />
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

export const ArchitecturePage: React.FC = () => {
  return (
    <Routes>
      <Route 
        path="/limits" 
        element={
          <PlaceholderComponent 
            title="限制器控制" 
            description="系统资源限制配置、使用情况监控和违规检查"
          />
        } 
      />
      <Route 
        path="/enforcement" 
        element={
          <PlaceholderComponent 
            title="运行时强制执行" 
            description="实时资源监控、阈值检查和自动化响应机制"
          />
        } 
      />
      <Route 
        path="/hotreload" 
        element={
          <PlaceholderComponent 
            title="配置热重载" 
            description="配置文件监控、自动重载和回滚管理"
          />
        } 
      />
      <Route 
        path="/health" 
        element={
          <PlaceholderComponent 
            title="健康检查" 
            description="系统健康状态监控、依赖检查和告警配置"
          />
        } 
      />
      <Route 
        path="/resources" 
        element={
          <PlaceholderComponent 
            title="系统资源监控" 
            description="CPU、内存、磁盘和网络资源的实时监控和分析"
          />
        } 
      />
      <Route 
        path="/recovery" 
        element={
          <PlaceholderComponent 
            title="故障恢复控制" 
            description="故障检测、恢复策略执行和状态备份管理"
          />
        } 
      />
      <Route 
        path="*" 
        element={
          <PlaceholderComponent 
            title="Architecture系统架构模块" 
            description="请从左侧菜单选择具体的功能模块"
          />
        } 
      />
    </Routes>
  );
};