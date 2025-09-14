import React from 'react';
import { Routes, Route } from 'react-router-dom';
import { Card, Typography } from 'antd';
import { SafetyOutlined } from '@ant-design/icons';

const { Title, Text } = Typography;

// 占位符组件
const PlaceholderComponent: React.FC<{ title: string; description: string }> = ({ title, description }) => (
  <div className="p-6">
    <Card>
      <div className="text-center py-12">
        <SafetyOutlined className="text-6xl text-gray-300 mb-4" />
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

export const SettingsPage: React.FC = () => {
  return (
    <Routes>
      <Route 
        path="/profile" 
        element={
          <PlaceholderComponent 
            title="个人资料" 
            description="用户信息管理、密码修改和头像设置"
          />
        } 
      />
      <Route 
        path="/preferences" 
        element={
          <PlaceholderComponent 
            title="偏好设置" 
            description="主题配置、语言设置和界面自定义"
          />
        } 
      />
      <Route 
        path="/notifications" 
        element={
          <PlaceholderComponent 
            title="通知设置" 
            description="消息通知配置、告警订阅和推送设置"
          />
        } 
      />
      <Route 
        path="/security" 
        element={
          <PlaceholderComponent 
            title="安全设置" 
            description="二次验证、访问权限和安全日志"
          />
        } 
      />
      <Route 
        path="*" 
        element={
          <PlaceholderComponent 
            title="系统设置" 
            description="请从左侧菜单选择具体的设置项目"
          />
        } 
      />
    </Routes>
  );
};