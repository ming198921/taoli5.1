import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import type {
  DistributedTracingConfig,
  TraceData,
  MetricDefinition,
  AlertRule,
  Alert,
  LogEntry,
  VisualizationConfig
} from '@/types/observability';

// Observability模块状态类型
interface ObservabilityState {
  // 分布式追踪状态
  tracing: {
    enabled: boolean;
    config: DistributedTracingConfig;
    serviceMap: {
      services: Array<{
        name: string;
        requestCount: number;
        errorRate: number;
        avgDuration: number;
        versions: string[];
      }>;
      dependencies: Array<{
        source: string;
        target: string;
        requestCount: number;
        errorRate: number;
        avgLatency: number;
      }>;
    };
    recentTraces: TraceData[];
    anomalies: Array<{
      timestamp: string;
      service: string;
      operation: string;
      type: string;
      severity: string;
      description: string;
    }>;
  };
  
  // 指标收集状态
  metrics: {
    definitions: MetricDefinition[];
    recentValues: Array<{
      metricName: string;
      value: number;
      timestamp: string;
      labels: Record<string, string>;
    }>;
    stats: {
      totalMetrics: number;
      uniqueSeries: number;
      dataPointsPerMinute: number;
      storageSizeBytes: number;
    };
  };
  
  // 告警状态
  alerting: {
    rules: AlertRule[];
    activeAlerts: Alert[];
    alertHistory: Array<{
      alertId: string;
      ruleName: string;
      severity: string;
      status: string;
      triggeredAt: string;
      resolvedAt?: string;
    }>;
    notificationChannels: Array<{
      name: string;
      type: string;
      enabled: boolean;
      successRate: number;
    }>;
    stats: {
      totalAlerts: number;
      bySeverity: Record<string, number>;
      byStatus: Record<string, number>;
      avgResolutionTime: number;
      falsePositiveRate: number;
    };
  };
  
  // 日志聚合状态
  logging: {
    recentLogs: LogEntry[];
    stats: {
      totalLogs: number;
      byLevel: Record<string, number>;
      byService: Record<string, number>;
      errorRate: number;
      logRatePerMinute: number;
    };
    patterns: Array<{
      pattern: string;
      count: number;
      percentage: number;
      firstSeen: string;
      lastSeen: string;
    }>;
    retentionPolicy: {
      defaultRetentionDays: number;
      totalStorageGB: number;
      cleanupSchedule: string;
    };
  };
  
  // 可视化管理状态
  visualization: {
    dashboards: Array<{
      dashboardId: string;
      name: string;
      description: string;
      panelCount: number;
      createdBy: string;
      lastModified: string;
      isPublic: boolean;
    }>;
    activeDashboard: VisualizationConfig | null;
    templates: Array<{
      templateId: string;
      name: string;
      description: string;
      usageCount: number;
      tags: string[];
    }>;
  };
  
  // 整体状态
  overallStatus: 'healthy' | 'warning' | 'critical' | 'maintenance';
  lastUpdate: string | null;
  activeAlerts: Array<{
    id: string;
    level: 'info' | 'warning' | 'error' | 'critical';
    component: string;
    message: string;
    timestamp: string;
    acknowledged: boolean;
  }>;
}

// 初始状态
const initialState: ObservabilityState = {
  tracing: {
    enabled: false,
    config: {} as DistributedTracingConfig,
    serviceMap: {
      services: [],
      dependencies: [],
    },
    recentTraces: [],
    anomalies: [],
  },
  metrics: {
    definitions: [],
    recentValues: [],
    stats: {
      totalMetrics: 0,
      uniqueSeries: 0,
      dataPointsPerMinute: 0,
      storageSizeBytes: 0,
    },
  },
  alerting: {
    rules: [],
    activeAlerts: [],
    alertHistory: [],
    notificationChannels: [],
    stats: {
      totalAlerts: 0,
      bySeverity: {},
      byStatus: {},
      avgResolutionTime: 0,
      falsePositiveRate: 0,
    },
  },
  logging: {
    recentLogs: [],
    stats: {
      totalLogs: 0,
      byLevel: {},
      byService: {},
      errorRate: 0,
      logRatePerMinute: 0,
    },
    patterns: [],
    retentionPolicy: {
      defaultRetentionDays: 30,
      totalStorageGB: 0,
      cleanupSchedule: 'daily',
    },
  },
  visualization: {
    dashboards: [],
    activeDashboard: null,
    templates: [],
  },
  overallStatus: 'critical',
  lastUpdate: null,
  activeAlerts: [],
};

// Observability slice
const observabilitySlice = createSlice({
  name: 'observability',
  initialState,
  reducers: {
    // 分布式追踪管理
    updateTracingConfig: (state, action: PayloadAction<{ enabled?: boolean; config?: Partial<DistributedTracingConfig> }>) => {
      if (action.payload.enabled !== undefined) {
        state.tracing.enabled = action.payload.enabled;
      }
      if (action.payload.config) {
        state.tracing.config = { ...state.tracing.config, ...action.payload.config };
      }
    },
    
    updateServiceMap: (state, action: PayloadAction<Partial<ObservabilityState['tracing']['serviceMap']>>) => {
      state.tracing.serviceMap = { ...state.tracing.serviceMap, ...action.payload };
    },
    
    addTrace: (state, action: PayloadAction<TraceData>) => {
      state.tracing.recentTraces.unshift(action.payload);
      // 保持最近100条记录
      if (state.tracing.recentTraces.length > 100) {
        state.tracing.recentTraces = state.tracing.recentTraces.slice(0, 100);
      }
    },
    
    addAnomaly: (state, action: PayloadAction<ObservabilityState['tracing']['anomalies'][number]>) => {
      state.tracing.anomalies.unshift(action.payload);
      // 保持最近50条记录
      if (state.tracing.anomalies.length > 50) {
        state.tracing.anomalies = state.tracing.anomalies.slice(0, 50);
      }
    },
    
    // 指标管理
    addMetricDefinition: (state, action: PayloadAction<MetricDefinition>) => {
      const existingIndex = state.metrics.definitions.findIndex(m => m.name === action.payload.name);
      if (existingIndex >= 0) {
        state.metrics.definitions[existingIndex] = action.payload;
      } else {
        state.metrics.definitions.push(action.payload);
      }
    },
    
    removeMetricDefinition: (state, action: PayloadAction<string>) => {
      state.metrics.definitions = state.metrics.definitions.filter(m => m.name !== action.payload);
    },
    
    addMetricValue: (state, action: PayloadAction<ObservabilityState['metrics']['recentValues'][number]>) => {
      state.metrics.recentValues.unshift(action.payload);
      // 保持最近1000个数据点
      if (state.metrics.recentValues.length > 1000) {
        state.metrics.recentValues = state.metrics.recentValues.slice(0, 1000);
      }
    },
    
    updateMetricStats: (state, action: PayloadAction<Partial<ObservabilityState['metrics']['stats']>>) => {
      state.metrics.stats = { ...state.metrics.stats, ...action.payload };
    },
    
    // 告警管理
    addAlertRule: (state, action: PayloadAction<AlertRule>) => {
      const existingIndex = state.alerting.rules.findIndex(r => r.id === action.payload.id);
      if (existingIndex >= 0) {
        state.alerting.rules[existingIndex] = action.payload;
      } else {
        state.alerting.rules.push(action.payload);
      }
    },
    
    removeAlertRule: (state, action: PayloadAction<string>) => {
      state.alerting.rules = state.alerting.rules.filter(r => r.id !== action.payload);
    },
    
    addActiveAlert: (state, action: PayloadAction<Alert>) => {
      const existingIndex = state.alerting.activeAlerts.findIndex(a => a.id === action.payload.id);
      if (existingIndex >= 0) {
        state.alerting.activeAlerts[existingIndex] = action.payload;
      } else {
        state.alerting.activeAlerts.push(action.payload);
      }
    },
    
    removeActiveAlert: (state, action: PayloadAction<string>) => {
      state.alerting.activeAlerts = state.alerting.activeAlerts.filter(a => a.id !== action.payload);
    },
    
    updateAlertStats: (state, action: PayloadAction<Partial<ObservabilityState['alerting']['stats']>>) => {
      state.alerting.stats = { ...state.alerting.stats, ...action.payload };
    },
    
    // 日志管理
    addLogEntry: (state, action: PayloadAction<LogEntry>) => {
      state.logging.recentLogs.unshift(action.payload);
      // 保持最近1000条记录
      if (state.logging.recentLogs.length > 1000) {
        state.logging.recentLogs = state.logging.recentLogs.slice(0, 1000);
      }
    },
    
    updateLogStats: (state, action: PayloadAction<Partial<ObservabilityState['logging']['stats']>>) => {
      state.logging.stats = { ...state.logging.stats, ...action.payload };
    },
    
    updateLogPatterns: (state, action: PayloadAction<ObservabilityState['logging']['patterns']>) => {
      state.logging.patterns = action.payload;
    },
    
    updateRetentionPolicy: (state, action: PayloadAction<Partial<ObservabilityState['logging']['retentionPolicy']>>) => {
      state.logging.retentionPolicy = { ...state.logging.retentionPolicy, ...action.payload };
    },
    
    // 可视化管理
    addDashboard: (state, action: PayloadAction<ObservabilityState['visualization']['dashboards'][number]>) => {
      const existingIndex = state.visualization.dashboards.findIndex(d => d.dashboardId === action.payload.dashboardId);
      if (existingIndex >= 0) {
        state.visualization.dashboards[existingIndex] = action.payload;
      } else {
        state.visualization.dashboards.push(action.payload);
      }
    },
    
    removeDashboard: (state, action: PayloadAction<string>) => {
      state.visualization.dashboards = state.visualization.dashboards.filter(d => d.dashboardId !== action.payload);
    },
    
    setActiveDashboard: (state, action: PayloadAction<VisualizationConfig | null>) => {
      state.visualization.activeDashboard = action.payload;
    },
    
    updateTemplates: (state, action: PayloadAction<ObservabilityState['visualization']['templates']>) => {
      state.visualization.templates = action.payload;
    },
    
    // 整体状态管理
    updateOverallStatus: (state, action: PayloadAction<ObservabilityState['overallStatus']>) => {
      state.overallStatus = action.payload;
      state.lastUpdate = new Date().toISOString();
    },
    
    // 告警管理
    addAlert: (state, action: PayloadAction<ObservabilityState['activeAlerts'][number]>) => {
      state.activeAlerts.push(action.payload);
    },
    
    removeAlert: (state, action: PayloadAction<string>) => {
      state.activeAlerts = state.activeAlerts.filter(alert => alert.id !== action.payload);
    },
    
    acknowledgeAlert: (state, action: PayloadAction<string>) => {
      const alert = state.activeAlerts.find(a => a.id === action.payload);
      if (alert) {
        alert.acknowledged = true;
      }
    },
    
    clearAllAlerts: (state) => {
      state.activeAlerts = [];
    },
    
    // 重置状态
    resetObservabilityState: () => initialState,
  },
});

// 导出actions
export const {
  updateTracingConfig,
  updateServiceMap,
  addTrace,
  addAnomaly,
  addMetricDefinition,
  removeMetricDefinition,
  addMetricValue,
  updateMetricStats,
  addAlertRule,
  removeAlertRule,
  addActiveAlert,
  removeActiveAlert,
  updateAlertStats,
  addLogEntry,
  updateLogStats,
  updateLogPatterns,
  updateRetentionPolicy,
  addDashboard,
  removeDashboard,
  setActiveDashboard,
  updateTemplates,
  updateOverallStatus,
  addAlert,
  removeAlert,
  acknowledgeAlert,
  clearAllAlerts,
  resetObservabilityState,
} = observabilitySlice.actions;

// 导出reducer
export default observabilitySlice.reducer;

// Selectors
export const selectObservabilityOverallStatus = (state: { observability: ObservabilityState }) => state.observability.overallStatus;
export const selectTracing = (state: { observability: ObservabilityState }) => state.observability.tracing;
export const selectMetrics = (state: { observability: ObservabilityState }) => state.observability.metrics;
export const selectAlerting = (state: { observability: ObservabilityState }) => state.observability.alerting;
export const selectLogging = (state: { observability: ObservabilityState }) => state.observability.logging;
export const selectVisualization = (state: { observability: ObservabilityState }) => state.observability.visualization;
export const selectObservabilityAlerts = (state: { observability: ObservabilityState }) => state.observability.activeAlerts;