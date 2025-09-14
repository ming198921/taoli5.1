import { apiCall, HttpMethod } from '../api/apiClient';

// 配置服务相关类型定义
export interface Configuration {
  key: string;
  value: any;
  type: 'string' | 'number' | 'boolean' | 'object' | 'array';
  description?: string;
  metadata?: any;
  created_at: string;
  updated_at: string;
}

export interface ConfigVersion {
  version: string;
  name: string;
  description?: string;
  changes: any[];
  created_at: string;
  deployed: boolean;
}

export interface HotReloadStatus {
  enabled: boolean;
  status: 'idle' | 'reloading' | 'success' | 'failed';
  last_reload: string;
  services: Record<string, any>;
}

/**
 * 配置服务 - 96个API接口
 * 端口: 4007
 * 功能: 配置管理、版本控制、热重载、环境管理
 */
export class ConfigService {
  
  // ==================== 基础配置管理API (24个) ====================
  
  /**
   * 列出所有配置
   */
  async listConfigs(): Promise<Configuration[]> {
    return apiCall(HttpMethod.GET, '/config/list');
  }
  
  /**
   * 获取配置值
   */
  async getConfig(key: string): Promise<Configuration> {
    return apiCall(HttpMethod.GET, `/config/${key}`);
  }
  
  /**
   * 设置配置值
   */
  async setConfig(key: string, value: any): Promise<void> {
    return apiCall(HttpMethod.PUT, `/config/${key}`, { value });
  }
  
  /**
   * 删除配置项
   */
  async deleteConfig(key: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/config/${key}`);
  }
  
  /**
   * 获取配置元数据
   */
  async getConfigMetadata(key: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/${key}/metadata`);
  }
  
  /**
   * 获取配置历史
   */
  async getConfigHistory(key: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/config/${key}/history`);
  }
  
  /**
   * 批量获取配置
   */
  async batchGetConfigs(keys: string[]): Promise<Configuration[]> {
    return apiCall(HttpMethod.POST, '/config/batch/get', { keys });
  }
  
  /**
   * 批量设置配置
   */
  async batchSetConfigs(configs: Record<string, any>): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/batch/set', { configs });
  }
  
  /**
   * 批量删除配置
   */
  async batchDeleteConfigs(keys: string[]): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/batch/delete', { keys });
  }
  
  /**
   * 搜索配置
   */
  async searchConfigs(query: string): Promise<Configuration[]> {
    return apiCall(HttpMethod.POST, '/config/search', { query });
  }
  
  /**
   * 获取配置树
   */
  async getConfigTree(): Promise<any> {
    return apiCall(HttpMethod.GET, '/config/tree');
  }
  
  /**
   * 获取配置子树
   */
  async getConfigSubTree(path: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/tree/${path}`);
  }
  
  /**
   * 导出配置
   */
  async exportConfigs(format: string = 'json'): Promise<any> {
    return apiCall(HttpMethod.POST, '/config/export', { format });
  }
  
  /**
   * 导入配置
   */
  async importConfigs(configs: any): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/import', { configs });
  }
  
  /**
   * 验证配置
   */
  async validateConfig(config: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/config/validate', { config });
  }
  
  /**
   * 获取配置模式
   */
  async getConfigSchema(): Promise<any> {
    return apiCall(HttpMethod.GET, '/config/schema');
  }
  
  /**
   * 更新配置模式
   */
  async updateConfigSchema(schema: any): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/schema', { schema });
  }
  
  /**
   * 获取默认配置
   */
  async getDefaultConfigs(): Promise<any> {
    return apiCall(HttpMethod.GET, '/config/defaults');
  }
  
  /**
   * 设置默认配置
   */
  async setDefaultConfigs(defaults: any): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/defaults', { defaults });
  }
  
  /**
   * 配置差异比较
   */
  async compareConfigs(config1: any, config2: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/config/diff', { config1, config2 });
  }
  
  /**
   * 合并配置
   */
  async mergeConfigs(configs: any[]): Promise<any> {
    return apiCall(HttpMethod.POST, '/config/merge', { configs });
  }
  
  /**
   * 备份配置
   */
  async backupConfigs(): Promise<{ backup_id: string }> {
    return apiCall(HttpMethod.POST, '/config/backup');
  }
  
  /**
   * 恢复配置
   */
  async restoreConfigs(backup_id: string): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/restore', { backup_id });
  }
  
  /**
   * 获取配置统计
   */
  async getConfigStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/config/stats');
  }
  
  // ==================== 版本控制API (24个) ====================
  
  /**
   * 列出所有版本
   */
  async listVersions(): Promise<ConfigVersion[]> {
    return apiCall(HttpMethod.GET, '/config/versions');
  }
  
  /**
   * 创建新版本
   */
  async createVersion(name: string): Promise<ConfigVersion> {
    return apiCall(HttpMethod.POST, '/config/versions', { name });
  }
  
  /**
   * 获取版本详情
   */
  async getVersion(version: string): Promise<ConfigVersion> {
    return apiCall(HttpMethod.GET, `/config/versions/${version}`);
  }
  
  /**
   * 删除版本
   */
  async deleteVersion(version: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/config/versions/${version}`);
  }
  
  /**
   * 部署版本
   */
  async deployVersion(version: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/versions/${version}/deploy`);
  }
  
  /**
   * 回滚版本
   */
  async rollbackVersion(version: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/versions/${version}/rollback`);
  }
  
  /**
   * 比较版本
   */
  async compareVersions(version1: string, version2: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/versions/${version1}/compare/${version2}`);
  }
  
  /**
   * 获取版本变更
   */
  async getVersionChanges(version: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/config/versions/${version}/changes`);
  }
  
  /**
   * 获取当前版本
   */
  async getCurrentVersion(): Promise<ConfigVersion> {
    return apiCall(HttpMethod.GET, '/config/versions/current');
  }
  
  /**
   * 获取最新版本
   */
  async getLatestVersion(): Promise<ConfigVersion> {
    return apiCall(HttpMethod.GET, '/config/versions/latest');
  }
  
  /**
   * 验证版本
   */
  async validateVersion(version: string): Promise<any> {
    return apiCall(HttpMethod.POST, `/config/versions/${version}/validate`);
  }
  
  /**
   * 检查冲突
   */
  async checkVersionConflicts(version: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/config/versions/${version}/conflicts`);
  }
  
  /**
   * 创建分支
   */
  async createBranch(name: string): Promise<any> {
    return apiCall(HttpMethod.POST, '/config/versions/branch', { name });
  }
  
  /**
   * 合并版本
   */
  async mergeVersions(from: string, to: string): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/versions/merge', { from, to });
  }
  
  /**
   * 标记版本
   */
  async tagVersion(version: string, tag: string): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/versions/tag', { version, tag });
  }
  
  /**
   * 列出标签
   */
  async listTags(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/config/versions/tags');
  }
  
  /**
   * 获取标签版本
   */
  async getTaggedVersion(tag: string): Promise<ConfigVersion> {
    return apiCall(HttpMethod.GET, `/config/versions/tags/${tag}`);
  }
  
  /**
   * 锁定版本
   */
  async lockVersion(version: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/versions/${version}/lock`);
  }
  
  /**
   * 解锁版本
   */
  async unlockVersion(version: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/versions/${version}/unlock`);
  }
  
  /**
   * 克隆版本
   */
  async cloneVersion(version: string): Promise<ConfigVersion> {
    return apiCall(HttpMethod.POST, `/config/versions/${version}/clone`);
  }
  
  /**
   * 垃圾回收版本
   */
  async gcVersions(): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/versions/gc');
  }
  
  /**
   * 获取版本审计
   */
  async getVersionAudit(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/config/versions/audit');
  }
  
  /**
   * 获取版本权限
   */
  async getVersionPermissions(): Promise<any> {
    return apiCall(HttpMethod.GET, '/config/versions/permissions');
  }
  
  /**
   * 设置版本权限
   */
  async setVersionPermissions(permissions: any): Promise<void> {
    return apiCall(HttpMethod.PUT, '/config/versions/permissions', permissions);
  }
  
  // ==================== 热重载API (18个) ====================
  
  /**
   * 获取重载状态
   */
  async getHotReloadStatus(): Promise<HotReloadStatus> {
    return apiCall(HttpMethod.GET, '/config/hot-reload/status');
  }
  
  /**
   * 启用热重载
   */
  async enableHotReload(): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/hot-reload/enable');
  }
  
  /**
   * 禁用热重载
   */
  async disableHotReload(): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/hot-reload/disable');
  }
  
  /**
   * 触发重载
   */
  async triggerHotReload(): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/hot-reload/trigger');
  }
  
  /**
   * 验证重载
   */
  async validateHotReload(config: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/config/hot-reload/validate', { config });
  }
  
  /**
   * 预览重载
   */
  async previewHotReload(config: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/config/hot-reload/preview', { config });
  }
  
  /**
   * 回滚重载
   */
  async rollbackHotReload(): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/hot-reload/rollback');
  }
  
  /**
   * 获取重载历史
   */
  async getHotReloadHistory(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/config/hot-reload/history');
  }
  
  /**
   * 列出重载服务
   */
  async listHotReloadServices(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/config/hot-reload/services');
  }
  
  /**
   * 获取服务重载状态
   */
  async getServiceHotReloadStatus(service: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/hot-reload/services/${service}`);
  }
  
  /**
   * 触发服务重载
   */
  async triggerServiceHotReload(service: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/hot-reload/services/${service}/trigger`);
  }
  
  /**
   * 批量重载
   */
  async batchHotReload(services: string[]): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/hot-reload/batch', { services });
  }
  
  /**
   * 计划重载
   */
  async scheduleHotReload(time: string): Promise<{ id: string }> {
    return apiCall(HttpMethod.POST, '/config/hot-reload/schedule', { time });
  }
  
  /**
   * 获取计划重载
   */
  async getScheduledHotReload(id: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/hot-reload/schedule/${id}`);
  }
  
  /**
   * 取消计划重载
   */
  async cancelScheduledHotReload(id: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/config/hot-reload/schedule/${id}`);
  }
  
  /**
   * 列出重载钩子
   */
  async listHotReloadHooks(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/config/hot-reload/hooks');
  }
  
  /**
   * 添加重载钩子
   */
  async addHotReloadHook(hook: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/config/hot-reload/hooks', { hook });
  }
  
  /**
   * 删除重载钩子
   */
  async deleteHotReloadHook(id: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/config/hot-reload/hooks/${id}`);
  }
  
  // ==================== 环境管理API (30个) ====================
  
  /**
   * 列出所有环境
   */
  async listEnvironments(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/config/environments');
  }
  
  /**
   * 创建新环境
   */
  async createEnvironment(name: string): Promise<any> {
    return apiCall(HttpMethod.POST, '/config/environments', { name });
  }
  
  /**
   * 获取环境详情
   */
  async getEnvironment(env: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}`);
  }
  
  /**
   * 更新环境
   */
  async updateEnvironment(env: string, updates: any): Promise<void> {
    return apiCall(HttpMethod.PUT, `/config/environments/${env}`, updates);
  }
  
  /**
   * 删除环境
   */
  async deleteEnvironment(env: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/config/environments/${env}`);
  }
  
  /**
   * 获取环境配置
   */
  async getEnvironmentConfig(env: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/config`);
  }
  
  /**
   * 设置环境配置
   */
  async setEnvironmentConfig(env: string, config: any): Promise<void> {
    return apiCall(HttpMethod.PUT, `/config/environments/${env}/config`, config);
  }
  
  /**
   * 激活环境
   */
  async activateEnvironment(env: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/activate`);
  }
  
  /**
   * 停用环境
   */
  async deactivateEnvironment(env: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/deactivate`);
  }
  
  /**
   * 获取环境状态
   */
  async getEnvironmentStatus(env: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/status`);
  }
  
  /**
   * 克隆环境
   */
  async cloneEnvironment(env: string, name: string): Promise<any> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/clone`, { name });
  }
  
  /**
   * 同步环境
   */
  async syncEnvironment(env: string, target: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/sync`, { target });
  }
  
  /**
   * 比较环境
   */
  async compareEnvironments(env: string, other: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/diff/${other}`);
  }
  
  /**
   * 提升环境
   */
  async promoteEnvironment(env: string, target: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/promote`, { target });
  }
  
  /**
   * 获取环境变量
   */
  async getEnvironmentVariables(env: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/variables`);
  }
  
  /**
   * 设置环境变量
   */
  async setEnvironmentVariables(env: string, variables: any): Promise<void> {
    return apiCall(HttpMethod.PUT, `/config/environments/${env}/variables`, variables);
  }
  
  /**
   * 获取环境密钥
   */
  async getEnvironmentSecrets(env: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/secrets`);
  }
  
  /**
   * 设置环境密钥
   */
  async setEnvironmentSecrets(env: string, secrets: any): Promise<void> {
    return apiCall(HttpMethod.PUT, `/config/environments/${env}/secrets`, secrets);
  }
  
  /**
   * 获取环境权限
   */
  async getEnvironmentPermissions(env: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/permissions`);
  }
  
  /**
   * 设置环境权限
   */
  async setEnvironmentPermissions(env: string, permissions: any): Promise<void> {
    return apiCall(HttpMethod.PUT, `/config/environments/${env}/permissions`, permissions);
  }
  
  /**
   * 获取环境审计
   */
  async getEnvironmentAudit(env: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/audit`);
  }
  
  /**
   * 备份环境
   */
  async backupEnvironment(env: string): Promise<{ backup_id: string }> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/backup`);
  }
  
  /**
   * 恢复环境
   */
  async restoreEnvironment(env: string, backup_id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/restore`, { backup_id });
  }
  
  /**
   * 环境健康检查
   */
  async checkEnvironmentHealth(env: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/health`);
  }
  
  /**
   * 验证环境
   */
  async validateEnvironment(env: string): Promise<any> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/validate`);
  }
  
  /**
   * 获取环境指标
   */
  async getEnvironmentMetrics(env: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/metrics`);
  }
  
  /**
   * 重置环境
   */
  async resetEnvironment(env: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/reset`);
  }
  
  /**
   * 获取环境模板
   */
  async getEnvironmentTemplates(env: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/templates`);
  }
  
  /**
   * 应用环境模板
   */
  async applyEnvironmentTemplate(env: string, template: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/apply-template`, { template });
  }
  
  /**
   * 获取当前环境
   */
  async getCurrentEnvironment(): Promise<any> {
    return apiCall(HttpMethod.GET, '/config/environments/current');
  }
}

// 导出单例实例
export const configService = new ConfigService(); 

// 配置服务相关类型定义
export interface Configuration {
  key: string;
  value: any;
  type: 'string' | 'number' | 'boolean' | 'object' | 'array';
  description?: string;
  metadata?: any;
  created_at: string;
  updated_at: string;
}

export interface ConfigVersion {
  version: string;
  name: string;
  description?: string;
  changes: any[];
  created_at: string;
  deployed: boolean;
}

export interface HotReloadStatus {
  enabled: boolean;
  status: 'idle' | 'reloading' | 'success' | 'failed';
  last_reload: string;
  services: Record<string, any>;
}

/**
 * 配置服务 - 96个API接口
 * 端口: 4007
 * 功能: 配置管理、版本控制、热重载、环境管理
 */
export class ConfigService {
  
  // ==================== 基础配置管理API (24个) ====================
  
  /**
   * 列出所有配置
   */
  async listConfigs(): Promise<Configuration[]> {
    return apiCall(HttpMethod.GET, '/config/list');
  }
  
  /**
   * 获取配置值
   */
  async getConfig(key: string): Promise<Configuration> {
    return apiCall(HttpMethod.GET, `/config/${key}`);
  }
  
  /**
   * 设置配置值
   */
  async setConfig(key: string, value: any): Promise<void> {
    return apiCall(HttpMethod.PUT, `/config/${key}`, { value });
  }
  
  /**
   * 删除配置项
   */
  async deleteConfig(key: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/config/${key}`);
  }
  
  /**
   * 获取配置元数据
   */
  async getConfigMetadata(key: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/${key}/metadata`);
  }
  
  /**
   * 获取配置历史
   */
  async getConfigHistory(key: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/config/${key}/history`);
  }
  
  /**
   * 批量获取配置
   */
  async batchGetConfigs(keys: string[]): Promise<Configuration[]> {
    return apiCall(HttpMethod.POST, '/config/batch/get', { keys });
  }
  
  /**
   * 批量设置配置
   */
  async batchSetConfigs(configs: Record<string, any>): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/batch/set', { configs });
  }
  
  /**
   * 批量删除配置
   */
  async batchDeleteConfigs(keys: string[]): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/batch/delete', { keys });
  }
  
  /**
   * 搜索配置
   */
  async searchConfigs(query: string): Promise<Configuration[]> {
    return apiCall(HttpMethod.POST, '/config/search', { query });
  }
  
  /**
   * 获取配置树
   */
  async getConfigTree(): Promise<any> {
    return apiCall(HttpMethod.GET, '/config/tree');
  }
  
  /**
   * 获取配置子树
   */
  async getConfigSubTree(path: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/tree/${path}`);
  }
  
  /**
   * 导出配置
   */
  async exportConfigs(format: string = 'json'): Promise<any> {
    return apiCall(HttpMethod.POST, '/config/export', { format });
  }
  
  /**
   * 导入配置
   */
  async importConfigs(configs: any): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/import', { configs });
  }
  
  /**
   * 验证配置
   */
  async validateConfig(config: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/config/validate', { config });
  }
  
  /**
   * 获取配置模式
   */
  async getConfigSchema(): Promise<any> {
    return apiCall(HttpMethod.GET, '/config/schema');
  }
  
  /**
   * 更新配置模式
   */
  async updateConfigSchema(schema: any): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/schema', { schema });
  }
  
  /**
   * 获取默认配置
   */
  async getDefaultConfigs(): Promise<any> {
    return apiCall(HttpMethod.GET, '/config/defaults');
  }
  
  /**
   * 设置默认配置
   */
  async setDefaultConfigs(defaults: any): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/defaults', { defaults });
  }
  
  /**
   * 配置差异比较
   */
  async compareConfigs(config1: any, config2: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/config/diff', { config1, config2 });
  }
  
  /**
   * 合并配置
   */
  async mergeConfigs(configs: any[]): Promise<any> {
    return apiCall(HttpMethod.POST, '/config/merge', { configs });
  }
  
  /**
   * 备份配置
   */
  async backupConfigs(): Promise<{ backup_id: string }> {
    return apiCall(HttpMethod.POST, '/config/backup');
  }
  
  /**
   * 恢复配置
   */
  async restoreConfigs(backup_id: string): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/restore', { backup_id });
  }
  
  /**
   * 获取配置统计
   */
  async getConfigStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/config/stats');
  }
  
  // ==================== 版本控制API (24个) ====================
  
  /**
   * 列出所有版本
   */
  async listVersions(): Promise<ConfigVersion[]> {
    return apiCall(HttpMethod.GET, '/config/versions');
  }
  
  /**
   * 创建新版本
   */
  async createVersion(name: string): Promise<ConfigVersion> {
    return apiCall(HttpMethod.POST, '/config/versions', { name });
  }
  
  /**
   * 获取版本详情
   */
  async getVersion(version: string): Promise<ConfigVersion> {
    return apiCall(HttpMethod.GET, `/config/versions/${version}`);
  }
  
  /**
   * 删除版本
   */
  async deleteVersion(version: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/config/versions/${version}`);
  }
  
  /**
   * 部署版本
   */
  async deployVersion(version: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/versions/${version}/deploy`);
  }
  
  /**
   * 回滚版本
   */
  async rollbackVersion(version: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/versions/${version}/rollback`);
  }
  
  /**
   * 比较版本
   */
  async compareVersions(version1: string, version2: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/versions/${version1}/compare/${version2}`);
  }
  
  /**
   * 获取版本变更
   */
  async getVersionChanges(version: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/config/versions/${version}/changes`);
  }
  
  /**
   * 获取当前版本
   */
  async getCurrentVersion(): Promise<ConfigVersion> {
    return apiCall(HttpMethod.GET, '/config/versions/current');
  }
  
  /**
   * 获取最新版本
   */
  async getLatestVersion(): Promise<ConfigVersion> {
    return apiCall(HttpMethod.GET, '/config/versions/latest');
  }
  
  /**
   * 验证版本
   */
  async validateVersion(version: string): Promise<any> {
    return apiCall(HttpMethod.POST, `/config/versions/${version}/validate`);
  }
  
  /**
   * 检查冲突
   */
  async checkVersionConflicts(version: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/config/versions/${version}/conflicts`);
  }
  
  /**
   * 创建分支
   */
  async createBranch(name: string): Promise<any> {
    return apiCall(HttpMethod.POST, '/config/versions/branch', { name });
  }
  
  /**
   * 合并版本
   */
  async mergeVersions(from: string, to: string): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/versions/merge', { from, to });
  }
  
  /**
   * 标记版本
   */
  async tagVersion(version: string, tag: string): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/versions/tag', { version, tag });
  }
  
  /**
   * 列出标签
   */
  async listTags(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/config/versions/tags');
  }
  
  /**
   * 获取标签版本
   */
  async getTaggedVersion(tag: string): Promise<ConfigVersion> {
    return apiCall(HttpMethod.GET, `/config/versions/tags/${tag}`);
  }
  
  /**
   * 锁定版本
   */
  async lockVersion(version: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/versions/${version}/lock`);
  }
  
  /**
   * 解锁版本
   */
  async unlockVersion(version: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/versions/${version}/unlock`);
  }
  
  /**
   * 克隆版本
   */
  async cloneVersion(version: string): Promise<ConfigVersion> {
    return apiCall(HttpMethod.POST, `/config/versions/${version}/clone`);
  }
  
  /**
   * 垃圾回收版本
   */
  async gcVersions(): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/versions/gc');
  }
  
  /**
   * 获取版本审计
   */
  async getVersionAudit(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/config/versions/audit');
  }
  
  /**
   * 获取版本权限
   */
  async getVersionPermissions(): Promise<any> {
    return apiCall(HttpMethod.GET, '/config/versions/permissions');
  }
  
  /**
   * 设置版本权限
   */
  async setVersionPermissions(permissions: any): Promise<void> {
    return apiCall(HttpMethod.PUT, '/config/versions/permissions', permissions);
  }
  
  // ==================== 热重载API (18个) ====================
  
  /**
   * 获取重载状态
   */
  async getHotReloadStatus(): Promise<HotReloadStatus> {
    return apiCall(HttpMethod.GET, '/config/hot-reload/status');
  }
  
  /**
   * 启用热重载
   */
  async enableHotReload(): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/hot-reload/enable');
  }
  
  /**
   * 禁用热重载
   */
  async disableHotReload(): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/hot-reload/disable');
  }
  
  /**
   * 触发重载
   */
  async triggerHotReload(): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/hot-reload/trigger');
  }
  
  /**
   * 验证重载
   */
  async validateHotReload(config: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/config/hot-reload/validate', { config });
  }
  
  /**
   * 预览重载
   */
  async previewHotReload(config: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/config/hot-reload/preview', { config });
  }
  
  /**
   * 回滚重载
   */
  async rollbackHotReload(): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/hot-reload/rollback');
  }
  
  /**
   * 获取重载历史
   */
  async getHotReloadHistory(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/config/hot-reload/history');
  }
  
  /**
   * 列出重载服务
   */
  async listHotReloadServices(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/config/hot-reload/services');
  }
  
  /**
   * 获取服务重载状态
   */
  async getServiceHotReloadStatus(service: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/hot-reload/services/${service}`);
  }
  
  /**
   * 触发服务重载
   */
  async triggerServiceHotReload(service: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/hot-reload/services/${service}/trigger`);
  }
  
  /**
   * 批量重载
   */
  async batchHotReload(services: string[]): Promise<void> {
    return apiCall(HttpMethod.POST, '/config/hot-reload/batch', { services });
  }
  
  /**
   * 计划重载
   */
  async scheduleHotReload(time: string): Promise<{ id: string }> {
    return apiCall(HttpMethod.POST, '/config/hot-reload/schedule', { time });
  }
  
  /**
   * 获取计划重载
   */
  async getScheduledHotReload(id: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/hot-reload/schedule/${id}`);
  }
  
  /**
   * 取消计划重载
   */
  async cancelScheduledHotReload(id: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/config/hot-reload/schedule/${id}`);
  }
  
  /**
   * 列出重载钩子
   */
  async listHotReloadHooks(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/config/hot-reload/hooks');
  }
  
  /**
   * 添加重载钩子
   */
  async addHotReloadHook(hook: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/config/hot-reload/hooks', { hook });
  }
  
  /**
   * 删除重载钩子
   */
  async deleteHotReloadHook(id: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/config/hot-reload/hooks/${id}`);
  }
  
  // ==================== 环境管理API (30个) ====================
  
  /**
   * 列出所有环境
   */
  async listEnvironments(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/config/environments');
  }
  
  /**
   * 创建新环境
   */
  async createEnvironment(name: string): Promise<any> {
    return apiCall(HttpMethod.POST, '/config/environments', { name });
  }
  
  /**
   * 获取环境详情
   */
  async getEnvironment(env: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}`);
  }
  
  /**
   * 更新环境
   */
  async updateEnvironment(env: string, updates: any): Promise<void> {
    return apiCall(HttpMethod.PUT, `/config/environments/${env}`, updates);
  }
  
  /**
   * 删除环境
   */
  async deleteEnvironment(env: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/config/environments/${env}`);
  }
  
  /**
   * 获取环境配置
   */
  async getEnvironmentConfig(env: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/config`);
  }
  
  /**
   * 设置环境配置
   */
  async setEnvironmentConfig(env: string, config: any): Promise<void> {
    return apiCall(HttpMethod.PUT, `/config/environments/${env}/config`, config);
  }
  
  /**
   * 激活环境
   */
  async activateEnvironment(env: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/activate`);
  }
  
  /**
   * 停用环境
   */
  async deactivateEnvironment(env: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/deactivate`);
  }
  
  /**
   * 获取环境状态
   */
  async getEnvironmentStatus(env: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/status`);
  }
  
  /**
   * 克隆环境
   */
  async cloneEnvironment(env: string, name: string): Promise<any> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/clone`, { name });
  }
  
  /**
   * 同步环境
   */
  async syncEnvironment(env: string, target: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/sync`, { target });
  }
  
  /**
   * 比较环境
   */
  async compareEnvironments(env: string, other: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/diff/${other}`);
  }
  
  /**
   * 提升环境
   */
  async promoteEnvironment(env: string, target: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/promote`, { target });
  }
  
  /**
   * 获取环境变量
   */
  async getEnvironmentVariables(env: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/variables`);
  }
  
  /**
   * 设置环境变量
   */
  async setEnvironmentVariables(env: string, variables: any): Promise<void> {
    return apiCall(HttpMethod.PUT, `/config/environments/${env}/variables`, variables);
  }
  
  /**
   * 获取环境密钥
   */
  async getEnvironmentSecrets(env: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/secrets`);
  }
  
  /**
   * 设置环境密钥
   */
  async setEnvironmentSecrets(env: string, secrets: any): Promise<void> {
    return apiCall(HttpMethod.PUT, `/config/environments/${env}/secrets`, secrets);
  }
  
  /**
   * 获取环境权限
   */
  async getEnvironmentPermissions(env: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/permissions`);
  }
  
  /**
   * 设置环境权限
   */
  async setEnvironmentPermissions(env: string, permissions: any): Promise<void> {
    return apiCall(HttpMethod.PUT, `/config/environments/${env}/permissions`, permissions);
  }
  
  /**
   * 获取环境审计
   */
  async getEnvironmentAudit(env: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/audit`);
  }
  
  /**
   * 备份环境
   */
  async backupEnvironment(env: string): Promise<{ backup_id: string }> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/backup`);
  }
  
  /**
   * 恢复环境
   */
  async restoreEnvironment(env: string, backup_id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/restore`, { backup_id });
  }
  
  /**
   * 环境健康检查
   */
  async checkEnvironmentHealth(env: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/health`);
  }
  
  /**
   * 验证环境
   */
  async validateEnvironment(env: string): Promise<any> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/validate`);
  }
  
  /**
   * 获取环境指标
   */
  async getEnvironmentMetrics(env: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/metrics`);
  }
  
  /**
   * 重置环境
   */
  async resetEnvironment(env: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/reset`);
  }
  
  /**
   * 获取环境模板
   */
  async getEnvironmentTemplates(env: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/config/environments/${env}/templates`);
  }
  
  /**
   * 应用环境模板
   */
  async applyEnvironmentTemplate(env: string, template: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/config/environments/${env}/apply-template`, { template });
  }
  
  /**
   * 获取当前环境
   */
  async getCurrentEnvironment(): Promise<any> {
    return apiCall(HttpMethod.GET, '/config/environments/current');
  }
}

// 导出单例实例
export const configService = new ConfigService(); 