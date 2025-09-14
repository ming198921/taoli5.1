// Architecture系统架构相关类型定义

export interface SystemLimits {
  max_exchanges: number;
  max_symbols_per_exchange: number;
  max_daily_volume_usd: number;
  max_requests_per_second: number;
  max_websocket_connections: number;
  max_memory_usage_gb: number;
  max_cpu_usage_percent: number;
  max_concurrent_strategies: number;
  max_open_positions: number;
}

export interface RuntimeEnforcementConfig {
  enabled: boolean;
  enforcement_mode: 'warn' | 'throttle' | 'block';
  check_interval_ms: number;
  grace_period_ms: number;
  escalation_levels: EscalationLevel[];
  resource_thresholds: ResourceThresholds;
  circuit_breaker_config: CircuitBreakerConfig;
}

export interface EscalationLevel {
  level: number;
  threshold_percent: number;
  actions: ('log' | 'alert' | 'throttle' | 'shutdown')[];
  cooldown_seconds: number;
  notification_channels: string[];
}

export interface ResourceThresholds {
  cpu: {
    warning_percent: number;
    critical_percent: number;
    sustained_duration_seconds: number;
  };
  memory: {
    warning_percent: number;
    critical_percent: number;
    oom_protection_enabled: boolean;
  };
  disk: {
    warning_percent: number;
    critical_percent: number;
    cleanup_enabled: boolean;
  };
  network: {
    max_bandwidth_mbps: number;
    max_connections: number;
    timeout_seconds: number;
  };
}

export interface CircuitBreakerConfig {
  enabled: boolean;
  failure_threshold: number;
  recovery_timeout_seconds: number;
  half_open_max_calls: number;
  reset_timeout_seconds: number;
  monitored_operations: string[];
}

export interface ConfigurationHotReload {
  enabled: boolean;
  watch_directories: string[];
  reload_delay_ms: number;
  validation_enabled: boolean;
  backup_enabled: boolean;
  rollback_on_error: boolean;
  notification_channels: string[];
  excluded_files: string[];
}

export interface HealthCheckConfig {
  enabled: boolean;
  endpoint: string;
  check_interval_seconds: number;
  timeout_seconds: number;
  checks: HealthCheck[];
  dependencies: DependencyCheck[];
  alerting: HealthAlertConfig;
}

export interface HealthCheck {
  name: string;
  type: 'database' | 'api' | 'cache' | 'queue' | 'external_service' | 'custom';
  config: Record<string, any>;
  timeout_seconds: number;
  critical: boolean;
  retry_attempts: number;
  retry_delay_seconds: number;
}

export interface DependencyCheck {
  name: string;
  url: string;
  method: 'GET' | 'POST' | 'HEAD';
  expected_status: number;
  timeout_seconds: number;
  headers?: Record<string, string>;
  payload?: any;
}

export interface HealthAlertConfig {
  enabled: boolean;
  channels: ('email' | 'slack' | 'webhook' | 'pagerduty')[];
  escalation_policy: string;
  silence_duration_minutes: number;
  group_alerts: boolean;
}

export interface SystemResourceUsage {
  timestamp: string;
  cpu: {
    usage_percent: number;
    load_average: [number, number, number];
    core_count: number;
    temperature_celsius?: number;
  };
  memory: {
    total_gb: number;
    used_gb: number;
    available_gb: number;
    cached_gb: number;
    swap_used_gb: number;
    swap_total_gb: number;
  };
  disk: {
    total_gb: number;
    used_gb: number;
    available_gb: number;
    io_read_bps: number;
    io_write_bps: number;
    iops_read: number;
    iops_write: number;
  };
  network: {
    interfaces: NetworkInterface[];
    total_bytes_sent: number;
    total_bytes_received: number;
    connections_active: number;
    connections_established: number;
  };
}

export interface NetworkInterface {
  name: string;
  bytes_sent: number;
  bytes_received: number;
  packets_sent: number;
  packets_received: number;
  errors_in: number;
  errors_out: number;
  dropped_in: number;
  dropped_out: number;
}

export interface FaultRecoveryConfig {
  enabled: boolean;
  recovery_strategies: RecoveryStrategy[];
  max_recovery_attempts: number;
  recovery_timeout_seconds: number;
  state_backup_interval_minutes: number;
  disaster_recovery_enabled: boolean;
  cross_region_replication: boolean;
}

export interface RecoveryStrategy {
  name: string;
  trigger_conditions: string[];
  actions: RecoveryAction[];
  priority: number;
  timeout_seconds: number;
  success_criteria: string[];
}

export interface RecoveryAction {
  type: 'restart_service' | 'switch_endpoint' | 'rollback_config' | 'scale_resources' | 'notify_admin';
  config: Record<string, any>;
  timeout_seconds: number;
  retry_attempts: number;
  depends_on?: string[];
}