// Architecture系统架构控制API
import { apiClient } from './client';
import type {
  SystemLimits,
  RuntimeEnforcementConfig,
  SystemResourceUsage,
  ConfigurationHotReload,
  HealthCheckConfig,
  HealthCheck,
  FaultRecoveryConfig,
  RecoveryStrategy,
  ResourceThresholds,
  CircuitBreakerConfig
} from '@/types/architecture';

export const architectureAPI = {
  // 3.1 限制器控制
  limits: {
    // 获取系统限制配置
    getSystemLimits: () =>
      apiClient.get<SystemLimits>('/api/architecture/limits/system'),
    
    // 更新系统限制
    updateSystemLimits: (limits: Partial<SystemLimits>) =>
      apiClient.put<SystemLimits>('/api/architecture/limits/system', limits),
    
    // 获取当前使用情况
    getCurrentUsage: () =>
      apiClient.get<{
        exchanges: { current: number; limit: number; utilization_percent: number };
        symbols: { current: number; limit: number; utilization_percent: number };
        daily_volume_usd: { current: number; limit: number; utilization_percent: number };
        requests_per_second: { current: number; limit: number; utilization_percent: number };
        websocket_connections: { current: number; limit: number; utilization_percent: number };
        memory_usage_gb: { current: number; limit: number; utilization_percent: number };
        cpu_usage_percent: { current: number; limit: number; utilization_percent: number };
        concurrent_strategies: { current: number; limit: number; utilization_percent: number };
        open_positions: { current: number; limit: number; utilization_percent: number };
      }>('/api/architecture/limits/usage'),
    
    // 检查限制违规
    checkLimitViolations: () =>
      apiClient.get<Array<{
        limit_name: string;
        current_value: number;
        limit_value: number;
        violation_percent: number;
        severity: 'warning' | 'critical';
        suggested_action: string;
      }>>('/api/architecture/limits/violations'),
    
    // 临时调整限制
    temporaryAdjustment: (adjustments: Record<string, number>, duration_minutes: number) =>
      apiClient.post<{ adjustment_id: string; expires_at: string; active_adjustments: Record<string, number> }>('/api/architecture/limits/temporary-adjust', {
        adjustments,
        duration_minutes
      }),
    
    // 恢复默认限制
    resetToDefaults: (confirm: boolean = false) =>
      apiClient.post('/api/architecture/limits/reset-defaults', { confirm }),
  },

  // 3.2 运行时强制执行控制
  enforcement: {
    // 获取强制执行配置
    getConfig: () =>
      apiClient.get<RuntimeEnforcementConfig>('/api/architecture/enforcement/config'),
    
    // 更新强制执行配置
    updateConfig: (config: Partial<RuntimeEnforcementConfig>) =>
      apiClient.put<RuntimeEnforcementConfig>('/api/architecture/enforcement/config', config),
    
    // 启用/禁用强制执行
    setEnabled: (enabled: boolean) =>
      apiClient.post('/api/architecture/enforcement/enabled', { enabled }),
    
    // 获取强制执行状态
    getStatus: () =>
      apiClient.get<{
        enabled: boolean;
        mode: string;
        active_violations: number;
        total_checks: number;
        last_check_time: string;
        escalation_level: number;
        circuit_breaker_active: boolean;
      }>('/api/architecture/enforcement/status'),
    
    // 手动触发检查
    triggerCheck: () =>
      apiClient.post<{
        check_id: string;
        violations_found: number;
        actions_taken: string[];
        next_check_time: string;
      }>('/api/architecture/enforcement/trigger-check'),
    
    // 获取违规历史
    getViolationHistory: (timeRange?: { start: string; end: string }) =>
      apiClient.get<Array<{
        timestamp: string;
        violation_type: string;
        severity: string;
        current_value: number;
        threshold_value: number;
        actions_taken: string[];
        resolved: boolean;
        resolution_time?: string;
      }>>('/api/architecture/enforcement/violations/history', { params: timeRange }),
    
    // 设置资源阈值
    setResourceThresholds: (thresholds: ResourceThresholds) =>
      apiClient.put('/api/architecture/enforcement/thresholds', thresholds),
    
    // 配置断路器
    configureCircuitBreaker: (config: CircuitBreakerConfig) =>
      apiClient.put('/api/architecture/enforcement/circuit-breaker', config),
    
    // 重置断路器
    resetCircuitBreaker: (operation?: string) =>
      apiClient.post('/api/architecture/enforcement/circuit-breaker/reset', { operation }),
  },

  // 3.3 配置热重载控制
  hotReload: {
    // 获取热重载配置
    getConfig: () =>
      apiClient.get<ConfigurationHotReload>('/api/architecture/hot-reload/config'),
    
    // 更新热重载配置
    updateConfig: (config: Partial<ConfigurationHotReload>) =>
      apiClient.put<ConfigurationHotReload>('/api/architecture/hot-reload/config', config),
    
    // 启用/禁用热重载
    setEnabled: (enabled: boolean) =>
      apiClient.post('/api/architecture/hot-reload/enabled', { enabled }),
    
    // 获取监控状态
    getWatchStatus: () =>
      apiClient.get<{
        enabled: boolean;
        watched_directories: string[];
        last_reload_time?: string;
        pending_reloads: number;
        total_reloads: number;
        failed_reloads: number;
        current_config_hash: string;
      }>('/api/architecture/hot-reload/status'),
    
    // 手动触发重载
    triggerReload: (validate: boolean = true) =>
      apiClient.post<{
        reload_id: string;
        status: string;
        changes_detected: string[];
        validation_passed: boolean;
        backup_created: boolean;
      }>('/api/architecture/hot-reload/trigger', { validate }),
    
    // 验证配置
    validateConfig: (configPath?: string) =>
      apiClient.post<{
        valid: boolean;
        errors: string[];
        warnings: string[];
        config_hash: string;
        validation_time_ms: number;
      }>('/api/architecture/hot-reload/validate', { config_path: configPath }),
    
    // 回滚配置
    rollbackConfig: (backupId?: string) =>
      apiClient.post<{
        rollback_id: string;
        status: string;
        previous_config_restored: boolean;
        restart_required: boolean;
      }>('/api/architecture/hot-reload/rollback', { backup_id: backupId }),
    
    // 获取配置变更历史
    getChangeHistory: (limit: number = 50) =>
      apiClient.get<Array<{
        timestamp: string;
        config_file: string;
        change_type: string;
        changes: Record<string, any>;
        triggered_by: string;
        success: boolean;
        error_message?: string;
      }>>('/api/architecture/hot-reload/history', { params: { limit } }),
    
    // 获取配置备份列表
    getBackups: () =>
      apiClient.get<Array<{
        backup_id: string;
        created_at: string;
        config_hash: string;
        size_bytes: number;
        description: string;
      }>>('/api/architecture/hot-reload/backups'),
  },

  // 3.4 健康检查控制
  healthCheck: {
    // 获取健康检查配置
    getConfig: () =>
      apiClient.get<HealthCheckConfig>('/api/architecture/health/config'),
    
    // 更新健康检查配置
    updateConfig: (config: Partial<HealthCheckConfig>) =>
      apiClient.put<HealthCheckConfig>('/api/architecture/health/config', config),
    
    // 执行单次健康检查
    runCheck: (checkName?: string) =>
      apiClient.post<{
        check_id: string;
        overall_status: 'healthy' | 'degraded' | 'unhealthy';
        checks: Array<{
          name: string;
          status: 'pass' | 'fail' | 'warn';
          response_time_ms: number;
          error_message?: string;
          details: Record<string, any>;
        }>;
        execution_time_ms: number;
      }>('/api/architecture/health/check', { check_name: checkName }),
    
    // 获取健康状态
    getStatus: () =>
      apiClient.get<{
        overall_status: 'healthy' | 'degraded' | 'unhealthy';
        last_check_time: string;
        next_check_time: string;
        total_checks: number;
        failed_checks: number;
        uptime_percent: number;
        issues: Array<{
          check_name: string;
          severity: string;
          message: string;
          first_seen: string;
          last_seen: string;
        }>;
      }>('/api/architecture/health/status'),
    
    // 添加自定义健康检查
    addCustomCheck: (check: HealthCheck) =>
      apiClient.post<{ check_id: string; status: string }>('/api/architecture/health/checks/add', check),
    
    // 移除健康检查
    removeCheck: (checkName: string) =>
      apiClient.delete(`/api/architecture/health/checks/${checkName}`),
    
    // 获取健康检查历史
    getCheckHistory: (checkName?: string, timeRange?: { start: string; end: string }) =>
      apiClient.get<Array<{
        timestamp: string;
        check_name: string;
        status: string;
        response_time_ms: number;
        error_message?: string;
        details: Record<string, any>;
      }>>('/api/architecture/health/history', { params: { check_name: checkName, ...timeRange } }),
    
    // 获取依赖状态
    getDependenciesStatus: () =>
      apiClient.get<Array<{
        name: string;
        url: string;
        status: 'online' | 'offline' | 'degraded';
        response_time_ms: number;
        last_check: string;
        uptime_percent: number;
        error_message?: string;
      }>>('/api/architecture/health/dependencies'),
    
    // 设置健康检查告警
    configureAlerting: (config: { enabled: boolean; channels: string[]; escalation_policy: string }) =>
      apiClient.put('/api/architecture/health/alerting', config),
  },

  // 3.5 系统资源监控
  resources: {
    // 获取实时资源使用情况
    getCurrent: () =>
      apiClient.get<SystemResourceUsage>('/api/architecture/resources/current'),
    
    // 获取资源使用历史
    getHistory: (timeRange: { start: string; end: string }, resolution?: 'minute' | 'hour' | 'day') =>
      apiClient.get<SystemResourceUsage[]>('/api/architecture/resources/history', { 
        params: { ...timeRange, resolution: resolution || 'minute' } 
      }),
    
    // 获取资源告警
    getAlerts: () =>
      apiClient.get<Array<{
        resource_type: string;
        severity: 'warning' | 'critical';
        message: string;
        current_value: number;
        threshold_value: number;
        duration_seconds: number;
        first_seen: string;
      }>>('/api/architecture/resources/alerts'),
    
    // 设置资源阈值
    setThresholds: (thresholds: {
      cpu_warning: number;
      cpu_critical: number;
      memory_warning: number;
      memory_critical: number;
      disk_warning: number;
      disk_critical: number;
      network_warning: number;
      network_critical: number;
    }) =>
      apiClient.put('/api/architecture/resources/thresholds', thresholds),
    
    // 获取性能分析
    getPerformanceAnalysis: (timeRange: { start: string; end: string }) =>
      apiClient.get<{
        summary: {
          avg_cpu_usage: number;
          max_cpu_usage: number;
          avg_memory_usage: number;
          max_memory_usage: number;
          total_disk_io: number;
          total_network_io: number;
        };
        trends: Array<{
          metric: string;
          trend: 'increasing' | 'decreasing' | 'stable';
          rate_per_hour: number;
        }>;
        anomalies: Array<{
          timestamp: string;
          metric: string;
          value: number;
          expected_range: [number, number];
          severity: string;
        }>;
        recommendations: string[];
      }>('/api/architecture/resources/analysis', { params: timeRange }),
    
    // 资源清理
    cleanup: (resources: ('logs' | 'temp_files' | 'cache' | 'old_backups')[]) =>
      apiClient.post<{
        cleanup_id: string;
        freed_space_gb: number;
        cleaned_items: Record<string, number>;
        duration_seconds: number;
      }>('/api/architecture/resources/cleanup', { resources }),
  },

  // 3.6 故障恢复控制
  faultRecovery: {
    // 获取故障恢复配置
    getConfig: () =>
      apiClient.get<FaultRecoveryConfig>('/api/architecture/fault-recovery/config'),
    
    // 更新故障恢复配置
    updateConfig: (config: Partial<FaultRecoveryConfig>) =>
      apiClient.put<FaultRecoveryConfig>('/api/architecture/fault-recovery/config', config),
    
    // 获取恢复策略
    getStrategies: () =>
      apiClient.get<RecoveryStrategy[]>('/api/architecture/fault-recovery/strategies'),
    
    // 添加恢复策略
    addStrategy: (strategy: Omit<RecoveryStrategy, 'name'> & { name: string }) =>
      apiClient.post<{ strategy_id: string }>('/api/architecture/fault-recovery/strategies/add', strategy),
    
    // 测试恢复策略
    testStrategy: (strategyName: string) =>
      apiClient.post<{
        test_id: string;
        strategy_name: string;
        test_successful: boolean;
        execution_time_seconds: number;
        actions_executed: string[];
        errors: string[];
      }>('/api/architecture/fault-recovery/strategies/test', { strategy_name: strategyName }),
    
    // 手动触发恢复
    triggerRecovery: (reason: string, strategy?: string) =>
      apiClient.post<{
        recovery_id: string;
        strategy_used: string;
        status: 'in_progress' | 'completed' | 'failed';
        actions_taken: string[];
        estimated_completion_time?: string;
      }>('/api/architecture/fault-recovery/trigger', { reason, strategy }),
    
    // 获取恢复历史
    getRecoveryHistory: (timeRange?: { start: string; end: string }) =>
      apiClient.get<Array<{
        recovery_id: string;
        triggered_at: string;
        trigger_reason: string;
        strategy_used: string;
        status: string;
        duration_seconds: number;
        success: boolean;
        actions_taken: string[];
        error_message?: string;
      }>>('/api/architecture/fault-recovery/history', { params: timeRange }),
    
    // 获取当前状态
    getRecoveryStatus: () =>
      apiClient.get<{
        enabled: boolean;
        active_recoveries: number;
        last_recovery_time?: string;
        total_recoveries: number;
        success_rate: number;
        available_strategies: string[];
        backup_status: {
          last_backup: string;
          backup_size_gb: number;
          backup_health: 'good' | 'stale' | 'corrupted';
        };
      }>('/api/architecture/fault-recovery/status'),
    
    // 创建状态备份
    createBackup: (description?: string) =>
      apiClient.post<{
        backup_id: string;
        created_at: string;
        size_gb: number;
        components_backed_up: string[];
      }>('/api/architecture/fault-recovery/backup', { description }),
    
    // 从备份恢复
    restoreFromBackup: (backupId: string, confirm: boolean = false) =>
      apiClient.post<{
        restore_id: string;
        status: 'in_progress' | 'completed' | 'failed';
        estimated_completion_time?: string;
        components_to_restore: string[];
      }>('/api/architecture/fault-recovery/restore', { backup_id: backupId, confirm }),
  },
};