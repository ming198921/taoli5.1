import { apiCall, HttpMethod } from '../api/apiClient';

// 清洗服务相关类型定义
export interface CleaningRule {
  id: string;
  name: string;
  enabled: boolean;
  pattern: string;
  action: string;
  priority: number;
  created_at: string;
  updated_at: string;
}

export interface CleaningConfig {
  rules: CleaningRule[];
  exchanges: Record<string, any>;
  quality: {
    score: number;
    threshold: number;
  };
}

export interface ExchangeConfig {
  id: string;
  name: string;
  enabled: boolean;
  symbols: string[];
  rules: any[];
  status: 'active' | 'inactive';
  metrics: any;
}

export interface QualityMetrics {
  score: number;
  issues: any[];
  trends: any;
  benchmarks: any;
  statistics: any;
}

/**
 * 清洗服务 - 52个API接口
 * 端口: 4002
 * 功能: 市场数据清洗、规范化、质量控制
 */
export class CleaningService {
  
  // ==================== 清洗规则管理API (20个) ====================
  
  /**
   * 列出所有清洗规则
   */
  async listCleaningRules(): Promise<CleaningRule[]> {
    return apiCall(HttpMethod.GET, '/cleaning/rules/list');
  }
  
  /**
   * 创建新的清洗规则
   */
  async createCleaningRule(name: string, pattern: string, action: string): Promise<CleaningRule> {
    return apiCall(HttpMethod.POST, '/cleaning/rules/create', { name, pattern, action });
  }
  
  /**
   * 获取特定规则详情
   */
  async getCleaningRule(id: string): Promise<CleaningRule> {
    return apiCall(HttpMethod.GET, `/cleaning/rules/${id}`);
  }
  
  /**
   * 更新清洗规则
   */
  async updateCleaningRule(id: string, updates: Partial<CleaningRule>): Promise<CleaningRule> {
    return apiCall(HttpMethod.PUT, `/cleaning/rules/${id}`, updates);
  }
  
  /**
   * 删除清洗规则
   */
  async deleteCleaningRule(id: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/cleaning/rules/${id}`);
  }
  
  /**
   * 启用清洗规则
   */
  async enableCleaningRule(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/cleaning/rules/${id}/enable`);
  }
  
  /**
   * 禁用清洗规则
   */
  async disableCleaningRule(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/cleaning/rules/${id}/disable`);
  }
  
  /**
   * 测试清洗规则
   */
  async testCleaningRule(rule: any, data: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/cleaning/rules/test', { rule, data });
  }
  
  /**
   * 验证清洗规则
   */
  async validateCleaningRule(rule: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/cleaning/rules/validate', { rule });
  }
  
  /**
   * 导出清洗规则
   */
  async exportCleaningRules(): Promise<CleaningRule[]> {
    return apiCall(HttpMethod.GET, '/cleaning/rules/export');
  }
  
  /**
   * 导入清洗规则
   */
  async importCleaningRules(rules: CleaningRule[]): Promise<void> {
    return apiCall(HttpMethod.POST, '/cleaning/rules/import', { rules });
  }
  
  /**
   * 获取规则模板
   */
  async getRuleTemplates(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/cleaning/rules/templates');
  }
  
  /**
   * 从模板创建规则
   */
  async createRuleFromTemplate(template: string): Promise<CleaningRule> {
    return apiCall(HttpMethod.POST, `/cleaning/rules/templates/${template}`);
  }
  
  /**
   * 搜索清洗规则
   */
  async searchCleaningRules(query: string): Promise<CleaningRule[]> {
    return apiCall(HttpMethod.POST, '/cleaning/rules/search', { query });
  }
  
  /**
   * 批量启用规则
   */
  async batchEnableRules(ids: string[]): Promise<void> {
    return apiCall(HttpMethod.POST, '/cleaning/rules/batch/enable', { ids });
  }
  
  /**
   * 批量禁用规则
   */
  async batchDisableRules(ids: string[]): Promise<void> {
    return apiCall(HttpMethod.POST, '/cleaning/rules/batch/disable', { ids });
  }
  
  /**
   * 批量删除规则
   */
  async batchDeleteRules(ids: string[]): Promise<void> {
    return apiCall(HttpMethod.POST, '/cleaning/rules/batch/delete', { ids });
  }
  
  /**
   * 获取规则历史
   */
  async getRuleHistory(id: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/cleaning/rules/history/${id}`);
  }
  
  /**
   * 获取规则统计
   */
  async getRuleStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/cleaning/rules/stats');
  }
  
  /**
   * 获取规则依赖
   */
  async getRuleDependencies(id: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/cleaning/rules/dependencies/${id}`);
  }
  
  // ==================== 交易所配置API (16个) ====================
  
  /**
   * 列出所有交易所
   */
  async listExchanges(): Promise<ExchangeConfig[]> {
    return apiCall(HttpMethod.GET, '/cleaning/exchanges');
  }
  
  /**
   * 获取交易所配置
   */
  async getExchangeConfig(id: string): Promise<ExchangeConfig> {
    return apiCall(HttpMethod.GET, `/cleaning/exchanges/${id}/config`);
  }
  
  /**
   * 更新交易所配置
   */
  async updateExchangeConfig(id: string, config: Partial<ExchangeConfig>): Promise<void> {
    return apiCall(HttpMethod.PUT, `/cleaning/exchanges/${id}/config`, config);
  }
  
  /**
   * 获取交易所状态
   */
  async getExchangeStatus(id: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/cleaning/exchanges/${id}/status`);
  }
  
  /**
   * 启用交易所
   */
  async enableExchange(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/cleaning/exchanges/${id}/enable`);
  }
  
  /**
   * 禁用交易所
   */
  async disableExchange(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/cleaning/exchanges/${id}/disable`);
  }
  
  /**
   * 获取交易对列表
   */
  async getExchangeSymbols(id: string): Promise<string[]> {
    return apiCall(HttpMethod.GET, `/cleaning/exchanges/${id}/symbols`);
  }
  
  /**
   * 更新交易对配置
   */
  async updateExchangeSymbols(id: string, symbols: string[]): Promise<void> {
    return apiCall(HttpMethod.PUT, `/cleaning/exchanges/${id}/symbols`, { symbols });
  }
  
  /**
   * 获取交易所规则
   */
  async getExchangeRules(id: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/cleaning/exchanges/${id}/rules`);
  }
  
  /**
   * 更新交易所规则
   */
  async updateExchangeRules(id: string, rules: any[]): Promise<void> {
    return apiCall(HttpMethod.PUT, `/cleaning/exchanges/${id}/rules`, { rules });
  }
  
  /**
   * 测试交易所连接
   */
  async testExchangeConnection(id: string): Promise<any> {
    return apiCall(HttpMethod.POST, `/cleaning/exchanges/${id}/test`);
  }
  
  /**
   * 获取交易所指标
   */
  async getExchangeMetrics(id: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/cleaning/exchanges/${id}/metrics`);
  }
  
  /**
   * 重置交易所配置
   */
  async resetExchangeConfig(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/cleaning/exchanges/${id}/reset`);
  }
  
  /**
   * 获取配置历史
   */
  async getExchangeConfigHistory(id: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/cleaning/exchanges/${id}/history`);
  }
  
  /**
   * 批量更新配置
   */
  async batchUpdateExchangeConfig(updates: Record<string, any>): Promise<void> {
    return apiCall(HttpMethod.POST, '/cleaning/exchanges/batch/update', updates);
  }
  
  /**
   * 获取配置模板
   */
  async getExchangeConfigTemplates(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/cleaning/exchanges/templates');
  }
  
  // ==================== 数据质量API (16个) ====================
  
  /**
   * 获取数据质量分数
   */
  async getQualityScore(): Promise<{ score: number }> {
    return apiCall(HttpMethod.GET, '/cleaning/quality/score');
  }
  
  /**
   * 获取质量指标
   */
  async getQualityMetrics(): Promise<QualityMetrics> {
    return apiCall(HttpMethod.GET, '/cleaning/quality/metrics');
  }
  
  /**
   * 获取质量问题
   */
  async getQualityIssues(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/cleaning/quality/issues');
  }
  
  /**
   * 解决质量问题
   */
  async resolveQualityIssue(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/cleaning/quality/issues/${id}/resolve`);
  }
  
  /**
   * 获取质量趋势
   */
  async getQualityTrends(): Promise<any> {
    return apiCall(HttpMethod.GET, '/cleaning/quality/trends');
  }
  
  /**
   * 分析数据质量
   */
  async analyzeDataQuality(data: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/cleaning/quality/analyze', { data });
  }
  
  /**
   * 获取质量报告
   */
  async getQualityReports(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/cleaning/quality/reports');
  }
  
  /**
   * 生成质量报告
   */
  async generateQualityReport(period: string = '24h'): Promise<any> {
    return apiCall(HttpMethod.POST, '/cleaning/quality/reports', { period });
  }
  
  /**
   * 获取质量基准
   */
  async getQualityBenchmarks(): Promise<any> {
    return apiCall(HttpMethod.GET, '/cleaning/quality/benchmarks');
  }
  
  /**
   * 设置质量基准
   */
  async setQualityBenchmarks(score: number): Promise<void> {
    return apiCall(HttpMethod.PUT, '/cleaning/quality/benchmarks', { score });
  }
  
  /**
   * 获取质量告警
   */
  async getQualityAlerts(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/cleaning/quality/alerts');
  }
  
  /**
   * 创建质量告警
   */
  async createQualityAlert(threshold: number): Promise<any> {
    return apiCall(HttpMethod.POST, '/cleaning/quality/alerts', { threshold });
  }
  
  /**
   * 数据验证结果
   */
  async getValidationResults(): Promise<any> {
    return apiCall(HttpMethod.GET, '/cleaning/quality/validation');
  }
  
  /**
   * 执行数据验证
   */
  async executeDataValidation(rules: any[]): Promise<any> {
    return apiCall(HttpMethod.POST, '/cleaning/quality/validation', { rules });
  }
  
  /**
   * 获取质量统计
   */
  async getQualityStatistics(): Promise<any> {
    return apiCall(HttpMethod.GET, '/cleaning/quality/statistics');
  }
  
  /**
   * 优化数据质量
   */
  async optimizeDataQuality(): Promise<any> {
    return apiCall(HttpMethod.POST, '/cleaning/quality/optimize');
  }
}

// 导出单例实例
export const cleaningService = new CleaningService(); 

// 清洗服务相关类型定义
export interface CleaningRule {
  id: string;
  name: string;
  enabled: boolean;
  pattern: string;
  action: string;
  priority: number;
  created_at: string;
  updated_at: string;
}

export interface CleaningConfig {
  rules: CleaningRule[];
  exchanges: Record<string, any>;
  quality: {
    score: number;
    threshold: number;
  };
}

export interface ExchangeConfig {
  id: string;
  name: string;
  enabled: boolean;
  symbols: string[];
  rules: any[];
  status: 'active' | 'inactive';
  metrics: any;
}

export interface QualityMetrics {
  score: number;
  issues: any[];
  trends: any;
  benchmarks: any;
  statistics: any;
}

/**
 * 清洗服务 - 52个API接口
 * 端口: 4002
 * 功能: 市场数据清洗、规范化、质量控制
 */
export class CleaningService {
  
  // ==================== 清洗规则管理API (20个) ====================
  
  /**
   * 列出所有清洗规则
   */
  async listCleaningRules(): Promise<CleaningRule[]> {
    return apiCall(HttpMethod.GET, '/cleaning/rules/list');
  }
  
  /**
   * 创建新的清洗规则
   */
  async createCleaningRule(name: string, pattern: string, action: string): Promise<CleaningRule> {
    return apiCall(HttpMethod.POST, '/cleaning/rules/create', { name, pattern, action });
  }
  
  /**
   * 获取特定规则详情
   */
  async getCleaningRule(id: string): Promise<CleaningRule> {
    return apiCall(HttpMethod.GET, `/cleaning/rules/${id}`);
  }
  
  /**
   * 更新清洗规则
   */
  async updateCleaningRule(id: string, updates: Partial<CleaningRule>): Promise<CleaningRule> {
    return apiCall(HttpMethod.PUT, `/cleaning/rules/${id}`, updates);
  }
  
  /**
   * 删除清洗规则
   */
  async deleteCleaningRule(id: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/cleaning/rules/${id}`);
  }
  
  /**
   * 启用清洗规则
   */
  async enableCleaningRule(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/cleaning/rules/${id}/enable`);
  }
  
  /**
   * 禁用清洗规则
   */
  async disableCleaningRule(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/cleaning/rules/${id}/disable`);
  }
  
  /**
   * 测试清洗规则
   */
  async testCleaningRule(rule: any, data: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/cleaning/rules/test', { rule, data });
  }
  
  /**
   * 验证清洗规则
   */
  async validateCleaningRule(rule: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/cleaning/rules/validate', { rule });
  }
  
  /**
   * 导出清洗规则
   */
  async exportCleaningRules(): Promise<CleaningRule[]> {
    return apiCall(HttpMethod.GET, '/cleaning/rules/export');
  }
  
  /**
   * 导入清洗规则
   */
  async importCleaningRules(rules: CleaningRule[]): Promise<void> {
    return apiCall(HttpMethod.POST, '/cleaning/rules/import', { rules });
  }
  
  /**
   * 获取规则模板
   */
  async getRuleTemplates(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/cleaning/rules/templates');
  }
  
  /**
   * 从模板创建规则
   */
  async createRuleFromTemplate(template: string): Promise<CleaningRule> {
    return apiCall(HttpMethod.POST, `/cleaning/rules/templates/${template}`);
  }
  
  /**
   * 搜索清洗规则
   */
  async searchCleaningRules(query: string): Promise<CleaningRule[]> {
    return apiCall(HttpMethod.POST, '/cleaning/rules/search', { query });
  }
  
  /**
   * 批量启用规则
   */
  async batchEnableRules(ids: string[]): Promise<void> {
    return apiCall(HttpMethod.POST, '/cleaning/rules/batch/enable', { ids });
  }
  
  /**
   * 批量禁用规则
   */
  async batchDisableRules(ids: string[]): Promise<void> {
    return apiCall(HttpMethod.POST, '/cleaning/rules/batch/disable', { ids });
  }
  
  /**
   * 批量删除规则
   */
  async batchDeleteRules(ids: string[]): Promise<void> {
    return apiCall(HttpMethod.POST, '/cleaning/rules/batch/delete', { ids });
  }
  
  /**
   * 获取规则历史
   */
  async getRuleHistory(id: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/cleaning/rules/history/${id}`);
  }
  
  /**
   * 获取规则统计
   */
  async getRuleStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/cleaning/rules/stats');
  }
  
  /**
   * 获取规则依赖
   */
  async getRuleDependencies(id: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/cleaning/rules/dependencies/${id}`);
  }
  
  // ==================== 交易所配置API (16个) ====================
  
  /**
   * 列出所有交易所
   */
  async listExchanges(): Promise<ExchangeConfig[]> {
    return apiCall(HttpMethod.GET, '/cleaning/exchanges');
  }
  
  /**
   * 获取交易所配置
   */
  async getExchangeConfig(id: string): Promise<ExchangeConfig> {
    return apiCall(HttpMethod.GET, `/cleaning/exchanges/${id}/config`);
  }
  
  /**
   * 更新交易所配置
   */
  async updateExchangeConfig(id: string, config: Partial<ExchangeConfig>): Promise<void> {
    return apiCall(HttpMethod.PUT, `/cleaning/exchanges/${id}/config`, config);
  }
  
  /**
   * 获取交易所状态
   */
  async getExchangeStatus(id: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/cleaning/exchanges/${id}/status`);
  }
  
  /**
   * 启用交易所
   */
  async enableExchange(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/cleaning/exchanges/${id}/enable`);
  }
  
  /**
   * 禁用交易所
   */
  async disableExchange(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/cleaning/exchanges/${id}/disable`);
  }
  
  /**
   * 获取交易对列表
   */
  async getExchangeSymbols(id: string): Promise<string[]> {
    return apiCall(HttpMethod.GET, `/cleaning/exchanges/${id}/symbols`);
  }
  
  /**
   * 更新交易对配置
   */
  async updateExchangeSymbols(id: string, symbols: string[]): Promise<void> {
    return apiCall(HttpMethod.PUT, `/cleaning/exchanges/${id}/symbols`, { symbols });
  }
  
  /**
   * 获取交易所规则
   */
  async getExchangeRules(id: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/cleaning/exchanges/${id}/rules`);
  }
  
  /**
   * 更新交易所规则
   */
  async updateExchangeRules(id: string, rules: any[]): Promise<void> {
    return apiCall(HttpMethod.PUT, `/cleaning/exchanges/${id}/rules`, { rules });
  }
  
  /**
   * 测试交易所连接
   */
  async testExchangeConnection(id: string): Promise<any> {
    return apiCall(HttpMethod.POST, `/cleaning/exchanges/${id}/test`);
  }
  
  /**
   * 获取交易所指标
   */
  async getExchangeMetrics(id: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/cleaning/exchanges/${id}/metrics`);
  }
  
  /**
   * 重置交易所配置
   */
  async resetExchangeConfig(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/cleaning/exchanges/${id}/reset`);
  }
  
  /**
   * 获取配置历史
   */
  async getExchangeConfigHistory(id: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/cleaning/exchanges/${id}/history`);
  }
  
  /**
   * 批量更新配置
   */
  async batchUpdateExchangeConfig(updates: Record<string, any>): Promise<void> {
    return apiCall(HttpMethod.POST, '/cleaning/exchanges/batch/update', updates);
  }
  
  /**
   * 获取配置模板
   */
  async getExchangeConfigTemplates(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/cleaning/exchanges/templates');
  }
  
  // ==================== 数据质量API (16个) ====================
  
  /**
   * 获取数据质量分数
   */
  async getQualityScore(): Promise<{ score: number }> {
    return apiCall(HttpMethod.GET, '/cleaning/quality/score');
  }
  
  /**
   * 获取质量指标
   */
  async getQualityMetrics(): Promise<QualityMetrics> {
    return apiCall(HttpMethod.GET, '/cleaning/quality/metrics');
  }
  
  /**
   * 获取质量问题
   */
  async getQualityIssues(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/cleaning/quality/issues');
  }
  
  /**
   * 解决质量问题
   */
  async resolveQualityIssue(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/cleaning/quality/issues/${id}/resolve`);
  }
  
  /**
   * 获取质量趋势
   */
  async getQualityTrends(): Promise<any> {
    return apiCall(HttpMethod.GET, '/cleaning/quality/trends');
  }
  
  /**
   * 分析数据质量
   */
  async analyzeDataQuality(data: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/cleaning/quality/analyze', { data });
  }
  
  /**
   * 获取质量报告
   */
  async getQualityReports(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/cleaning/quality/reports');
  }
  
  /**
   * 生成质量报告
   */
  async generateQualityReport(period: string = '24h'): Promise<any> {
    return apiCall(HttpMethod.POST, '/cleaning/quality/reports', { period });
  }
  
  /**
   * 获取质量基准
   */
  async getQualityBenchmarks(): Promise<any> {
    return apiCall(HttpMethod.GET, '/cleaning/quality/benchmarks');
  }
  
  /**
   * 设置质量基准
   */
  async setQualityBenchmarks(score: number): Promise<void> {
    return apiCall(HttpMethod.PUT, '/cleaning/quality/benchmarks', { score });
  }
  
  /**
   * 获取质量告警
   */
  async getQualityAlerts(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/cleaning/quality/alerts');
  }
  
  /**
   * 创建质量告警
   */
  async createQualityAlert(threshold: number): Promise<any> {
    return apiCall(HttpMethod.POST, '/cleaning/quality/alerts', { threshold });
  }
  
  /**
   * 数据验证结果
   */
  async getValidationResults(): Promise<any> {
    return apiCall(HttpMethod.GET, '/cleaning/quality/validation');
  }
  
  /**
   * 执行数据验证
   */
  async executeDataValidation(rules: any[]): Promise<any> {
    return apiCall(HttpMethod.POST, '/cleaning/quality/validation', { rules });
  }
  
  /**
   * 获取质量统计
   */
  async getQualityStatistics(): Promise<any> {
    return apiCall(HttpMethod.GET, '/cleaning/quality/statistics');
  }
  
  /**
   * 优化数据质量
   */
  async optimizeDataQuality(): Promise<any> {
    return apiCall(HttpMethod.POST, '/cleaning/quality/optimize');
  }
}

// 导出单例实例
export const cleaningService = new CleaningService(); 