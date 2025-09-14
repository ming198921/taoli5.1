import { apiCall, HttpMethod } from '../api/apiClient';
import axios from 'axios';

// AI模型服务专用客户端 - 直连AI服务
const aiApiClient = axios.create({
  baseURL: 'http://localhost:4006/api',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// AI模型服务API调用函数
const aiApiCall = async (method: HttpMethod, url: string, data?: any) => {
  try {
    const response = await aiApiClient.request({
      method,
      url,
      data
    });
    return response.data;
  } catch (error) {
    throw error;
  }
};

// AI模型服务相关类型定义
export interface AIModel {
  id: string;
  name: string;
  type: 'classification' | 'regression' | 'clustering' | 'reinforcement';
  status: 'training' | 'deployed' | 'inactive' | 'error';
  version: string;
  accuracy: number;
  created_at: string;
  updated_at: string;
  metadata: any;
}

export interface TrainingJob {
  id: string;
  model_id: string;
  status: 'running' | 'completed' | 'failed' | 'cancelled';
  progress: number;
  dataset_id: string;
  config: any;
  metrics: any;
  logs: string[];
  started_at: string;
  completed_at?: string;
}

export interface Dataset {
  id: string;
  name: string;
  type: 'training' | 'validation' | 'test';
  size: number;
  features: string[];
  target: string;
  created_at: string;
}

export interface PredictionResult {
  model_id: string;
  input: any;
  output: any;
  confidence: number;
  explanation?: any;
  timestamp: string;
}

/**
 * AI模型服务 - 48个API接口
 * 端口: 4006
 * 功能: 机器学习模型管理、训练、推理、特征工程
 */
export class AIModelService {
  
  // ==================== 模型管理API (16个) ====================
  
  /**
   * 列出所有模型
   */
  async listModels(): Promise<AIModel[]> {
    return aiApiCall(HttpMethod.GET, '/ml/models');
  }
  
  /**
   * 创建新模型
   */
  async createModel(name: string, type: string): Promise<AIModel> {
    return apiCall(HttpMethod.POST, '/models', { name, type });
  }
  
  /**
   * 获取模型详情
   */
  async getModel(id: string): Promise<AIModel> {
    return apiCall(HttpMethod.GET, `/models/${id}`);
  }
  
  /**
   * 更新模型
   */
  async updateModel(id: string, updates: Partial<AIModel>): Promise<AIModel> {
    return apiCall(HttpMethod.PUT, `/models/${id}`, updates);
  }
  
  /**
   * 删除模型
   */
  async deleteModel(id: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/models/${id}`);
  }
  
  /**
   * 获取模型状态
   */
  async getModelStatus(id: string): Promise<{ status: string }> {
    return apiCall(HttpMethod.GET, `/models/${id}/status`);
  }
  
  /**
   * 获取模型元数据
   */
  async getModelMetadata(id: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/models/${id}/metadata`);
  }
  
  /**
   * 更新模型元数据
   */
  async updateModelMetadata(id: string, metadata: any): Promise<void> {
    return apiCall(HttpMethod.PUT, `/models/${id}/metadata`, metadata);
  }
  
  /**
   * 获取模型版本
   */
  async getModelVersions(id: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/models/${id}/versions`);
  }
  
  /**
   * 创建模型版本
   */
  async createModelVersion(id: string): Promise<any> {
    return apiCall(HttpMethod.POST, `/models/${id}/versions`);
  }
  
  /**
   * 获取特定版本
   */
  async getModelVersion(id: string, version: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/models/${id}/versions/${version}`);
  }
  
  /**
   * 部署模型
   */
  async deployModel(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/models/${id}/deploy`);
  }
  
  /**
   * 取消部署
   */
  async undeployModel(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/models/${id}/undeploy`);
  }
  
  /**
   * 回滚模型
   */
  async rollbackModel(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/models/${id}/rollback`);
  }
  
  /**
   * 搜索模型
   */
  async searchModels(query: string): Promise<AIModel[]> {
    return apiCall(HttpMethod.POST, '/models/search', { query });
  }
  
  /**
   * 克隆模型
   */
  async cloneModel(id: string): Promise<AIModel> {
    return apiCall(HttpMethod.POST, `/models/${id}/clone`);
  }
  
  // ==================== 训练API (12个) ====================
  
  /**
   * 列出训练任务
   */
  async listTrainingJobs(): Promise<TrainingJob[]> {
    return aiApiCall(HttpMethod.GET, '/ml/training/jobs');
  }
  
  /**
   * 创建训练任务
   */
  async createTrainingJob(model_id: string, config: any): Promise<TrainingJob> {
    return apiCall(HttpMethod.POST, '/training/jobs', { model_id, ...config });
  }
  
  /**
   * 获取训练任务
   */
  async getTrainingJob(id: string): Promise<TrainingJob> {
    return apiCall(HttpMethod.GET, `/training/jobs/${id}`);
  }
  
  /**
   * 停止训练任务
   */
  async stopTrainingJob(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/training/jobs/${id}/stop`);
  }
  
  /**
   * 获取训练进度
   */
  async getTrainingProgress(id: string): Promise<{ progress: number }> {
    return apiCall(HttpMethod.GET, `/training/jobs/${id}/progress`);
  }
  
  /**
   * 获取训练日志
   */
  async getTrainingLogs(id: string): Promise<string[]> {
    return apiCall(HttpMethod.GET, `/training/jobs/${id}/logs`);
  }
  
  /**
   * 列出数据集
   */
  async listDatasets(): Promise<Dataset[]> {
    return aiApiCall(HttpMethod.GET, '/ml/training/datasets');
  }
  
  /**
   * 创建数据集
   */
  async createDataset(name: string): Promise<Dataset> {
    return apiCall(HttpMethod.POST, '/training/datasets', { name });
  }
  
  /**
   * 获取数据集详情
   */
  async getDataset(id: string): Promise<Dataset> {
    return apiCall(HttpMethod.GET, `/training/datasets/${id}`);
  }
  
  /**
   * 上传数据集
   */
  async uploadDataset(id: string, formData: FormData): Promise<void> {
    return apiCall(HttpMethod.POST, `/training/datasets/${id}/upload`, formData);
  }
  
  /**
   * 获取训练指标
   */
  async getTrainingMetrics(): Promise<any> {
    return apiCall(HttpMethod.GET, '/training/metrics');
  }
  
  /**
   * 超参数调优
   */
  async hyperparameterTuning(config: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/training/hyperparameter-tuning', config);
  }
  
  // ==================== 推理API (12个) ====================
  
  /**
   * 执行预测
   */
  async predict(data: any): Promise<PredictionResult> {
    return apiCall(HttpMethod.POST, '/inference/predict', { data });
  }
  
  /**
   * 批量预测
   */
  async batchPredict(batch: any[]): Promise<PredictionResult[]> {
    return apiCall(HttpMethod.POST, '/inference/batch-predict', { batch });
  }
  
  /**
   * 获取推理模型
   */
  async getInferenceModels(): Promise<AIModel[]> {
    return apiCall(HttpMethod.GET, '/inference/models');
  }
  
  /**
   * 加载推理模型
   */
  async loadInferenceModel(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/inference/models/${id}/load`);
  }
  
  /**
   * 卸载推理模型
   */
  async unloadInferenceModel(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/inference/models/${id}/unload`);
  }
  
  /**
   * 获取推理统计
   */
  async getInferenceStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/inference/stats');
  }
  
  /**
   * 获取推理性能
   */
  async getInferencePerformance(): Promise<any> {
    return apiCall(HttpMethod.GET, '/inference/performance');
  }
  
  /**
   * 模型解释
   */
  async explainPrediction(data: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/inference/explain', { data });
  }
  
  /**
   * 推理健康检查
   */
  async getInferenceHealth(): Promise<any> {
    return apiCall(HttpMethod.GET, '/inference/health');
  }
  
  /**
   * 推理基准测试
   */
  async benchmarkInference(): Promise<any> {
    return apiCall(HttpMethod.POST, '/inference/benchmark');
  }
  
  /**
   * 获取推理队列
   */
  async getInferenceQueue(): Promise<any> {
    return apiCall(HttpMethod.GET, '/inference/queue');
  }
  
  /**
   * 优化推理
   */
  async optimizeInference(): Promise<any> {
    return apiCall(HttpMethod.POST, '/inference/optimize');
  }
  
  // ==================== 特征工程API (8个) ====================
  
  /**
   * 列出特征
   */
  async listFeatures(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/features/list');
  }
  
  /**
   * 提取特征
   */
  async extractFeatures(data: any[]): Promise<any> {
    return apiCall(HttpMethod.POST, '/features/extract', { data });
  }
  
  /**
   * 特征转换
   */
  async transformFeatures(features: any[]): Promise<any> {
    return apiCall(HttpMethod.POST, '/features/transform', { features });
  }
  
  /**
   * 特征选择
   */
  async selectFeatures(method: string): Promise<any> {
    return apiCall(HttpMethod.POST, '/features/select', { method });
  }
  
  /**
   * 获取特征重要性
   */
  async getFeatureImportance(): Promise<any> {
    return apiCall(HttpMethod.GET, '/features/importance');
  }
  
  /**
   * 特征工程
   */
  async engineerFeatures(pipeline: any[]): Promise<any> {
    return apiCall(HttpMethod.POST, '/features/engineering', { pipeline });
  }
  
  /**
   * 获取特征统计
   */
  async getFeatureStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/features/stats');
  }
  
  /**
   * 验证特征
   */
  async validateFeatures(features: any[]): Promise<any> {
    return apiCall(HttpMethod.POST, '/features/validate', { features });
  }
}

// 导出单例实例
export const aiModelService = new AIModelService(); 
import axios from 'axios';

// AI模型服务专用客户端 - 直连AI服务
const aiApiClient = axios.create({
  baseURL: 'http://localhost:4006/api',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// AI模型服务API调用函数
const aiApiCall = async (method: HttpMethod, url: string, data?: any) => {
  try {
    const response = await aiApiClient.request({
      method,
      url,
      data
    });
    return response.data;
  } catch (error) {
    throw error;
  }
};

// AI模型服务相关类型定义
export interface AIModel {
  id: string;
  name: string;
  type: 'classification' | 'regression' | 'clustering' | 'reinforcement';
  status: 'training' | 'deployed' | 'inactive' | 'error';
  version: string;
  accuracy: number;
  created_at: string;
  updated_at: string;
  metadata: any;
}

export interface TrainingJob {
  id: string;
  model_id: string;
  status: 'running' | 'completed' | 'failed' | 'cancelled';
  progress: number;
  dataset_id: string;
  config: any;
  metrics: any;
  logs: string[];
  started_at: string;
  completed_at?: string;
}

export interface Dataset {
  id: string;
  name: string;
  type: 'training' | 'validation' | 'test';
  size: number;
  features: string[];
  target: string;
  created_at: string;
}

export interface PredictionResult {
  model_id: string;
  input: any;
  output: any;
  confidence: number;
  explanation?: any;
  timestamp: string;
}

/**
 * AI模型服务 - 48个API接口
 * 端口: 4006
 * 功能: 机器学习模型管理、训练、推理、特征工程
 */
export class AIModelService {
  
  // ==================== 模型管理API (16个) ====================
  
  /**
   * 列出所有模型
   */
  async listModels(): Promise<AIModel[]> {
    return aiApiCall(HttpMethod.GET, '/ml/models');
  }
  
  /**
   * 创建新模型
   */
  async createModel(name: string, type: string): Promise<AIModel> {
    return apiCall(HttpMethod.POST, '/models', { name, type });
  }
  
  /**
   * 获取模型详情
   */
  async getModel(id: string): Promise<AIModel> {
    return apiCall(HttpMethod.GET, `/models/${id}`);
  }
  
  /**
   * 更新模型
   */
  async updateModel(id: string, updates: Partial<AIModel>): Promise<AIModel> {
    return apiCall(HttpMethod.PUT, `/models/${id}`, updates);
  }
  
  /**
   * 删除模型
   */
  async deleteModel(id: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/models/${id}`);
  }
  
  /**
   * 获取模型状态
   */
  async getModelStatus(id: string): Promise<{ status: string }> {
    return apiCall(HttpMethod.GET, `/models/${id}/status`);
  }
  
  /**
   * 获取模型元数据
   */
  async getModelMetadata(id: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/models/${id}/metadata`);
  }
  
  /**
   * 更新模型元数据
   */
  async updateModelMetadata(id: string, metadata: any): Promise<void> {
    return apiCall(HttpMethod.PUT, `/models/${id}/metadata`, metadata);
  }
  
  /**
   * 获取模型版本
   */
  async getModelVersions(id: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/models/${id}/versions`);
  }
  
  /**
   * 创建模型版本
   */
  async createModelVersion(id: string): Promise<any> {
    return apiCall(HttpMethod.POST, `/models/${id}/versions`);
  }
  
  /**
   * 获取特定版本
   */
  async getModelVersion(id: string, version: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/models/${id}/versions/${version}`);
  }
  
  /**
   * 部署模型
   */
  async deployModel(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/models/${id}/deploy`);
  }
  
  /**
   * 取消部署
   */
  async undeployModel(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/models/${id}/undeploy`);
  }
  
  /**
   * 回滚模型
   */
  async rollbackModel(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/models/${id}/rollback`);
  }
  
  /**
   * 搜索模型
   */
  async searchModels(query: string): Promise<AIModel[]> {
    return apiCall(HttpMethod.POST, '/models/search', { query });
  }
  
  /**
   * 克隆模型
   */
  async cloneModel(id: string): Promise<AIModel> {
    return apiCall(HttpMethod.POST, `/models/${id}/clone`);
  }
  
  // ==================== 训练API (12个) ====================
  
  /**
   * 列出训练任务
   */
  async listTrainingJobs(): Promise<TrainingJob[]> {
    return aiApiCall(HttpMethod.GET, '/ml/training/jobs');
  }
  
  /**
   * 创建训练任务
   */
  async createTrainingJob(model_id: string, config: any): Promise<TrainingJob> {
    return apiCall(HttpMethod.POST, '/training/jobs', { model_id, ...config });
  }
  
  /**
   * 获取训练任务
   */
  async getTrainingJob(id: string): Promise<TrainingJob> {
    return apiCall(HttpMethod.GET, `/training/jobs/${id}`);
  }
  
  /**
   * 停止训练任务
   */
  async stopTrainingJob(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/training/jobs/${id}/stop`);
  }
  
  /**
   * 获取训练进度
   */
  async getTrainingProgress(id: string): Promise<{ progress: number }> {
    return apiCall(HttpMethod.GET, `/training/jobs/${id}/progress`);
  }
  
  /**
   * 获取训练日志
   */
  async getTrainingLogs(id: string): Promise<string[]> {
    return apiCall(HttpMethod.GET, `/training/jobs/${id}/logs`);
  }
  
  /**
   * 列出数据集
   */
  async listDatasets(): Promise<Dataset[]> {
    return aiApiCall(HttpMethod.GET, '/ml/training/datasets');
  }
  
  /**
   * 创建数据集
   */
  async createDataset(name: string): Promise<Dataset> {
    return apiCall(HttpMethod.POST, '/training/datasets', { name });
  }
  
  /**
   * 获取数据集详情
   */
  async getDataset(id: string): Promise<Dataset> {
    return apiCall(HttpMethod.GET, `/training/datasets/${id}`);
  }
  
  /**
   * 上传数据集
   */
  async uploadDataset(id: string, formData: FormData): Promise<void> {
    return apiCall(HttpMethod.POST, `/training/datasets/${id}/upload`, formData);
  }
  
  /**
   * 获取训练指标
   */
  async getTrainingMetrics(): Promise<any> {
    return apiCall(HttpMethod.GET, '/training/metrics');
  }
  
  /**
   * 超参数调优
   */
  async hyperparameterTuning(config: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/training/hyperparameter-tuning', config);
  }
  
  // ==================== 推理API (12个) ====================
  
  /**
   * 执行预测
   */
  async predict(data: any): Promise<PredictionResult> {
    return apiCall(HttpMethod.POST, '/inference/predict', { data });
  }
  
  /**
   * 批量预测
   */
  async batchPredict(batch: any[]): Promise<PredictionResult[]> {
    return apiCall(HttpMethod.POST, '/inference/batch-predict', { batch });
  }
  
  /**
   * 获取推理模型
   */
  async getInferenceModels(): Promise<AIModel[]> {
    return apiCall(HttpMethod.GET, '/inference/models');
  }
  
  /**
   * 加载推理模型
   */
  async loadInferenceModel(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/inference/models/${id}/load`);
  }
  
  /**
   * 卸载推理模型
   */
  async unloadInferenceModel(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/inference/models/${id}/unload`);
  }
  
  /**
   * 获取推理统计
   */
  async getInferenceStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/inference/stats');
  }
  
  /**
   * 获取推理性能
   */
  async getInferencePerformance(): Promise<any> {
    return apiCall(HttpMethod.GET, '/inference/performance');
  }
  
  /**
   * 模型解释
   */
  async explainPrediction(data: any): Promise<any> {
    return apiCall(HttpMethod.POST, '/inference/explain', { data });
  }
  
  /**
   * 推理健康检查
   */
  async getInferenceHealth(): Promise<any> {
    return apiCall(HttpMethod.GET, '/inference/health');
  }
  
  /**
   * 推理基准测试
   */
  async benchmarkInference(): Promise<any> {
    return apiCall(HttpMethod.POST, '/inference/benchmark');
  }
  
  /**
   * 获取推理队列
   */
  async getInferenceQueue(): Promise<any> {
    return apiCall(HttpMethod.GET, '/inference/queue');
  }
  
  /**
   * 优化推理
   */
  async optimizeInference(): Promise<any> {
    return apiCall(HttpMethod.POST, '/inference/optimize');
  }
  
  // ==================== 特征工程API (8个) ====================
  
  /**
   * 列出特征
   */
  async listFeatures(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/features/list');
  }
  
  /**
   * 提取特征
   */
  async extractFeatures(data: any[]): Promise<any> {
    return apiCall(HttpMethod.POST, '/features/extract', { data });
  }
  
  /**
   * 特征转换
   */
  async transformFeatures(features: any[]): Promise<any> {
    return apiCall(HttpMethod.POST, '/features/transform', { features });
  }
  
  /**
   * 特征选择
   */
  async selectFeatures(method: string): Promise<any> {
    return apiCall(HttpMethod.POST, '/features/select', { method });
  }
  
  /**
   * 获取特征重要性
   */
  async getFeatureImportance(): Promise<any> {
    return apiCall(HttpMethod.GET, '/features/importance');
  }
  
  /**
   * 特征工程
   */
  async engineerFeatures(pipeline: any[]): Promise<any> {
    return apiCall(HttpMethod.POST, '/features/engineering', { pipeline });
  }
  
  /**
   * 获取特征统计
   */
  async getFeatureStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/features/stats');
  }
  
  /**
   * 验证特征
   */
  async validateFeatures(features: any[]): Promise<any> {
    return apiCall(HttpMethod.POST, '/features/validate', { features });
  }
}

// 导出单例实例
export const aiModelService = new AIModelService(); 