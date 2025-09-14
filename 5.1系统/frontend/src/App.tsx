import { BrowserRouter as Router, Routes, Route, Navigate, useNavigate, useLocation } from 'react-router-dom';
import { Layout, Menu, Avatar, Dropdown, Badge, Button } from 'antd';
import {
  DashboardOutlined,
  DatabaseOutlined,
  MonitorOutlined,
  SettingOutlined,
  LogoutOutlined,
  UserOutlined,
  BellOutlined,
  MenuFoldOutlined,
  MenuUnfoldOutlined,
  FileTextOutlined,
  AreaChartOutlined,
  SwapOutlined,
  RobotOutlined,
  ControlOutlined
} from '@ant-design/icons';
import { useState, useEffect } from 'react';

// 页面组件
import Dashboard from './pages/Dashboard';
import SystemControl from './pages/SystemControl';
import LoggingModule from './pages/LoggingModule';
import CleaningModule from './pages/CleaningModule';
import StrategyModule from './pages/StrategyModule';
import PerformanceModule from './pages/PerformanceModule';
import TradingModule from './pages/TradingModule';
import AIModelModule from './pages/AIModelModule';
import ConfigModule from './pages/ConfigModule';

const { Header, Content, Sider } = Layout;

function AppContent() {
  console.log('🚀 App组件开始渲染');
  const [collapsed, setCollapsed] = useState(false);
  const [notifications, setNotifications] = useState(3);
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    // 模拟通知更新
    const interval = setInterval(() => {
      setNotifications(prev => Math.max(0, prev + Math.floor(Math.random() * 3) - 1));
    }, 30000);
    
    return () => clearInterval(interval);
  }, []);

  const userMenuItems = [
    {
      key: 'profile',
      icon: <UserOutlined />,
      label: '个人资料',
    },
    {
      key: 'settings',
      icon: <SettingOutlined />,
      label: '设置',
    },
    {
      type: 'divider' as const,
    },
    {
      key: 'logout',
      icon: <LogoutOutlined />,
      label: '退出登录',
    },
  ];

  const menuItems = [
    {
      key: 'dashboard',
      icon: <DashboardOutlined />,
      label: '仪表板',
      path: '/dashboard'
    },
    {
      key: 'system',
      icon: <ControlOutlined />,
      label: '系统控制',
      path: '/system'
    },
    {
      key: 'logging',
      icon: <FileTextOutlined />,
      label: '日志服务',
      path: '/logging'
    },
    {
      key: 'cleaning',
      icon: <DatabaseOutlined />,
      label: '清洗服务',
      path: '/cleaning'
    },
    {
      key: 'strategy',
      icon: <MonitorOutlined />,
      label: '策略服务',
      path: '/strategy'
    },
    {
      key: 'performance',
      icon: <AreaChartOutlined />,
      label: '性能服务',
      path: '/performance'
    },
    {
      key: 'trading',
      icon: <SwapOutlined />,
      label: '交易服务',
      path: '/trading'
    },
    {
      key: 'ai-model',
      icon: <RobotOutlined />,
      label: 'AI模型服务',
      path: '/ai-model'
    },
    {
      key: 'config',
      icon: <SettingOutlined />,
      label: '配置服务',
      path: '/config'
    }
  ];

  // 根据当前路径获取选中的菜单项
  const getSelectedKey = () => {
    const path = location.pathname;
    for (const item of menuItems) {
      if (path.startsWith(item.path)) {
        return item.key;
      }
    }
    return 'dashboard';
  };

  const handleMenuClick = (path: string) => {
    navigate(path);
  };

  return (
      <Layout style={{ minHeight: '100vh' }}>
        <Sider 
          collapsible 
          collapsed={collapsed} 
          onCollapse={setCollapsed}
          breakpoint="lg"
          collapsedWidth="0"
          style={{
            overflow: 'auto',
            height: '100vh',
            position: 'fixed',
            left: 0,
            top: 0,
            bottom: 0,
          }}
        >
          <div style={{ 
            height: 32, 
            margin: 16, 
            background: 'rgba(255, 255, 255, 0.2)',
            borderRadius: 6,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: 'white',
            fontWeight: 'bold'
          }}>
            {collapsed ? '5.1' : '5.1套利系统'}
          </div>
          
          <Menu 
            theme="dark" 
            selectedKeys={[getSelectedKey()]} 
            mode="inline"
            onClick={({ key }) => {
              const item = menuItems.find(item => item.key === key);
              if (item) {
                handleMenuClick(item.path);
              }
            }}
            items={menuItems.map(item => ({
              key: item.key,
              icon: item.icon,
              label: item.label
            }))}
          />
        </Sider>
        
        <Layout style={{ marginLeft: collapsed ? 0 : 200 }}>
          <Header style={{ 
            padding: '0 24px', 
            background: '#fff',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            boxShadow: '0 1px 4px rgba(0,21,41,.08)'
          }}>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <Button
                type="text"
                icon={collapsed ? <MenuUnfoldOutlined /> : <MenuFoldOutlined />}
                onClick={() => setCollapsed(!collapsed)}
                style={{
                  fontSize: '16px',
                  width: 64,
                  height: 64,
                }}
              />
              <h2 style={{ margin: 0, marginLeft: 16 }}>5.1高频套利交易系统</h2>
            </div>
            
            <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
              <Badge count={notifications} size="small">
                <BellOutlined style={{ fontSize: 18 }} />
              </Badge>
              
              <Dropdown
                menu={{ items: userMenuItems }}
                placement="bottomRight"
                trigger={['click']}
              >
                <div style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                  <Avatar size="small" icon={<UserOutlined />} />
                  <span style={{ marginLeft: 8 }}>管理员</span>
                </div>
              </Dropdown>
            </div>
          </Header>
          
          <Content style={{ 
            margin: '24px 16px 0', 
            overflow: 'initial',
            minHeight: 'calc(100vh - 112px)'
          }}>
            <div style={{
              padding: 24,
              minHeight: 360,
              background: '#fff',
              borderRadius: 8,
              boxShadow: '0 1px 3px rgba(0,0,0,0.12), 0 1px 2px rgba(0,0,0,0.24)'
            }}>
              <Routes>
                <Route path="/" element={<Navigate to="/dashboard" replace />} />
                <Route path="/dashboard" element={<Dashboard />} />
                <Route path="/system/*" element={<SystemControl />} />
                <Route path="/logging/*" element={<LoggingModule />} />
                <Route path="/cleaning/*" element={<CleaningModule />} />
                <Route path="/strategy/*" element={<StrategyModule />} />
                <Route path="/performance/*" element={<PerformanceModule />} />
                <Route path="/trading/*" element={<TradingModule />} />
                <Route path="/ai-model/*" element={<AIModelModule />} />
                <Route path="/config/*" element={<ConfigModule />} />
                <Route path="*" element={<Navigate to="/dashboard" replace />} />
              </Routes>
            </div>
          </Content>
        </Layout>
      </Layout>
  );
}

function App() {
  return (
    <Router
      future={{
        v7_startTransition: true,
        v7_relativeSplatPath: true
      }}
    >
      <AppContent />
    </Router>
  );
}

export default App;

  // 根据当前路径获取选中的菜单项
  const getSelectedKey = () => {
    const path = location.pathname;
    for (const item of menuItems) {
      if (path.startsWith(item.path)) {
        return item.key;
      }
    }
    return 'dashboard';
  };

  const handleMenuClick = (path: string) => {
    navigate(path);
  };

  return (
      <Layout style={{ minHeight: '100vh' }}>
        <Sider 
          collapsible 
          collapsed={collapsed} 
          onCollapse={setCollapsed}
          breakpoint="lg"
          collapsedWidth="0"
          style={{
            overflow: 'auto',
            height: '100vh',
            position: 'fixed',
            left: 0,
            top: 0,
            bottom: 0,
          }}
        >
          <div style={{ 
            height: 32, 
            margin: 16, 
            background: 'rgba(255, 255, 255, 0.2)',
            borderRadius: 6,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: 'white',
            fontWeight: 'bold'
          }}>
            {collapsed ? '5.1' : '5.1套利系统'}
          </div>
          
          <Menu 
            theme="dark" 
            selectedKeys={[getSelectedKey()]} 
            mode="inline"
            onClick={({ key }) => {
              const item = menuItems.find(item => item.key === key);
              if (item) {
                handleMenuClick(item.path);
              }
            }}
            items={menuItems.map(item => ({
              key: item.key,
              icon: item.icon,
              label: item.label
            }))}
          />
        </Sider>
        
        <Layout style={{ marginLeft: collapsed ? 0 : 200 }}>
          <Header style={{ 
            padding: '0 24px', 
            background: '#fff',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            boxShadow: '0 1px 4px rgba(0,21,41,.08)'
          }}>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <Button
                type="text"
                icon={collapsed ? <MenuUnfoldOutlined /> : <MenuFoldOutlined />}
                onClick={() => setCollapsed(!collapsed)}
                style={{
                  fontSize: '16px',
                  width: 64,
                  height: 64,
                }}
              />
              <h2 style={{ margin: 0, marginLeft: 16 }}>5.1高频套利交易系统</h2>
            </div>
            
            <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
              <Badge count={notifications} size="small">
                <BellOutlined style={{ fontSize: 18 }} />
              </Badge>
              
              <Dropdown
                menu={{ items: userMenuItems }}
                placement="bottomRight"
                trigger={['click']}
              >
                <div style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                  <Avatar size="small" icon={<UserOutlined />} />
                  <span style={{ marginLeft: 8 }}>管理员</span>
                </div>
              </Dropdown>
            </div>
          </Header>
          
          <Content style={{ 
            margin: '24px 16px 0', 
            overflow: 'initial',
            minHeight: 'calc(100vh - 112px)'
          }}>
            <div style={{
              padding: 24,
              minHeight: 360,
              background: '#fff',
              borderRadius: 8,
              boxShadow: '0 1px 3px rgba(0,0,0,0.12), 0 1px 2px rgba(0,0,0,0.24)'
            }}>
              <Routes>
                <Route path="/" element={<Navigate to="/dashboard" replace />} />
                <Route path="/dashboard" element={<Dashboard />} />
                <Route path="/system/*" element={<SystemControl />} />
                <Route path="/logging/*" element={<LoggingModule />} />
                <Route path="/cleaning/*" element={<CleaningModule />} />
                <Route path="/strategy/*" element={<StrategyModule />} />
                <Route path="/performance/*" element={<PerformanceModule />} />
                <Route path="/trading/*" element={<TradingModule />} />
                <Route path="/ai-model/*" element={<AIModelModule />} />
                <Route path="/config/*" element={<ConfigModule />} />
                <Route path="*" element={<Navigate to="/dashboard" replace />} />
              </Routes>
            </div>
          </Content>
        </Layout>
      </Layout>
  );
}

function App() {
  return (
    <Router
      future={{
        v7_startTransition: true,
        v7_relativeSplatPath: true
      }}
    >
      <AppContent />
    </Router>
  );
}

export default App;