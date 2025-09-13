/**
 * 仪表板服务 - 完整的数据可视化和分析功能
 */

import { HttpClient } from '../core/http-client';
import {
  SankeyData,
  ProfitCurveData,
  FlowHistoryItem,
  DashboardStats,
  ApiResponse,
  PaginationQuery,
  PaginatedResponse,
} from '../types';

export class DashboardService {
  constructor(private httpClient: HttpClient) {}

  /**
   * 获取仪表板统计概览
   */
  public async getDashboardStats(timeRange?: {
    startTime: string;
    endTime: string;
  }): Promise<DashboardStats> {
    const response = await this.httpClient.get<DashboardStats>('/api/dashboard/stats', {
      params: timeRange,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取仪表板统计失败');
  }

  /**
   * 获取Sankey图数据
   */
  public async getSankeyData(params?: {
    timeRange?: { startTime: string; endTime: string };
    minFlowValue?: number;
    exchangeFilter?: string[];
  }): Promise<SankeyData> {
    const response = await this.httpClient.get<SankeyData>('/api/dashboard/sankey', {
      params,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取Sankey数据失败');
  }

  /**
   * 获取利润曲线数据
   */
  public async getProfitCurve(params?: {
    timeRange?: { startTime: string; endTime: string };
    granularity?: 'hour' | 'day' | 'week' | 'month';
    strategy?: string;
  }): Promise<ProfitCurveData> {
    const response = await this.httpClient.get<ProfitCurveData>('/api/dashboard/profit-curve', {
      params,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取利润曲线失败');
  }

  /**
   * 获取流动历史记录
   */
  public async getFlowHistory(
    query?: PaginationQuery & {
      timeRange?: { startTime: string; endTime: string };
      status?: 'pending' | 'completed' | 'failed';
      exchange?: string;
      symbol?: string;
      minProfit?: number;
    }
  ): Promise<PaginatedResponse<FlowHistoryItem>> {
    const response = await this.httpClient.get<PaginatedResponse<FlowHistoryItem>>(
      '/api/dashboard/flow-history',
      { params: query }
    );
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取流动历史失败');
  }

  /**
   * 获取实时交易流
   */
  public async getRealTimeFlows(): Promise<FlowHistoryItem[]> {
    const response = await this.httpClient.get<FlowHistoryItem[]>('/api/dashboard/realtime-flows');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取实时交易流失败');
  }

  /**
   * 获取交易对分析
   */
  public async getSymbolAnalysis(
    symbol: string,
    timeRange?: { startTime: string; endTime: string }
  ): Promise<{
    symbol: string;
    totalProfit: number;
    tradeCount: number;
    winRate: number;
    averageProfit: number;
    bestTrade: {
      profit: number;
      timestamp: string;
      exchanges: string[];
    };
    worstTrade: {
      profit: number;
      timestamp: string;
      exchanges: string[];
    };
    profitDistribution: {
      range: string;
      count: number;
      percentage: number;
    }[];
    hourlyStats: {
      hour: number;
      avgProfit: number;
      tradeCount: number;
    }[];
  }> {
    const response = await this.httpClient.get(`/api/dashboard/symbol-analysis/${symbol}`, {
      params: timeRange,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取交易对分析失败');
  }

  /**
   * 获取交易所表现分析
   */
  public async getExchangePerformance(
    timeRange?: { startTime: string; endTime: string }
  ): Promise<{
    exchange: string;
    totalVolume: number;
    profitContribution: number;
    averageLatency: number;
    uptime: number;
    errorRate: number;
    topSymbols: {
      symbol: string;
      volume: number;
      profit: number;
    }[];
  }[]> {
    const response = await this.httpClient.get('/api/dashboard/exchange-performance', {
      params: timeRange,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取交易所表现失败');
  }

  /**
   * 获取风险分析
   */
  public async getRiskAnalysis(
    timeRange?: { startTime: string; endTime: string }
  ): Promise<{
    totalRiskScore: number;
    riskFactors: {
      factor: string;
      score: number;
      impact: 'low' | 'medium' | 'high';
      description: string;
    }[];
    recommendations: string[];
    portfolioRisk: {
      var95: number;
      var99: number;
      expectedShortfall: number;
      maxDrawdown: number;
      sharpeRatio: number;
      sortinoRatio: number;
    };
    concentrationRisk: {
      topSymbols: { symbol: string; exposure: number }[];
      topExchanges: { exchange: string; exposure: number }[];
    };
  }> {
    const response = await this.httpClient.get('/api/dashboard/risk-analysis', {
      params: timeRange,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取风险分析失败');
  }

  /**
   * 获取市场机会分析
   */
  public async getMarketOpportunityAnalysis(): Promise<{
    totalOpportunities: number;
    activeOpportunities: number;
    averageProfitability: number;
    topOpportunities: {
      id: string;
      symbol: string;
      profit: number;
      profitPercent: number;
      exchanges: string[];
      riskScore: number;
    }[];
    marketTrends: {
      timeframe: string;
      direction: 'bullish' | 'bearish' | 'neutral';
      confidence: number;
      factors: string[];
    }[];
    volatilityAnalysis: {
      symbol: string;
      currentVolatility: number;
      averageVolatility: number;
      trend: 'increasing' | 'decreasing' | 'stable';
    }[];
  }> {
    const response = await this.httpClient.get('/api/dashboard/market-opportunities');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取市场机会分析失败');
  }

  /**
   * 获取性能指标
   */
  public async getPerformanceMetrics(
    timeRange?: { startTime: string; endTime: string }
  ): Promise<{
    period: string;
    totalReturn: number;
    annualizedReturn: number;
    volatility: number;
    sharpeRatio: number;
    sortinoRatio: number;
    maxDrawdown: number;
    winRate: number;
    profitFactor: number;
    averageWin: number;
    averageLoss: number;
    largestWin: number;
    largestLoss: number;
    consecutiveWins: number;
    consecutiveLosses: number;
    monthlyReturns: {
      month: string;
      return: number;
      trades: number;
    }[];
  }> {
    const response = await this.httpClient.get('/api/dashboard/performance-metrics', {
      params: timeRange,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取性能指标失败');
  }

  /**
   * 生成自定义报告
   */
  public async generateCustomReport(config: {
    timeRange: { startTime: string; endTime: string };
    reportType: 'daily' | 'weekly' | 'monthly' | 'custom';
    includeSections: ('summary' | 'profit' | 'risk' | 'performance' | 'trades' | 'recommendations')[];
    filters?: {
      exchanges?: string[];
      symbols?: string[];
      minProfit?: number;
      strategies?: string[];
    };
    format: 'json' | 'pdf' | 'excel';
  }): Promise<{
    reportId: string;
    status: 'generating' | 'completed' | 'failed';
    downloadUrl?: string;
    estimatedTime?: number;
  }> {
    const response = await this.httpClient.post('/api/dashboard/generate-report', config);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '生成报告失败');
  }

  /**
   * 获取报告状态
   */
  public async getReportStatus(reportId: string): Promise<{
    reportId: string;
    status: 'generating' | 'completed' | 'failed';
    progress: number;
    downloadUrl?: string;
    error?: string;
  }> {
    const response = await this.httpClient.get(`/api/dashboard/reports/${reportId}/status`);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取报告状态失败');
  }

  /**
   * 获取历史报告列表
   */
  public async getReportHistory(
    query?: PaginationQuery
  ): Promise<PaginatedResponse<{
    reportId: string;
    reportType: string;
    generatedAt: string;
    timeRange: { startTime: string; endTime: string };
    format: string;
    status: string;
    downloadUrl?: string;
  }>> {
    const response = await this.httpClient.get('/api/dashboard/reports', {
      params: query,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取报告历史失败');
  }

  /**
   * 创建自定义图表
   */
  public async createCustomChart(config: {
    chartType: 'line' | 'bar' | 'pie' | 'scatter' | 'heatmap';
    dataSource: string;
    metrics: string[];
    dimensions: string[];
    filters?: Record<string, any>;
    timeRange?: { startTime: string; endTime: string };
    title: string;
    description?: string;
  }): Promise<{
    chartId: string;
    data: any;
    config: any;
  }> {
    const response = await this.httpClient.post('/api/dashboard/custom-charts', config);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '创建自定义图表失败');
  }

  /**
   * 获取自定义图表列表
   */
  public async getCustomCharts(): Promise<{
    chartId: string;
    title: string;
    description?: string;
    chartType: string;
    createdAt: string;
    lastUpdated: string;
  }[]> {
    const response = await this.httpClient.get('/api/dashboard/custom-charts');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取自定义图表失败');
  }

  /**
   * 更新自定义图表
   */
  public async updateCustomChart(
    chartId: string, 
    updates: Partial<{
      title: string;
      description: string;
      config: any;
      filters: Record<string, any>;
    }>
  ): Promise<void> {
    const response = await this.httpClient.put(`/api/dashboard/custom-charts/${chartId}`, updates);
    
    if (!response.success) {
      throw new Error(response.message || '更新自定义图表失败');
    }
  }

  /**
   * 删除自定义图表
   */
  public async deleteCustomChart(chartId: string): Promise<void> {
    const response = await this.httpClient.delete(`/api/dashboard/custom-charts/${chartId}`);
    
    if (!response.success) {
      throw new Error(response.message || '删除自定义图表失败');
    }
  }

  /**
   * 导出数据
   */
  public async exportData(config: {
    dataType: 'flows' | 'profits' | 'trades' | 'opportunities';
    timeRange: { startTime: string; endTime: string };
    format: 'csv' | 'json' | 'excel';
    filters?: Record<string, any>;
  }): Promise<{
    exportId: string;
    downloadUrl: string;
    expiresAt: string;
  }> {
    const response = await this.httpClient.post('/api/dashboard/export', config);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '导出数据失败');
  }
}