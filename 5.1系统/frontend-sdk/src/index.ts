/**
 * 5.1套利系统前端SDK - 统一导出
 * 提供完整的API访问能力，实现前端100%控制后端系统
 */

// 主SDK类
export { ArbitrageSystemSDK } from './sdk';

// 核心客户端
export { HttpClient } from './core/http-client';
export { WebSocketClient } from './core/websocket-client';

// 服务类
export { AuthService } from './services/auth.service';
export { QingxiService } from './services/qingxi.service';
export { DashboardService } from './services/dashboard.service';
export { MonitoringService } from './services/monitoring.service';
export { SystemService } from './services/system.service';

// 类型定义
export * from './types';

// 默认导出SDK类
export default ArbitrageSystemSDK;