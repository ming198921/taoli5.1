import { apiCall, HttpMethod } from '../api/apiClient';

// 性能服务相关类型定义
export interface CpuInfo {
  usage: number;
  cores: number;
  frequency: string;
  governor: string;
  temperature: number;
  processes: any[];
}

export interface MemoryInfo {
  usage: number;
  total: number;
  available: number;
  swap: {
    usage: number;
    total: number;
  };
  cache: number;
  fragmentation: number;
}

export interface NetworkInfo {
  interfaces: any[];
  stats: any;
  bandwidth: any;
  latency: number;
  connections: any[];
}

export interface DiskInfo {
  usage: number;
  io_stats: any;
  iops: number;
  latency: number;
  scheduler: string;
}

/**
 * 性能服务 - 67个API接口
 * 端口: 4004
 * 功能: 系统性能优化、资源监控、基准测试
 */
export class PerformanceService {
  
  // ==================== CPU优化API (18个) ====================
  
  /**
   * 获取CPU使用率
   */
  async getCpuUsage(): Promise<{ usage: number }> {
    return apiCall(HttpMethod.GET, '/performance/cpu/usage');
  }
  
  /**
   * 获取CPU核心信息
   */
  async getCpuCores(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/cpu/cores');
  }
  
  /**
   * 获取CPU频率
   */
  async getCpuFrequency(): Promise<{ frequency: string }> {
    return apiCall(HttpMethod.GET, '/performance/cpu/frequency');
  }
  
  /**
   * 设置CPU频率
   */
  async setCpuFrequency(freq: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/cpu/frequency', { freq });
  }
  
  /**
   * 获取CPU调度器
   */
  async getCpuGovernor(): Promise<{ governor: string }> {
    return apiCall(HttpMethod.GET, '/performance/cpu/governor');
  }
  
  /**
   * 设置CPU调度器
   */
  async setCpuGovernor(governor: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/cpu/governor', { governor });
  }
  
  /**
   * 获取进程CPU亲和性
   */
  async getProcessCpuAffinity(process: string): Promise<{ cores: number[] }> {
    return apiCall(HttpMethod.GET, `/performance/cpu/affinity/${process}`);
  }
  
  /**
   * 设置进程CPU亲和性
   */
  async setProcessCpuAffinity(process: string, cores: number[]): Promise<void> {
    return apiCall(HttpMethod.PUT, `/performance/cpu/affinity/${process}`, { cores });
  }
  
  /**
   * 获取CPU缓存统计
   */
  async getCpuCacheStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/cpu/cache');
  }
  
  /**
   * 刷新CPU缓存
   */
  async flushCpuCache(): Promise<void> {
    return apiCall(HttpMethod.POST, '/performance/cpu/cache/flush');
  }
  
  /**
   * 获取CPU温度
   */
  async getCpuTemperature(): Promise<{ temperature: number }> {
    return apiCall(HttpMethod.GET, '/performance/cpu/temperature');
  }
  
  /**
   * 获取CPU节流状态
   */
  async getCpuThrottling(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/cpu/throttling');
  }
  
  /**
   * 获取CPU拓扑
   */
  async getCpuTopology(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/cpu/topology');
  }
  
  /**
   * 获取进程CPU使用
   */
  async getProcessCpuUsage(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/performance/cpu/processes');
  }
  
  /**
   * 优化CPU性能
   */
  async optimizeCpu(): Promise<any> {
    return apiCall(HttpMethod.POST, '/performance/cpu/optimize');
  }
  
  /**
   * 运行CPU基准测试
   */
  async benchmarkCpu(): Promise<any> {
    return apiCall(HttpMethod.POST, '/performance/cpu/benchmark');
  }
  
  /**
   * 获取调度器信息
   */
  async getSchedulerInfo(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/cpu/scheduler');
  }
  
  /**
   * 调优调度器
   */
  async tuneScheduler(policy: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/cpu/scheduler', { policy });
  }
  
  // ==================== 内存优化API (16个) ====================
  
  /**
   * 获取内存使用情况
   */
  async getMemoryUsage(): Promise<MemoryInfo> {
    return apiCall(HttpMethod.GET, '/performance/memory/usage');
  }
  
  /**
   * 获取交换空间使用
   */
  async getSwapUsage(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/memory/swap');
  }
  
  /**
   * 配置交换空间
   */
  async configureSwap(size: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/memory/swap', { size });
  }
  
  /**
   * 获取内存缓存
   */
  async getMemoryCache(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/memory/cache');
  }
  
  /**
   * 清理内存缓存
   */
  async clearMemoryCache(): Promise<void> {
    return apiCall(HttpMethod.POST, '/performance/memory/cache/clear');
  }
  
  /**
   * 获取内存碎片
   */
  async getMemoryFragmentation(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/memory/fragmentation');
  }
  
  /**
   * 内存压缩
   */
  async compactMemory(): Promise<void> {
    return apiCall(HttpMethod.POST, '/performance/memory/compaction');
  }
  
  /**
   * 获取大页配置
   */
  async getHugePagesConfig(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/memory/huge-pages');
  }
  
  /**
   * 配置大页
   */
  async configureHugePages(enabled: boolean): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/memory/huge-pages', { enabled });
  }
  
  /**
   * 获取NUMA信息
   */
  async getNumaInfo(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/memory/numa');
  }
  
  /**
   * 优化NUMA
   */
  async optimizeNuma(policy: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/memory/numa', { policy });
  }
  
  /**
   * 获取内存压力
   */
  async getMemoryPressure(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/memory/pressure');
  }
  
  /**
   * 检测内存泄漏
   */
  async detectMemoryLeaks(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/performance/memory/leaks');
  }
  
  /**
   * 获取GC统计
   */
  async getGcStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/memory/gc');
  }
  
  /**
   * 触发垃圾回收
   */
  async triggerGc(): Promise<void> {
    return apiCall(HttpMethod.POST, '/performance/memory/gc');
  }
  
  /**
   * 优化内存
   */
  async optimizeMemory(): Promise<any> {
    return apiCall(HttpMethod.POST, '/performance/memory/optimize');
  }
  
  // ==================== 网络优化API (15个) ====================
  
  /**
   * 获取网络接口
   */
  async getNetworkInterfaces(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/performance/network/interfaces');
  }
  
  /**
   * 获取网络统计
   */
  async getNetworkStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/network/stats');
  }
  
  /**
   * 获取带宽信息
   */
  async getBandwidthInfo(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/network/bandwidth');
  }
  
  /**
   * 测量网络延迟
   */
  async measureNetworkLatency(): Promise<{ latency: number }> {
    return apiCall(HttpMethod.GET, '/performance/network/latency');
  }
  
  /**
   * 获取网络连接
   */
  async getNetworkConnections(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/performance/network/connections');
  }
  
  /**
   * 获取TCP调优参数
   */
  async getTcpTuningParams(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/network/tcp-tuning');
  }
  
  /**
   * 设置TCP调优参数
   */
  async setTcpTuningParams(params: any): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/network/tcp-tuning', params);
  }
  
  /**
   * 获取缓冲区大小
   */
  async getBufferSizes(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/network/buffer-sizes');
  }
  
  /**
   * 设置缓冲区大小
   */
  async setBufferSizes(sizes: any): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/network/buffer-sizes', sizes);
  }
  
  /**
   * 获取拥塞控制算法
   */
  async getCongestionControl(): Promise<{ algo: string }> {
    return apiCall(HttpMethod.GET, '/performance/network/congestion');
  }
  
  /**
   * 设置拥塞控制算法
   */
  async setCongestionControl(algo: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/network/congestion', { algo });
  }
  
  /**
   * 获取队列规则
   */
  async getQueueRules(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/network/queue');
  }
  
  /**
   * 设置队列规则
   */
  async setQueueRules(rules: any): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/network/queue', rules);
  }
  
  /**
   * 优化网络性能
   */
  async optimizeNetwork(): Promise<any> {
    return apiCall(HttpMethod.POST, '/performance/network/optimize');
  }
  
  /**
   * 运行网络测试
   */
  async runNetworkTest(): Promise<any> {
    return apiCall(HttpMethod.POST, '/performance/network/test');
  }
  
  // ==================== 磁盘I/O优化API (18个) ====================
  
  /**
   * 获取磁盘使用情况
   */
  async getDiskUsage(): Promise<DiskInfo> {
    return apiCall(HttpMethod.GET, '/performance/disk/usage');
  }
  
  /**
   * 获取I/O统计
   */
  async getIoStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/disk/io-stats');
  }
  
  /**
   * 测量IOPS
   */
  async measureIops(): Promise<{ iops: number }> {
    return apiCall(HttpMethod.GET, '/performance/disk/iops');
  }
  
  /**
   * 测量磁盘延迟
   */
  async measureDiskLatency(): Promise<{ latency: number }> {
    return apiCall(HttpMethod.GET, '/performance/disk/latency');
  }
  
  /**
   * 获取I/O调度器
   */
  async getIoScheduler(): Promise<{ scheduler: string }> {
    return apiCall(HttpMethod.GET, '/performance/disk/scheduler');
  }
  
  /**
   * 设置I/O调度器
   */
  async setIoScheduler(scheduler: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/disk/scheduler', { scheduler });
  }
  
  /**
   * 获取队列深度
   */
  async getQueueDepth(): Promise<{ depth: number }> {
    return apiCall(HttpMethod.GET, '/performance/disk/queue-depth');
  }
  
  /**
   * 设置队列深度
   */
  async setQueueDepth(depth: number): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/disk/queue-depth', { depth });
  }
  
  /**
   * 获取预读设置
   */
  async getReadAheadSettings(): Promise<{ size: string }> {
    return apiCall(HttpMethod.GET, '/performance/disk/read-ahead');
  }
  
  /**
   * 设置预读
   */
  async setReadAhead(size: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/disk/read-ahead', { size });
  }
  
  /**
   * 获取磁盘缓存
   */
  async getDiskCache(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/disk/cache');
  }
  
  /**
   * 配置磁盘缓存
   */
  async configureDiskCache(enabled: boolean): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/disk/cache', { enabled });
  }
  
  /**
   * 获取挂载选项
   */
  async getMountOptions(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/disk/mount-options');
  }
  
  /**
   * 设置挂载选项
   */
  async setMountOptions(options: any): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/disk/mount-options', options);
  }
  
  /**
   * 磁盘碎片整理
   */
  async defragmentDisk(): Promise<any> {
    return apiCall(HttpMethod.POST, '/performance/disk/defrag');
  }
  
  /**
   * SSD TRIM
   */
  async trimSsd(): Promise<any> {
    return apiCall(HttpMethod.POST, '/performance/disk/trim');
  }
  
  /**
   * 运行磁盘基准测试
   */
  async benchmarkDisk(): Promise<any> {
    return apiCall(HttpMethod.POST, '/performance/disk/benchmark');
  }
  
  /**
   * 优化磁盘性能
   */
  async optimizeDisk(): Promise<any> {
    return apiCall(HttpMethod.POST, '/performance/disk/optimize');
  }
}

// 导出单例实例
export const performanceService = new PerformanceService(); 

// 性能服务相关类型定义
export interface CpuInfo {
  usage: number;
  cores: number;
  frequency: string;
  governor: string;
  temperature: number;
  processes: any[];
}

export interface MemoryInfo {
  usage: number;
  total: number;
  available: number;
  swap: {
    usage: number;
    total: number;
  };
  cache: number;
  fragmentation: number;
}

export interface NetworkInfo {
  interfaces: any[];
  stats: any;
  bandwidth: any;
  latency: number;
  connections: any[];
}

export interface DiskInfo {
  usage: number;
  io_stats: any;
  iops: number;
  latency: number;
  scheduler: string;
}

/**
 * 性能服务 - 67个API接口
 * 端口: 4004
 * 功能: 系统性能优化、资源监控、基准测试
 */
export class PerformanceService {
  
  // ==================== CPU优化API (18个) ====================
  
  /**
   * 获取CPU使用率
   */
  async getCpuUsage(): Promise<{ usage: number }> {
    return apiCall(HttpMethod.GET, '/performance/cpu/usage');
  }
  
  /**
   * 获取CPU核心信息
   */
  async getCpuCores(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/cpu/cores');
  }
  
  /**
   * 获取CPU频率
   */
  async getCpuFrequency(): Promise<{ frequency: string }> {
    return apiCall(HttpMethod.GET, '/performance/cpu/frequency');
  }
  
  /**
   * 设置CPU频率
   */
  async setCpuFrequency(freq: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/cpu/frequency', { freq });
  }
  
  /**
   * 获取CPU调度器
   */
  async getCpuGovernor(): Promise<{ governor: string }> {
    return apiCall(HttpMethod.GET, '/performance/cpu/governor');
  }
  
  /**
   * 设置CPU调度器
   */
  async setCpuGovernor(governor: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/cpu/governor', { governor });
  }
  
  /**
   * 获取进程CPU亲和性
   */
  async getProcessCpuAffinity(process: string): Promise<{ cores: number[] }> {
    return apiCall(HttpMethod.GET, `/performance/cpu/affinity/${process}`);
  }
  
  /**
   * 设置进程CPU亲和性
   */
  async setProcessCpuAffinity(process: string, cores: number[]): Promise<void> {
    return apiCall(HttpMethod.PUT, `/performance/cpu/affinity/${process}`, { cores });
  }
  
  /**
   * 获取CPU缓存统计
   */
  async getCpuCacheStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/cpu/cache');
  }
  
  /**
   * 刷新CPU缓存
   */
  async flushCpuCache(): Promise<void> {
    return apiCall(HttpMethod.POST, '/performance/cpu/cache/flush');
  }
  
  /**
   * 获取CPU温度
   */
  async getCpuTemperature(): Promise<{ temperature: number }> {
    return apiCall(HttpMethod.GET, '/performance/cpu/temperature');
  }
  
  /**
   * 获取CPU节流状态
   */
  async getCpuThrottling(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/cpu/throttling');
  }
  
  /**
   * 获取CPU拓扑
   */
  async getCpuTopology(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/cpu/topology');
  }
  
  /**
   * 获取进程CPU使用
   */
  async getProcessCpuUsage(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/performance/cpu/processes');
  }
  
  /**
   * 优化CPU性能
   */
  async optimizeCpu(): Promise<any> {
    return apiCall(HttpMethod.POST, '/performance/cpu/optimize');
  }
  
  /**
   * 运行CPU基准测试
   */
  async benchmarkCpu(): Promise<any> {
    return apiCall(HttpMethod.POST, '/performance/cpu/benchmark');
  }
  
  /**
   * 获取调度器信息
   */
  async getSchedulerInfo(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/cpu/scheduler');
  }
  
  /**
   * 调优调度器
   */
  async tuneScheduler(policy: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/cpu/scheduler', { policy });
  }
  
  // ==================== 内存优化API (16个) ====================
  
  /**
   * 获取内存使用情况
   */
  async getMemoryUsage(): Promise<MemoryInfo> {
    return apiCall(HttpMethod.GET, '/performance/memory/usage');
  }
  
  /**
   * 获取交换空间使用
   */
  async getSwapUsage(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/memory/swap');
  }
  
  /**
   * 配置交换空间
   */
  async configureSwap(size: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/memory/swap', { size });
  }
  
  /**
   * 获取内存缓存
   */
  async getMemoryCache(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/memory/cache');
  }
  
  /**
   * 清理内存缓存
   */
  async clearMemoryCache(): Promise<void> {
    return apiCall(HttpMethod.POST, '/performance/memory/cache/clear');
  }
  
  /**
   * 获取内存碎片
   */
  async getMemoryFragmentation(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/memory/fragmentation');
  }
  
  /**
   * 内存压缩
   */
  async compactMemory(): Promise<void> {
    return apiCall(HttpMethod.POST, '/performance/memory/compaction');
  }
  
  /**
   * 获取大页配置
   */
  async getHugePagesConfig(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/memory/huge-pages');
  }
  
  /**
   * 配置大页
   */
  async configureHugePages(enabled: boolean): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/memory/huge-pages', { enabled });
  }
  
  /**
   * 获取NUMA信息
   */
  async getNumaInfo(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/memory/numa');
  }
  
  /**
   * 优化NUMA
   */
  async optimizeNuma(policy: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/memory/numa', { policy });
  }
  
  /**
   * 获取内存压力
   */
  async getMemoryPressure(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/memory/pressure');
  }
  
  /**
   * 检测内存泄漏
   */
  async detectMemoryLeaks(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/performance/memory/leaks');
  }
  
  /**
   * 获取GC统计
   */
  async getGcStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/memory/gc');
  }
  
  /**
   * 触发垃圾回收
   */
  async triggerGc(): Promise<void> {
    return apiCall(HttpMethod.POST, '/performance/memory/gc');
  }
  
  /**
   * 优化内存
   */
  async optimizeMemory(): Promise<any> {
    return apiCall(HttpMethod.POST, '/performance/memory/optimize');
  }
  
  // ==================== 网络优化API (15个) ====================
  
  /**
   * 获取网络接口
   */
  async getNetworkInterfaces(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/performance/network/interfaces');
  }
  
  /**
   * 获取网络统计
   */
  async getNetworkStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/network/stats');
  }
  
  /**
   * 获取带宽信息
   */
  async getBandwidthInfo(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/network/bandwidth');
  }
  
  /**
   * 测量网络延迟
   */
  async measureNetworkLatency(): Promise<{ latency: number }> {
    return apiCall(HttpMethod.GET, '/performance/network/latency');
  }
  
  /**
   * 获取网络连接
   */
  async getNetworkConnections(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/performance/network/connections');
  }
  
  /**
   * 获取TCP调优参数
   */
  async getTcpTuningParams(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/network/tcp-tuning');
  }
  
  /**
   * 设置TCP调优参数
   */
  async setTcpTuningParams(params: any): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/network/tcp-tuning', params);
  }
  
  /**
   * 获取缓冲区大小
   */
  async getBufferSizes(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/network/buffer-sizes');
  }
  
  /**
   * 设置缓冲区大小
   */
  async setBufferSizes(sizes: any): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/network/buffer-sizes', sizes);
  }
  
  /**
   * 获取拥塞控制算法
   */
  async getCongestionControl(): Promise<{ algo: string }> {
    return apiCall(HttpMethod.GET, '/performance/network/congestion');
  }
  
  /**
   * 设置拥塞控制算法
   */
  async setCongestionControl(algo: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/network/congestion', { algo });
  }
  
  /**
   * 获取队列规则
   */
  async getQueueRules(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/network/queue');
  }
  
  /**
   * 设置队列规则
   */
  async setQueueRules(rules: any): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/network/queue', rules);
  }
  
  /**
   * 优化网络性能
   */
  async optimizeNetwork(): Promise<any> {
    return apiCall(HttpMethod.POST, '/performance/network/optimize');
  }
  
  /**
   * 运行网络测试
   */
  async runNetworkTest(): Promise<any> {
    return apiCall(HttpMethod.POST, '/performance/network/test');
  }
  
  // ==================== 磁盘I/O优化API (18个) ====================
  
  /**
   * 获取磁盘使用情况
   */
  async getDiskUsage(): Promise<DiskInfo> {
    return apiCall(HttpMethod.GET, '/performance/disk/usage');
  }
  
  /**
   * 获取I/O统计
   */
  async getIoStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/disk/io-stats');
  }
  
  /**
   * 测量IOPS
   */
  async measureIops(): Promise<{ iops: number }> {
    return apiCall(HttpMethod.GET, '/performance/disk/iops');
  }
  
  /**
   * 测量磁盘延迟
   */
  async measureDiskLatency(): Promise<{ latency: number }> {
    return apiCall(HttpMethod.GET, '/performance/disk/latency');
  }
  
  /**
   * 获取I/O调度器
   */
  async getIoScheduler(): Promise<{ scheduler: string }> {
    return apiCall(HttpMethod.GET, '/performance/disk/scheduler');
  }
  
  /**
   * 设置I/O调度器
   */
  async setIoScheduler(scheduler: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/disk/scheduler', { scheduler });
  }
  
  /**
   * 获取队列深度
   */
  async getQueueDepth(): Promise<{ depth: number }> {
    return apiCall(HttpMethod.GET, '/performance/disk/queue-depth');
  }
  
  /**
   * 设置队列深度
   */
  async setQueueDepth(depth: number): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/disk/queue-depth', { depth });
  }
  
  /**
   * 获取预读设置
   */
  async getReadAheadSettings(): Promise<{ size: string }> {
    return apiCall(HttpMethod.GET, '/performance/disk/read-ahead');
  }
  
  /**
   * 设置预读
   */
  async setReadAhead(size: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/disk/read-ahead', { size });
  }
  
  /**
   * 获取磁盘缓存
   */
  async getDiskCache(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/disk/cache');
  }
  
  /**
   * 配置磁盘缓存
   */
  async configureDiskCache(enabled: boolean): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/disk/cache', { enabled });
  }
  
  /**
   * 获取挂载选项
   */
  async getMountOptions(): Promise<any> {
    return apiCall(HttpMethod.GET, '/performance/disk/mount-options');
  }
  
  /**
   * 设置挂载选项
   */
  async setMountOptions(options: any): Promise<void> {
    return apiCall(HttpMethod.PUT, '/performance/disk/mount-options', options);
  }
  
  /**
   * 磁盘碎片整理
   */
  async defragmentDisk(): Promise<any> {
    return apiCall(HttpMethod.POST, '/performance/disk/defrag');
  }
  
  /**
   * SSD TRIM
   */
  async trimSsd(): Promise<any> {
    return apiCall(HttpMethod.POST, '/performance/disk/trim');
  }
  
  /**
   * 运行磁盘基准测试
   */
  async benchmarkDisk(): Promise<any> {
    return apiCall(HttpMethod.POST, '/performance/disk/benchmark');
  }
  
  /**
   * 优化磁盘性能
   */
  async optimizeDisk(): Promise<any> {
    return apiCall(HttpMethod.POST, '/performance/disk/optimize');
  }
}

// 导出单例实例
export const performanceService = new PerformanceService(); 