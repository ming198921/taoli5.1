#!/usr/bin/env python3
"""
快速修复工作区依赖问题
"""

import os
import re

def fix_cargo_toml(file_path):
    """修复单个Cargo.toml文件的依赖问题"""
    if not os.path.exists(file_path):
        return
        
    with open(file_path, 'r') as f:
        content = f.read()
    
    # 定义在workspace中不存在的依赖
    missing_deps = [
        'futures',
        'metrics', 
        'metrics-exporter-prometheus',
        'prometheus',
        'linfa',
        'linfa-trees',
        'linfa-linear',
        'linfa-logistic',
        'atomic',
        'core_affinity',
        'memmap2',
        'num_cpus',
        'rand',
        'rand_distr',
        'rand_core',
        'approx',
        'linfa-elasticnet',
        'linfa-svm',
        'linfa-clustering',
        'linfa-preprocessing',
        'nuid',
        'tokio-util',
        'aligned-vec'
    ]
    
    # 注释掉这些依赖
    for dep in missing_deps:
        pattern = rf'^({dep}\s*=\s*{{.*workspace.*}})$'
        replacement = rf'# \1  # 未在workspace中定义'
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
    
    # 写回文件
    with open(file_path, 'w') as f:
        f.write(content)
    
    print(f"✅ 修复: {file_path}")

def main():
    print("🔧 修复工作区依赖问题...")
    
    # 需要修复的文件
    files_to_fix = [
        'orchestrator/Cargo.toml',
        'adapters/Cargo.toml',
        'common/Cargo.toml',
        'strategy/Cargo.toml'
    ]
    
    for file_path in files_to_fix:
        fix_cargo_toml(file_path)
    
    print("✅ 所有依赖问题已修复")

if __name__ == "__main__":
    main() 
"""
快速修复工作区依赖问题
"""

import os
import re

def fix_cargo_toml(file_path):
    """修复单个Cargo.toml文件的依赖问题"""
    if not os.path.exists(file_path):
        return
        
    with open(file_path, 'r') as f:
        content = f.read()
    
    # 定义在workspace中不存在的依赖
    missing_deps = [
        'futures',
        'metrics', 
        'metrics-exporter-prometheus',
        'prometheus',
        'linfa',
        'linfa-trees',
        'linfa-linear',
        'linfa-logistic',
        'atomic',
        'core_affinity',
        'memmap2',
        'num_cpus',
        'rand',
        'rand_distr',
        'rand_core',
        'approx',
        'linfa-elasticnet',
        'linfa-svm',
        'linfa-clustering',
        'linfa-preprocessing',
        'nuid',
        'tokio-util',
        'aligned-vec'
    ]
    
    # 注释掉这些依赖
    for dep in missing_deps:
        pattern = rf'^({dep}\s*=\s*{{.*workspace.*}})$'
        replacement = rf'# \1  # 未在workspace中定义'
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
    
    # 写回文件
    with open(file_path, 'w') as f:
        f.write(content)
    
    print(f"✅ 修复: {file_path}")

def main():
    print("🔧 修复工作区依赖问题...")
    
    # 需要修复的文件
    files_to_fix = [
        'orchestrator/Cargo.toml',
        'adapters/Cargo.toml',
        'common/Cargo.toml',
        'strategy/Cargo.toml'
    ]
    
    for file_path in files_to_fix:
        fix_cargo_toml(file_path)
    
    print("✅ 所有依赖问题已修复")

if __name__ == "__main__":
    main() 