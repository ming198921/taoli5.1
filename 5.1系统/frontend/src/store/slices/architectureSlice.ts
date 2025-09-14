import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import type {
  SystemLimits,
  RuntimeEnforcementConfig,
  SystemResourceUsage,
  ConfigurationHotReload,
  HealthCheckConfig,
  FaultRecoveryConfig,
  ResourceThresholds
} from '@/types/architecture';

// Architecture模块状态类型
interface ArchitectureState {
  // 限制器状态
  limits: {
    config: SystemLimits;
    currentUsage: {
      exchanges: { current: number; limit: number; utilization: number };
      symbols: { current: number; limit: number; utilization: number };
      dailyVolume: { current: number; limit: number; utilization: number };
      requestsPerSecond: { current: number; limit: number; utilization: number };
      websocketConnections: { current: number; limit: number; utilization: number };
      memory: { current: number; limit: number; utilization: number };
      cpu: { current: number; limit: number; utilization: number };
      strategies: { current: number; limit: number; utilization: number };
      positions: { current: number; limit: number; utilization: number };
    };
    violations: Array<{
      limitName: string;
      currentValue: number;
      limitValue: number;
      violationPercent: number;
      severity: 'warning' | 'critical';
      suggestedAction: string;
      timestamp: string;
    }>;
    temporaryAdjustments: Array<{
      id: string;
      adjustments: Record<string, number>;
      expiresAt: string;
      isActive: boolean;
    }>;
  };
  
  // 运行时强制执行状态
  enforcement: {
    enabled: boolean;
    config: RuntimeEnforcementConfig;
    status: {
      mode: 'warn' | 'throttle' | 'block';
      activeViolations: number;
      totalChecks: number;
      lastCheckTime: string;
      escalationLevel: number;
      circuitBreakerActive: boolean;
    };
    violationHistory: Array<{
      timestamp: string;
      violationType: string;
      severity: string;
      currentValue: number;
      thresholdValue: number;
      actionsTaken: string[];
      resolved: boolean;
      resolutionTime?: string;
    }>;
    resourceThresholds: ResourceThresholds;
  };
  
  // 配置热重载状态
  hotReload: {
    enabled: boolean;
    config: ConfigurationHotReload;
    status: {
      watchedDirectories: string[];
      lastReloadTime?: string;
      pendingReloads: number;
      totalReloads: number;
      failedReloads: number;
      currentConfigHash: string;
    };
    changeHistory: Array<{
      timestamp: string;
      configFile: string;
      changeType: string;
      changes: Record<string, any>;
      triggeredBy: string;
      success: boolean;
      errorMessage?: string;
    }>;
    backups: Array<{
      backupId: string;
      createdAt: string;
      configHash: string;
      sizeBytes: number;
      description: string;
    }>;
  };
  
  // 健康检查状态
  healthCheck: {
    enabled: boolean;
    config: HealthCheckConfig;
    overallStatus: 'healthy' | 'degraded' | 'unhealthy';
    lastCheckTime: string;
    nextCheckTime: string;
    checks: {
      [checkName: string]: {
        name: string;
        status: 'pass' | 'fail' | 'warn';
        responseTime: number;
        errorMessage?: string;
        details: Record<string, any>;
        lastCheck: string;
      };
    };
    dependencies: Array<{
      name: string;
      url: string;
      status: 'online' | 'offline' | 'degraded';
      responseTime: number;
      lastCheck: string;
      uptimePercent: number;
      errorMessage?: string;
    }>;
    issues: Array<{
      checkName: string;
      severity: string;
      message: string;
      firstSeen: string;
      lastSeen: string;
    }>;
    stats: {
      totalChecks: number;
      failedChecks: number;
      uptimePercent: number;
    };
  };
  
  // 系统资源监控状态
  resources: {
    current: SystemResourceUsage | null;
    history: SystemResourceUsage[];
    alerts: Array<{
      resourceType: string;
      severity: 'warning' | 'critical';
      message: string;
      currentValue: number;
      thresholdValue: number;
      duration: number;
      firstSeen: string;
    }>;
    thresholds: {
      cpuWarning: number;
      cpuCritical: number;
      memoryWarning: number;
      memoryCritical: number;
      diskWarning: number;
      diskCritical: number;
      networkWarning: number;
      networkCritical: number;
    };
    analysis: {
      summary: {
        avgCpuUsage: number;
        maxCpuUsage: number;
        avgMemoryUsage: number;
        maxMemoryUsage: number;
        totalDiskIO: number;
        totalNetworkIO: number;
      };
      trends: Array<{
        metric: string;
        trend: 'increasing' | 'decreasing' | 'stable';
        ratePerHour: number;
      }>;
      anomalies: Array<{
        timestamp: string;
        metric: string;
        value: number;
        expectedRange: [number, number];
        severity: string;
      }>;
      recommendations: string[];
    };
  };
  
  // 故障恢复状态
  faultRecovery: {
    enabled: boolean;
    config: FaultRecoveryConfig;
    status: {
      activeRecoveries: number;
      lastRecoveryTime?: string;
      totalRecoveries: number;
      successRate: number;
      availableStrategies: string[];
    };
    strategies: Array<{
      name: string;
      priority: number;
      triggerConditions: string[];
      actions: Array<{
        type: string;
        config: Record<string, any>;
        timeout: number;
      }>;
      successCriteria: string[];
    }>;
    recoveryHistory: Array<{
      recoveryId: string;
      triggeredAt: string;
      triggerReason: string;
      strategyUsed: string;
      status: 'in_progress' | 'completed' | 'failed';
      duration: number;
      success: boolean;
      actionsTaken: string[];
      errorMessage?: string;
    }>;
    backupStatus: {
      lastBackup: string;
      backupSizeGB: number;
      backupHealth: 'good' | 'stale' | 'corrupted';
    };
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
const initialState: ArchitectureState = {
  limits: {
    config: {} as SystemLimits,
    currentUsage: {
      exchanges: { current: 0, limit: 0, utilization: 0 },
      symbols: { current: 0, limit: 0, utilization: 0 },
      dailyVolume: { current: 0, limit: 0, utilization: 0 },
      requestsPerSecond: { current: 0, limit: 0, utilization: 0 },
      websocketConnections: { current: 0, limit: 0, utilization: 0 },
      memory: { current: 0, limit: 0, utilization: 0 },
      cpu: { current: 0, limit: 0, utilization: 0 },
      strategies: { current: 0, limit: 0, utilization: 0 },
      positions: { current: 0, limit: 0, utilization: 0 },
    },
    violations: [],
    temporaryAdjustments: [],
  },
  enforcement: {
    enabled: false,
    config: {} as RuntimeEnforcementConfig,
    status: {
      mode: 'warn',
      activeViolations: 0,
      totalChecks: 0,
      lastCheckTime: '',
      escalationLevel: 0,
      circuitBreakerActive: false,
    },
    violationHistory: [],
    resourceThresholds: {} as ResourceThresholds,
  },
  hotReload: {
    enabled: false,
    config: {} as ConfigurationHotReload,
    status: {
      watchedDirectories: [],
      pendingReloads: 0,
      totalReloads: 0,
      failedReloads: 0,
      currentConfigHash: '',
    },
    changeHistory: [],
    backups: [],
  },
  healthCheck: {
    enabled: false,
    config: {} as HealthCheckConfig,
    overallStatus: 'unhealthy',
    lastCheckTime: '',
    nextCheckTime: '',
    checks: {},
    dependencies: [],
    issues: [],
    stats: {
      totalChecks: 0,
      failedChecks: 0,
      uptimePercent: 0,
    },
  },
  resources: {
    current: null,
    history: [],
    alerts: [],
    thresholds: {
      cpuWarning: 70,
      cpuCritical: 90,
      memoryWarning: 80,
      memoryCritical: 95,
      diskWarning: 80,
      diskCritical: 95,
      networkWarning: 80,
      networkCritical: 95,
    },
    analysis: {
      summary: {
        avgCpuUsage: 0,
        maxCpuUsage: 0,
        avgMemoryUsage: 0,
        maxMemoryUsage: 0,
        totalDiskIO: 0,
        totalNetworkIO: 0,
      },
      trends: [],
      anomalies: [],
      recommendations: [],
    },
  },
  faultRecovery: {
    enabled: false,
    config: {} as FaultRecoveryConfig,
    status: {
      activeRecoveries: 0,
      totalRecoveries: 0,
      successRate: 0,
      availableStrategies: [],
    },
    strategies: [],
    recoveryHistory: [],
    backupStatus: {
      lastBackup: '',
      backupSizeGB: 0,
      backupHealth: 'stale',
    },
  },
  overallStatus: 'critical',
  lastUpdate: null,
  activeAlerts: [],
};

// Architecture slice
const architectureSlice = createSlice({
  name: 'architecture',
  initialState,
  reducers: {
    // 限制器管理
    updateSystemLimits: (state, action: PayloadAction<Partial<SystemLimits>>) => {
      state.limits.config = { ...state.limits.config, ...action.payload };
    },
    
    updateCurrentUsage: (state, action: PayloadAction<Partial<ArchitectureState['limits']['currentUsage']>>) => {
      state.limits.currentUsage = { ...state.limits.currentUsage, ...action.payload };
    },
    
    addLimitViolation: (state, action: PayloadAction<ArchitectureState['limits']['violations'][number]>) => {
      state.limits.violations.push(action.payload);
    },
    
    removeLimitViolation: (state, action: PayloadAction<string>) => {
      state.limits.violations = state.limits.violations.filter(v => v.limitName !== action.payload);
    },
    
    addTemporaryAdjustment: (state, action: PayloadAction<ArchitectureState['limits']['temporaryAdjustments'][number]>) => {
      state.limits.temporaryAdjustments.push(action.payload);
    },
    
    removeTemporaryAdjustment: (state, action: PayloadAction<string>) => {
      state.limits.temporaryAdjustments = state.limits.temporaryAdjustments.filter(adj => adj.id !== action.payload);
    },
    
    // 运行时强制执行管理
    updateEnforcementConfig: (state, action: PayloadAction<{ enabled?: boolean; config?: Partial<RuntimeEnforcementConfig> }>) => {
      if (action.payload.enabled !== undefined) {
        state.enforcement.enabled = action.payload.enabled;
      }
      if (action.payload.config) {
        state.enforcement.config = { ...state.enforcement.config, ...action.payload.config };
      }
    },
    
    updateEnforcementStatus: (state, action: PayloadAction<Partial<ArchitectureState['enforcement']['status']>>) => {
      state.enforcement.status = { ...state.enforcement.status, ...action.payload };
    },
    
    addViolationHistory: (state, action: PayloadAction<ArchitectureState['enforcement']['violationHistory'][number]>) => {
      state.enforcement.violationHistory.push(action.payload);
      // 保持最近1000条记录
      if (state.enforcement.violationHistory.length > 1000) {
        state.enforcement.violationHistory = state.enforcement.violationHistory.slice(-1000);
      }
    },
    
    updateResourceThresholds: (state, action: PayloadAction<Partial<ResourceThresholds>>) => {
      state.enforcement.resourceThresholds = { ...state.enforcement.resourceThresholds, ...action.payload };
    },
    
    // 配置热重载管理
    updateHotReloadConfig: (state, action: PayloadAction<{ enabled?: boolean; config?: Partial<ConfigurationHotReload> }>) => {
      if (action.payload.enabled !== undefined) {
        state.hotReload.enabled = action.payload.enabled;
      }
      if (action.payload.config) {
        state.hotReload.config = { ...state.hotReload.config, ...action.payload.config };
      }
    },
    
    updateHotReloadStatus: (state, action: PayloadAction<Partial<ArchitectureState['hotReload']['status']>>) => {
      state.hotReload.status = { ...state.hotReload.status, ...action.payload };
    },
    
    addChangeHistory: (state, action: PayloadAction<ArchitectureState['hotReload']['changeHistory'][number]>) => {
      state.hotReload.changeHistory.unshift(action.payload);
      // 保持最近100条记录
      if (state.hotReload.changeHistory.length > 100) {
        state.hotReload.changeHistory = state.hotReload.changeHistory.slice(0, 100);
      }
    },
    
    addBackup: (state, action: PayloadAction<ArchitectureState['hotReload']['backups'][number]>) => {
      state.hotReload.backups.unshift(action.payload);
      // 保持最近50个备份记录
      if (state.hotReload.backups.length > 50) {
        state.hotReload.backups = state.hotReload.backups.slice(0, 50);
      }
    },
    
    // 健康检查管理
    updateHealthCheckConfig: (state, action: PayloadAction<{ enabled?: boolean; config?: Partial<HealthCheckConfig> }>) => {
      if (action.payload.enabled !== undefined) {
        state.healthCheck.enabled = action.payload.enabled;
      }
      if (action.payload.config) {
        state.healthCheck.config = { ...state.healthCheck.config, ...action.payload.config };
      }
    },
    
    updateHealthCheckStatus: (state, action: PayloadAction<{
      overallStatus?: ArchitectureState['healthCheck']['overallStatus'];
      lastCheckTime?: string;
      nextCheckTime?: string;
    }>) => {
      if (action.payload.overallStatus !== undefined) {
        state.healthCheck.overallStatus = action.payload.overallStatus;
      }
      if (action.payload.lastCheckTime !== undefined) {
        state.healthCheck.lastCheckTime = action.payload.lastCheckTime;
      }
      if (action.payload.nextCheckTime !== undefined) {
        state.healthCheck.nextCheckTime = action.payload.nextCheckTime;
      }
    },
    
    updateHealthCheckResult: (state, action: PayloadAction<{ checkName: string; result: ArchitectureState['healthCheck']['checks'][string] }>) => {
      const { checkName, result } = action.payload;
      state.healthCheck.checks[checkName] = result;
    },
    
    updateDependencies: (state, action: PayloadAction<ArchitectureState['healthCheck']['dependencies']>) => {
      state.healthCheck.dependencies = action.payload;
    },
    
    addHealthIssue: (state, action: PayloadAction<ArchitectureState['healthCheck']['issues'][number]>) => {
      const existingIssue = state.healthCheck.issues.find(issue => issue.checkName === action.payload.checkName);
      if (existingIssue) {
        existingIssue.lastSeen = action.payload.lastSeen;
        existingIssue.message = action.payload.message;
      } else {
        state.healthCheck.issues.push(action.payload);
      }
    },
    
    removeHealthIssue: (state, action: PayloadAction<string>) => {
      state.healthCheck.issues = state.healthCheck.issues.filter(issue => issue.checkName !== action.payload);
    },
    
    updateHealthStats: (state, action: PayloadAction<Partial<ArchitectureState['healthCheck']['stats']>>) => {
      state.healthCheck.stats = { ...state.healthCheck.stats, ...action.payload };
    },
    
    // 系统资源监控管理
    updateCurrentResources: (state, action: PayloadAction<SystemResourceUsage>) => {
      state.resources.current = action.payload;
      // 添加到历史记录
      state.resources.history.push(action.payload);
      // 保持最近1000条记录
      if (state.resources.history.length > 1000) {
        state.resources.history = state.resources.history.slice(-1000);
      }
    },
    
    addResourceAlert: (state, action: PayloadAction<ArchitectureState['resources']['alerts'][number]>) => {
      // 检查是否已存在相同类型的告警
      const existingAlert = state.resources.alerts.find(alert => 
        alert.resourceType === action.payload.resourceType && 
        alert.severity === action.payload.severity
      );
      
      if (existingAlert) {
        existingAlert.currentValue = action.payload.currentValue;
        existingAlert.duration = action.payload.duration;
      } else {
        state.resources.alerts.push(action.payload);
      }
    },
    
    removeResourceAlert: (state, action: PayloadAction<string>) => {
      state.resources.alerts = state.resources.alerts.filter(alert => alert.resourceType !== action.payload);
    },
    
    
    updateResourceAnalysis: (state, action: PayloadAction<Partial<ArchitectureState['resources']['analysis']>>) => {
      state.resources.analysis = { ...state.resources.analysis, ...action.payload };
    },
    
    // 故障恢复管理
    updateFaultRecoveryConfig: (state, action: PayloadAction<{ enabled?: boolean; config?: Partial<FaultRecoveryConfig> }>) => {
      if (action.payload.enabled !== undefined) {
        state.faultRecovery.enabled = action.payload.enabled;
      }
      if (action.payload.config) {
        state.faultRecovery.config = { ...state.faultRecovery.config, ...action.payload.config };
      }
    },
    
    updateFaultRecoveryStatus: (state, action: PayloadAction<Partial<ArchitectureState['faultRecovery']['status']>>) => {
      state.faultRecovery.status = { ...state.faultRecovery.status, ...action.payload };
    },
    
    addRecoveryStrategy: (state, action: PayloadAction<ArchitectureState['faultRecovery']['strategies'][number]>) => {
      state.faultRecovery.strategies.push(action.payload);
    },
    
    removeRecoveryStrategy: (state, action: PayloadAction<string>) => {
      state.faultRecovery.strategies = state.faultRecovery.strategies.filter(strategy => strategy.name !== action.payload);
    },
    
    addRecoveryHistory: (state, action: PayloadAction<ArchitectureState['faultRecovery']['recoveryHistory'][number]>) => {
      state.faultRecovery.recoveryHistory.unshift(action.payload);
      // 保持最近100条记录
      if (state.faultRecovery.recoveryHistory.length > 100) {
        state.faultRecovery.recoveryHistory = state.faultRecovery.recoveryHistory.slice(0, 100);
      }
    },
    
    updateBackupStatus: (state, action: PayloadAction<Partial<ArchitectureState['faultRecovery']['backupStatus']>>) => {
      state.faultRecovery.backupStatus = { ...state.faultRecovery.backupStatus, ...action.payload };
    },
    
    // 整体状态管理
    updateOverallStatus: (state, action: PayloadAction<ArchitectureState['overallStatus']>) => {
      state.overallStatus = action.payload;
      state.lastUpdate = new Date().toISOString();
    },
    
    // 告警管理
    addAlert: (state, action: PayloadAction<ArchitectureState['activeAlerts'][number]>) => {
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
    resetArchitectureState: () => initialState,
  },
});

// 导出actions
export const {
  updateSystemLimits,
  updateCurrentUsage,
  addLimitViolation,
  removeLimitViolation,
  addTemporaryAdjustment,
  removeTemporaryAdjustment,
  updateEnforcementConfig,
  updateEnforcementStatus,
  addViolationHistory,
  updateResourceThresholds,
  updateHotReloadConfig,
  updateHotReloadStatus,
  addChangeHistory,
  addBackup,
  updateHealthCheckConfig,
  updateHealthCheckStatus,
  updateHealthCheckResult,
  updateDependencies,
  addHealthIssue,
  removeHealthIssue,
  updateHealthStats,
  updateCurrentResources,
  addResourceAlert,
  removeResourceAlert,
  updateResourceAnalysis,
  updateFaultRecoveryConfig,
  updateFaultRecoveryStatus,
  addRecoveryStrategy,
  removeRecoveryStrategy,
  addRecoveryHistory,
  updateBackupStatus,
  updateOverallStatus,
  addAlert,
  removeAlert,
  acknowledgeAlert,
  clearAllAlerts,
  resetArchitectureState,
} = architectureSlice.actions;

// 导出reducer
export default architectureSlice.reducer;

// Selectors
export const selectArchitectureOverallStatus = (state: { architecture: ArchitectureState }) => state.architecture.overallStatus;
export const selectSystemLimits = (state: { architecture: ArchitectureState }) => state.architecture.limits;
export const selectEnforcement = (state: { architecture: ArchitectureState }) => state.architecture.enforcement;
export const selectHotReload = (state: { architecture: ArchitectureState }) => state.architecture.hotReload;
export const selectHealthCheck = (state: { architecture: ArchitectureState }) => state.architecture.healthCheck;
export const selectResources = (state: { architecture: ArchitectureState }) => state.architecture.resources;
export const selectFaultRecovery = (state: { architecture: ArchitectureState }) => state.architecture.faultRecovery;
export const selectArchitectureAlerts = (state: { architecture: ArchitectureState }) => state.architecture.activeAlerts;