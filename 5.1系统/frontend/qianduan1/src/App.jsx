import React, { useEffect } from 'react';
import {
  Routes,
  Route,
  useLocation,
  Navigate
} from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

import './css/style.css';
import './charts/ChartjsConfig';

// Import pages - 现有页面
import Dashboard from './pages/Dashboard';
import SystemControl from './pages/system/SystemControl.jsx';
import QingxiModule from './pages/qingxi/QingxiModule.jsx';
import CelueModule from './pages/celue/CelueModule.jsx';
import RiskModule from './pages/risk/RiskModule.jsx';
import ArchitectureModule from './pages/architecture/ArchitectureModule.jsx';
import ObservabilityModule from './pages/observability/ObservabilityModule.jsx';

// Import new pages - 387个API接口对应的新页面
import LoggingModule from './pages/logging/LoggingModule.jsx';
import CleaningModule from './pages/cleaning/CleaningModule.jsx';
import StrategyMonitoringModule from './pages/strategy-monitoring/StrategyMonitoringModule.jsx';
import PerformanceModule from './pages/performance/PerformanceModule.jsx';
import TradingModule from './pages/trading/TradingModule.jsx';
import AIModelModule from './pages/ai-model/AIModelModule.jsx';
import ConfigModule from './pages/config/ConfigModule.jsx';

// Create QueryClient instance
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      refetchOnMount: true,
      refetchOnReconnect: true,
      retry: 3,
      staleTime: 5 * 60 * 1000, // 5 minutes
    },
  },
});

function App() {
  const location = useLocation();

  useEffect(() => {
    document.querySelector('html').style.scrollBehavior = 'auto'
    window.scroll({ top: 0 })
    document.querySelector('html').style.scrollBehavior = ''
  }, [location.pathname]); // triggered on route change

  return (
    <QueryClientProvider client={queryClient}>
      <Routes>
        {/* 重定向根路径到仪表板 */}
        <Route path="/" element={<Navigate to="/dashboard" replace />} />
        
        {/* 主仪表板 - 387个API状态总览 */}
        <Route path="/dashboard" element={<Dashboard />} />
        
        {/* 现有模块路由 */}
        <Route path="/system/*" element={<SystemControl />} />
        <Route path="/qingxi/*" element={<QingxiModule />} />
        <Route path="/celue/*" element={<CelueModule />} />
        <Route path="/risk/*" element={<RiskModule />} />
        <Route path="/architecture/*" element={<ArchitectureModule />} />
        <Route path="/observability/*" element={<ObservabilityModule />} />
        
        {/* 新增模块路由 - 387个API接口对应 */}
        
        {/* 日志监控模块 (45个API) */}
        <Route path="/logging/*" element={<LoggingModule />} />
        
        {/* 清洗配置模块 (52个API) */}
        <Route path="/cleaning/*" element={<CleaningModule />} />
        
        {/* 策略监控模块 (38个API) */}
        <Route path="/strategy-monitoring/*" element={<StrategyMonitoringModule />} />
        
        {/* 性能调优模块 (67个API) */}
        <Route path="/performance/*" element={<PerformanceModule />} />
        
        {/* 交易监控模块 (41个API) */}
        <Route path="/trading/*" element={<TradingModule />} />
        
        {/* AI模型管理模块 (48个API) */}
        <Route path="/ai-model/*" element={<AIModelModule />} />
        
        {/* 配置管理模块 (96个API) */}
        <Route path="/config/*" element={<ConfigModule />} />
        
        {/* 404页面 */}
        <Route path="*" element={<Navigate to="/dashboard" replace />} />
      </Routes>
    </QueryClientProvider>
  );
}

export default App;
