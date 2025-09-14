import React from 'react';
import { Routes, Route } from 'react-router-dom';
import { Card, Typography } from 'antd';
import { ApiOutlined } from '@ant-design/icons';

const { Title, Text } = Typography;

// 占位符组件
const PlaceholderComponent: React.FC<{ title: string; description: string }> = ({ title, description }) => (
  <div className="p-6">
    <Card>
      <div className="text-center py-12">
        <ApiOutlined className="text-6xl text-gray-300 mb-4" />
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

export const CeLuePage: React.FC = () => {
  return (
    <Routes>
      <Route 
        path="/ml" 
        element={
          <PlaceholderComponent 
            title="AI/ML模型管理" 
            description="机器学习模型的训练、部署、验证和在线学习控制"
          />
        } 
      />
      <Route 
        path="/production" 
        element={
          <PlaceholderComponent 
            title="生产级API执行器" 
            description="套利交易执行、订单管理和API健康监控"
          />
        } 
      />
      <Route 
        path="/shadow" 
        element={
          <PlaceholderComponent 
            title="影子交易系统" 
            description="虚拟交易测试、回测分析和风险场景测试"
          />
        } 
      />
      <Route 
        path="/approval" 
        element={
          <PlaceholderComponent 
            title="审批工作流系统" 
            description="工作流管理、审批流程控制和权限管理"
          />
        } 
      />
      <Route 
        path="/strategies" 
        element={
          <PlaceholderComponent 
            title="策略编排器" 
            description="交易策略管理、参数调优和性能监控"
          />
        } 
      />
      <Route 
        path="/risk" 
        element={
          <PlaceholderComponent 
            title="风险管理控制" 
            description="风控规则管理、实时监控和紧急控制机制"
          />
        } 
      />
      <Route 
        path="/orders" 
        element={
          <PlaceholderComponent 
            title="订单执行管理" 
            description="订单创建、执行监控和智能路由配置"
          />
        } 
      />
      <Route 
        path="*" 
        element={
          <PlaceholderComponent 
            title="CeLue策略执行模块" 
            description="请从左侧菜单选择具体的功能模块"
          />
        } 
      />
    </Routes>
  );
};