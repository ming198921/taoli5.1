import React from 'react';

export const SimpleDashboard: React.FC = () => {
  console.log('SimpleDashboard rendering...');
  
  return (
    <div style={{ 
      padding: '24px', 
      background: '#ffffff', 
      minHeight: '400px',
      border: '2px solid #1890ff',
      borderRadius: '8px',
      margin: '20px'
    }}>
      <h1 style={{ color: '#1890ff', fontSize: '24px' }}>5.1套利系统仪表板</h1>
      <div style={{ 
        background: '#f0f2f5', 
        padding: '16px', 
        borderRadius: '4px',
        margin: '16px 0'
      }}>
        <h3>系统状态: 正常运行</h3>
        <p>当前时间: {new Date().toLocaleString()}</p>
        <p>版本: 5.1.0</p>
        <p>状态: 已连接后端服务</p>
      </div>
      <div style={{ color: '#52c41a', fontWeight: 'bold' }}>
        ✅ 如果你能看到这个页面，说明前端应用已经成功渲染！
      </div>
    </div>
  );
};