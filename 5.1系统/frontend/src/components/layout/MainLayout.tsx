import React, { useState, useEffect } from 'react';
import { Layout, Menu, Avatar, Dropdown, Badge, Button, Tooltip, Switch, Space } from 'antd';
import {
  DashboardOutlined,
  DatabaseOutlined,
  SettingOutlined,
  MonitorOutlined,
  SafetyOutlined,
  BellOutlined,
  UserOutlined,
  LogoutOutlined,
  MoonOutlined,
  SunOutlined,
  MenuFoldOutlined,
  MenuUnfoldOutlined,
  WifiOutlined,
  DisconnectOutlined,
  ApiOutlined,
  ThunderboltOutlined,
} from '@ant-design/icons';
import { useLocation, useNavigate } from 'react-router-dom';
import { useAppDispatch, useAppSelector } from '@/store/hooks';
import { 
  toggleSidebar, 
  toggleTheme, 
  selectUnreadNotificationCount,
  selectConnectionStatus,
  selectSystemHealth 
} from '@/store/slices/appSlice';
import { logoutUser } from '@/store/slices/authSlice';

const { Header, Sider, Content } = Layout;

interface MainLayoutProps {
  children: React.ReactNode;
}

export const MainLayout: React.FC<MainLayoutProps> = ({ children }) => {
  const dispatch = useAppDispatch();
  const navigate = useNavigate();
  const location = useLocation();
  
  const { 
    sidebarCollapsed, 
    theme,
    isAuthenticated 
  } = useAppSelector(state => state.app);
  
  const { user } = useAppSelector(state => state.auth);
  const unreadCount = useAppSelector(selectUnreadNotificationCount);
  const connectionStatus = useAppSelector(selectConnectionStatus);
  const systemHealth = useAppSelector(selectSystemHealth);

  // 菜单项定义
  const menuItems = [
    {
      key: '/dashboard',
      icon: <DashboardOutlined />,
      label: '系统概览',
    },
    {
      key: '/qingxi',
      icon: <DatabaseOutlined />,
      label: '数据处理',
      children: [
        { key: '/qingxi/collector', label: '数据收集器' },
        { key: '/qingxi/processor', label: '批处理器' },
        { key: '/qingxi/ccxt', label: 'CCXT适配器' },
        { key: '/qingxi/time', label: '时间管理' },
        { key: '/qingxi/memory', label: '内存管理' },
        { key: '/qingxi/sources', label: '第三方数据源' },
        { key: '/qingxi/nats', label: 'NATS队列' },
      ],
    },
    {
      key: '/celue',
      icon: <ApiOutlined />,
      label: '策略执行',
      children: [
        { key: '/celue/ml', label: 'AI/ML模型' },
        { key: '/celue/production', label: '生产API' },
        { key: '/celue/shadow', label: '影子交易' },
        { key: '/celue/approval', label: '审批工作流' },
        { key: '/celue/strategies', label: '策略管理' },
        { key: '/celue/risk', label: '风险控制' },
        { key: '/celue/orders', label: '订单管理' },
      ],
    },
    {
      key: '/risk',
      icon: <ThunderboltOutlined />,
      label: 'AI风控管理',
    },
    {
      key: '/architecture',
      icon: <SettingOutlined />,
      label: '系统架构',
      children: [
        { key: '/architecture/limits', label: '限制器控制' },
        { key: '/architecture/enforcement', label: '运行时强制' },
        { key: '/architecture/hotreload', label: '配置热重载' },
        { key: '/architecture/health', label: '健康检查' },
        { key: '/architecture/resources', label: '资源监控' },
        { key: '/architecture/recovery', label: '故障恢复' },
      ],
    },
    {
      key: '/observability',
      icon: <MonitorOutlined />,
      label: '可观测性',
      children: [
        { key: '/observability/tracing', label: '分布式追踪' },
        { key: '/observability/metrics', label: '指标收集' },
        { key: '/observability/alerts', label: '告警规则' },
        { key: '/observability/logs', label: '日志聚合' },
        { key: '/observability/dashboards', label: '可视化管理' },
      ],
    },
    {
      key: '/settings',
      icon: <SafetyOutlined />,
      label: '系统设置',
    },
  ];

  // 用户下拉菜单
  const userMenuItems = [
    {
      key: 'profile',
      icon: <UserOutlined />,
      label: '个人资料',
      onClick: () => navigate('/settings/profile'),
    },
    {
      key: 'preferences',
      icon: <SettingOutlined />,
      label: '偏好设置',
      onClick: () => navigate('/settings/preferences'),
    },
    {
      type: 'divider' as const,
    },
    {
      key: 'logout',
      icon: <LogoutOutlined />,
      label: '退出登录',
      onClick: async () => {
        await dispatch(logoutUser());
        navigate('/login');
      },
    },
  ];

  // 获取当前选中的菜单项
  const getSelectedKeys = (): string[] => {
    const path = location.pathname;
    
    // 找到最匹配的菜单项
    for (const item of menuItems) {
      if (item.children) {
        for (const child of item.children) {
          if (path.startsWith(child.key)) {
            return [child.key];
          }
        }
      }
      if (path.startsWith(item.key)) {
        return [item.key];
      }
    }
    
    return ['/dashboard'];
  };

  // 获取展开的菜单项
  const getOpenKeys = (): string[] => {
    const path = location.pathname;
    const openKeys: string[] = [];
    
    for (const item of menuItems) {
      if (item.children) {
        for (const child of item.children) {
          if (path.startsWith(child.key)) {
            openKeys.push(item.key);
            break;
          }
        }
      }
    }
    
    return openKeys;
  };

  // 连接状态指示器
  const renderConnectionStatus = () => {
    const apiConnected = connectionStatus.api === 'connected';
    const wsConnected = connectionStatus.websocket === 'connected';
    
    return (
      <Space size="small">
        <Tooltip title={`API连接: ${apiConnected ? '已连接' : '未连接'}`}>
          <div className="flex items-center">
            {apiConnected ? (
              <WifiOutlined className="text-green-500" />
            ) : (
              <DisconnectOutlined className="text-red-500" />
            )}
          </div>
        </Tooltip>
        
        <Tooltip title={`WebSocket: ${wsConnected ? '已连接' : '未连接'}`}>
          <div className="flex items-center">
            <div className={`w-2 h-2 rounded-full ${wsConnected ? 'bg-green-500' : 'bg-red-500'} animate-pulse`} />
          </div>
        </Tooltip>
      </Space>
    );
  };

  // 系统健康状态指示器
  const renderHealthStatus = () => {
    const status = systemHealth.overall;
    let color = 'text-gray-400';
    let text = '未知';
    
    switch (status) {
      case 'healthy':
        color = 'text-green-500';
        text = '健康';
        break;
      case 'warning':
        color = 'text-yellow-500';
        text = '警告';
        break;
      case 'error':
        color = 'text-red-500';
        text = '错误';
        break;
    }
    
    return (
      <Tooltip title={`系统状态: ${text}`}>
        <div className={`flex items-center ${color}`}>
          <div className={`w-2 h-2 rounded-full ${color.replace('text-', 'bg-')} mr-1`} />
          <span className="text-xs">{text}</span>
        </div>
      </Tooltip>
    );
  };

  return (
    <Layout className="min-h-screen">
      {/* 侧边栏 */}
      <Sider
        trigger={null}
        collapsible
        collapsed={sidebarCollapsed}
        width={256}
        theme="dark"
        className="fixed left-0 top-0 h-full z-10"
      >
        {/* Logo */}
        <div className="h-16 flex items-center justify-center border-b border-gray-700">
          {sidebarCollapsed ? (
            <div className="text-white text-lg font-bold">5.1</div>
          ) : (
            <div className="text-white">
              <div className="text-lg font-bold">5.1套利系统</div>
              <div className="text-xs text-gray-400">v{systemHealth.version || '1.0.0'}</div>
            </div>
          )}
        </div>
        
        {/* 菜单 */}
        <Menu
          theme="dark"
          mode="inline"
          selectedKeys={getSelectedKeys()}
          defaultOpenKeys={getOpenKeys()}
          items={menuItems}
          onClick={({ key }) => navigate(key)}
          className="border-r-0"
        />
      </Sider>

      {/* 主内容区域 */}
      <Layout style={{ marginLeft: sidebarCollapsed ? 80 : 256, transition: 'margin-left 0.2s' }}>
        {/* 头部 */}
        <Header className="bg-white shadow-sm px-4 flex items-center justify-between h-16 fixed w-full z-10">
          <div className="flex items-center space-x-4">
            {/* 折叠按钮 */}
            <Button
              type="text"
              icon={sidebarCollapsed ? <MenuUnfoldOutlined /> : <MenuFoldOutlined />}
              onClick={() => dispatch(toggleSidebar())}
              className="text-gray-600 hover:text-gray-800"
            />
            
            {/* 连接状态 */}
            {renderConnectionStatus()}
            
            {/* 系统健康状态 */}
            {renderHealthStatus()}
          </div>

          <div className="flex items-center space-x-4">
            {/* 主题切换 */}
            <Tooltip title={`切换到${theme === 'light' ? '暗色' : '亮色'}主题`}>
              <Switch
                checkedChildren={<MoonOutlined />}
                unCheckedChildren={<SunOutlined />}
                checked={theme === 'dark'}
                onChange={() => dispatch(toggleTheme())}
              />
            </Tooltip>

            {/* 通知 */}
            <Tooltip title="通知">
              <Badge count={unreadCount} size="small">
                <Button
                  type="text"
                  icon={<BellOutlined />}
                  onClick={() => navigate('/settings/notifications')}
                  className="text-gray-600 hover:text-gray-800"
                />
              </Badge>
            </Tooltip>

            {/* 用户菜单 */}
            <Dropdown
              menu={{ items: userMenuItems }}
              placement="bottomRight"
              trigger={['click']}
            >
              <div className="flex items-center space-x-2 cursor-pointer hover:bg-gray-50 px-2 py-1 rounded">
                <Avatar size="small" icon={<UserOutlined />} src={user?.avatar} />
                <span className="text-sm text-gray-700 hidden sm:inline">
                  {user?.fullName || user?.username || '用户'}
                </span>
              </div>
            </Dropdown>
          </div>
        </Header>

        {/* 内容区域 */}
        <Content className="mt-16">
          {children}
        </Content>
      </Layout>
    </Layout>
  );
};