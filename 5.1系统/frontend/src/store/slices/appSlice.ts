import { createSlice, createAsyncThunk, PayloadAction } from '@reduxjs/toolkit';
import { apiClient } from '@/api/client';

// 应用状态类型
interface AppState {
  isLoading: boolean;
  isAuthenticated: boolean;
  theme: 'light' | 'dark';
  language: 'zh-CN' | 'en-US';
  sidebarCollapsed: boolean;
  systemHealth: {
    overall: 'healthy' | 'warning' | 'error' | 'unknown';
    lastCheck: string | null;
    uptime: number;
    version: string | null;
  };
  globalError: string | null;
  notifications: Notification[];
  connectionStatus: {
    api: 'connected' | 'disconnected' | 'connecting';
    websocket: 'connected' | 'disconnected' | 'connecting';
  };
  performance: {
    memoryUsage: number;
    cpuUsage: number;
    networkLatency: number;
  };
}

interface Notification {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error';
  title: string;
  message: string;
  timestamp: string;
  read: boolean;
  actions?: Array<{
    label: string;
    action: string;
  }>;
}

// 初始状态
const initialState: AppState = {
  isLoading: true,
  isAuthenticated: !!localStorage.getItem('auth_token'),
  theme: (localStorage.getItem('theme') as 'light' | 'dark') || 'light',
  language: (localStorage.getItem('language') as 'zh-CN' | 'en-US') || 'zh-CN',
  sidebarCollapsed: localStorage.getItem('sidebar_collapsed') === 'true',
  systemHealth: {
    overall: 'unknown',
    lastCheck: null,
    uptime: 0,
    version: null,
  },
  globalError: null,
  notifications: [],
  connectionStatus: {
    api: 'disconnected',
    websocket: 'disconnected',
  },
  performance: {
    memoryUsage: 0,
    cpuUsage: 0,
    networkLatency: 0,
  },
};

// 异步thunks
export const initializeApp = createAsyncThunk(
  'app/initialize',
  async (_, { rejectWithValue, dispatch }) => {
    try {
      console.log('🚀 应用初始化中...');
      
      // 执行健康检查获取真实数据
      const healthResult = await dispatch(checkSystemHealth());
      console.log('📋 健康检查结果:', healthResult);
      
      const apiConnected = healthResult.type === 'app/checkSystemHealth/fulfilled';
      console.log('🔗 API连接状态:', apiConnected);
      
      const result = {
        apiConnected,
        version: '5.1.0',
        timestamp: new Date().toISOString(),
      };
      
      console.log('✅ 应用初始化完成:', result);
      return result;
    } catch (error) {
      console.error('❌ 应用初始化失败:', error);
      // 即使失败也要返回基础数据，避免无限loading
      return {
        apiConnected: false,
        version: '5.1.0',
        timestamp: new Date().toISOString(),
      };
    }
  }
);

export const checkSystemHealth = createAsyncThunk(
  'app/checkSystemHealth',
  async (_, { rejectWithValue }) => {
    try {
      console.log('🚀 开始健康检查...');
      
      const [health, systemStatus] = await Promise.all([
        apiClient.healthCheck(),
        apiClient.getSystemStatus()
      ]);
      
      console.log('✅ 健康检查结果:', { health, systemStatus });
      
      const timestamp = new Date().toISOString();
      
      const result = {
        healthy: health,
        timestamp,
        systemStatus,
      };
      
      console.log('📤 返回健康检查数据:', result);
      return result;
    } catch (error) {
      console.error('❌ 健康检查失败:', error);
      return rejectWithValue('健康检查失败');
    }
  }
);

// App slice
const appSlice = createSlice({
  name: 'app',
  initialState,
  reducers: {
    // 认证状态
    setAuthenticated: (state, action: PayloadAction<boolean>) => {
      state.isAuthenticated = action.payload;
      if (!action.payload) {
        localStorage.removeItem('auth_token');
      }
    },
    
    // 主题切换
    toggleTheme: (state) => {
      state.theme = state.theme === 'light' ? 'dark' : 'light';
      localStorage.setItem('theme', state.theme);
    },
    
    setTheme: (state, action: PayloadAction<'light' | 'dark'>) => {
      state.theme = action.payload;
      localStorage.setItem('theme', state.theme);
    },
    
    // 语言设置
    setLanguage: (state, action: PayloadAction<'zh-CN' | 'en-US'>) => {
      state.language = action.payload;
      localStorage.setItem('language', state.language);
    },
    
    // 侧边栏折叠
    toggleSidebar: (state) => {
      state.sidebarCollapsed = !state.sidebarCollapsed;
      localStorage.setItem('sidebar_collapsed', String(state.sidebarCollapsed));
    },
    
    setSidebarCollapsed: (state, action: PayloadAction<boolean>) => {
      state.sidebarCollapsed = action.payload;
      localStorage.setItem('sidebar_collapsed', String(state.sidebarCollapsed));
    },
    
    // 全局错误
    setGlobalError: (state, action: PayloadAction<string | null>) => {
      state.globalError = action.payload;
    },
    
    clearGlobalError: (state) => {
      state.globalError = null;
    },
    
    // 通知管理
    addNotification: (state, action: PayloadAction<Omit<Notification, 'id' | 'timestamp' | 'read'>>) => {
      const notification: Notification = {
        ...action.payload,
        id: `notification_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
        timestamp: new Date().toISOString(),
        read: false,
      };
      state.notifications.unshift(notification);
      
      // 保持最多50个通知
      if (state.notifications.length > 50) {
        state.notifications = state.notifications.slice(0, 50);
      }
    },
    
    markNotificationRead: (state, action: PayloadAction<string>) => {
      const notification = state.notifications.find(n => n.id === action.payload);
      if (notification) {
        notification.read = true;
      }
    },
    
    markAllNotificationsRead: (state) => {
      state.notifications.forEach(notification => {
        notification.read = true;
      });
    },
    
    removeNotification: (state, action: PayloadAction<string>) => {
      state.notifications = state.notifications.filter(n => n.id !== action.payload);
    },
    
    clearAllNotifications: (state) => {
      state.notifications = [];
    },
    
    // 连接状态
    setApiConnectionStatus: (state, action: PayloadAction<'connected' | 'disconnected' | 'connecting'>) => {
      state.connectionStatus.api = action.payload;
    },
    
    setWebSocketConnectionStatus: (state, action: PayloadAction<'connected' | 'disconnected' | 'connecting'>) => {
      state.connectionStatus.websocket = action.payload;
    },
    
    // 性能指标
    updatePerformanceMetrics: (state, action: PayloadAction<Partial<AppState['performance']>>) => {
      state.performance = {
        ...state.performance,
        ...action.payload,
      };
    },
    
    // 系统健康状态
    updateSystemHealth: (state, action: PayloadAction<Partial<AppState['systemHealth']>>) => {
      state.systemHealth = {
        ...state.systemHealth,
        ...action.payload,
      };
    },
  },
  
  extraReducers: (builder) => {
    builder
      // 应用初始化
      .addCase(initializeApp.pending, (state) => {
        state.isLoading = true;
        state.globalError = null;
      })
      .addCase(initializeApp.fulfilled, (state, action) => {
        console.log('initializeApp.fulfilled:', action.payload);
        state.isLoading = false;
        state.connectionStatus.api = action.payload.apiConnected ? 'connected' : 'disconnected';
        state.systemHealth = {
          ...state.systemHealth,
          overall: action.payload.apiConnected ? 'healthy' : 'error',
          lastCheck: action.payload.timestamp,
          version: action.payload.version,
        };
      })
      .addCase(initializeApp.rejected, (state, action) => {
        state.isLoading = false;
        state.globalError = action.payload as string;
        state.connectionStatus.api = 'disconnected';
        state.systemHealth.overall = 'error';
      })
      
      // 系统健康检查
      .addCase(checkSystemHealth.fulfilled, (state, action) => {
        const { healthy, timestamp, systemStatus } = action.payload;
        console.log('📊 系统健康状态更新:', { healthy, timestamp, systemStatus });
        
        // 计算运行时间（秒转小时）
        const uptimeHours = systemStatus?.uptime ? Math.round(systemStatus.uptime / 3600 * 10) / 10 : 0;
        
        state.systemHealth = {
          ...state.systemHealth,
          overall: healthy ? 'healthy' : 'error',
          lastCheck: timestamp,
          uptime: uptimeHours,
          version: systemStatus?.version || '5.1.0',
        };
        state.connectionStatus.api = healthy ? 'connected' : 'disconnected';
        
        // 更新性能数据（如果有的话）
        if (systemStatus) {
          state.performance = {
            memoryUsage: systemStatus.memoryUsage || 0,
            cpuUsage: systemStatus.cpuUsage || 0,
            networkLatency: systemStatus.networkLatency || 0,
          };
        }
        
        console.log('📊 更新后的系统健康状态:', state.systemHealth);
        console.log('🔌 更新后的连接状态:', state.connectionStatus);
      })
      .addCase(checkSystemHealth.rejected, (state) => {
        state.systemHealth.overall = 'error';
        state.connectionStatus.api = 'disconnected';
      });
  },
});

// 导出actions
export const {
  setAuthenticated,
  toggleTheme,
  setTheme,
  setLanguage,
  toggleSidebar,
  setSidebarCollapsed,
  setGlobalError,
  clearGlobalError,
  addNotification,
  markNotificationRead,
  markAllNotificationsRead,
  removeNotification,
  clearAllNotifications,
  setApiConnectionStatus,
  setWebSocketConnectionStatus,
  updatePerformanceMetrics,
  updateSystemHealth,
} = appSlice.actions;

// 导出reducer
export default appSlice.reducer;

// Selectors
export const selectIsAuthenticated = (state: { app: AppState }) => state.app.isAuthenticated;
export const selectTheme = (state: { app: AppState }) => state.app.theme;
export const selectLanguage = (state: { app: AppState }) => state.app.language;
export const selectSidebarCollapsed = (state: { app: AppState }) => state.app.sidebarCollapsed;
export const selectSystemHealth = (state: { app: AppState }) => state.app.systemHealth;
export const selectGlobalError = (state: { app: AppState }) => state.app.globalError;
export const selectNotifications = (state: { app: AppState }) => state.app.notifications;
export const selectUnreadNotificationCount = (state: { app: AppState }) => 
  state.app.notifications.filter(n => !n.read).length;
export const selectConnectionStatus = (state: { app: AppState }) => state.app.connectionStatus;
export const selectPerformanceMetrics = (state: { app: AppState }) => state.app.performance;