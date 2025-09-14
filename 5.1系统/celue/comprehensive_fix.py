#!/usr/bin/env python3
import re
import os

def clean_scheduler_file():
    """彻底清理scheduler.rs中的重复定义"""
    file_path = 'src/strategy/scheduler.rs'
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 通过行号范围删除重复的块
    lines = content.split('\n')
    
    # 删除800-1056行（重复的定义）
    new_lines = lines[:800] + ['', '// End of scheduler module']
    
    content = '\n'.join(new_lines)
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)
    
    print(f"已清理: {file_path}")

def clean_opportunity_pool_file():
    """清理opportunity_pool.rs中的重复定义"""
    file_path = 'src/strategy/opportunity_pool.rs'
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 删除重复的import和struct定义
    lines = content.split('\n')
    new_lines = []
    found_first_config = False
    
    for i, line in enumerate(lines):
        # 跳过重复的OpportunityPoolConfig定义（第558行之后的）
        if i >= 558 and '#[derive(Debug, Clone, Serialize, Deserialize)]' in line:
            if i + 1 < len(lines) and 'pub struct OpportunityPoolConfig' in lines[i + 1]:
                if found_first_config:
                    # 跳到文件末尾，删除这个重复定义
                    break
        
        # 标记找到第一个config
        if 'pub struct OpportunityPoolConfig' in line:
            found_first_config = True
        
        new_lines.append(line)
    
    content = '\n'.join(new_lines)
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)
    
    print(f"已清理: {file_path}")

def fix_adaptive_profit_errors():
    """修复adaptive_profit.rs中的错误"""
    file_path = 'src/strategy/adaptive_profit.rs'
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 修复类型转换错误
    content = content.replace(
        'hyperparams.max_depth.unwrap_or(10)',
        'hyperparams.max_depth.unwrap_or(10) as u16'
    )
    
    # 修复RandomForest model存储
    content = content.replace(
        '*model = Some(Box::new(rf) as Box<dyn std::any::Any + Send + Sync>);',
        '*model = Some(Box::new(rf));'
    )
    
    # 修复缺失的StrategyError变体
    content = content.replace(
        'StrategyError::InsufficientData(',
        'StrategyError::ValidationError('
    )
    content = content.replace(
        'StrategyError::ModelNotFound(',
        'StrategyError::ModelTrainingError('
    )
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)
    
    print(f"已修复: {file_path}")

def fix_ml_models_errors():
    """修复ml_models.rs中的错误"""
    file_path = 'src/strategy/ml_models.rs'
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 修复ModelNotFound错误
    content = content.replace(
        'StrategyError::ModelNotFound(',
        'StrategyError::ModelTrainingError('
    )
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)
    
    print(f"已修复: {file_path}")

def main():
    clean_scheduler_file()
    clean_opportunity_pool_file()
    fix_adaptive_profit_errors()
    fix_ml_models_errors()
    print("所有修复完成!")

if __name__ == "__main__":
    main() 
import re
import os

def clean_scheduler_file():
    """彻底清理scheduler.rs中的重复定义"""
    file_path = 'src/strategy/scheduler.rs'
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 通过行号范围删除重复的块
    lines = content.split('\n')
    
    # 删除800-1056行（重复的定义）
    new_lines = lines[:800] + ['', '// End of scheduler module']
    
    content = '\n'.join(new_lines)
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)
    
    print(f"已清理: {file_path}")

def clean_opportunity_pool_file():
    """清理opportunity_pool.rs中的重复定义"""
    file_path = 'src/strategy/opportunity_pool.rs'
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 删除重复的import和struct定义
    lines = content.split('\n')
    new_lines = []
    found_first_config = False
    
    for i, line in enumerate(lines):
        # 跳过重复的OpportunityPoolConfig定义（第558行之后的）
        if i >= 558 and '#[derive(Debug, Clone, Serialize, Deserialize)]' in line:
            if i + 1 < len(lines) and 'pub struct OpportunityPoolConfig' in lines[i + 1]:
                if found_first_config:
                    # 跳到文件末尾，删除这个重复定义
                    break
        
        # 标记找到第一个config
        if 'pub struct OpportunityPoolConfig' in line:
            found_first_config = True
        
        new_lines.append(line)
    
    content = '\n'.join(new_lines)
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)
    
    print(f"已清理: {file_path}")

def fix_adaptive_profit_errors():
    """修复adaptive_profit.rs中的错误"""
    file_path = 'src/strategy/adaptive_profit.rs'
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 修复类型转换错误
    content = content.replace(
        'hyperparams.max_depth.unwrap_or(10)',
        'hyperparams.max_depth.unwrap_or(10) as u16'
    )
    
    # 修复RandomForest model存储
    content = content.replace(
        '*model = Some(Box::new(rf) as Box<dyn std::any::Any + Send + Sync>);',
        '*model = Some(Box::new(rf));'
    )
    
    # 修复缺失的StrategyError变体
    content = content.replace(
        'StrategyError::InsufficientData(',
        'StrategyError::ValidationError('
    )
    content = content.replace(
        'StrategyError::ModelNotFound(',
        'StrategyError::ModelTrainingError('
    )
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)
    
    print(f"已修复: {file_path}")

def fix_ml_models_errors():
    """修复ml_models.rs中的错误"""
    file_path = 'src/strategy/ml_models.rs'
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 修复ModelNotFound错误
    content = content.replace(
        'StrategyError::ModelNotFound(',
        'StrategyError::ModelTrainingError('
    )
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)
    
    print(f"已修复: {file_path}")

def main():
    clean_scheduler_file()
    clean_opportunity_pool_file()
    fix_adaptive_profit_errors()
    fix_ml_models_errors()
    print("所有修复完成!")

if __name__ == "__main__":
    main() 