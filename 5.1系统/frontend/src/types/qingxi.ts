// QingXi数据处理模块相关类型定义

export interface TimeManagerConfig {
  enabled: boolean;
  timezone: string;
  syncInterval: number;
}

export interface MemoryManagerConfig {
  enabled: boolean;
  maxMemoryMB: number;
  cleanupInterval: number;
}

export interface SystemLimits {
  maxConnections: number;
  maxMemoryMB: number;
  maxCpuPercent: number;
}

export interface MarketDataCollectorConfig {
  id: string;
  name: string;
  exchange: string;
  symbols: string[];
  enabled: boolean;
  websocket_url: string;
  rest_api_url: string;
  api_key?: string;
  api_secret?: string;
  rate_limit: number;
  connection_timeout_ms: number;
  heartbeat_interval_ms: number;
  reconnect_interval_sec: number;
  max_reconnect_attempts: number;
  buffer_size: number;
  batch_size: number;
}

export interface BatchProcessorConfig {
  enabled: boolean;
  batch_size: number;
  flush_interval_ms: number;
  max_queue_size: number;
  worker_threads: number;
  enable_compression: boolean;
  enable_simd: boolean;
  memory_pool_size_mb: number;
}

export interface CCXTConfig {
  version: string;
  available_exchanges: string[];
  enabled_exchanges: string[];
  fees_cache_ttl_minutes: number;
  rate_limit_global: number;
  sandbox_mode: boolean;
  api_timeout_ms: number;
}

export interface CacheConfig {
  enabled: boolean;
  type: 'lru' | 'redis' | 'memory';
  max_size_mb: number;
  ttl_seconds: number;
  eviction_policy: 'lru' | 'lfu' | 'ttl';
  compression_enabled: boolean;
  statistics_enabled: boolean;
}

export interface QualityMetrics {
  data_freshness_ms: number;
  accuracy_score: number;
  completeness_percent: number;
  consistency_score: number;
  anomaly_count: number;
  error_rate_percent: number;
  throughput_per_second: number;
  latency_p95_ms: number;
}

export interface DataSourceHealth {
  source_id: string;
  exchange: string;
  status: 'healthy' | 'degraded' | 'offline' | 'error';
  last_update: string;
  latency_ms: number;
  error_count: number;
  message_count: number;
  uptime_percent: number;
  last_error?: string;
  last_error_time?: string;
}

export interface ThirdPartyDataSource {
  id: string;
  name: string;
  type: 'price_aggregator' | 'news_sentiment' | 'onchain_data' | 'macro_economic' | 'social_sentiment' | 'regulatory_news';
  provider: string;
  enabled: boolean;
  api_endpoint: string;
  update_frequency: number;
  data_quality_score: number;
  cost_per_request?: number;
  rate_limit: number;
  config: Record<string, any>;
}

export interface NATSConfig {
  servers: string[];
  connection_timeout_ms: number;
  max_reconnect_attempts: number;
  reconnect_wait_ms: number;
  enable_jetstream: boolean;
  max_payload_bytes: number;
  subjects_config: {
    [subject: string]: {
      max_msgs: number;
      max_bytes: number;
      retention: 'limits' | 'interest' | 'workqueue';
    };
  };
}

export interface HighPrecisionTimeConfig {
  ntp_servers: string[];
  sync_interval_ms: number;
  max_drift_ms: number;
  enable_hardware_clock: boolean;
  calibration_samples: number;
  outlier_threshold_ms: number;
}

export interface ZeroCopyMemoryConfig {
  enabled: boolean;
  pool_count: number;
  pool_size_mb: number;
  alignment_bytes: number;
  enable_hugepages: boolean;
  enable_numa_awareness: boolean;
  gc_threshold_percent: number;
  defragmentation_enabled: boolean;
}