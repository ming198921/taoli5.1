# 🚀 5.1套利系统微服务自动化 - 快速使用指南

## ✅ 系统验证结果

**测试时间**: 2025-09-10 02:45
**测试状态**: ✅ 全部通过
**自动修复测试**: ✅ 成功

### 系统状态
- **8个微服务**: 全部健康运行
- **前端界面**: 正常访问 (本地 + 外网)
- **API接口**: 387个接口正常
- **自动修复**: 功能验证通过

## 🌐 访问地址

### 前端Dashboard
- **本地访问**: http://localhost:3003/dashboard
- **外网访问**: http://57.183.21.242:3003/dashboard

### API服务
- **统一网关**: http://localhost:3000/health
- **API服务器**: http://localhost:3001/health

## 🛠️ 管理工具使用

### 1. 系统综合仪表板 (推荐)
```bash
cd /home/ubuntu/5.1xitong
./system-dashboard.sh
```
**功能**: 交互式管理界面，一键操作

### 2. 服务管理器
```bash
./auto-service-manager.sh status    # 检查状态
./auto-service-manager.sh start     # 启动所有服务
./auto-service-manager.sh restart   # 重启所有服务
./auto-service-manager.sh repair    # 自动修复
```

### 3. 诊断工具
```bash
node microservice-diagnostic-tool.js status    # 详细健康检查
node microservice-diagnostic-tool.js monitor   # 持续监控
node microservice-diagnostic-tool.js repair    # 自动修复
```

### 4. 自愈式监控
```bash
python3 self-healing-monitor.py status  # JSON格式状态
python3 self-healing-monitor.py repair  # Python自动修复
```

## 🔧 常用操作

### 快速检查系统状态
```bash
./system-dashboard.sh status
```

### 一键启动所有服务
```bash
./auto-service-manager.sh start
```

### 自动修复故障
```bash
./system-dashboard.sh repair
```

### 生成系统报告
```bash
./system-dashboard.sh report
```

### 清理日志文件
```bash
./system-dashboard.sh cleanup
```

## 🚨 故障排除

### 如果前端无法访问
```bash
# 检查前端状态
ss -tlnp | grep ":3003"

# 重启前端
cd /home/ubuntu/arbitrage-frontend-v5.1
./start-frontend.sh
```

### 如果微服务异常
```bash
# 自动修复
./auto-service-manager.sh repair

# 或重启所有服务
./auto-service-manager.sh restart
```

### 如果系统负载高
```bash
# 查看资源使用
node microservice-diagnostic-tool.js resources

# 清理系统
./system-dashboard.sh cleanup
```

## 📊 监控建议

### 生产环境
- 建议每30-60秒检查一次
- 启用自动修复功能
- 定期生成系统报告

### 启动持续监控
```bash
node microservice-diagnostic-tool.js monitor 30000
```

## 🎯 核心服务说明

### 关键服务 (Critical)
- **config-service** (4007): 配置管理
- **logging-service** (4001): 日志服务
- **unified-gateway** (3000): 统一网关
- **trading-service** (4005): 交易服务
- **strategy-service** (4003): 策略服务

### 普通服务 (Normal)
- **cleaning-service** (4002): 清算服务
- **performance-service** (4004): 性能监控
- **ai-model-service** (4006): AI模型服务

## 🔐 安全提醒

1. 定期备份配置和日志
2. 监控系统资源使用
3. 及时更新系统依赖
4. 检查防火墙设置

## 📱 移动端访问

可通过手机浏览器访问:
**http://57.183.21.242:3003/dashboard**

## 📞 技术支持

### 日志文件位置
```
/home/ubuntu/5.1xitong/logs/
├── auto-service-manager.log
├── self-healing-monitor.log
├── system_report_*.txt
└── auto_repair_test_*.txt
```

### 收集支持信息
```bash
./system-dashboard.sh report
tar -czf support_$(date +%Y%m%d).tar.gz /home/ubuntu/5.1xitong/logs/
```

---

## 🎉 恭喜！

**5.1套利系统微服务自动化诊断与修复平台已成功部署并通过全面测试！**

**系统特性**:
- ✅ 8个微服务100%健康运行
- ✅ 387个API接口统一管理
- ✅ 自动故障检测与修复
- ✅ 实时监控与告警
- ✅ 前端Dashboard完美运行
- ✅ 外网访问正常
- ✅ 自动修复功能验证通过

**推荐开始使用**: `./system-dashboard.sh`

---
*最后更新: 2025-09-10*  
*状态: 生产就绪 🚀*