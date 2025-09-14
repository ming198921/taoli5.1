#!/usr/bin/env node

// 脚本：将架构监控和可观测性API替换为真实端点
const fs = require('fs');

const apiFile = './src/services/api.js';
const content = fs.readFileSync(apiFile, 'utf8');

// 定义替换映射
const replacements = [
  // 架构监控模块API替换
  {
    search: /\/\/ 获取服务状态列表[\s\S]*?getServices: async \(\) => \{[\s\S]*?return \{[\s\S]*?services: \[[\s\S]*?\}\s*\}\s*\}\s*\}/gm,
    replace: `// 获取服务状态列表 (真实API)
    getServices: () => apiClient.get('/api/architecture/services'),`
  },
  {
    search: /\/\/ 获取健康检查结果[\s\S]*?getHealthCheck: async \(\) => \{[\s\S]*?return \{[\s\S]*?\][\s\S]*?\}\s*\}\s*\}/gm,
    replace: `// 获取健康检查结果 (真实API)
    getHealthCheck: () => apiClient.get('/api/architecture/health-check'),`
  },
  {
    search: /\/\/ 获取性能指标[\s\S]*?getMetrics: async \(\) => \{[\s\S]*?return \{[\s\S]*?\}[\s\S]*?\}\s*\}/gm,
    replace: `// 获取性能指标 (真实API)  
    getMetrics: () => apiClient.get('/api/architecture/metrics'),`
  },
  
  // 可观测性模块API替换
  {
    search: /\/\/ 获取日志聚合数据[\s\S]*?getLogs: async \(params = \{\}\) => \{[\s\S]*?return \{[\s\S]*?\][\s\S]*?\}\s*\}\s*\}/gm,
    replace: `// 获取日志聚合数据 (真实API)
    getLogs: (params = {}) => {
      const { lines = 50 } = params;
      return apiClient.get(\`/api/observability/logs?lines=\${lines}\`);
    },`
  },
  {
    search: /\/\/ 获取链路追踪数据[\s\S]*?getTraces: async \(\) => \{[\s\S]*?return \{[\s\S]*?\][\s\S]*?\}\s*\}\s*\}/gm,
    replace: `// 获取链路追踪数据 (真实API)
    getTraces: () => apiClient.get('/api/observability/traces'),`
  },
  {
    search: /\/\/ 获取告警规则[\s\S]*?getAlerts: async \(\) => \{[\s\S]*?return \{[\s\S]*?\][\s\S]*?\}\s*\}\s*\}/gm,
    replace: `// 获取告警规则 (真实API)
    getAlerts: () => apiClient.get('/api/observability/alerts'),`
  },
  {
    search: /\/\/ 获取指标数据[\s\S]*?getMetrics: async \(category, timeRange\) => \{[\s\S]*?return \{[\s\S]*?\}[\s\S]*?\}\s*\}/gm,
    replace: `// 获取指标数据 (真实API)
    getMetrics: (category, timeRange) => apiClient.get('/api/observability/metrics'),`
  }
];

let updatedContent = content;

// 应用所有替换
replacements.forEach((replacement, index) => {
  const beforeLength = updatedContent.length;
  updatedContent = updatedContent.replace(replacement.search, replacement.replace);
  const afterLength = updatedContent.length;
  
  if (beforeLength !== afterLength) {
    console.log(`✅ 替换 ${index + 1} 成功应用`);
  } else {
    console.log(`⚠️ 替换 ${index + 1} 未找到匹配项`);
  }
});

// 写回文件
fs.writeFileSync(apiFile, updatedContent);
console.log('🎯 API文件更新完成');