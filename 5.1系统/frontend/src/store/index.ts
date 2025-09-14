import { configureStore } from '@reduxjs/toolkit';
import { setupListeners } from '@reduxjs/toolkit/query';

// Slices
import appSlice from './slices/appSlice';
import authSlice from './slices/authSlice';
import qingxiSlice from './slices/qingxiSlice';
import celueSlice from './slices/celueSlice';
import architectureSlice from './slices/architectureSlice';
import observabilitySlice from './slices/observabilitySlice';

// Note: RTK Query APIs will be implemented as needed

// 配置store
export const store = configureStore({
  reducer: {
    // 常规slices
    app: appSlice,
    auth: authSlice,
    qingxi: qingxiSlice,
    celue: celueSlice,
    architecture: architectureSlice,
    observability: observabilitySlice,
    
    // RTK Query API slices will be added as needed
  },
  
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      // 序列化检查配置
      serializableCheck: {
        ignoredActions: [
          // RTK Query actions
          'persist/PERSIST',
          'persist/REHYDRATE',
          'persist/REGISTER',
        ],
        ignoredPaths: [
          // 忽略复杂对象路径
          'register',
          'rehydrate',
        ],
      },
      // 不可变性检查配置
      immutableCheck: {
        ignoredPaths: ['register', 'rehydrate'],
      },
    }),
  
  // 开发工具配置
  devTools: import.meta.env.DEV && {
    name: '5.1套利系统前端',
    trace: true,
    traceLimit: 25,
    actionSanitizer: (action) => ({
      ...action,
      // 过滤敏感信息
      type: action.type,
    }),
    stateSanitizer: (state) => ({
      ...state,
      // 过滤敏感状态
      auth: state.auth ? { ...state.auth, token: '***' } : state.auth,
    }),
  },
});

// 设置RTK Query listeners
setupListeners(store.dispatch);

// 类型定义
export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;

// 热重载支持
if (import.meta.env.DEV && import.meta.hot) {
  import.meta.hot.accept(['./slices/appSlice'], () => {
    // 重新加载reducer
    const newAppSlice = require('./slices/appSlice').default;
    store.replaceReducer({
      ...store.getState(),
      app: newAppSlice,
    });
  });
}