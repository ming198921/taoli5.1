# 5.1系统模块重构路线图

## 🎯 重构目标
移除配置转换层，实现各模块直接使用统一的`ConfigCenter`，确保三个模块紧密联系且易于管理。

## 📋 重构任务清单

### 1. Qingxi数据处理模块重构 ❌ 
**当前状态：** 配置加载失败，模块未启动
**目标：** 直接使用`ConfigCenter`而非`Settings::load()`

#### 需要修改的文件：
- `qingxi/qingxi/src/central_manager.rs`
- `qingxi/qingxi/src/settings.rs` 
- `qingxi/qingxi/src/main.rs`

#### 具体重构任务：
```rust
// 当前方式 (需要移除)
let settings = Settings::load()?;
let (manager, handle) = CentralManager::new(&settings);

// 目标方式 (需要实现)
let config_center = Arc<ConfigCenter>::clone(&shared_config);
let (manager, handle) = CentralManager::new_with_config_center(config_center);
```

#### 重构步骤：
1. 修改`CentralManager::new()`签名，接受`Arc<ConfigCenter>`参数
2. 在`CentralManager`内部直接调用`config_center.get_exchange_configs()`等方法
3. 移除对`settings.rs`的依赖
4. 更新`src/main.rs`中的启动逻辑

### 2. Celue策略执行模块重构 ⚠️
**当前状态：** 使用临时配置模式启动成功
**目标：** 直接使用`ConfigCenter`而非`SystemConfig::default()`

#### 需要修改的文件：
- `celue/orchestrator/src/engine.rs`
- `celue/orchestrator/src/config.rs`
- `celue/strategy/src/plugins/triangular.rs`

#### 具体重构任务：
```rust
// 当前方式 (需要移除)  
let celue_config = CelueSystemConfig::default();
let engine = ConfigurableArbitrageEngine::new(&celue_config, strategy_context);

// 目标方式 (需要实现)
let config_center = Arc<ConfigCenter>::clone(&shared_config);
let engine = ConfigurableArbitrageEngine::new_with_config_center(config_center, strategy_context);
```

#### 重构步骤：
1. 修改`ConfigurableArbitrageEngine::new()`签名
2. 在引擎内部调用`config_center.get_strategy_configs()`
3. 删除或重构`CelueSystemConfig`以减少配置重复
4. 确保策略配置直接从`ConfigCenter`读取

### 3. AI风控模块重构 ⚠️
**当前状态：** 使用临时配置模式启动成功  
**目标：** 直接使用`ConfigCenter`而非`SystemConfig::default()`

#### 需要修改的文件：
- `celue/orchestrator/src/risk.rs`
- 相关风控算法文件

#### 具体重构任务：
```rust
// 当前方式 (需要移除)
let celue_config = CelueSystemConfig::default();
let risk_controller = Arc::new(DynamicRiskController::from_system_config(&celue_config));

// 目标方式 (需要实现)
let config_center = Arc<ConfigCenter>::clone(&shared_config);
let risk_controller = Arc::new(DynamicRiskController::new_with_config_center(config_center));
```

#### 重构步骤：
1. 修改`DynamicRiskController::from_system_config()`方法
2. 在风控器内部调用`config_center.get_risk_config()`
3. 确保所有风险参数直接从`ConfigCenter`读取
4. 移除对`CelueSystemConfig`的依赖

## 🔧 重构实施原则

### 1. 统一配置来源
- 所有模块**必须**直接使用`ConfigCenter`
- 禁止中间配置转换层
- 禁止硬编码和占位符

### 2. 配置接口设计
```rust
// 推荐的模块初始化模式
pub struct ModuleName {
    config: Arc<ConfigCenter>,
    // ... other fields
}

impl ModuleName {
    pub fn new(config: Arc<ConfigCenter>) -> Self {
        // 直接从ConfigCenter获取所需配置
        let module_config = config.get_module_specific_config().await;
        Self { config, /* ... */ }
    }
}
```

### 3. 配置热更新支持
- 各模块应支持通过`ConfigCenter`的配置变更通知
- 实现配置热重载机制
- 确保配置变更的原子性和一致性

## 📊 重构优先级

1. **高优先级：** Qingxi模块 - 当前未启动，影响整个数据流
2. **中优先级：** Celue模块 - 已启动但需要完善配置集成
3. **中优先级：** AI风控模块 - 已启动但需要完善配置集成

## ✅ 验收标准

### 重构完成后系统应满足：
1. 所有三个模块成功启动 ✅
2. 配置完全统一，无转换层 ✅
3. 支持配置热更新 ✅
4. 模块间紧密协作 ✅
5. 代码结构清晰易维护 ✅

## 🎯 最终目标架构

```
ConfigCenter (统一配置源)
    ├── Qingxi模块 (直接读取)
    ├── Celue模块 (直接读取)  
    └── AI风控模块 (直接读取)
```

**无中间层，无转换器，配置直达各模块！** 