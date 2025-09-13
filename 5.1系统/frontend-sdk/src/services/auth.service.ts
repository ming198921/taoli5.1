/**
 * 认证服务 - 完整的用户认证和权限管理
 */

import { HttpClient } from '../core/http-client';
import {
  LoginRequest,
  LoginResponse,
  UserInfo,
  RefreshTokenRequest,
  ChangePasswordRequest,
  CreateUserRequest,
  UserRole,
  ApiResponse,
  PaginationQuery,
  PaginatedResponse,
} from '../types';

export class AuthService {
  constructor(private httpClient: HttpClient) {}

  /**
   * 用户登录
   */
  public async login(request: LoginRequest): Promise<LoginResponse> {
    const response = await this.httpClient.post<LoginResponse>('/api/auth/login', request);
    
    if (response.success && response.data) {
      // 存储令牌
      this.httpClient.setAuthToken(response.data.token);
      this.httpClient.storeRefreshToken(response.data.refresh_token);
      
      return response.data;
    }
    
    throw new Error(response.message || '登录失败');
  }

  /**
   * 用户登出
   */
  public async logout(): Promise<void> {
    try {
      await this.httpClient.post('/api/auth/logout');
    } finally {
      // 无论成功失败都清理本地令牌
      this.httpClient.clearAuthToken();
      this.httpClient.clearStoredTokens();
    }
  }

  /**
   * 刷新令牌
   */
  public async refreshToken(request: RefreshTokenRequest): Promise<LoginResponse> {
    const response = await this.httpClient.post<LoginResponse>('/api/auth/refresh', request);
    
    if (response.success && response.data) {
      this.httpClient.setAuthToken(response.data.token);
      return response.data;
    }
    
    throw new Error(response.message || '令牌刷新失败');
  }

  /**
   * 获取当前用户信息
   */
  public async getCurrentUser(): Promise<UserInfo> {
    const response = await this.httpClient.get<UserInfo>('/api/auth/me');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取用户信息失败');
  }

  /**
   * 修改密码
   */
  public async changePassword(request: ChangePasswordRequest): Promise<void> {
    const response = await this.httpClient.post('/api/auth/change-password', request);
    
    if (!response.success) {
      throw new Error(response.message || '密码修改失败');
    }
  }

  /**
   * 验证令牌有效性
   */
  public async validateToken(token: string): Promise<boolean> {
    try {
      const response = await this.httpClient.post<{ valid: boolean }>('/api/auth/validate', {
        token,
      });
      
      return response.success && response.data?.valid === true;
    } catch {
      return false;
    }
  }

  /**
   * 获取用户权限列表
   */
  public async getUserPermissions(userId?: string): Promise<string[]> {
    const url = userId ? `/api/auth/users/${userId}/permissions` : '/api/auth/permissions';
    const response = await this.httpClient.get<string[]>(url);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取权限失败');
  }

  /**
   * 检查用户是否有特定权限
   */
  public async hasPermission(permission: string): Promise<boolean> {
    try {
      const permissions = await this.getUserPermissions();
      return permissions.includes(permission) || permissions.includes('*');
    } catch {
      return false;
    }
  }

  /**
   * 创建用户（管理员功能）
   */
  public async createUser(request: CreateUserRequest): Promise<UserInfo> {
    const response = await this.httpClient.post<UserInfo>('/api/auth/users', request);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '用户创建失败');
  }

  /**
   * 获取用户列表（管理员功能）
   */
  public async getUsers(query?: PaginationQuery): Promise<PaginatedResponse<UserInfo>> {
    const response = await this.httpClient.get<PaginatedResponse<UserInfo>>('/api/auth/users', {
      params: query,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取用户列表失败');
  }

  /**
   * 获取用户详情（管理员功能）
   */
  public async getUser(userId: string): Promise<UserInfo> {
    const response = await this.httpClient.get<UserInfo>(`/api/auth/users/${userId}`);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取用户详情失败');
  }

  /**
   * 更新用户信息（管理员功能）
   */
  public async updateUser(
    userId: string, 
    updates: Partial<Omit<UserInfo, 'id' | 'created_at'>>
  ): Promise<UserInfo> {
    const response = await this.httpClient.put<UserInfo>(`/api/auth/users/${userId}`, updates);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '用户更新失败');
  }

  /**
   * 删除用户（管理员功能）
   */
  public async deleteUser(userId: string): Promise<void> {
    const response = await this.httpClient.delete(`/api/auth/users/${userId}`);
    
    if (!response.success) {
      throw new Error(response.message || '用户删除失败');
    }
  }

  /**
   * 启用/禁用用户（管理员功能）
   */
  public async setUserStatus(userId: string, active: boolean): Promise<void> {
    const response = await this.httpClient.put(`/api/auth/users/${userId}/status`, {
      active,
    });
    
    if (!response.success) {
      throw new Error(response.message || '用户状态更新失败');
    }
  }

  /**
   * 重置用户密码（管理员功能）
   */
  public async resetUserPassword(userId: string, newPassword: string): Promise<void> {
    const response = await this.httpClient.post(`/api/auth/users/${userId}/reset-password`, {
      new_password: newPassword,
    });
    
    if (!response.success) {
      throw new Error(response.message || '密码重置失败');
    }
  }

  /**
   * 获取用户角色列表
   */
  public getUserRoles(): UserRole[] {
    return Object.values(UserRole);
  }

  /**
   * 获取角色权限映射
   */
  public async getRolePermissions(): Promise<Record<UserRole, string[]>> {
    const response = await this.httpClient.get<Record<UserRole, string[]>>('/api/auth/role-permissions');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取角色权限失败');
  }

  /**
   * 检查用户是否有管理员权限
   */
  public async isAdmin(): Promise<boolean> {
    try {
      const user = await this.getCurrentUser();
      return [UserRole.SuperAdmin, UserRole.Admin].includes(user.role);
    } catch {
      return false;
    }
  }

  /**
   * 自动登录（使用存储的令牌）
   */
  public async autoLogin(): Promise<UserInfo | null> {
    const token = this.httpClient.getStoredAuthToken();
    
    if (!token) {
      return null;
    }

    try {
      // 验证令牌
      const isValid = await this.validateToken(token);
      if (!isValid) {
        this.httpClient.clearStoredTokens();
        return null;
      }

      // 设置令牌并获取用户信息
      this.httpClient.setAuthToken(token);
      return await this.getCurrentUser();
    } catch {
      this.httpClient.clearStoredTokens();
      return null;
    }
  }

  /**
   * 获取会话信息
   */
  public async getSessionInfo(): Promise<{
    user: UserInfo;
    loginTime: string;
    expiresAt: string;
    permissions: string[];
  }> {
    const response = await this.httpClient.get<{
      user: UserInfo;
      loginTime: string;
      expiresAt: string;
      permissions: string[];
    }>('/api/auth/session');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取会话信息失败');
  }

  /**
   * 获取登录历史（管理员功能）
   */
  public async getLoginHistory(
    userId?: string, 
    query?: PaginationQuery
  ): Promise<PaginatedResponse<{
    id: string;
    userId: string;
    username: string;
    loginTime: string;
    ipAddress: string;
    userAgent: string;
    success: boolean;
  }>> {
    const url = userId ? `/api/auth/users/${userId}/login-history` : '/api/auth/login-history';
    const response = await this.httpClient.get(url, { params: query });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取登录历史失败');
  }
}