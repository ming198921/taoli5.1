#!/usr/bin/env python3
"""
编译警告修复脚本
系统性修复Rust编译过程中的未使用变量和导入警告
"""

import re
import os

def fix_unused_variables(file_path, patterns):
    """修复未使用变量警告"""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    
    for pattern, replacement in patterns:
        content = re.sub(pattern, replacement, content)
    
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"✅ 修复了 {file_path}")
        return True
    return False

def main():
    """主修复流程"""
    
    # 修复 strategy/src/dynamic_fee_calculator.rs
    dynamic_fee_patterns = [
        (r'(\s+)fee_type: FeeType,', r'\1_fee_type: FeeType,'),
        (r'(\s+)target_volume_usd: f64,', r'\1_target_volume_usd: f64,'),
    ]
    
    # 修复 strategy/src/plugins/triangular.rs  
    triangular_patterns = [
        (r'(\s+)exchange_filter: Option<&str>', r'\1_exchange_filter: Option<&str>'),
        (r'\.flat_map\(\|&currency_a\|', r'.flat_map(|&_currency_a|'),
        (r'(\s+)ctx: &StrategyContext,', r'\1_ctx: &StrategyContext,'),
    ]
    
    files_to_fix = [
        ('strategy/src/dynamic_fee_calculator.rs', dynamic_fee_patterns),
        ('strategy/src/plugins/triangular.rs', triangular_patterns),
    ]
    
    total_fixed = 0
    
    for file_path, patterns in files_to_fix:
        if os.path.exists(file_path):
            if fix_unused_variables(file_path, patterns):
                total_fixed += 1
        else:
            print(f"⚠️  文件不存在: {file_path}")
    
    print(f"\n🎯 总计修复了 {total_fixed} 个文件的警告")

if __name__ == "__main__":
    main() 
"""
编译警告修复脚本
系统性修复Rust编译过程中的未使用变量和导入警告
"""

import re
import os

def fix_unused_variables(file_path, patterns):
    """修复未使用变量警告"""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    
    for pattern, replacement in patterns:
        content = re.sub(pattern, replacement, content)
    
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"✅ 修复了 {file_path}")
        return True
    return False

def main():
    """主修复流程"""
    
    # 修复 strategy/src/dynamic_fee_calculator.rs
    dynamic_fee_patterns = [
        (r'(\s+)fee_type: FeeType,', r'\1_fee_type: FeeType,'),
        (r'(\s+)target_volume_usd: f64,', r'\1_target_volume_usd: f64,'),
    ]
    
    # 修复 strategy/src/plugins/triangular.rs  
    triangular_patterns = [
        (r'(\s+)exchange_filter: Option<&str>', r'\1_exchange_filter: Option<&str>'),
        (r'\.flat_map\(\|&currency_a\|', r'.flat_map(|&_currency_a|'),
        (r'(\s+)ctx: &StrategyContext,', r'\1_ctx: &StrategyContext,'),
    ]
    
    files_to_fix = [
        ('strategy/src/dynamic_fee_calculator.rs', dynamic_fee_patterns),
        ('strategy/src/plugins/triangular.rs', triangular_patterns),
    ]
    
    total_fixed = 0
    
    for file_path, patterns in files_to_fix:
        if os.path.exists(file_path):
            if fix_unused_variables(file_path, patterns):
                total_fixed += 1
        else:
            print(f"⚠️  文件不存在: {file_path}")
    
    print(f"\n🎯 总计修复了 {total_fixed} 个文件的警告")

if __name__ == "__main__":
    main() 