import { createSlice, createAsyncThunk, PayloadAction } from '@reduxjs/toolkit';
import { apiClient } from '@/api/client';

// 认证状态类型
interface AuthState {
  isLoading: boolean;
  isAuthenticated: boolean;
  user: UserInfo | null;
  token: string | null;
  refreshToken: string | null;
  permissions: string[];
  roles: string[];
  lastLoginTime: string | null;
  sessionExpiry: string | null;
  loginAttempts: number;
  isLocked: boolean;
  lockExpiry: string | null;
  twoFactorRequired: boolean;
  twoFactorToken: string | null;
}

interface UserInfo {
  id: string;
  username: string;
  email: string;
  fullName: string;
  avatar?: string;
  department?: string;
  position?: string;
  phone?: string;
  lastLogin?: string;
  createdAt: string;
  status: 'active' | 'inactive' | 'suspended';
}

interface LoginRequest {
  username: string;
  password: string;
  captcha?: string;
  remember?: boolean;
}

interface TwoFactorRequest {
  token: string;
  code: string;
}

// 初始状态
const initialState: AuthState = {
  isLoading: false,
  isAuthenticated: !!localStorage.getItem('auth_token'),
  user: null,
  token: localStorage.getItem('auth_token'),
  refreshToken: localStorage.getItem('refresh_token'),
  permissions: [],
  roles: [],
  lastLoginTime: localStorage.getItem('last_login_time'),
  sessionExpiry: localStorage.getItem('session_expiry'),
  loginAttempts: 0,
  isLocked: false,
  lockExpiry: null,
  twoFactorRequired: false,
  twoFactorToken: null,
};

// 异步thunks
export const loginUser = createAsyncThunk(
  'auth/login',
  async (loginData: LoginRequest, { rejectWithValue }) => {
    try {
      const response = await apiClient.post<{
        user: UserInfo;
        token: string;
        refreshToken: string;
        permissions: string[];
        roles: string[];
        expiresAt: string;
        twoFactorRequired?: boolean;
        twoFactorToken?: string;
      }>('/api/auth/login', loginData);

      const { user, token, refreshToken, permissions, roles, expiresAt, twoFactorRequired, twoFactorToken } = response;

      if (twoFactorRequired) {
        return {
          twoFactorRequired: true,
          twoFactorToken: twoFactorToken!,
          user: null,
          token: null,
          refreshToken: null,
          permissions: [],
          roles: [],
          expiresAt: null,
        };
      }

      // 存储到localStorage
      localStorage.setItem('auth_token', token);
      localStorage.setItem('refresh_token', refreshToken);
      localStorage.setItem('last_login_time', new Date().toISOString());
      localStorage.setItem('session_expiry', expiresAt);
      localStorage.setItem('user_info', JSON.stringify(user));

      return {
        user,
        token,
        refreshToken,
        permissions,
        roles,
        expiresAt,
        twoFactorRequired: false,
        twoFactorToken: null,
      };
    } catch (error: any) {
      const message = error.response?.data?.message || '登录失败';
      const code = error.response?.data?.code;
      
      if (code === 'ACCOUNT_LOCKED') {
        return rejectWithValue({
          message: '账户已被锁定，请稍后重试',
          locked: true,
          lockExpiry: error.response?.data?.lockExpiry,
        });
      }
      
      return rejectWithValue({ message, locked: false });
    }
  }
);

export const verifyTwoFactor = createAsyncThunk(
  'auth/verifyTwoFactor',
  async (twoFactorData: TwoFactorRequest, { rejectWithValue }) => {
    try {
      const response = await apiClient.post<{
        user: UserInfo;
        token: string;
        refreshToken: string;
        permissions: string[];
        roles: string[];
        expiresAt: string;
      }>('/api/auth/verify-2fa', twoFactorData);

      const { user, token, refreshToken, permissions, roles, expiresAt } = response;

      // 存储到localStorage
      localStorage.setItem('auth_token', token);
      localStorage.setItem('refresh_token', refreshToken);
      localStorage.setItem('last_login_time', new Date().toISOString());
      localStorage.setItem('session_expiry', expiresAt);
      localStorage.setItem('user_info', JSON.stringify(user));

      return {
        user,
        token,
        refreshToken,
        permissions,
        roles,
        expiresAt,
      };
    } catch (error: any) {
      return rejectWithValue(error.response?.data?.message || '二次验证失败');
    }
  }
);

export const logoutUser = createAsyncThunk(
  'auth/logout',
  async (_, { getState, rejectWithValue }) => {
    try {
      const state = getState() as { auth: AuthState };
      if (state.auth.token) {
        await apiClient.post('/api/auth/logout');
      }
      
      // 清除localStorage
      localStorage.removeItem('auth_token');
      localStorage.removeItem('refresh_token');
      localStorage.removeItem('last_login_time');
      localStorage.removeItem('session_expiry');
      localStorage.removeItem('user_info');
      
      return null;
    } catch (error) {
      // 即使登出接口失败也要清除本地数据
      localStorage.removeItem('auth_token');
      localStorage.removeItem('refresh_token');
      localStorage.removeItem('last_login_time');
      localStorage.removeItem('session_expiry');
      localStorage.removeItem('user_info');
      
      return null;
    }
  }
);

export const refreshToken = createAsyncThunk(
  'auth/refreshToken',
  async (_, { getState, rejectWithValue }) => {
    try {
      const state = getState() as { auth: AuthState };
      const refreshToken = state.auth.refreshToken;
      
      if (!refreshToken) {
        throw new Error('No refresh token available');
      }

      const response = await apiClient.post<{
        token: string;
        refreshToken: string;
        expiresAt: string;
      }>('/api/auth/refresh', { refreshToken });

      const { token, refreshToken: newRefreshToken, expiresAt } = response;

      // 更新localStorage
      localStorage.setItem('auth_token', token);
      localStorage.setItem('refresh_token', newRefreshToken);
      localStorage.setItem('session_expiry', expiresAt);

      return {
        token,
        refreshToken: newRefreshToken,
        expiresAt,
      };
    } catch (error: any) {
      // Token刷新失败，清除认证状态
      localStorage.removeItem('auth_token');
      localStorage.removeItem('refresh_token');
      localStorage.removeItem('session_expiry');
      localStorage.removeItem('user_info');
      
      return rejectWithValue('Token refresh failed');
    }
  }
);

export const getCurrentUser = createAsyncThunk(
  'auth/getCurrentUser',
  async (_, { rejectWithValue }) => {
    try {
      const response = await apiClient.get<{
        user: UserInfo;
        permissions: string[];
        roles: string[];
      }>('/api/auth/me');

      return response;
    } catch (error: any) {
      return rejectWithValue(error.response?.data?.message || '获取用户信息失败');
    }
  }
);

export const changePassword = createAsyncThunk(
  'auth/changePassword',
  async (passwordData: { currentPassword: string; newPassword: string }, { rejectWithValue }) => {
    try {
      await apiClient.post('/api/auth/change-password', passwordData);
      return null;
    } catch (error: any) {
      return rejectWithValue(error.response?.data?.message || '密码修改失败');
    }
  }
);

// Auth slice
const authSlice = createSlice({
  name: 'auth',
  initialState,
  reducers: {
    // 清除错误状态
    clearAuthError: (state) => {
      state.isLoading = false;
    },
    
    // 设置用户信息
    setUser: (state, action: PayloadAction<UserInfo>) => {
      state.user = action.payload;
      localStorage.setItem('user_info', JSON.stringify(action.payload));
    },
    
    // 更新权限
    updatePermissions: (state, action: PayloadAction<string[]>) => {
      state.permissions = action.payload;
    },
    
    // 更新角色
    updateRoles: (state, action: PayloadAction<string[]>) => {
      state.roles = action.payload;
    },
    
    // 重置登录尝试次数
    resetLoginAttempts: (state) => {
      state.loginAttempts = 0;
      state.isLocked = false;
      state.lockExpiry = null;
    },
    
    // 清除二次验证状态
    clearTwoFactorState: (state) => {
      state.twoFactorRequired = false;
      state.twoFactorToken = null;
    },
    
    // 检查会话过期
    checkSessionExpiry: (state) => {
      const now = new Date().getTime();
      const expiry = state.sessionExpiry ? new Date(state.sessionExpiry).getTime() : 0;
      
      if (expiry > 0 && now >= expiry) {
        // 会话已过期，清除认证状态
        state.isAuthenticated = false;
        state.user = null;
        state.token = null;
        state.refreshToken = null;
        state.permissions = [];
        state.roles = [];
        state.sessionExpiry = null;
        
        localStorage.removeItem('auth_token');
        localStorage.removeItem('refresh_token');
        localStorage.removeItem('session_expiry');
        localStorage.removeItem('user_info');
      }
    },
  },
  
  extraReducers: (builder) => {
    builder
      // 用户登录
      .addCase(loginUser.pending, (state) => {
        state.isLoading = true;
        state.loginAttempts += 1;
      })
      .addCase(loginUser.fulfilled, (state, action) => {
        state.isLoading = false;
        
        if (action.payload.twoFactorRequired) {
          state.twoFactorRequired = true;
          state.twoFactorToken = action.payload.twoFactorToken;
        } else {
          state.isAuthenticated = true;
          state.user = action.payload.user;
          state.token = action.payload.token;
          state.refreshToken = action.payload.refreshToken;
          state.permissions = action.payload.permissions;
          state.roles = action.payload.roles;
          state.sessionExpiry = action.payload.expiresAt;
          state.lastLoginTime = new Date().toISOString();
          state.loginAttempts = 0;
          state.isLocked = false;
          state.lockExpiry = null;
          state.twoFactorRequired = false;
          state.twoFactorToken = null;
        }
      })
      .addCase(loginUser.rejected, (state, action: any) => {
        state.isLoading = false;
        
        if (action.payload?.locked) {
          state.isLocked = true;
          state.lockExpiry = action.payload.lockExpiry;
        }
      })
      
      // 二次验证
      .addCase(verifyTwoFactor.pending, (state) => {
        state.isLoading = true;
      })
      .addCase(verifyTwoFactor.fulfilled, (state, action) => {
        state.isLoading = false;
        state.isAuthenticated = true;
        state.user = action.payload.user;
        state.token = action.payload.token;
        state.refreshToken = action.payload.refreshToken;
        state.permissions = action.payload.permissions;
        state.roles = action.payload.roles;
        state.sessionExpiry = action.payload.expiresAt;
        state.lastLoginTime = new Date().toISOString();
        state.twoFactorRequired = false;
        state.twoFactorToken = null;
        state.loginAttempts = 0;
      })
      .addCase(verifyTwoFactor.rejected, (state) => {
        state.isLoading = false;
      })
      
      // 用户登出
      .addCase(logoutUser.fulfilled, (state) => {
        state.isAuthenticated = false;
        state.user = null;
        state.token = null;
        state.refreshToken = null;
        state.permissions = [];
        state.roles = [];
        state.lastLoginTime = null;
        state.sessionExpiry = null;
        state.loginAttempts = 0;
        state.isLocked = false;
        state.lockExpiry = null;
        state.twoFactorRequired = false;
        state.twoFactorToken = null;
      })
      
      // Token刷新
      .addCase(refreshToken.fulfilled, (state, action) => {
        state.token = action.payload.token;
        state.refreshToken = action.payload.refreshToken;
        state.sessionExpiry = action.payload.expiresAt;
      })
      .addCase(refreshToken.rejected, (state) => {
        state.isAuthenticated = false;
        state.user = null;
        state.token = null;
        state.refreshToken = null;
        state.permissions = [];
        state.roles = [];
        state.sessionExpiry = null;
      })
      
      // 获取当前用户
      .addCase(getCurrentUser.fulfilled, (state, action) => {
        state.user = action.payload.user;
        state.permissions = action.payload.permissions;
        state.roles = action.payload.roles;
      })
      .addCase(getCurrentUser.rejected, (state) => {
        // 获取用户信息失败可能意味着token无效
        state.isAuthenticated = false;
        state.user = null;
        state.token = null;
        state.refreshToken = null;
        state.permissions = [];
        state.roles = [];
      })
      
      // 修改密码
      .addCase(changePassword.pending, (state) => {
        state.isLoading = true;
      })
      .addCase(changePassword.fulfilled, (state) => {
        state.isLoading = false;
      })
      .addCase(changePassword.rejected, (state) => {
        state.isLoading = false;
      });
  },
});

// 导出actions
export const {
  clearAuthError,
  setUser,
  updatePermissions,
  updateRoles,
  resetLoginAttempts,
  clearTwoFactorState,
  checkSessionExpiry,
} = authSlice.actions;

// 导出reducer
export default authSlice.reducer;

// Selectors
export const selectIsAuthenticated = (state: { auth: AuthState }) => state.auth.isAuthenticated;
export const selectCurrentUser = (state: { auth: AuthState }) => state.auth.user;
export const selectUserPermissions = (state: { auth: AuthState }) => state.auth.permissions;
export const selectUserRoles = (state: { auth: AuthState }) => state.auth.roles;
export const selectAuthLoading = (state: { auth: AuthState }) => state.auth.isLoading;
export const selectIsLocked = (state: { auth: AuthState }) => state.auth.isLocked;
export const selectTwoFactorRequired = (state: { auth: AuthState }) => state.auth.twoFactorRequired;
export const selectSessionExpiry = (state: { auth: AuthState }) => state.auth.sessionExpiry;