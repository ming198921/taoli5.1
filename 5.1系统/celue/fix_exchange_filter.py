#!/usr/bin/env python3
"""
修复exchange_filter参数的过度修改
只有真正未使用的参数才应该加下划线前缀
"""

import re

def fix_exchange_filter():
    file_path = 'strategy/src/plugins/triangular.rs'
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 修复被误改的正在使用的exchange_filter参数
    fixes = [
        # 修复函数参数定义（这些确实在使用）
        (r'pub fn discover_triangular_paths_optimized_v2\(&self, _exchange_filter: Option<&str>', 
         r'pub fn discover_triangular_paths_optimized_v2(&self, exchange_filter: Option<&str>'),
        
        # 只修复真正未使用的那些（在函数体内没有使用exchange_filter的）
        (r'fn find_triangular_cycles_parallel_v2\(&self, _exchange_filter: Option<&str>', 
         r'fn find_triangular_cycles_parallel_v2(&self, _exchange_filter: Option<&str>'),  # 这个确实未使用
    ]
    
    original_content = content
    
    for pattern, replacement in fixes:
        content = re.sub(pattern, replacement, content)
    
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"✅ 修复了 exchange_filter 参数使用问题")
    else:
        print("ℹ️  无需修复")

if __name__ == "__main__":
    fix_exchange_filter() 
"""
修复exchange_filter参数的过度修改
只有真正未使用的参数才应该加下划线前缀
"""

import re

def fix_exchange_filter():
    file_path = 'strategy/src/plugins/triangular.rs'
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 修复被误改的正在使用的exchange_filter参数
    fixes = [
        # 修复函数参数定义（这些确实在使用）
        (r'pub fn discover_triangular_paths_optimized_v2\(&self, _exchange_filter: Option<&str>', 
         r'pub fn discover_triangular_paths_optimized_v2(&self, exchange_filter: Option<&str>'),
        
        # 只修复真正未使用的那些（在函数体内没有使用exchange_filter的）
        (r'fn find_triangular_cycles_parallel_v2\(&self, _exchange_filter: Option<&str>', 
         r'fn find_triangular_cycles_parallel_v2(&self, _exchange_filter: Option<&str>'),  # 这个确实未使用
    ]
    
    original_content = content
    
    for pattern, replacement in fixes:
        content = re.sub(pattern, replacement, content)
    
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"✅ 修复了 exchange_filter 参数使用问题")
    else:
        print("ℹ️  无需修复")

if __name__ == "__main__":
    fix_exchange_filter() 