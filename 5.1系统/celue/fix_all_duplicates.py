#!/usr/bin/env python3
"""全面清理所有重复代码"""

import os
import re

def find_duplicate_marker(content, marker):
    """找到重复的标记位置"""
    positions = []
    start = 0
    while True:
        pos = content.find(marker, start)
        if pos == -1:
            break
        positions.append(pos)
        start = pos + 1
    return positions

def clean_file(file_path):
    """清理文件中的重复内容"""
    with open(file_path, 'r') as f:
        content = f.read()
    
    original_len = len(content)
    
    # 对于performance模块下的文件，找到第一个完整定义后删除后续重复
    if 'performance/' in file_path:
        # 查找文档注释
        doc_positions = find_duplicate_marker(content, '\n//! \n//! ')
        if len(doc_positions) > 1:
            # 只保留第一个
            content = content[:doc_positions[1]]
            
        # 对于cpu_affinity.rs，查找struct CpuAffinityManager
        if file_path.endswith('cpu_affinity.rs'):
            struct_positions = find_duplicate_marker(content, '\npub struct CpuAffinityManager')
            if len(struct_positions) > 1:
                # 找到第一个完整实现的结束
                first_end = content.find('\n}\n\n', struct_positions[0])
                if first_end > 0:
                    content = content[:first_end + 3]
                    
        # 对于triangular_arbitrage.rs
        elif file_path.endswith('triangular_arbitrage.rs'):
            struct_positions = find_duplicate_marker(content, '\npub struct TriangularArbitrageDetector')
            if len(struct_positions) > 1:
                first_end = content.find('\n}\n\n', struct_positions[0])
                if first_end > 0:
                    content = content[:first_end + 3]
                    
        # 对于market_analysis.rs
        elif file_path.endswith('market_analysis.rs'):
            struct_positions = find_duplicate_marker(content, '\npub struct MarketAnalyzer')
            if len(struct_positions) > 1:
                first_end = content.find('\n}\n\n', struct_positions[0])
                if first_end > 0:
                    content = content[:first_end + 3]
    
    # 对于strategy模块下的文件
    elif 'strategy/' in file_path:
        # 查找重复的struct定义
        for struct_name in ['StrategyRegistry', 'GlobalOpportunityPool', 'StrategyFailureDetector']:
            struct_positions = find_duplicate_marker(content, f'\npub struct {struct_name}')
            if len(struct_positions) > 1:
                # 找到第一个完整实现的结束
                first_impl_end = content.find(f'\n}}\n\nimpl Default for {struct_name}', struct_positions[0])
                if first_impl_end == -1:
                    first_impl_end = content.find('\n}\n\n#[cfg(test)]', struct_positions[0])
                if first_impl_end == -1:
                    first_impl_end = content.find('\n}\n\n', struct_positions[0] + 1000)
                
                if first_impl_end > 0:
                    # 找到Default实现的结束
                    default_end = content.find('\n}\n\n', first_impl_end + 10)
                    if default_end > 0:
                        content = content[:default_end + 3]
                    else:
                        content = content[:first_impl_end + 3]
                    break
    
    # 对于mod.rs文件，删除重复的模块声明
    if file_path.endswith('mod.rs'):
        lines = content.split('\n')
        seen_modules = set()
        new_lines = []
        
        for line in lines:
            if line.startswith('pub mod '):
                module_name = line.split()[2].rstrip(';')
                if module_name not in seen_modules:
                    seen_modules.add(module_name)
                    new_lines.append(line)
            else:
                new_lines.append(line)
        
        content = '\n'.join(new_lines)
    
    # 保存清理后的文件
    if len(content) < original_len:
        with open(file_path, 'w') as f:
            f.write(content)
        print(f"清理了 {file_path} ({original_len} -> {len(content)} 字节)")

# 需要清理的文件列表
files_to_clean = [
    'src/performance/cpu_affinity.rs',
    'src/performance/triangular_arbitrage.rs', 
    'src/performance/market_analysis.rs',
    'src/performance/mod.rs',
    'src/strategy/registry.rs',
    'src/strategy/opportunity_pool.rs',
    'src/strategy/failure_detector.rs',
]

for file_path in files_to_clean:
    if os.path.exists(file_path):
        clean_file(file_path)

print("清理完成！") 