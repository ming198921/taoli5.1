// Observability监控追踪相关类型定义

export interface DistributedTracingConfig {
  enabled: boolean;
  sampler_type: 'const' | 'probabilistic' | 'rate_limiting' | 'adaptive';
  sampler_param: number;
  service_name: string;
  service_version: string;
  jaeger_endpoint?: string;
  otlp_endpoint?: string;
  tags: Record<string, string>;
  propagation_formats: ('jaeger' | 'b3' | 'w3c')[];
}

export interface TraceData {
  trace_id: string;
  span_id: string;
  parent_span_id?: string;
  operation_name: string;
  service_name: string;
  start_time: string;
  duration_microseconds: number;
  tags: Record<string, any>;
  logs: TraceLog[];
  status: 'ok' | 'error' | 'timeout';
  baggage?: Record<string, string>;
}

export interface TraceLog {
  timestamp: string;
  level: 'debug' | 'info' | 'warn' | 'error';
  message: string;
  fields: Record<string, any>;
}

export interface MetricDefinition {
  name: string;
  type: 'counter' | 'gauge' | 'histogram' | 'summary';
  description: string;
  unit: string;
  labels: string[];
  namespace: string;
  subsystem?: string;
}

export interface MetricValue {
  metric_name: string;
  value: number;
  timestamp: string;
  labels: Record<string, string>;
  tags?: Record<string, string>;
}

export interface MetricQuery {
  query: string;
  start_time: string;
  end_time: string;
  step?: string;
  timeout?: string;
}

export interface AlertRule {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
  query: string;
  condition: 'gt' | 'gte' | 'lt' | 'lte' | 'eq' | 'neq';
  threshold: number;
  duration_seconds: number;
  severity: 'info' | 'warning' | 'critical';
  labels: Record<string, string>;
  annotations: Record<string, string>;
  notification_channels: string[];
  escalation_policy?: string;
  inhibit_rules?: string[];
}

export interface Alert {
  id: string;
  rule_id: string;
  name: string;
  status: 'firing' | 'pending' | 'resolved';
  severity: 'info' | 'warning' | 'critical';
  message: string;
  labels: Record<string, string>;
  annotations: Record<string, string>;
  starts_at: string;
  ends_at?: string;
  resolved_at?: string;
  generator_url?: string;
  fingerprint: string;
}

export interface LogEntry {
  timestamp: string;
  level: 'trace' | 'debug' | 'info' | 'warn' | 'error' | 'fatal';
  service: string;
  message: string;
  fields: Record<string, any>;
  trace_id?: string;
  span_id?: string;
  request_id?: string;
  user_id?: string;
  session_id?: string;
}

export interface LogQuery {
  query: string;
  start_time: string;
  end_time: string;
  limit?: number;
  direction?: 'forward' | 'backward';
  regexp?: string;
  labels?: Record<string, string>;
}

export interface VisualizationConfig {
  dashboard_id: string;
  name: string;
  description: string;
  tags: string[];
  panels: DashboardPanel[];
  variables: DashboardVariable[];
  time_range: {
    from: string;
    to: string;
  };
  refresh_interval: string;
  auto_refresh: boolean;
}

export interface DashboardPanel {
  id: string;
  title: string;
  type: 'graph' | 'table' | 'stat' | 'gauge' | 'bar' | 'heatmap' | 'text';
  position: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
  targets: PanelTarget[];
  options: PanelOptions;
  field_config: FieldConfig;
}

export interface PanelTarget {
  query: string;
  legend: string;
  format: 'time_series' | 'table' | 'heatmap';
  step?: string;
  interval?: string;
  instant?: boolean;
}

export interface PanelOptions {
  legend: {
    show: boolean;
    position: 'bottom' | 'right' | 'top';
    values: string[];
  };
  tooltip: {
    mode: 'single' | 'multi' | 'none';
    sort: 'none' | 'asc' | 'desc';
  };
  graph: {
    show_lines: boolean;
    show_points: boolean;
    point_radius: number;
    line_width: number;
    fill_opacity: number;
  };
}

export interface FieldConfig {
  defaults: {
    unit: string;
    min?: number;
    max?: number;
    decimals?: number;
    color: ColorConfig;
    mappings: ValueMapping[];
    thresholds: ThresholdConfig;
  };
  overrides: FieldOverride[];
}

export interface ColorConfig {
  mode: 'palette-classic' | 'palette-modern' | 'continuous' | 'single';
  scheme?: string;
  value?: string;
}

export interface ValueMapping {
  type: 'value' | 'range' | 'regex';
  options: Record<string, any>;
}

export interface ThresholdConfig {
  mode: 'absolute' | 'percentage';
  steps: ThresholdStep[];
}

export interface ThresholdStep {
  color: string;
  value: number;
  state?: 'normal' | 'warning' | 'critical';
}

export interface FieldOverride {
  matcher: {
    type: 'byName' | 'byRegex' | 'byType';
    options: string;
  };
  properties: FieldProperty[];
}

export interface FieldProperty {
  id: string;
  value: any;
}

export interface DashboardVariable {
  name: string;
  type: 'query' | 'datasource' | 'interval' | 'custom' | 'constant';
  label: string;
  description: string;
  query?: string;
  options?: VariableOption[];
  multi: boolean;
  include_all: boolean;
  all_value?: string;
  current: VariableOption;
  hide: 'label' | 'variable' | 'nothing';
}

export interface VariableOption {
  text: string;
  value: string;
  selected: boolean;
}