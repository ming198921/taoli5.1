import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import type { 
  MarketDataCollectorConfig, 
  BatchProcessorConfig, 
  CCXTConfig, 
  TimeManagerConfig,
  MemoryManagerConfig,
  SystemLimits 
} from '@/types/qingxi';

// QingXi模块状态类型
interface QingXiState {
  // 数据收集器状态
  collectors: {
    [key: string]: {
      id: string;
      name: string;
      status: 'running' | 'stopped' | 'error' | 'paused';
      config: MarketDataCollectorConfig;
      stats: {
        totalMessages: number;
        messagesPerSecond: number;
        errorCount: number;
        lastUpdate: string;
        uptime: number;
      };
      health: 'healthy' | 'warning' | 'critical';
      lastError?: string;
    };
  };
  
  // 批处理器状态
  batchProcessor: {
    status: 'running' | 'stopped' | 'error' | 'paused';
    config: BatchProcessorConfig;
    currentBatch: {
      id: string | null;
      size: number;
      startTime: string | null;
      estimatedCompletion: string | null;
    };
    stats: {
      totalBatches: number;
      successfulBatches: number;
      failedBatches: number;
      avgProcessingTime: number;
      throughput: number;
    };
    queue: {
      pending: number;
      processing: number;
      completed: number;
      failed: number;
    };
  };
  
  // CCXT适配器状态
  ccxtAdapters: {
    [exchange: string]: {
      status: 'connected' | 'disconnected' | 'error' | 'connecting';
      config: CCXTConfig;
      stats: {
        requestCount: number;
        errorCount: number;
        avgResponseTime: number;
        lastRequest: string | null;
        rateLimitRemaining: number;
        rateLimitReset: string | null;
      };
      supportedSymbols: string[];
      lastError?: string;
    };
  };
  
  // 时间管理器状态
  timeManager: {
    status: 'running' | 'stopped' | 'error';
    config: TimeManagerConfig;
    currentTime: string;
    timezone: string;
    ntp: {
      synchronized: boolean;
      lastSync: string | null;
      offset: number;
      accuracy: number;
    };
    schedule: {
      marketOpen: boolean;
      nextEvent: string | null;
      tradingHours: {
        [market: string]: {
          isOpen: boolean;
          nextOpen: string | null;
          nextClose: string | null;
        };
      };
    };
  };
  
  // 内存管理器状态
  memoryManager: {
    status: 'normal' | 'warning' | 'critical';
    config: MemoryManagerConfig;
    usage: {
      total: number;
      used: number;
      available: number;
      cached: number;
      buffers: number;
    };
    gc: {
      lastRun: string | null;
      frequency: number;
      avgDuration: number;
      objectsCollected: number;
    };
    limits: SystemLimits;
  };
  
  // 第三方数据源状态
  thirdPartyDataSources: {
    [sourceId: string]: {
      id: string;
      name: string;
      type: string;
      status: 'active' | 'inactive' | 'error' | 'rate_limited';
      config: Record<string, any>;
      stats: {
        requestCount: number;
        successCount: number;
        errorCount: number;
        avgResponseTime: number;
        lastRequest: string | null;
        dataQuality: number; // 0-1
      };
      rateLimit: {
        remaining: number;
        reset: string | null;
        limit: number;
      };
      lastError?: string;
    };
  };
  
  // NATS消息队列状态
  natsQueue: {
    status: 'connected' | 'disconnected' | 'reconnecting' | 'error';
    config: {
      servers: string[];
      clusterId: string;
      clientId: string;
    };
    stats: {
      messagesPublished: number;
      messagesReceived: number;
      bytesPublished: number;
      bytesReceived: number;
      subscriptions: number;
      reconnectCount: number;
    };
    health: {
      lastHeartbeat: string | null;
      latency: number;
      pendingMessages: number;
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
const initialState: QingXiState = {
  collectors: {},
  batchProcessor: {
    status: 'stopped',
    config: {} as BatchProcessorConfig,
    currentBatch: {
      id: null,
      size: 0,
      startTime: null,
      estimatedCompletion: null,
    },
    stats: {
      totalBatches: 0,
      successfulBatches: 0,
      failedBatches: 0,
      avgProcessingTime: 0,
      throughput: 0,
    },
    queue: {
      pending: 0,
      processing: 0,
      completed: 0,
      failed: 0,
    },
  },
  ccxtAdapters: {},
  timeManager: {
    status: 'stopped',
    config: {} as TimeManagerConfig,
    currentTime: new Date().toISOString(),
    timezone: 'UTC',
    ntp: {
      synchronized: false,
      lastSync: null,
      offset: 0,
      accuracy: 0,
    },
    schedule: {
      marketOpen: false,
      nextEvent: null,
      tradingHours: {},
    },
  },
  memoryManager: {
    status: 'normal',
    config: {} as MemoryManagerConfig,
    usage: {
      total: 0,
      used: 0,
      available: 0,
      cached: 0,
      buffers: 0,
    },
    gc: {
      lastRun: null,
      frequency: 0,
      avgDuration: 0,
      objectsCollected: 0,
    },
    limits: {} as SystemLimits,
  },
  thirdPartyDataSources: {},
  natsQueue: {
    status: 'disconnected',
    config: {
      servers: [],
      clusterId: '',
      clientId: '',
    },
    stats: {
      messagesPublished: 0,
      messagesReceived: 0,
      bytesPublished: 0,
      bytesReceived: 0,
      subscriptions: 0,
      reconnectCount: 0,
    },
    health: {
      lastHeartbeat: null,
      latency: 0,
      pendingMessages: 0,
    },
  },
  overallStatus: 'critical',
  lastUpdate: null,
  activeAlerts: [],
};

// QingXi slice
const qingxiSlice = createSlice({
  name: 'qingxi',
  initialState,
  reducers: {
    // 数据收集器管理
    updateCollectorStatus: (state, action: PayloadAction<{ id: string; status: QingXiState['collectors'][string]['status'] }>) => {
      const { id, status } = action.payload;
      if (state.collectors[id]) {
        state.collectors[id].status = status;
      }
    },
    
    addCollector: (state, action: PayloadAction<QingXiState['collectors'][string]>) => {
      const collector = action.payload;
      state.collectors[collector.id] = collector;
    },
    
    removeCollector: (state, action: PayloadAction<string>) => {
      delete state.collectors[action.payload];
    },
    
    updateCollectorConfig: (state, action: PayloadAction<{ id: string; config: Partial<MarketDataCollectorConfig> }>) => {
      const { id, config } = action.payload;
      if (state.collectors[id]) {
        state.collectors[id].config = { ...state.collectors[id].config, ...config };
      }
    },
    
    updateCollectorStats: (state, action: PayloadAction<{ id: string; stats: Partial<QingXiState['collectors'][string]['stats']> }>) => {
      const { id, stats } = action.payload;
      if (state.collectors[id]) {
        state.collectors[id].stats = { ...state.collectors[id].stats, ...stats };
        state.collectors[id].stats.lastUpdate = new Date().toISOString();
      }
    },
    
    // 批处理器管理
    updateBatchProcessorStatus: (state, action: PayloadAction<QingXiState['batchProcessor']['status']>) => {
      state.batchProcessor.status = action.payload;
    },
    
    updateBatchProcessorConfig: (state, action: PayloadAction<Partial<BatchProcessorConfig>>) => {
      state.batchProcessor.config = { ...state.batchProcessor.config, ...action.payload };
    },
    
    updateBatchProcessorStats: (state, action: PayloadAction<Partial<QingXiState['batchProcessor']['stats']>>) => {
      state.batchProcessor.stats = { ...state.batchProcessor.stats, ...action.payload };
    },
    
    updateCurrentBatch: (state, action: PayloadAction<Partial<QingXiState['batchProcessor']['currentBatch']>>) => {
      state.batchProcessor.currentBatch = { ...state.batchProcessor.currentBatch, ...action.payload };
    },
    
    updateBatchQueue: (state, action: PayloadAction<Partial<QingXiState['batchProcessor']['queue']>>) => {
      state.batchProcessor.queue = { ...state.batchProcessor.queue, ...action.payload };
    },
    
    // CCXT适配器管理
    addCCXTAdapter: (state, action: PayloadAction<{ exchange: string; adapter: QingXiState['ccxtAdapters'][string] }>) => {
      const { exchange, adapter } = action.payload;
      state.ccxtAdapters[exchange] = adapter;
    },
    
    removeCCXTAdapter: (state, action: PayloadAction<string>) => {
      delete state.ccxtAdapters[action.payload];
    },
    
    updateCCXTAdapterStatus: (state, action: PayloadAction<{ exchange: string; status: QingXiState['ccxtAdapters'][string]['status'] }>) => {
      const { exchange, status } = action.payload;
      if (state.ccxtAdapters[exchange]) {
        state.ccxtAdapters[exchange].status = status;
      }
    },
    
    updateCCXTAdapterStats: (state, action: PayloadAction<{ exchange: string; stats: Partial<QingXiState['ccxtAdapters'][string]['stats']> }>) => {
      const { exchange, stats } = action.payload;
      if (state.ccxtAdapters[exchange]) {
        state.ccxtAdapters[exchange].stats = { ...state.ccxtAdapters[exchange].stats, ...stats };
      }
    },
    
    // 时间管理器管理
    updateTimeManagerStatus: (state, action: PayloadAction<QingXiState['timeManager']['status']>) => {
      state.timeManager.status = action.payload;
    },
    
    updateTimeManagerConfig: (state, action: PayloadAction<Partial<TimeManagerConfig>>) => {
      state.timeManager.config = { ...state.timeManager.config, ...action.payload };
    },
    
    updateCurrentTime: (state, action: PayloadAction<string>) => {
      state.timeManager.currentTime = action.payload;
    },
    
    updateNTPStatus: (state, action: PayloadAction<Partial<QingXiState['timeManager']['ntp']>>) => {
      state.timeManager.ntp = { ...state.timeManager.ntp, ...action.payload };
    },
    
    updateSchedule: (state, action: PayloadAction<Partial<QingXiState['timeManager']['schedule']>>) => {
      state.timeManager.schedule = { ...state.timeManager.schedule, ...action.payload };
    },
    
    // 内存管理器管理
    updateMemoryManagerStatus: (state, action: PayloadAction<QingXiState['memoryManager']['status']>) => {
      state.memoryManager.status = action.payload;
    },
    
    updateMemoryUsage: (state, action: PayloadAction<Partial<QingXiState['memoryManager']['usage']>>) => {
      state.memoryManager.usage = { ...state.memoryManager.usage, ...action.payload };
    },
    
    updateGCStats: (state, action: PayloadAction<Partial<QingXiState['memoryManager']['gc']>>) => {
      state.memoryManager.gc = { ...state.memoryManager.gc, ...action.payload };
    },
    
    // 第三方数据源管理
    addThirdPartyDataSource: (state, action: PayloadAction<QingXiState['thirdPartyDataSources'][string]>) => {
      const source = action.payload;
      state.thirdPartyDataSources[source.id] = source;
    },
    
    removeThirdPartyDataSource: (state, action: PayloadAction<string>) => {
      delete state.thirdPartyDataSources[action.payload];
    },
    
    updateThirdPartyDataSourceStatus: (state, action: PayloadAction<{ id: string; status: QingXiState['thirdPartyDataSources'][string]['status'] }>) => {
      const { id, status } = action.payload;
      if (state.thirdPartyDataSources[id]) {
        state.thirdPartyDataSources[id].status = status;
      }
    },
    
    updateThirdPartyDataSourceStats: (state, action: PayloadAction<{ id: string; stats: Partial<QingXiState['thirdPartyDataSources'][string]['stats']> }>) => {
      const { id, stats } = action.payload;
      if (state.thirdPartyDataSources[id]) {
        state.thirdPartyDataSources[id].stats = { ...state.thirdPartyDataSources[id].stats, ...stats };
      }
    },
    
    // NATS队列管理
    updateNATSQueueStatus: (state, action: PayloadAction<QingXiState['natsQueue']['status']>) => {
      state.natsQueue.status = action.payload;
    },
    
    updateNATSQueueStats: (state, action: PayloadAction<Partial<QingXiState['natsQueue']['stats']>>) => {
      state.natsQueue.stats = { ...state.natsQueue.stats, ...action.payload };
    },
    
    updateNATSQueueHealth: (state, action: PayloadAction<Partial<QingXiState['natsQueue']['health']>>) => {
      state.natsQueue.health = { ...state.natsQueue.health, ...action.payload };
    },
    
    // 整体状态管理
    updateOverallStatus: (state, action: PayloadAction<QingXiState['overallStatus']>) => {
      state.overallStatus = action.payload;
      state.lastUpdate = new Date().toISOString();
    },
    
    // 告警管理
    addAlert: (state, action: PayloadAction<QingXiState['activeAlerts'][number]>) => {
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
    resetQingXiState: () => initialState,
  },
});

// 导出actions
export const {
  updateCollectorStatus,
  addCollector,
  removeCollector,
  updateCollectorConfig,
  updateCollectorStats,
  updateBatchProcessorStatus,
  updateBatchProcessorConfig,
  updateBatchProcessorStats,
  updateCurrentBatch,
  updateBatchQueue,
  addCCXTAdapter,
  removeCCXTAdapter,
  updateCCXTAdapterStatus,
  updateCCXTAdapterStats,
  updateTimeManagerStatus,
  updateTimeManagerConfig,
  updateCurrentTime,
  updateNTPStatus,
  updateSchedule,
  updateMemoryManagerStatus,
  updateMemoryUsage,
  updateGCStats,
  addThirdPartyDataSource,
  removeThirdPartyDataSource,
  updateThirdPartyDataSourceStatus,
  updateThirdPartyDataSourceStats,
  updateNATSQueueStatus,
  updateNATSQueueStats,
  updateNATSQueueHealth,
  updateOverallStatus,
  addAlert,
  removeAlert,
  acknowledgeAlert,
  clearAllAlerts,
  resetQingXiState,
} = qingxiSlice.actions;

// 导出reducer
export default qingxiSlice.reducer;

// Selectors
export const selectQingXiOverallStatus = (state: { qingxi: QingXiState }) => state.qingxi.overallStatus;
export const selectCollectors = (state: { qingxi: QingXiState }) => state.qingxi.collectors;
export const selectBatchProcessor = (state: { qingxi: QingXiState }) => state.qingxi.batchProcessor;
export const selectCCXTAdapters = (state: { qingxi: QingXiState }) => state.qingxi.ccxtAdapters;
export const selectTimeManager = (state: { qingxi: QingXiState }) => state.qingxi.timeManager;
export const selectMemoryManager = (state: { qingxi: QingXiState }) => state.qingxi.memoryManager;
export const selectThirdPartyDataSources = (state: { qingxi: QingXiState }) => state.qingxi.thirdPartyDataSources;
export const selectNATSQueue = (state: { qingxi: QingXiState }) => state.qingxi.natsQueue;
export const selectQingXiAlerts = (state: { qingxi: QingXiState }) => state.qingxi.activeAlerts;
export const selectUnacknowledgedAlerts = (state: { qingxi: QingXiState }) => 
  state.qingxi.activeAlerts.filter(alert => !alert.acknowledged);