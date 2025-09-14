# 5.1套利系统前端控制界面 (v2.0)

基于 **Cruip Tailwind Dashboard** 模板重构的5.1高频套利系统前端管理界面，采用现代化技术栈，提供稳定、高性能的系统控制面板。

## 🚀 核心特性

- ✅ **现代化技术栈**: React 19 + Tailwind CSS v4 + Vite 6
- ✅ **完整API集成**: 统一的API服务层，完全兼容后端接口
- ✅ **实时数据同步**: 5秒间隔自动刷新系统状态和性能指标
- ✅ **响应式设计**: 支持桌面端和移动端，完美适配各种屏幕尺寸
- ✅ **模块化架构**: 6大核心模块独立管理
- ✅ **高性能优化**: 组件懒加载、代码分割、缓存优化
- ✅ **错误边界**: 完善的错误处理和用户友好的错误提示

Created and maintained for 5.1套利系统 | Based on Cruip Tailwind Dashboard Template

## 🎯 访问地址

- **开发环境**: http://57.183.21.242:3000
- **后端API**: http://57.183.21.242:8080
- **本地开发**: http://localhost:3000

## 📋 系统架构

### 技术栈对比

| 组件 | 旧版本 | 新版本 | 优势 |
|------|--------|--------|------|
| **React** | 18.2.0 | 19.0.0 | 新特性、性能优化 |
| **样式框架** | Ant Design | Tailwind CSS v4 | 更轻量、更灵活 |
| **构建工具** | Vite 4.4.5 | Vite 6.3.5 | 更快的构建速度 |
| **状态管理** | Redux Toolkit | React Query + Context | 服务端状态管理优化 |
| **路由** | React Router 6.15.0 | React Router 7.0.2 | 最新路由特性 |

### 6大核心模块

1. **系统控制** (`/system`) - 系统启停控制、实时状态监控、性能指标
2. **清洗模块** (`/qingxi`) - 数据收集器管理、数据质量监控  
3. **策略模块** (`/celue`) - 套利策略管理、执行监控、收益分析
4. **风险管理** (`/risk`) - 风险指标监控、预警管理、资金安全
5. **架构监控** (`/architecture`) - 系统架构监控、服务健康检查
6. **可观测性** (`/observability`) - 日志聚合、链路追踪、告警管理

## 🛠️ 快速开始

### 系统要求

- Node.js >= 18.0.0
- npm >= 9.0.0
- 后端API服务运行在 `http://57.183.21.242:8080`

### 安装和运行

```bash
# 1. 进入项目目录
cd /home/ubuntu/arbitrage-frontend-v5.1/qianduan1

# 2. 安装依赖
npm install

# 3. 启动开发服务器 (端口 3000)
npm run dev

# 4. 访问应用
# 本地: http://localhost:3000
# 网络: http://57.183.21.242:3000
```

### 可用脚本

```bash
npm run dev        # 启动开发服务器 (端口 3000)
npm run build      # 构建生产版本
npm run preview    # 预览生产版本 (端口 4001)
npm run lint       # 代码检查
```

### 项目结构

```
qianduan1/
├── src/
│   ├── components/           # 可复用组件
│   ├── pages/               # 页面组件
│   │   ├── system/          # 系统控制页面
│   │   ├── qingxi/          # 清洗模块页面
│   │   ├── celue/           # 策略模块页面
│   │   ├── risk/            # 风险管理页面
│   │   ├── architecture/    # 架构监控页面
│   │   └── observability/   # 可观测性页面
│   ├── services/            # API服务层
│   │   └── api.js           # 统一API接口
│   ├── utils/               # 工具函数
│   │   ├── constants.js     # 常量定义
│   │   └── helpers.js       # 辅助函数
│   └── store/               # 状态管理
├── package.json             # 依赖配置
├── vite.config.js          # Vite构建配置
└── .env                    # 环境变量配置
```

## 🔧 API集成说明

### 统一API服务层

新版前端采用统一的API服务层设计，所有API调用都通过 `src/services/api.js` 进行：

```javascript
import apiService from '@/services/api.js';

// 系统控制API
const systemStatus = await apiService.system.getStatus();
await apiService.system.start();
await apiService.system.stop();

// 风险管理API
const riskStatus = await apiService.risk.getStatus();
const riskMetrics = await apiService.risk.getMetrics();
```

### 代理配置

开发环境通过Vite代理自动转发API请求到后端服务器：

```javascript
// vite.config.js
server: {
  proxy: {
    '/api': {
      target: 'http://57.183.21.242:8080',
      changeOrigin: true,
      secure: false
    }
  }
}
```

## 🚀 部署说明

### 开发环境 (当前状态)

- **前端地址**: http://57.183.21.242:3000
- **后端API**: http://57.183.21.242:8080  
- **状态**: ✅ 运行中

### 生产环境部署

```bash
# 1. 构建生产版本
npm run build

# 2. 部署到Web服务器
# dist目录包含所有构建产物
```

## 📈 性能优化特性

- **代码分割**: 自动分包，vendor、charts、utils分离
- **缓存策略**: React Query 5分钟缓存 + 5秒自动刷新
- **懒加载**: 组件级别的懒加载
- **响应式**: 完美适配移动端和桌面端

## 🐛 故障排除

### 常见问题

1. **API连接失败**: 检查后端服务是否运行在8080端口
2. **端口冲突**: 修改package.json中的端口配置
3. **依赖安装失败**: 清除node_modules重新安装

## 📝 版本历史

### v2.0.0 (2025-09-06) - 当前版本
- 🆕 基于Cruip Tailwind Dashboard模板完全重构
- 🆕 React 19 + Tailwind CSS v4 + Vite 6技术栈
- 🆕 统一API服务层，完善错误处理
- 🆕 实时数据同步，5秒自动刷新系统状态
- 🆕 响应式设计，移动端适配
- 🆕 系统控制模块完整实现
- 🆕 性能优化，代码分割

### v1.x (历史版本)
- React 18 + Ant Design架构
- 基础系统控制功能
- 简单的API对接

---

## 🙏 致谢

基于 [Cruip Tailwind Dashboard Template](https://github.com/cruip/tailwind-dashboard-template) 构建

**5.1套利系统开发团队** | 最后更新: 2025-09-06
