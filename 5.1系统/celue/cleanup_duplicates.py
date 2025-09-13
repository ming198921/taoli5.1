#!/usr/bin/env python3
"""清理重复的代码定义"""

import re
import os

def remove_duplicate_blocks(file_path):
    """删除文件中的重复代码块"""
    with open(file_path, 'r') as f:
        content = f.read()
    
    # 删除simd_fixed_point.rs中从第二个"use crossbeam::queue::ArrayQueue;"开始的重复块
    if file_path.endswith('lockfree_structures.rs'):
        # 找到第一个完整的实现块的结束
        first_impl_end = content.find('} \n\nuse crossbeam::queue::ArrayQueue;')
        if first_impl_end > 0:
            # 保留第一个实现，删除后面的重复
            content = content[:first_impl_end + 2]  # 保留 "} "
            print(f"清理了 {file_path} 中的重复定义")
    
    # 写回文件
    with open(file_path, 'w') as f:
        f.write(content)

def fix_strategy_error_display(file_path):
    """修复StrategyError的Display实现"""
    with open(file_path, 'r') as f:
        content = f.read()
    
    # 找到Display实现
    display_start = content.find('impl fmt::Display for StrategyError {')
    if display_start > 0:
        # 找到match语句
        match_start = content.find('match self {', display_start)
        match_end = content.find('}\n    }', match_start)
        
        if match_start > 0 and match_end > 0:
            # 添加缺失的match分支
            new_match = '''match self {
            StrategyError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
            StrategyError::InsufficientLiquidity => write!(f, "Insufficient liquidity"),
            StrategyError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            StrategyError::RiskLimitExceeded(msg) => write!(f, "Risk limit exceeded: {}", msg),
            StrategyError::MarketDataStale => write!(f, "Market data is stale"),
            StrategyError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            StrategyError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            StrategyError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            StrategyError::StrategyDisabled => write!(f, "Strategy disabled"),
            StrategyError::ModelTrainingError(msg) => write!(f, "Model training error: {}", msg),
            StrategyError::PredictionError(msg) => write!(f, "Prediction error: {}", msg),
            StrategyError::FeatureEngineeringError(msg) => write!(f, "Feature engineering error: {}", msg),
            StrategyError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }'''
            content = content[:match_start] + new_match + content[match_end:]
            print(f"修复了 {file_path} 中的Display实现")
    
    with open(file_path, 'w') as f:
        f.write(content)

def fix_adaptive_profit_types(file_path):
    """修复adaptive_profit.rs中的类型错误"""
    with open(file_path, 'r') as f:
        content = f.read()
    
    # 修复f64到f32的转换
    content = content.replace('hyperparams.min_samples_split as f64', 'hyperparams.min_samples_split as f32')
    content = content.replace('hyperparams.min_samples_leaf as f64', 'hyperparams.min_samples_leaf as f32')
    
    # 修复DenseMatrix转换
    old_matrix = 'let x_train = DenseMatrix::from_2d_array(&features.view().to_owned());'
    new_matrix = '''let feature_rows: Vec<Vec<f64>> = features.outer_iter()
                    .map(|row| row.to_vec())
                    .collect();
                let feature_refs: Vec<&[f64]> = feature_rows.iter()
                    .map(|row| row.as_slice())
                    .collect();
                let x_train = DenseMatrix::from_2d_array(&feature_refs);'''
    content = content.replace(old_matrix, new_matrix)
    
    # 修复max_depth类型
    content = content.replace(
        '.with_max_depth(hyperparams.max_depth.unwrap_or(10))',
        '.with_max_depth(hyperparams.max_depth.unwrap_or(10) as u16)'
    )
    
    print(f"修复了 {file_path} 中的类型错误")
    
    with open(file_path, 'w') as f:
        f.write(content)

# 执行清理
if __name__ == "__main__":
    # 清理lockfree_structures.rs
    remove_duplicate_blocks('src/performance/lockfree_structures.rs')
    
    # 修复strategy/core.rs
    fix_strategy_error_display('src/strategy/core.rs')
    
    # 修复adaptive_profit.rs
    fix_adaptive_profit_types('src/strategy/adaptive_profit.rs')
    
    print("清理完成！") 