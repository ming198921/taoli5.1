// Observability监控追踪API
import { apiClient } from './client';
import type {
  DistributedTracingConfig,
  TraceData,
  MetricDefinition,
  MetricValue,
  MetricQuery,
  AlertRule,
  Alert,
  LogEntry,
  LogQuery,
  VisualizationConfig,
  DashboardPanel,
  DashboardVariable
} from '@/types/observability';

export const observabilityAPI = {
  // 4.1 分布式追踪控制
  tracing: {
    // 追踪配置管理
    getConfig: () =>
      apiClient.get<DistributedTracingConfig>('/api/observability/tracing/config'),
    
    updateConfig: (config: Partial<DistributedTracingConfig>) =>
      apiClient.put<DistributedTracingConfig>('/api/observability/tracing/config', config),
    
    // 启用/禁用追踪
    setEnabled: (enabled: boolean) =>
      apiClient.post('/api/observability/tracing/enabled', { enabled }),
    
    // 追踪数据查询
    getTraces: (query: { 
      service?: string; 
      operation?: string; 
      start_time: string; 
      end_time: string; 
      limit?: number;
      tags?: Record<string, string>;
    }) =>
      apiClient.get<TraceData[]>('/api/observability/tracing/traces', { params: query }),
    
    getTrace: (traceId: string) =>
      apiClient.get<TraceData>(`/api/observability/tracing/traces/${traceId}`),
    
    // 服务依赖图
    getServiceMap: (timeRange?: { start: string; end: string }) =>
      apiClient.get<{
        services: Array<{
          name: string;
          request_count: number;
          error_rate: number;
          avg_duration_ms: number;
          versions: string[];
        }>;
        dependencies: Array<{
          source: string;
          target: string;
          request_count: number;
          error_rate: number;
          avg_latency_ms: number;
        }>;
      }>('/api/observability/tracing/service-map', { params: timeRange }),
    
    // 性能分析
    getServicePerformance: (serviceName: string, timeRange: { start: string; end: string }) =>
      apiClient.get<{
        service: string;
        total_requests: number;
        error_rate: number;
        avg_duration_ms: number;
        p95_duration_ms: number;
        p99_duration_ms: number;
        operations: Array<{
          name: string;
          request_count: number;
          error_rate: number;
          avg_duration_ms: number;
        }>;
        errors: Array<{
          error_type: string;
          count: number;
          last_seen: string;
          sample_trace_id: string;
        }>;
      }>(`/api/observability/tracing/services/${serviceName}/performance`, { params: timeRange }),
    
    // 异常检测
    getAnomalies: (timeRange: { start: string; end: string }) =>
      apiClient.get<Array<{
        timestamp: string;
        service: string;
        operation: string;
        anomaly_type: 'latency_spike' | 'error_burst' | 'throughput_drop';
        severity: 'low' | 'medium' | 'high';
        description: string;
        affected_traces: string[];
        root_cause?: string;
      }>>('/api/observability/tracing/anomalies', { params: timeRange }),
    
    // 手动创建span
    createSpan: (span: {
      trace_id?: string;
      parent_span_id?: string;
      operation_name: string;
      service_name: string;
      tags?: Record<string, any>;
      start_time?: string;
    }) =>
      apiClient.post<{ span_id: string; trace_id: string }>('/api/observability/tracing/spans', span),
    
    // 完成span
    finishSpan: (spanId: string, data: { end_time?: string; tags?: Record<string, any>; logs?: Array<{ timestamp: string; fields: Record<string, any> }> }) =>
      apiClient.put(`/api/observability/tracing/spans/${spanId}/finish`, data),
  },

  // 4.2 指标收集控制
  metrics: {
    // 指标定义管理
    defineMetric: (metric: MetricDefinition) =>
      apiClient.post<{ metric_id: string; status: string }>('/api/observability/metrics/define', metric),
    
    getMetricDefinitions: () =>
      apiClient.get<MetricDefinition[]>('/api/observability/metrics/definitions'),
    
    updateMetricDefinition: (metricName: string, updates: Partial<MetricDefinition>) =>
      apiClient.put(`/api/observability/metrics/definitions/${metricName}`, updates),
    
    deleteMetricDefinition: (metricName: string) =>
      apiClient.delete(`/api/observability/metrics/definitions/${metricName}`),
    
    // 指标数据操作
    recordMetric: (metric: MetricValue) =>
      apiClient.post('/api/observability/metrics/record', metric),
    
    recordBatchMetrics: (metrics: MetricValue[]) =>
      apiClient.post('/api/observability/metrics/batch', { metrics }),
    
    // 指标查询
    queryMetrics: (query: MetricQuery) =>
      apiClient.get<{
        metric_name: string;
        query: string;
        result_type: 'matrix' | 'vector' | 'scalar';
        data: Array<{
          timestamp: string;
          value: number;
          labels?: Record<string, string>;
        }>;
        stats: {
          execution_time_ms: number;
          samples_examined: number;
        };
      }>('/api/observability/metrics/query', { params: query }),
    
    queryRange: (query: MetricQuery & { step: string }) =>
      apiClient.get<{
        metric_name: string;
        result: Array<{
          metric: Record<string, string>;
          values: Array<[string, string]>; // [timestamp, value]
        }>;
        stats: {
          execution_time_ms: number;
          samples_examined: number;
        };
      }>('/api/observability/metrics/query/range', { params: query }),
    
    // 聚合函数
    getAggregation: (metricName: string, aggregation: 'sum' | 'avg' | 'min' | 'max' | 'count', timeRange: { start: string; end: string }, groupBy?: string[]) =>
      apiClient.get<{
        aggregation: string;
        result: number | Record<string, number>;
        time_range: { start: string; end: string };
      }>(`/api/observability/metrics/${metricName}/aggregate`, { 
        params: { aggregation, ...timeRange, group_by: groupBy?.join(',') } 
      }),
    
    // 指标统计
    getMetricStats: (metricName: string, timeRange: { start: string; end: string }) =>
      apiClient.get<{
        metric_name: string;
        total_samples: number;
        unique_series: number;
        cardinality: Record<string, number>; // by label
        data_points_per_minute: number;
        storage_size_bytes: number;
      }>(`/api/observability/metrics/${metricName}/stats`, { params: timeRange }),
    
    // 自定义计算器
    incrementCounter: (name: string, labels?: Record<string, string>, value: number = 1) =>
      apiClient.post('/api/observability/metrics/counter/increment', { name, labels, value }),
    
    setGauge: (name: string, value: number, labels?: Record<string, string>) =>
      apiClient.post('/api/observability/metrics/gauge/set', { name, value, labels }),
    
    recordHistogram: (name: string, value: number, labels?: Record<string, string>) =>
      apiClient.post('/api/observability/metrics/histogram/record', { name, value, labels }),
  },

  // 4.3 告警规则控制
  alerting: {
    // 告警规则管理
    createRule: (rule: Omit<AlertRule, 'id'>) =>
      apiClient.post<{ rule_id: string; status: string }>('/api/observability/alerts/rules/create', rule),
    
    listRules: () =>
      apiClient.get<AlertRule[]>('/api/observability/alerts/rules'),
    
    getRule: (ruleId: string) =>
      apiClient.get<AlertRule>(`/api/observability/alerts/rules/${ruleId}`),
    
    updateRule: (ruleId: string, updates: Partial<AlertRule>) =>
      apiClient.put<AlertRule>(`/api/observability/alerts/rules/${ruleId}`, updates),
    
    deleteRule: (ruleId: string) =>
      apiClient.delete(`/api/observability/alerts/rules/${ruleId}`),
    
    // 告警规则操作
    enableRule: (ruleId: string) =>
      apiClient.post(`/api/observability/alerts/rules/${ruleId}/enable`),
    
    disableRule: (ruleId: string) =>
      apiClient.post(`/api/observability/alerts/rules/${ruleId}/disable`),
    
    testRule: (ruleId: string) =>
      apiClient.post<{
        rule_id: string;
        test_successful: boolean;
        current_value: number;
        threshold_value: number;
        would_trigger: boolean;
        evaluation_time_ms: number;
      }>(`/api/observability/alerts/rules/${ruleId}/test`),
    
    // 活动告警管理
    getActiveAlerts: (filters?: { severity?: string; status?: string; rule_id?: string }) =>
      apiClient.get<Alert[]>('/api/observability/alerts/active', { params: filters }),
    
    getAlert: (alertId: string) =>
      apiClient.get<Alert>(`/api/observability/alerts/${alertId}`),
    
    acknowledgeAlert: (alertId: string, comment?: string) =>
      apiClient.post(`/api/observability/alerts/${alertId}/acknowledge`, { comment }),
    
    resolveAlert: (alertId: string, comment?: string) =>
      apiClient.post(`/api/observability/alerts/${alertId}/resolve`, { comment }),
    
    silenceAlert: (alertId: string, duration_minutes: number, comment?: string) =>
      apiClient.post(`/api/observability/alerts/${alertId}/silence`, { duration_minutes, comment }),
    
    // 告警历史和统计
    getAlertHistory: (timeRange: { start: string; end: string }, filters?: { rule_id?: string; severity?: string }) =>
      apiClient.get<Array<{
        alert_id: string;
        rule_name: string;
        severity: string;
        status: string;
        triggered_at: string;
        resolved_at?: string;
        duration_minutes?: number;
        acknowledged_by?: string;
        resolution_comment?: string;
      }>>('/api/observability/alerts/history', { params: { ...timeRange, ...filters } }),
    
    getAlertStats: (timeRange: { start: string; end: string }) =>
      apiClient.get<{
        total_alerts: number;
        by_severity: Record<string, number>;
        by_status: Record<string, number>;
        avg_resolution_time_minutes: number;
        false_positive_rate: number;
        most_triggered_rules: Array<{ rule_name: string; count: number }>;
        alert_volume_trend: Array<{ date: string; count: number }>;
      }>('/api/observability/alerts/stats', { params: timeRange }),
    
    // 通知渠道管理
    testNotificationChannel: (channel: string, testMessage?: string) =>
      apiClient.post<{ success: boolean; response_time_ms: number; error_message?: string }>(`/api/observability/alerts/notifications/${channel}/test`, { test_message: testMessage }),
    
    getNotificationChannels: () =>
      apiClient.get<Array<{
        name: string;
        type: string;
        enabled: boolean;
        config: Record<string, any>;
        last_used: string;
        success_rate: number;
      }>>('/api/observability/alerts/notifications/channels'),
  },

  // 4.4 日志聚合控制
  logging: {
    // 日志查询
    searchLogs: (query: LogQuery) =>
      apiClient.get<{
        total_hits: number;
        logs: LogEntry[];
        aggregations?: Record<string, any>;
        execution_time_ms: number;
        query_hash: string;
      }>('/api/observability/logs/search', { params: query }),
    
    getLiveStreams: (filters?: { service?: string; level?: string; follow?: boolean }) =>
      apiClient.get<{
        stream_id: string;
        endpoint: string; // WebSocket endpoint for live streaming
        filters_applied: Record<string, any>;
      }>('/api/observability/logs/live', { params: filters }),
    
    // 日志统计分析
    getLogStats: (timeRange: { start: string; end: string }, service?: string) =>
      apiClient.get<{
        total_logs: number;
        by_level: Record<string, number>;
        by_service: Record<string, number>;
        error_rate: number;
        unique_errors: number;
        log_rate_per_minute: number;
      }>('/api/observability/logs/stats', { params: { ...timeRange, service } }),
    
    getLogPatterns: (timeRange: { start: string; end: string }, service?: string) =>
      apiClient.get<Array<{
        pattern: string;
        count: number;
        percentage: number;
        first_seen: string;
        last_seen: string;
        sample_logs: string[];
      }>>('/api/observability/logs/patterns', { params: { ...timeRange, service } }),
    
    // 错误日志分析
    getErrorAnalysis: (timeRange: { start: string; end: string }) =>
      apiClient.get<{
        total_errors: number;
        error_rate: number;
        top_errors: Array<{
          message: string;
          count: number;
          first_seen: string;
          last_seen: string;
          affected_services: string[];
          stack_trace?: string;
        }>;
        error_trends: Array<{
          timestamp: string;
          error_count: number;
          error_rate: number;
        }>;
      }>('/api/observability/logs/errors/analysis', { params: timeRange }),
    
    // 日志导出
    exportLogs: (query: LogQuery & { format: 'json' | 'csv' | 'txt' }) =>
      apiClient.post<{ export_id: string; download_url: string; expires_at: string }>('/api/observability/logs/export', query),
    
    getExportStatus: (exportId: string) =>
      apiClient.get<{ 
        status: 'pending' | 'processing' | 'completed' | 'failed';
        progress_percent?: number;
        download_url?: string;
        error_message?: string;
      }>(`/api/observability/logs/exports/${exportId}/status`),
    
    // 日志保留策略
    getRetentionPolicy: () =>
      apiClient.get<{
        default_retention_days: number;
        by_service: Record<string, number>;
        by_level: Record<string, number>;
        total_storage_gb: number;
        cleanup_schedule: string;
      }>('/api/observability/logs/retention/policy'),
    
    updateRetentionPolicy: (policy: {
      default_retention_days?: number;
      service_policies?: Record<string, number>;
      level_policies?: Record<string, number>;
    }) =>
      apiClient.put('/api/observability/logs/retention/policy', policy),
    
    manualCleanup: (beforeDate: string, confirm: boolean = false) =>
      apiClient.post<{
        cleanup_id: string;
        logs_deleted: number;
        storage_freed_gb: number;
        completion_time_seconds: number;
      }>('/api/observability/logs/retention/cleanup', { before_date: beforeDate, confirm }),
  },

  // 4.5 可视化管理控制
  visualization: {
    // 仪表板管理
    createDashboard: (dashboard: Omit<VisualizationConfig, 'dashboard_id'>) =>
      apiClient.post<{ dashboard_id: string; status: string }>('/api/observability/dashboards/create', dashboard),
    
    listDashboards: (filters?: { tags?: string[]; created_by?: string }) =>
      apiClient.get<Array<{
        dashboard_id: string;
        name: string;
        description: string;
        tags: string[];
        created_by: string;
        created_at: string;
        last_modified: string;
        panel_count: number;
        is_public: boolean;
      }>>('/api/observability/dashboards', { params: filters }),
    
    getDashboard: (dashboardId: string) =>
      apiClient.get<VisualizationConfig>(`/api/observability/dashboards/${dashboardId}`),
    
    updateDashboard: (dashboardId: string, updates: Partial<VisualizationConfig>) =>
      apiClient.put<VisualizationConfig>(`/api/observability/dashboards/${dashboardId}`, updates),
    
    deleteDashboard: (dashboardId: string) =>
      apiClient.delete(`/api/observability/dashboards/${dashboardId}`),
    
    // 面板管理
    addPanel: (dashboardId: string, panel: Omit<DashboardPanel, 'id'>) =>
      apiClient.post<{ panel_id: string }>(`/api/observability/dashboards/${dashboardId}/panels`, panel),
    
    updatePanel: (dashboardId: string, panelId: string, updates: Partial<DashboardPanel>) =>
      apiClient.put(`/api/observability/dashboards/${dashboardId}/panels/${panelId}`, updates),
    
    deletePanel: (dashboardId: string, panelId: string) =>
      apiClient.delete(`/api/observability/dashboards/${dashboardId}/panels/${panelId}`),
    
    // 面板数据查询
    getPanelData: (dashboardId: string, panelId: string, timeRange?: { start: string; end: string }) =>
      apiClient.get<{
        panel_id: string;
        data: Array<{
          target: string;
          datapoints: Array<[number, string]>; // [value, timestamp]
        }>;
        query_time_ms: number;
      }>(`/api/observability/dashboards/${dashboardId}/panels/${panelId}/data`, { params: timeRange }),
    
    refreshPanelData: (dashboardId: string, panelId?: string) =>
      apiClient.post(`/api/observability/dashboards/${dashboardId}/refresh`, { panel_id: panelId }),
    
    // 变量管理
    addVariable: (dashboardId: string, variable: DashboardVariable) =>
      apiClient.post(`/api/observability/dashboards/${dashboardId}/variables`, variable),
    
    updateVariable: (dashboardId: string, variableName: string, updates: Partial<DashboardVariable>) =>
      apiClient.put(`/api/observability/dashboards/${dashboardId}/variables/${variableName}`, updates),
    
    deleteVariable: (dashboardId: string, variableName: string) =>
      apiClient.delete(`/api/observability/dashboards/${dashboardId}/variables/${variableName}`),
    
    // 仪表板分享和导出
    shareDashboard: (dashboardId: string, config: { public: boolean; expires_at?: string; password?: string }) =>
      apiClient.post<{ share_url: string; share_id: string; expires_at?: string }>(`/api/observability/dashboards/${dashboardId}/share`, config),
    
    exportDashboard: (dashboardId: string, format: 'json' | 'pdf' | 'png') =>
      apiClient.post<{ export_id: string; download_url: string; expires_at: string }>(`/api/observability/dashboards/${dashboardId}/export`, { format }),
    
    importDashboard: (dashboardData: any) =>
      apiClient.post<{ dashboard_id: string; imported_panels: number; warnings: string[] }>('/api/observability/dashboards/import', { dashboard_data: dashboardData }),
    
    // 模板管理
    saveDashboardAsTemplate: (dashboardId: string, templateName: string, description?: string) =>
      apiClient.post<{ template_id: string }>(`/api/observability/dashboards/${dashboardId}/save-template`, { 
        template_name: templateName, 
        description 
      }),
    
    listDashboardTemplates: () =>
      apiClient.get<Array<{
        template_id: string;
        name: string;
        description: string;
        created_by: string;
        created_at: string;
        usage_count: number;
        tags: string[];
      }>>('/api/observability/dashboards/templates'),
    
    createFromTemplate: (templateId: string, dashboardName: string) =>
      apiClient.post<{ dashboard_id: string }>('/api/observability/dashboards/create-from-template', {
        template_id: templateId,
        dashboard_name: dashboardName
      }),
  },
};