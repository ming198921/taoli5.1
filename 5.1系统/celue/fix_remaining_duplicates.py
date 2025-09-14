#!/usr/bin/env python3
"""清理剩余的重复定义"""

import os

def clean_scheduler(file_path):
    """清理scheduler.rs中的重复定义"""
    with open(file_path, 'r') as f:
        content = f.read()
    
    # 找到第一个StrategyScheduler定义
    first_pos = content.find('pub struct StrategyScheduler {')
    if first_pos > 0:
        # 找到这个定义块的结束(包括所有impl块)
        # 查找第二个定义的位置
        second_pos = content.find('pub struct StrategyScheduler {', first_pos + 1)
        if second_pos > 0:
            # 保留到第二个定义之前
            content = content[:second_pos]
    
    with open(file_path, 'w') as f:
        f.write(content.rstrip() + '\n')

def clean_config_manager(file_path):
    """清理config_manager.rs中的重复定义"""
    with open(file_path, 'r') as f:
        content = f.read()
    
    # 找到第一个StrategyConfigManager定义
    first_pos = content.find('pub struct StrategyConfigManager {')
    if first_pos > 0:
        second_pos = content.find('pub struct StrategyConfigManager {', first_pos + 1)
        if second_pos > 0:
            content = content[:second_pos]
    
    with open(file_path, 'w') as f:
        f.write(content.rstrip() + '\n')

def clean_market_state(file_path):
    """清理market_state.rs中的重复定义"""
    with open(file_path, 'r') as f:
        content = f.read()
    
    # 找到第一个MarketStateJudge定义
    first_pos = content.find('pub struct MarketStateJudge {')
    if first_pos > 0:
        second_pos = content.find('pub struct MarketStateJudge {', first_pos + 1)
        if second_pos > 0:
            content = content[:second_pos]
    
    with open(file_path, 'w') as f:
        f.write(content.rstrip() + '\n')

def clean_imports(file_path):
    """清理重复的imports"""
    with open(file_path, 'r') as f:
        lines = f.readlines()
    
    seen_imports = set()
    new_lines = []
    in_use_block = False
    use_block_lines = []
    
    for line in lines:
        stripped = line.strip()
        
        # 检测use块的开始
        if stripped.startswith('use ') and not in_use_block:
            in_use_block = True
            use_block_lines = [line]
        elif in_use_block:
            if stripped == '' or not (stripped.startswith('use ') or stripped.startswith(' ') or stripped.startswith('\t')):
                # use块结束，去重并添加
                unique_uses = []
                for use_line in use_block_lines:
                    if use_line.strip() and use_line not in seen_imports:
                        seen_imports.add(use_line)
                        unique_uses.append(use_line)
                new_lines.extend(unique_uses)
                new_lines.append(line)
                in_use_block = False
                use_block_lines = []
            else:
                use_block_lines.append(line)
        else:
            new_lines.append(line)
    
    # 处理文件末尾的use块
    if in_use_block:
        unique_uses = []
        for use_line in use_block_lines:
            if use_line.strip() and use_line not in seen_imports:
                seen_imports.add(use_line)
                unique_uses.append(use_line)
        new_lines.extend(unique_uses)
    
    with open(file_path, 'w') as f:
        f.writelines(new_lines)

# 执行清理
files_to_clean = [
    ('src/strategy/scheduler.rs', clean_scheduler),
    ('src/strategy/config_manager.rs', clean_config_manager),
    ('src/strategy/market_state.rs', clean_market_state),
]

for file_path, clean_func in files_to_clean:
    if os.path.exists(file_path):
        clean_func(file_path)
        print(f"清理了 {file_path}")

# 清理所有strategy文件的imports
strategy_files = [
    'src/strategy/scheduler.rs',
    'src/strategy/config_manager.rs', 
    'src/strategy/market_state.rs',
    'src/strategy/adaptive_profit.rs',
]

for file_path in strategy_files:
    if os.path.exists(file_path):
        clean_imports(file_path)
        print(f"清理了 {file_path} 的imports")

print("清理完成！") 