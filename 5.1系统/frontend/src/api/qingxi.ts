// QingXi数据处理模块API
import { apiClient } from './client';
import type {
  MarketDataCollectorConfig,
  BatchProcessorConfig,
  CacheConfig,
  QualityMetrics,
  DataSourceHealth,
  ThirdPartyDataSource,
  NATSConfig,
  HighPrecisionTimeConfig,
  ZeroCopyMemoryConfig
} from '@/types/qingxi';

export const qingxiAPI = {
  // 1.1 市场数据采集控制
  collectors: {
    create: (config: Omit<MarketDataCollectorConfig, 'id'>) =>
      apiClient.post<MarketDataCollectorConfig>('/api/qingxi/collectors/create', config),
    
    list: () =>
      apiClient.get<MarketDataCollectorConfig[]>('/api/qingxi/collectors/list'),
    
    update: (id: string, config: Partial<MarketDataCollectorConfig>) =>
      apiClient.put<MarketDataCollectorConfig>(`/api/qingxi/collectors/${id}/config`, config),
    
    delete: (id: string) =>
      apiClient.delete(`/api/qingxi/collectors/${id}`),
    
    start: (id: string) =>
      apiClient.post(`/api/qingxi/collectors/${id}/start`),
    
    stop: (id: string) =>
      apiClient.post(`/api/qingxi/collectors/${id}/stop`),
    
    restart: (id: string) =>
      apiClient.post(`/api/qingxi/collectors/${id}/restart`),
    
    getStatus: (id: string) =>
      apiClient.get<{ status: string; health: DataSourceHealth }>(`/api/qingxi/collectors/${id}/status`),
      
    // 清洗性能监控
    getCleaningPerformance: () =>
      apiClient.get<{
        overall_stats: {
          fastest_ms: number;
          slowest_ms: number;
          average_ms: number;
          total_count: number;
          last_update: number | null;
        };
        per_currency_stats: Record<string, {
          fastest_ms: number;
          slowest_ms: number;
          average_ms: number;
          total_count: number;
          last_update: number | null;
        }>;
        system_info: {
          v3_optimizations_enabled: boolean;
          target_range_ms: string;
          performance_status: string;
        };
      }>('/api/qingxi/cleaning/performance'),
  },

  // 1.2 批处理器控制
  batchProcessor: {
    getConfig: () =>
      apiClient.get<BatchProcessorConfig>('/api/qingxi/batch/config'),
    
    updateConfig: (config: BatchProcessorConfig) =>
      apiClient.put<BatchProcessorConfig>('/api/qingxi/batch/config', config),
    
    optimize: () =>
      apiClient.post<{ optimization_applied: string[]; performance_improvement: number }>('/api/qingxi/batch/optimize'),
    
    getStatistics: () =>
      apiClient.get<{
        processed_records: number;
        avg_processing_time_ms: number;
        throughput_per_second: number;
        error_rate: number;
        memory_usage_mb: number;
      }>('/api/qingxi/batch/statistics'),
    
    updateBufferSize: (size: number) =>
      apiClient.put('/api/qingxi/batch/buffer-size', { size }),
  },

  // 1.2 SIMD优化控制
  simd: {
    getStatus: () =>
      apiClient.get<{ enabled: boolean; instruction_set: string; performance_gain: number }>('/api/qingxi/simd/status'),
    
    enable: () =>
      apiClient.post('/api/qingxi/simd/enable'),
    
    disable: () =>
      apiClient.post('/api/qingxi/simd/disable'),
    
    getBenchmarks: () =>
      apiClient.get<{ benchmark_results: Array<{ operation: string; speedup: number; latency_reduction: number }> }>('/api/qingxi/simd/benchmarks'),
  },

  // 1.3 CCXT集成管理
  ccxt: {
    getVersion: () =>
      apiClient.get<{ version: string; last_update: string }>('/api/ccxt/version'),
    
    upgrade: () =>
      apiClient.post<{ new_version: string; changelog: string[] }>('/api/ccxt/upgrade'),
    
    getAvailableExchanges: () =>
      apiClient.get<string[]>('/api/ccxt/exchanges/available'),
    
    reloadLibrary: () =>
      apiClient.post('/api/ccxt/library/reload'),
    
    fetchFees: (exchange?: string) =>
      apiClient.post<{ updated_exchanges: string[]; timestamp: string }>('/api/ccxt/fees/fetch', { exchange }),
    
    getCurrentFees: (exchange: string) =>
      apiClient.get<{ maker: number; taker: number; withdrawal: Record<string, number>; last_updated: string }>(`/api/ccxt/fees/${exchange}/current`),
    
    refreshFeesCache: () =>
      apiClient.put('/api/ccxt/fees/cache/refresh'),
    
    getFeesHistory: (exchange: string) =>
      apiClient.get<Array<{ date: string; maker: number; taker: number }>>(`/api/ccxt/fees/history/${exchange}`),
    
    listAdapters: () =>
      apiClient.get<Array<{ exchange: string; version: string; status: string; last_sync: string }>>('/api/ccxt/adapters/list'),
    
    configureAdapter: (exchange: string, config: Record<string, any>) =>
      apiClient.post(`/api/ccxt/adapters/${exchange}/configure`, config),
    
    getAdapterStatus: (exchange: string) =>
      apiClient.get<{ status: string; health_score: number; last_error?: string }>(`/api/ccxt/adapters/${exchange}/status`),
    
    testAdapter: (exchange: string) =>
      apiClient.post<{ success: boolean; latency_ms: number; error?: string }>(`/api/ccxt/adapters/${exchange}/test`),
  },

  // 1.4 高精度时间管理
  time: {
    getCurrentPrecision: () =>
      apiClient.get<{ precision_microseconds: number; drift_ms: number; sync_status: string }>('/api/time/precision/current'),
    
    calibrate: () =>
      apiClient.post<{ calibration_result: string; new_precision_microseconds: number }>('/api/time/precision/calibrate'),
    
    getLatencyMeasurements: () =>
      apiClient.get<Array<{ target: string; latency_ms: number; jitter_ms: number; packet_loss: number }>>('/api/time/latency/measurements'),
    
    updateSyncConfig: (config: HighPrecisionTimeConfig) =>
      apiClient.put('/api/time/synchronization/config', config),
    
    getLatencyStats: () =>
      apiClient.get<{ avg_latency_ms: number; p95_latency_ms: number; p99_latency_ms: number }>('/api/time/latency/stats'),
    
    runLatencyBenchmark: () =>
      apiClient.post<{ benchmark_results: Array<{ endpoint: string; avg_latency_ms: number; min_latency_ms: number; max_latency_ms: number }> }>('/api/time/latency/benchmark'),
    
    getDriftDetection: () =>
      apiClient.get<{ clock_drift_ms: number; drift_rate_ppm: number; correction_needed: boolean }>('/api/time/drift/detection'),
  },

  // 1.5 零拷贝内存管理
  memory: {
    getPoolsStatus: () =>
      apiClient.get<Array<{ pool_id: string; size_mb: number; used_mb: number; fragmentation_percent: number; allocations: number }>>('/api/memory/pools/status'),
    
    optimizePools: () =>
      apiClient.post<{ optimization_applied: string[]; memory_saved_mb: number }>('/api/memory/pools/optimize'),
    
    getAllocationStats: () =>
      apiClient.get<{ total_allocated_mb: number; peak_allocated_mb: number; allocation_rate_per_second: number; gc_collections: number }>('/api/memory/allocation/stats'),
    
    resizePools: (poolId: string, newSizeMb: number) =>
      apiClient.post('/api/memory/pools/resize', { pool_id: poolId, new_size_mb: newSizeMb }),
    
    getZeroAllocMetrics: () =>
      apiClient.get<{ zero_alloc_hit_rate: number; memory_reuse_percent: number; performance_gain: number }>('/api/memory/zero-alloc/metrics'),
    
    tuneZeroAlloc: (config: ZeroCopyMemoryConfig) =>
      apiClient.post('/api/memory/zero-alloc/tune', config),
    
    getFragmentationAnalysis: () =>
      apiClient.get<{ total_fragmentation_percent: number; largest_free_block_mb: number; defragmentation_needed: boolean }>('/api/memory/fragmentation/analysis'),
  },

  // 1.6 第三方数据源集成管理
  thirdParty: {
    listSources: () =>
      apiClient.get<ThirdPartyDataSource[]>('/api/third-party/sources/list'),
    
    registerSource: (source: Omit<ThirdPartyDataSource, 'id'>) =>
      apiClient.post<ThirdPartyDataSource>('/api/third-party/sources/register', source),
    
    updateSourceConfig: (id: string, config: Partial<ThirdPartyDataSource>) =>
      apiClient.put<ThirdPartyDataSource>(`/api/third-party/sources/${id}/config`, config),
    
    deleteSource: (id: string) =>
      apiClient.delete(`/api/third-party/sources/${id}`),
    
    // 价格聚合器控制
    enablePriceAggregator: (providers: string[]) =>
      apiClient.post('/api/third-party/price-aggregator/enable', { providers }),
    
    getPriceProviders: () =>
      apiClient.get<Array<{ provider: string; reliability_score: number; latency_ms: number; coverage: string[] }>>('/api/third-party/price-aggregator/providers'),
    
    updateProviderWeights: (weights: Record<string, number>) =>
      apiClient.put('/api/third-party/price-aggregator/weights', weights),
    
    // 新闻情感分析
    enableNewsSentiment: (config: { sources: string[]; keywords: string[]; languages: string[] }) =>
      apiClient.post('/api/third-party/sentiment/news/enable', config),
    
    getNewsSentimentScore: (symbol?: string) =>
      apiClient.get<{ overall_score: number; positive_percent: number; negative_percent: number; neutral_percent: number; article_count: number }>('/api/third-party/sentiment/news/score', { params: { symbol } }),
    
    updateSentimentThreshold: (threshold: number) =>
      apiClient.put('/api/third-party/sentiment/threshold', { threshold }),
    
    // 链上数据监控
    enableOnchainData: (config: { blockchains: string[]; data_types: string[] }) =>
      apiClient.post('/api/third-party/onchain/enable', config),
    
    getOnchainMetrics: (blockchain?: string) =>
      apiClient.get<{ total_value_locked: number; active_addresses: number; transaction_volume: number; gas_price: number }>('/api/third-party/onchain/metrics', { params: { blockchain } }),
    
    updateOnchainConfig: (chain: string, config: Record<string, any>) =>
      apiClient.put(`/api/third-party/onchain/blockchain/${chain}/config`, config),
    
    // 宏观经济指标
    getMacroIndicators: () =>
      apiClient.get<Array<{ indicator: string; value: number; change_percent: number; last_updated: string }>>('/api/third-party/macro/indicators'),
    
    subscribeMacroData: (indicators: string[]) =>
      apiClient.post('/api/third-party/macro/subscribe', { indicators }),
    
    getMacroImpactAnalysis: (symbol: string) =>
      apiClient.get<{ correlation_strength: number; impact_factors: Array<{ factor: string; correlation: number; significance: number }> }>('/api/third-party/macro/impact/analysis', { params: { symbol } }),
    
    // 社交媒体情绪
    enableSocialPlatforms: (platforms: string[]) =>
      apiClient.post('/api/third-party/social/platforms/enable', { platforms }),
    
    getSocialSentiment: (symbol: string) =>
      apiClient.get<{ sentiment_score: number; volume: number; trending_topics: string[]; influential_posts: Array<{ platform: string; content: string; engagement: number }> }>(`/api/third-party/social/sentiment/${symbol}`),
    
    updateSocialKeywords: (keywords: string[]) =>
      apiClient.put('/api/third-party/social/keywords', { keywords }),
    
    // 监管公告监控
    enableRegulatoryAlerts: (config: { jurisdictions: string[]; keywords: string[] }) =>
      apiClient.post('/api/third-party/regulatory/alerts/enable', config),
    
    getRegulatoryUpdates: () =>
      apiClient.get<Array<{ jurisdiction: string; title: string; summary: string; impact_level: string; published_at: string }>>('/api/third-party/regulatory/updates'),
    
    updateRegulatoryJurisdictions: (jurisdictions: string[]) =>
      apiClient.put('/api/third-party/regulatory/jurisdictions', { jurisdictions }),
    
    // 数据融合与质量控制
    getFusionConfig: () =>
      apiClient.get<{ fusion_algorithm: string; confidence_threshold: number; data_sources: string[] }>('/api/third-party/fusion/config'),
    
    assessDataQuality: (sourceId?: string) =>
      apiClient.post<{ quality_score: number; completeness: number; timeliness: number; accuracy: number; consistency: number }>('/api/third-party/quality/assess', { source_id: sourceId }),
    
    getQualityReport: () =>
      apiClient.get<{ overall_quality: number; source_rankings: Array<{ source: string; quality_score: number; issues: string[] }> }>('/api/third-party/quality/report'),
    
    updateValidationRules: (rules: Array<{ rule_name: string; condition: string; action: string }>) =>
      apiClient.put('/api/third-party/validation/rules', { rules }),
  },

  // 1.7 NATS消息队列管理
  nats: {
    getConnectionStatus: () =>
      apiClient.get<{ connected: boolean; server: string; cluster_size: number; uptime_seconds: number }>('/api/nats/connection/status'),
    
    reconnect: () =>
      apiClient.post('/api/nats/connection/reconnect'),
    
    updateConnectionConfig: (config: NATSConfig) =>
      apiClient.put('/api/nats/connection/config', config),
    
    // 主题管理
    listSubjects: () =>
      apiClient.get<Array<{ subject: string; subscribers: number; messages_sent: number; messages_received: number; bytes_sent: number; bytes_received: number }>>('/api/nats/subjects/list'),
    
    publishToSubject: (subject: string, data: any) =>
      apiClient.post(`/api/nats/subjects/${subject}/publish`, { data }),
    
    subscribeToSubject: (subject: string) =>
      apiClient.post(`/api/nats/subjects/${subject}/subscribe`),
    
    unsubscribeFromSubject: (subject: string) =>
      apiClient.delete(`/api/nats/subjects/${subject}/unsubscribe`),
    
    // 消息监控
    getMessageStats: () =>
      apiClient.get<{ total_messages: number; messages_per_second: number; average_message_size_bytes: number; peak_throughput: number }>('/api/nats/messages/stats'),
    
    getMessageThroughput: () =>
      apiClient.get<Array<{ timestamp: string; messages_per_second: number; bytes_per_second: number }>>('/api/nats/messages/throughput'),
    
    getMessageLatency: () =>
      apiClient.get<{ avg_latency_ms: number; p95_latency_ms: number; p99_latency_ms: number; max_latency_ms: number }>('/api/nats/messages/latency'),
    
    // JetStream管理
    listStreams: () =>
      apiClient.get<Array<{ name: string; subjects: string[]; messages: number; bytes: number; consumers: number; retention: string }>>('/api/nats/jetstream/streams'),
    
    createStream: (config: { name: string; subjects: string[]; retention: string; max_msgs: number; max_bytes: number }) =>
      apiClient.post<{ stream_created: string; config: any }>('/api/nats/jetstream/streams/create', config),
    
    deleteStream: (stream: string) =>
      apiClient.delete(`/api/nats/jetstream/streams/${stream}`),
    
    getStreamConsumers: (stream: string) =>
      apiClient.get<Array<{ name: string; pending_messages: number; delivered_messages: number; redelivered_messages: number; ack_pending: number }>>(`/api/nats/jetstream/consumers/${stream}`),
  },

  // 1.8 数据缓存管理
  cache: {
    getPolicies: () =>
      apiClient.get<CacheConfig[]>('/api/qingxi/cache/policies'),
    
    updatePolicy: (type: string, policy: CacheConfig) =>
      apiClient.put(`/api/qingxi/cache/policies/${type}`, policy),
    
    clear: (pattern?: string) =>
      apiClient.post('/api/qingxi/cache/clear', { pattern }),
    
    getStats: () =>
      apiClient.get<{ hit_rate: number; miss_rate: number; eviction_rate: number; size_mb: number; entries: number }>('/api/qingxi/cache/stats'),
    
    warmup: (keys: string[]) =>
      apiClient.post('/api/qingxi/cache/warmup', { keys }),
    
    invalidateKey: (key: string) =>
      apiClient.delete(`/api/qingxi/cache/invalidate/${encodeURIComponent(key)}`),
    
    // LRU配置
    getLRUConfig: () =>
      apiClient.get<{ max_size: number; ttl_seconds: number; eviction_policy: string }>('/api/qingxi/cache/lru/config'),
    
    updateLRUSize: (size: number) =>
      apiClient.put('/api/qingxi/cache/lru/size', { size }),
    
    resetLRU: () =>
      apiClient.post('/api/qingxi/cache/lru/reset'),
  },

  // 数据质量监控
  quality: {
    getMetrics: (exchange?: string) =>
      apiClient.get<QualityMetrics>('/api/qingxi/quality/metrics', { params: { exchange } }),
    
    updateThresholds: (thresholds: Partial<QualityMetrics>) =>
      apiClient.post('/api/qingxi/quality/thresholds', thresholds),
    
    getAnomalies: (timeRange?: { start: string; end: string }) =>
      apiClient.get<Array<{ timestamp: string; type: string; severity: string; description: string; affected_symbols: string[] }>>('/api/qingxi/quality/anomalies', { params: timeRange }),
    
    calibrate: () =>
      apiClient.post<{ calibration_completed: boolean; adjustments_made: string[] }>('/api/qingxi/quality/calibrate'),
  },
};