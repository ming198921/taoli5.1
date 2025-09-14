// 统一的系统控制服务层
// 支持多种部署环境：本地systemd、AWS ECS、Kubernetes

export enum DeploymentType {
  SYSTEMD = 'systemd',
  ECS = 'ecs', 
  K8S = 'k8s',
  DIRECT = 'direct' // 直接进程控制
}

export interface SystemModule {
  name: string;
  status: 'running' | 'stopped' | 'starting' | 'stopping' | 'error';
  health?: 'healthy' | 'unhealthy' | 'unknown';
  lastHeartbeat?: number;
  metrics?: {
    cpu: number;
    memory: number;
    requests: number;
  };
}

export interface ControlResponse {
  success: boolean;
  message: string;
  data?: any;
}

// 抽象控制接口
interface SystemController {
  start(module: string): Promise<ControlResponse>;
  stop(module: string): Promise<ControlResponse>;
  restart(module: string): Promise<ControlResponse>;
  status(module: string): Promise<SystemModule>;
  logs(module: string, lines?: number): Promise<string[]>;
  updateConfig(module: string, config: any): Promise<ControlResponse>;
}

// Systemd控制器实现
class SystemdController implements SystemController {
  private baseUrl: string;
  
  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }
  
  async start(module: string): Promise<ControlResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/api/control/systemd/start`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ service: `arbitrage-${module}.service` })
      });
      return await response.json();
    } catch (error) {
      return {
        success: false,
        message: `Failed to start ${module}: ${error}`
      };
    }
  }
  
  async stop(module: string): Promise<ControlResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/api/control/systemd/stop`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ service: `arbitrage-${module}.service` })
      });
      return await response.json();
    } catch (error) {
      return {
        success: false,
        message: `Failed to stop ${module}: ${error}`
      };
    }
  }
  
  async restart(module: string): Promise<ControlResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/api/control/systemd/restart`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ service: `arbitrage-${module}.service` })
      });
      return await response.json();
    } catch (error) {
      return {
        success: false,
        message: `Failed to restart ${module}: ${error}`
      };
    }
  }
  
  async status(module: string): Promise<SystemModule> {
    try {
      const response = await fetch(`${this.baseUrl}/api/control/systemd/status`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
      });
      const data = await response.json();
      return data.data;
    } catch (error) {
      return {
        name: module,
        status: 'error',
        health: 'unknown'
      };
    }
  }
  
  async logs(module: string, lines: number = 100): Promise<string[]> {
    try {
      const response = await fetch(
        `${this.baseUrl}/api/control/systemd/logs?service=arbitrage-${module}.service&lines=${lines}`
      );
      const data = await response.json();
      return data.logs || [];
    } catch (error) {
      return [`Error fetching logs: ${error}`];
    }
  }
  
  async updateConfig(module: string, config: any): Promise<ControlResponse> {
    // Systemd服务通常通过重新加载配置文件和重启服务来更新
    try {
      // 1. 更新配置文件
      await fetch(`${this.baseUrl}/api/config/update`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ module, config })
      });
      
      // 2. 重启服务以应用新配置
      return await this.restart(module);
    } catch (error) {
      return {
        success: false,
        message: `Failed to update config: ${error}`
      };
    }
  }
}

// ECS控制器实现
class ECSController implements SystemController {
  private baseUrl: string;
  private cluster: string;
  
  constructor(baseUrl: string, cluster: string = 'arbitrage-cluster') {
    this.baseUrl = baseUrl;
    this.cluster = cluster;
  }
  
  async start(module: string): Promise<ControlResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/api/control/ecs/services`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          cluster: this.cluster,
          service: `arbitrage-${module}`,
          desiredCount: 1
        })
      });
      return await response.json();
    } catch (error) {
      return {
        success: false,
        message: `Failed to start ECS service: ${error}`
      };
    }
  }
  
  async stop(module: string): Promise<ControlResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/api/control/ecs/services`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          cluster: this.cluster,
          service: `arbitrage-${module}`,
          desiredCount: 0
        })
      });
      return await response.json();
    } catch (error) {
      return {
        success: false,
        message: `Failed to stop ECS service: ${error}`
      };
    }
  }
  
  async restart(module: string): Promise<ControlResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/api/control/ecs/restart`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          cluster: this.cluster,
          service: `arbitrage-${module}`
        })
      });
      return await response.json();
    } catch (error) {
      return {
        success: false,
        message: `Failed to restart ECS service: ${error}`
      };
    }
  }
  
  async status(module: string): Promise<SystemModule> {
    try {
      const response = await fetch(
        `${this.baseUrl}/api/control/ecs/services/${this.cluster}/${module}`
      );
      const data = await response.json();
      
      return {
        name: module,
        status: data.runningCount > 0 ? 'running' : 'stopped',
        health: data.healthStatus || 'unknown',
        metrics: {
          cpu: data.cpuUtilization || 0,
          memory: data.memoryUtilization || 0,
          requests: 0
        }
      };
    } catch (error) {
      return {
        name: module,
        status: 'error',
        health: 'unknown'
      };
    }
  }
  
  async logs(module: string, lines: number = 100): Promise<string[]> {
    try {
      const response = await fetch(
        `${this.baseUrl}/api/control/ecs/logs?cluster=${this.cluster}&service=${module}&lines=${lines}`
      );
      const data = await response.json();
      return data.logs || [];
    } catch (error) {
      return [`Error fetching logs: ${error}`];
    }
  }
  
  async updateConfig(module: string, config: any): Promise<ControlResponse> {
    try {
      // ECS使用环境变量或Parameter Store
      const response = await fetch(`${this.baseUrl}/api/control/ecs/update-task`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          cluster: this.cluster,
          service: `arbitrage-${module}`,
          environment: config
        })
      });
      return await response.json();
    } catch (error) {
      return {
        success: false,
        message: `Failed to update ECS config: ${error}`
      };
    }
  }
}

// K8s控制器实现
class K8sController implements SystemController {
  private baseUrl: string;
  private namespace: string;
  
  constructor(baseUrl: string, namespace: string = 'arbitrage') {
    this.baseUrl = baseUrl;
    this.namespace = namespace;
  }
  
  async start(module: string): Promise<ControlResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/api/control/k8s/scale`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          namespace: this.namespace,
          deployment: `arbitrage-${module}`,
          replicas: 1
        })
      });
      return await response.json();
    } catch (error) {
      return {
        success: false,
        message: `Failed to start K8s deployment: ${error}`
      };
    }
  }
  
  async stop(module: string): Promise<ControlResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/api/control/k8s/scale`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          namespace: this.namespace,
          deployment: `arbitrage-${module}`,
          replicas: 0
        })
      });
      return await response.json();
    } catch (error) {
      return {
        success: false,
        message: `Failed to stop K8s deployment: ${error}`
      };
    }
  }
  
  async restart(module: string): Promise<ControlResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/api/control/k8s/rollout-restart`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          namespace: this.namespace,
          deployment: `arbitrage-${module}`
        })
      });
      return await response.json();
    } catch (error) {
      return {
        success: false,
        message: `Failed to restart K8s deployment: ${error}`
      };
    }
  }
  
  async status(module: string): Promise<SystemModule> {
    try {
      const response = await fetch(
        `${this.baseUrl}/api/control/k8s/deployments/${this.namespace}/${module}`
      );
      const data = await response.json();
      
      return {
        name: module,
        status: data.readyReplicas > 0 ? 'running' : 'stopped',
        health: data.conditions?.find((c: any) => c.type === 'Available')?.status === 'True' 
          ? 'healthy' : 'unhealthy',
        metrics: {
          cpu: data.metrics?.cpu || 0,
          memory: data.metrics?.memory || 0,
          requests: data.metrics?.requests || 0
        }
      };
    } catch (error) {
      return {
        name: module,
        status: 'error',
        health: 'unknown'
      };
    }
  }
  
  async logs(module: string, lines: number = 100): Promise<string[]> {
    try {
      const response = await fetch(
        `${this.baseUrl}/api/control/k8s/logs?namespace=${this.namespace}&deployment=${module}&lines=${lines}`
      );
      const data = await response.json();
      return data.logs || [];
    } catch (error) {
      return [`Error fetching logs: ${error}`];
    }
  }
  
  async updateConfig(module: string, config: any): Promise<ControlResponse> {
    try {
      // K8s使用ConfigMap
      const response = await fetch(`${this.baseUrl}/api/control/k8s/configmap`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          namespace: this.namespace,
          name: `arbitrage-${module}-config`,
          data: config
        })
      });
      
      if (response.ok) {
        // 触发滚动更新以应用新配置
        return await this.restart(module);
      }
      
      return await response.json();
    } catch (error) {
      return {
        success: false,
        message: `Failed to update K8s config: ${error}`
      };
    }
  }
}

// 直接进程控制（当前实现）
class DirectController implements SystemController {
  private baseUrl: string;
  
  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }
  
  async start(module: string): Promise<ControlResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/api/system/start`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
      });
      return await response.json();
    } catch (error) {
      return {
        success: false,
        message: `Failed to start: ${error}`
      };
    }
  }
  
  async stop(module: string): Promise<ControlResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/api/system/stop`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
      });
      return await response.json();
    } catch (error) {
      return {
        success: false,
        message: `Failed to stop: ${error}`
      };
    }
  }
  
  async restart(module: string): Promise<ControlResponse> {
    await this.stop(module);
    await new Promise(resolve => setTimeout(resolve, 2000));
    return await this.start(module);
  }
  
  async status(module: string): Promise<SystemModule> {
    try {
      const response = await fetch(`${this.baseUrl}/api/system/status`);
      const data = await response.json();
      return {
        name: module,
        status: data.isRunning ? 'running' : 'stopped',
        health: data.isRunning ? 'healthy' : 'unknown'
      };
    } catch (error) {
      return {
        name: module,
        status: 'error',
        health: 'unknown'
      };
    }
  }
  
  async logs(module: string, lines: number = 100): Promise<string[]> {
    try {
      const response = await fetch(`${this.baseUrl}/api/system/logs?lines=${lines}`);
      const data = await response.json();
      return data.logs || [];
    } catch (error) {
      return [`Error fetching logs: ${error}`];
    }
  }
  
  async updateConfig(module: string, config: any): Promise<ControlResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/api/config/update`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config)
      });
      return await response.json();
    } catch (error) {
      return {
        success: false,
        message: `Failed to update config: ${error}`
      };
    }
  }
}

// 工厂类：根据环境自动选择控制器
export class SystemControlService {
  private controller: SystemController;
  private deploymentType: DeploymentType;
  
  constructor() {
    // 从环境变量或配置文件读取部署类型
    this.deploymentType = this.detectDeploymentType();
    this.controller = this.createController();
  }
  
  private detectDeploymentType(): DeploymentType {
    // 可以从环境变量或配置API获取
    const envType = process.env.REACT_APP_DEPLOYMENT_TYPE;
    
    if (envType && Object.values(DeploymentType).includes(envType as DeploymentType)) {
      return envType as DeploymentType;
    }
    
    // 默认使用直接控制
    return DeploymentType.DIRECT;
  }
  
  private createController(): SystemController {
    const baseUrl = process.env.REACT_APP_API_URL || 'http://localhost:8080';
    
    switch (this.deploymentType) {
      case DeploymentType.SYSTEMD:
        return new SystemdController(baseUrl);
      case DeploymentType.ECS:
        return new ECSController(baseUrl, process.env.REACT_APP_ECS_CLUSTER || 'default');
      case DeploymentType.K8S:
        return new K8sController(baseUrl, process.env.REACT_APP_K8S_NAMESPACE || 'default');
      case DeploymentType.DIRECT:
      default:
        return new DirectController(baseUrl);
    }
  }
  
  // 公共方法，直接委托给具体控制器
  async startModule(module: string): Promise<ControlResponse> {
    console.log(`Starting module ${module} using ${this.deploymentType} controller`);
    return this.controller.start(module);
  }
  
  async stopModule(module: string): Promise<ControlResponse> {
    console.log(`Stopping module ${module} using ${this.deploymentType} controller`);
    return this.controller.stop(module);
  }
  
  async restartModule(module: string): Promise<ControlResponse> {
    console.log(`Restarting module ${module} using ${this.deploymentType} controller`);
    return this.controller.restart(module);
  }
  
  async getModuleStatus(module: string): Promise<SystemModule> {
    return this.controller.status(module);
  }
  
  async getModuleLogs(module: string, lines?: number): Promise<string[]> {
    return this.controller.logs(module, lines);
  }
  
  async updateModuleConfig(module: string, config: any): Promise<ControlResponse> {
    console.log(`Updating config for ${module} using ${this.deploymentType} controller`);
    return this.controller.updateConfig(module, config);
  }
  
  // 批量操作
  async startAllModules(modules: string[]): Promise<Map<string, ControlResponse>> {
    const results = new Map<string, ControlResponse>();
    
    for (const module of modules) {
      results.set(module, await this.startModule(module));
    }
    
    return results;
  }
  
  async stopAllModules(modules: string[]): Promise<Map<string, ControlResponse>> {
    const results = new Map<string, ControlResponse>();
    
    for (const module of modules) {
      results.set(module, await this.stopModule(module));
    }
    
    return results;
  }
  
  async getAllModuleStatuses(modules: string[]): Promise<SystemModule[]> {
    const statuses: SystemModule[] = [];
    
    for (const module of modules) {
      statuses.push(await this.getModuleStatus(module));
    }
    
    return statuses;
  }
  
  // 获取当前部署类型
  getDeploymentType(): DeploymentType {
    return this.deploymentType;
  }
}

// 导出单例
const systemControl = new SystemControlService();
export default systemControl;