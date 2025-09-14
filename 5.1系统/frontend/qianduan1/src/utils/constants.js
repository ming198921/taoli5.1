/**
 * 5.1套利系统 - 常量定义
 */

// ===== API相关常量 =====
export const API_ENDPOINTS = {
  // 系统控制
  SYSTEM_STATUS: '/api/system/status',
  SYSTEM_START: '/api/system/start',
  SYSTEM_STOP: '/api/system/stop',
  SYSTEM_LOGS: '/api/system/logs',
  HEALTH: '/api/health',
  
  // 清洗模块
  QINGXI_COLLECTORS: '/api/qingxi/collectors',
  QINGXI_DATA_QUALITY: '/api/qingxi/data/quality',
  
  // 策略模块
  CELUE_STRATEGIES: '/api/celue/strategies/list',
  
  // 风险管理
  RISK_STATUS: '/api/risk/status',
  RISK_METRICS: '/api/risk/metrics',
  
  // 架构监控
  ARCHITECTURE_STATUS: '/api/architecture/status',
  
  // 可观测性
  OBSERVABILITY_LOGS: '/api/observability/logs',
  OBSERVABILITY_ALERTS: '/api/observability/alerts',
  
  // 配置管理
  CONFIG_UPDATE: '/api/config/update',
  
  // Systemd控制
  SYSTEMD_STATUS: '/api/control/systemd/status',
};

// ===== 系统状态常量 =====
export const SYSTEM_STATUS = {
  RUNNING: 'running',
  STOPPED: 'stopped',
  STARTING: 'starting',
  STOPPING: 'stopping',
  ERROR: 'error',
  UNKNOWN: 'unknown',
};

// ===== 模块状态常量 =====
export const MODULE_STATUS = {
  HEALTHY: 'healthy',
  UNHEALTHY: 'unhealthy',
  WARNING: 'warning',
  UNKNOWN: 'unknown',
};

// ===== 服务健康状态 =====
export const SERVICE_HEALTH = {
  UP: 'up',
  DOWN: 'down',
  DEGRADED: 'degraded',
  UNKNOWN: 'unknown',
};

// ===== 风险等级常量 =====
export const RISK_LEVELS = {
  LOW: 'low',
  MEDIUM: 'medium',
  HIGH: 'high',
  CRITICAL: 'critical',
};

// ===== 日志等级常量 =====
export const LOG_LEVELS = {
  DEBUG: 'debug',
  INFO: 'info',
  WARN: 'warn',
  ERROR: 'error',
  FATAL: 'fatal',
};

// ===== 数据收集器状态 =====
export const COLLECTOR_STATUS = {
  RUNNING: 'running',
  STOPPED: 'stopped',
  ERROR: 'error',
  INITIALIZING: 'initializing',
};

// ===== 策略状态 =====
export const STRATEGY_STATUS = {
  ACTIVE: 'active',
  INACTIVE: 'inactive',
  PAUSED: 'paused',
  ERROR: 'error',
};

// ===== 通知类型 =====
export const NOTIFICATION_TYPES = {
  SUCCESS: 'success',
  INFO: 'info',
  WARNING: 'warning',
  ERROR: 'error',
};

// ===== UI相关常量 =====
export const COLORS = {
  // 状态颜色
  SUCCESS: '#52c41a',
  WARNING: '#faad14',
  ERROR: '#ff4d4f',
  INFO: '#1890ff',
  
  // 系统主题色
  PRIMARY: '#1890ff',
  SECONDARY: '#722ed1',
  ACCENT: '#13c2c2',
  
  // 图表颜色
  CHART_COLORS: [
    '#1890ff', '#52c41a', '#faad14', '#ff4d4f',
    '#722ed1', '#13c2c2', '#eb2f96', '#f5222d'
  ],
};

// ===== 刷新间隔 =====
export const REFRESH_INTERVALS = {
  FAST: 5000,     // 5秒 - 用于关键指标
  NORMAL: 10000,  // 10秒 - 用于一般指标
  SLOW: 30000,    // 30秒 - 用于非关键指标
  VERY_SLOW: 60000, // 60秒 - 用于统计数据
};

// ===== 分页相关 =====
export const PAGINATION = {
  DEFAULT_PAGE_SIZE: 10,
  PAGE_SIZE_OPTIONS: [10, 20, 50, 100],
  MAX_PAGE_SIZE: 1000,
};

// ===== 时间格式 =====
export const TIME_FORMATS = {
  DATETIME: 'YYYY-MM-DD HH:mm:ss',
  DATE: 'YYYY-MM-DD',
  TIME: 'HH:mm:ss',
  TIMESTAMP: 'YYYY-MM-DD HH:mm:ss.SSS',
};

// ===== 路由路径 =====
export const ROUTES = {
  HOME: '/',
  SYSTEM: '/system',
  QINGXI: '/qingxi',
  CELUE: '/celue',
  RISK: '/risk',
  ARCHITECTURE: '/architecture',
  OBSERVABILITY: '/observability',
  SETTINGS: '/settings',
  PROFILE: '/profile',
  LOGIN: '/login',
};

// ===== 模块配置 =====
export const MODULES = {
  SYSTEM: {
    id: 'system',
    name: '系统控制',
    description: '系统启停控制、状态监控、性能指标',
    icon: '🎛️',
    path: ROUTES.SYSTEM,
  },
  QINGXI: {
    id: 'qingxi',
    name: '清洗模块',
    description: '数据收集器管理、数据质量监控',
    icon: '🔄',
    path: ROUTES.QINGXI,
  },
  CELUE: {
    id: 'celue',
    name: '策略模块',
    description: '套利策略管理、执行监控、收益分析',
    icon: '📈',
    path: ROUTES.CELUE,
  },
  RISK: {
    id: 'risk',
    name: '风险管理',
    description: '风险指标监控、预警管理、资金安全',
    icon: '🛡️',
    path: ROUTES.RISK,
  },
  ARCHITECTURE: {
    id: 'architecture',
    name: '架构监控',
    description: '系统架构监控、服务健康检查、性能分析',
    icon: '🏗️',
    path: ROUTES.ARCHITECTURE,
  },
  OBSERVABILITY: {
    id: 'observability',
    name: '可观测性',
    description: '日志聚合、链路追踪、告警管理',
    icon: '👁️',
    path: ROUTES.OBSERVABILITY,
  },
};

// ===== 默认配置 =====
export const DEFAULT_CONFIG = {
  // API配置
  API_TIMEOUT: 30000,
  API_RETRY_COUNT: 3,
  API_RETRY_DELAY: 1000,
  
  // 自动刷新配置
  AUTO_REFRESH: true,
  DEFAULT_REFRESH_INTERVAL: REFRESH_INTERVALS.NORMAL,
  
  // 表格配置
  TABLE_PAGE_SIZE: 10,
  TABLE_SHOW_SIZE_CHANGER: true,
  TABLE_SHOW_QUICK_JUMPER: true,
  
  // 图表配置
  CHART_ANIMATION: true,
  CHART_THEME: 'light',
  
  // 通知配置
  NOTIFICATION_DURATION: 4.5,
  NOTIFICATION_PLACEMENT: 'topRight',
};

// ===== 错误消息 =====
export const ERROR_MESSAGES = {
  NETWORK_ERROR: '网络连接失败，请检查网络连接',
  SERVER_ERROR: '服务器内部错误，请稍后重试',
  UNAUTHORIZED: '认证失败，请重新登录',
  FORBIDDEN: '权限不足，无法执行此操作',
  NOT_FOUND: '请求的资源不存在',
  TIMEOUT: '请求超时，请稍后重试',
  UNKNOWN_ERROR: '未知错误，请联系系统管理员',
  
  // 业务相关错误
  SYSTEM_START_FAILED: '系统启动失败',
  SYSTEM_STOP_FAILED: '系统停止失败',
  DATA_LOAD_FAILED: '数据加载失败',
  CONFIG_UPDATE_FAILED: '配置更新失败',
};

// ===== 成功消息 =====
export const SUCCESS_MESSAGES = {
  SYSTEM_STARTED: '系统启动成功',
  SYSTEM_STOPPED: '系统停止成功',
  CONFIG_UPDATED: '配置更新成功',
  DATA_LOADED: '数据加载成功',
  OPERATION_SUCCESS: '操作执行成功',
};

export default {
  API_ENDPOINTS,
  SYSTEM_STATUS,
  MODULE_STATUS,
  SERVICE_HEALTH,
  RISK_LEVELS,
  LOG_LEVELS,
  COLLECTOR_STATUS,
  STRATEGY_STATUS,
  NOTIFICATION_TYPES,
  COLORS,
  REFRESH_INTERVALS,
  PAGINATION,
  TIME_FORMATS,
  ROUTES,
  MODULES,
  DEFAULT_CONFIG,
  ERROR_MESSAGES,
  SUCCESS_MESSAGES,
};