#!/usr/bin/env python3
"""修复最终的编译错误"""

import re

# 1. 修复performance/mod.rs中的重复PerformanceMetrics
with open('src/performance/mod.rs', 'r') as f:
    content = f.read()

# 删除第二个和第三个PerformanceMetrics定义
first_metrics = content.find('#[derive(Debug, Clone)]\npub struct PerformanceMetrics')
if first_metrics > 0:
    second_metrics = content.find('#[derive(Debug, Clone)]\npub struct PerformanceMetrics', first_metrics + 1)
    if second_metrics > 0:
        # 找到第二个定义的结束
        second_end = content.find('\n}\n\n', second_metrics)
        if second_end > 0:
            content = content[:second_metrics] + content[second_end + 4:]

with open('src/performance/mod.rs', 'w') as f:
    f.write(content)

# 2. 修复opportunity_pool.rs中的重复定义
with open('src/strategy/opportunity_pool.rs', 'r') as f:
    content = f.read()

# 删除use语句中的重复
content = re.sub(r'use std::sync::Arc;\nuse tokio::sync::RwLock;\nuse serde::\{Deserialize, Serialize\};\nuse chrono::\{DateTime, Utc\};\nuse std::cmp::Ordering;\n.*\n.*\n', '', content)

with open('src/strategy/opportunity_pool.rs', 'w') as f:
    f.write(content)

# 3. 修复adaptive_profit.rs中的RandomForestRegressor类型
with open('src/strategy/adaptive_profit.rs', 'r') as f:
    content = f.read()

# 修复RandomForestRegressor泛型参数
content = content.replace(
    'model: Option<RandomForestRegressor<f64>>',
    'model: Option<Box<dyn std::any::Any + Send + Sync>>'
)

with open('src/strategy/adaptive_profit.rs', 'w') as f:
    f.write(content)

# 4. 添加缺失的StrategyError variants到core.rs
with open('src/strategy/core.rs', 'r') as f:
    content = f.read()

# 在StrategyError enum中添加缺失的variants
error_enum_start = content.find('pub enum StrategyError {')
if error_enum_start > 0:
    error_enum_end = content.find('ValidationError(String),', error_enum_start)
    if error_enum_end > 0:
        insert_pos = error_enum_end + len('ValidationError(String),')
        new_variants = '''
    InsufficientData(String),
    ModelNotFound(String),'''
        content = content[:insert_pos] + new_variants + content[insert_pos:]

# 更新Display实现
display_impl_start = content.find('impl std::fmt::Display for StrategyError {')
if display_impl_start > 0:
    match_start = content.find('match self {', display_impl_start)
    match_end = content.find('StrategyError::ValidationError(msg)', match_start)
    if match_end > 0:
        insert_pos = content.find('\n', match_end) + 1
        new_matches = '''            StrategyError::InsufficientData(msg) => write!(f, "Insufficient data: {}", msg),
            StrategyError::ModelNotFound(msg) => write!(f, "Model not found: {}", msg),
'''
        content = content[:insert_pos] + new_matches + content[insert_pos:]

with open('src/strategy/core.rs', 'w') as f:
    f.write(content)

print("修复完成！") 