/**
 * QingXi数据服务 - 完整的数据采集和市场信息管理
 */

import { HttpClient } from '../core/http-client';
import {
  MarketData,
  OrderBook,
  CollectorStatus,
  ArbitrageOpportunity,
  ApiResponse,
  PaginationQuery,
  PaginatedResponse,
} from '../types';

export class QingxiService {
  constructor(private httpClient: HttpClient) {}

  /**
   * 获取市场数据
   */
  public async getMarketData(
    symbol?: string, 
    exchange?: string
  ): Promise<MarketData[]> {
    const params: any = {};
    if (symbol) params.symbol = symbol;
    if (exchange) params.exchange = exchange;

    const response = await this.httpClient.get<MarketData[]>('/api/qingxi/market-data', {
      params,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取市场数据失败');
  }

  /**
   * 获取特定交易对的市场数据
   */
  public async getSymbolMarketData(symbol: string, exchange: string): Promise<MarketData> {
    const response = await this.httpClient.get<MarketData>(
      `/api/qingxi/market-data/${exchange}/${symbol}`
    );
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取交易对数据失败');
  }

  /**
   * 获取订单簿数据
   */
  public async getOrderBook(
    symbol: string, 
    exchange: string, 
    depth?: number
  ): Promise<OrderBook> {
    const params: any = {};
    if (depth) params.depth = depth;

    const response = await this.httpClient.get<OrderBook>(
      `/api/qingxi/orderbook/${exchange}/${symbol}`,
      { params }
    );
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取订单簿失败');
  }

  /**
   * 获取所有收集器状态
   */
  public async getCollectorStatus(): Promise<CollectorStatus[]> {
    const response = await this.httpClient.get<CollectorStatus[]>('/api/qingxi/collectors/status');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取收集器状态失败');
  }

  /**
   * 获取特定收集器状态
   */
  public async getCollectorStatusById(collectorId: string): Promise<CollectorStatus> {
    const response = await this.httpClient.get<CollectorStatus>(
      `/api/qingxi/collectors/${collectorId}/status`
    );
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取收集器状态失败');
  }

  /**
   * 启动收集器
   */
  public async startCollector(collectorId: string): Promise<void> {
    const response = await this.httpClient.post(`/api/qingxi/collectors/${collectorId}/start`);
    
    if (!response.success) {
      throw new Error(response.message || '启动收集器失败');
    }
  }

  /**
   * 停止收集器
   */
  public async stopCollector(collectorId: string): Promise<void> {
    const response = await this.httpClient.post(`/api/qingxi/collectors/${collectorId}/stop`);
    
    if (!response.success) {
      throw new Error(response.message || '停止收集器失败');
    }
  }

  /**
   * 重启收集器
   */
  public async restartCollector(collectorId: string): Promise<void> {
    const response = await this.httpClient.post(`/api/qingxi/collectors/${collectorId}/restart`);
    
    if (!response.success) {
      throw new Error(response.message || '重启收集器失败');
    }
  }

  /**
   * 获取套利机会
   */
  public async getArbitrageOpportunities(
    query?: PaginationQuery & {
      symbol?: string;
      minProfitPercent?: number;
      status?: 'active' | 'expired' | 'executed';
    }
  ): Promise<PaginatedResponse<ArbitrageOpportunity>> {
    const response = await this.httpClient.get<PaginatedResponse<ArbitrageOpportunity>>(
      '/api/qingxi/arbitrage-opportunities',
      { params: query }
    );
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取套利机会失败');
  }

  /**
   * 获取特定套利机会详情
   */
  public async getArbitrageOpportunity(opportunityId: string): Promise<ArbitrageOpportunity> {
    const response = await this.httpClient.get<ArbitrageOpportunity>(
      `/api/qingxi/arbitrage-opportunities/${opportunityId}`
    );
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取套利机会详情失败');
  }

  /**
   * 执行套利机会
   */
  public async executeArbitrageOpportunity(
    opportunityId: string,
    params: {
      amount: number;
      maxSlippage?: number;
      timeoutSeconds?: number;
    }
  ): Promise<{
    executionId: string;
    status: string;
    message: string;
  }> {
    const response = await this.httpClient.post<{
      executionId: string;
      status: string;
      message: string;
    }>(`/api/qingxi/arbitrage-opportunities/${opportunityId}/execute`, params);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '执行套利机会失败');
  }

  /**
   * 获取支持的交易所列表
   */
  public async getSupportedExchanges(): Promise<{
    id: string;
    name: string;
    status: 'active' | 'inactive';
    supportedFeatures: string[];
  }[]> {
    const response = await this.httpClient.get('/api/qingxi/exchanges');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取支持的交易所失败');
  }

  /**
   * 获取支持的交易对列表
   */
  public async getSupportedSymbols(exchange?: string): Promise<{
    symbol: string;
    baseAsset: string;
    quoteAsset: string;
    exchange: string;
    status: 'active' | 'inactive';
    lastPrice?: number;
  }[]> {
    const params: any = {};
    if (exchange) params.exchange = exchange;

    const response = await this.httpClient.get('/api/qingxi/symbols', { params });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取支持的交易对失败');
  }

  /**
   * 获取数据统计信息
   */
  public async getDataStats(): Promise<{
    totalSymbols: number;
    activeExchanges: number;
    dataPointsToday: number;
    averageLatency: number;
    uptime: number;
    lastUpdate: string;
  }> {
    const response = await this.httpClient.get('/api/qingxi/stats');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取数据统计失败');
  }

  /**
   * 获取历史数据
   */
  public async getHistoricalData(
    symbol: string,
    exchange: string,
    params: {
      startTime: string;
      endTime: string;
      interval?: '1m' | '5m' | '15m' | '1h' | '4h' | '1d';
    }
  ): Promise<{
    timestamp: number;
    open: number;
    high: number;
    low: number;
    close: number;
    volume: number;
  }[]> {
    const response = await this.httpClient.get(
      `/api/qingxi/historical/${exchange}/${symbol}`,
      { params }
    );
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取历史数据失败');
  }

  /**
   * 获取价格差异分析
   */
  public async getPriceSpreadAnalysis(
    symbol: string,
    exchanges: string[]
  ): Promise<{
    symbol: string;
    spreads: {
      exchange1: string;
      exchange2: string;
      spread: number;
      spreadPercent: number;
      volume1: number;
      volume2: number;
    }[];
    averageSpread: number;
    maxSpread: number;
    recommendedPair: {
      buyExchange: string;
      sellExchange: string;
      potentialProfit: number;
    };
  }> {
    const response = await this.httpClient.post('/api/qingxi/analysis/spread', {
      symbol,
      exchanges,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取价格差异分析失败');
  }

  /**
   * 配置数据收集器
   */
  public async configureCollector(
    collectorId: string,
    config: {
      symbols?: string[];
      updateInterval?: number;
      enableOrderbook?: boolean;
      orderbookDepth?: number;
      rateLimit?: number;
    }
  ): Promise<void> {
    const response = await this.httpClient.put(
      `/api/qingxi/collectors/${collectorId}/config`,
      config
    );
    
    if (!response.success) {
      throw new Error(response.message || '配置收集器失败');
    }
  }

  /**
   * 获取数据质量报告
   */
  public async getDataQualityReport(
    timeRange: { start: string; end: string }
  ): Promise<{
    period: string;
    totalDataPoints: number;
    missingData: number;
    dataQualityScore: number;
    exchangeQuality: {
      exchange: string;
      score: number;
      issues: string[];
    }[];
    recommendations: string[];
  }> {
    const response = await this.httpClient.get('/api/qingxi/quality-report', {
      params: timeRange,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取数据质量报告失败');
  }

  /**
   * 触发数据同步
   */
  public async triggerDataSync(params: {
    exchanges?: string[];
    symbols?: string[];
    forceRefresh?: boolean;
  } = {}): Promise<{
    syncId: string;
    status: string;
    estimatedDuration: number;
  }> {
    const response = await this.httpClient.post('/api/qingxi/sync', params);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '触发数据同步失败');
  }

  /**
   * 获取同步状态
   */
  public async getSyncStatus(syncId?: string): Promise<{
    id: string;
    status: 'running' | 'completed' | 'failed';
    progress: number;
    startTime: string;
    endTime?: string;
    processedItems: number;
    totalItems: number;
    errors?: string[];
  }[]> {
    const url = syncId ? `/api/qingxi/sync/${syncId}` : '/api/qingxi/sync';
    const response = await this.httpClient.get(url);
    
    if (response.success && response.data) {
      return syncId ? [response.data] : response.data;
    }
    
    throw new Error(response.message || '获取同步状态失败');
  }
}