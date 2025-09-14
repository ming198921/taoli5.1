#!/usr/bin/env python3
"""
外科手术式修复脚本 - 清理arbitrage_monitor.rs中的重复代码
"""

import re
import sys

def surgical_fix():
    file_path = "src/bin/arbitrage_monitor.rs"
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        print(f"📂 正在处理文件: {file_path}")
        print(f"📊 原始文件大小: {len(content)} 字符")
        
        # 1. 移除重复的use语句，保留第一个
        lines = content.split('\n')
        seen_uses = set()
        cleaned_lines = []
        
        for line in lines:
            # 检查use语句
            if line.strip().startswith('use ') and line.strip().endswith(';'):
                use_stmt = line.strip()
                if use_stmt not in seen_uses:
                    seen_uses.add(use_stmt)
                    cleaned_lines.append(line)
                else:
                    print(f"🗑️  移除重复use: {use_stmt}")
            else:
                cleaned_lines.append(line)
        
        content = '\n'.join(cleaned_lines)
        
        # 2. 移除重复的struct定义 - 保留第一个
        structs_to_fix = [
            'CelueMarketData',
            'PricePoint', 
            'ArbitrageOpportunity',
            'ArbitrageType',
            'ArbitrageStats'
        ]
        
        for struct_name in structs_to_fix:
            pattern = rf'#\[derive\([^\]]*\)\]\s*pub struct {struct_name} \{{[^}}]*\}}'
            matches = list(re.finditer(pattern, content, re.DOTALL))
            if len(matches) > 1:
                print(f"🗑️  发现{len(matches)}个重复struct {struct_name}, 保留第一个")
                # 移除除第一个之外的所有定义
                for match in reversed(matches[1:]):
                    content = content[:match.start()] + content[match.end():]
        
        # 3. 移除重复的impl块 - 保留第一个
        impl_pattern = r'impl ArbitrageMonitor \{[^}]*(?:\{[^}]*\}[^}]*)*\}'
        impl_matches = list(re.finditer(impl_pattern, content, re.DOTALL))
        
        if len(impl_matches) > 1:
            print(f"🗑️  发现{len(impl_matches)}个重复impl块, 保留第一个")
            # 移除除第一个之外的所有impl块
            for match in reversed(impl_matches[1:]):
                content = content[:match.start()] + content[match.end():]
        
        # 4. 移除重复的main函数 - 保留第一个
        main_pattern = r'#\[tokio::main\]\s*async fn main\(\)[^}]*\{[^}]*(?:\{[^}]*\}[^}]*)*\}'
        main_matches = list(re.finditer(main_pattern, content, re.DOTALL))
        
        if len(main_matches) > 1:
            print(f"🗑️  发现{len(main_matches)}个重复main函数, 保留第一个")
            # 移除除第一个之外的所有main函数
            for match in reversed(main_matches[1:]):
                content = content[:match.start()] + content[match.end():]
        
        # 5. 转换内部文档注释为普通注释
        content = re.sub(r'^\s*//!', '//', content, flags=re.MULTILINE)
        
        # 6. 确保colored导入正确
        if 'use colored::*;' not in content:
            # 在其他use语句后添加colored导入
            use_section_end = 0
            lines = content.split('\n')
            for i, line in enumerate(lines):
                if line.strip().startswith('use ') and line.strip().endswith(';'):
                    use_section_end = i
            
            if use_section_end > 0:
                lines.insert(use_section_end + 1, 'use colored::*;')
                content = '\n'.join(lines)
        
        # 写回文件
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        
        print(f"✅ 修复完成! 最终文件大小: {len(content)} 字符")
        return True
        
    except Exception as e:
        print(f"❌ 修复失败: {e}")
        return False

if __name__ == "__main__":
    success = surgical_fix()
    sys.exit(0 if success else 1) 
"""
外科手术式修复脚本 - 清理arbitrage_monitor.rs中的重复代码
"""

import re
import sys

def surgical_fix():
    file_path = "src/bin/arbitrage_monitor.rs"
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        print(f"📂 正在处理文件: {file_path}")
        print(f"📊 原始文件大小: {len(content)} 字符")
        
        # 1. 移除重复的use语句，保留第一个
        lines = content.split('\n')
        seen_uses = set()
        cleaned_lines = []
        
        for line in lines:
            # 检查use语句
            if line.strip().startswith('use ') and line.strip().endswith(';'):
                use_stmt = line.strip()
                if use_stmt not in seen_uses:
                    seen_uses.add(use_stmt)
                    cleaned_lines.append(line)
                else:
                    print(f"🗑️  移除重复use: {use_stmt}")
            else:
                cleaned_lines.append(line)
        
        content = '\n'.join(cleaned_lines)
        
        # 2. 移除重复的struct定义 - 保留第一个
        structs_to_fix = [
            'CelueMarketData',
            'PricePoint', 
            'ArbitrageOpportunity',
            'ArbitrageType',
            'ArbitrageStats'
        ]
        
        for struct_name in structs_to_fix:
            pattern = rf'#\[derive\([^\]]*\)\]\s*pub struct {struct_name} \{{[^}}]*\}}'
            matches = list(re.finditer(pattern, content, re.DOTALL))
            if len(matches) > 1:
                print(f"🗑️  发现{len(matches)}个重复struct {struct_name}, 保留第一个")
                # 移除除第一个之外的所有定义
                for match in reversed(matches[1:]):
                    content = content[:match.start()] + content[match.end():]
        
        # 3. 移除重复的impl块 - 保留第一个
        impl_pattern = r'impl ArbitrageMonitor \{[^}]*(?:\{[^}]*\}[^}]*)*\}'
        impl_matches = list(re.finditer(impl_pattern, content, re.DOTALL))
        
        if len(impl_matches) > 1:
            print(f"🗑️  发现{len(impl_matches)}个重复impl块, 保留第一个")
            # 移除除第一个之外的所有impl块
            for match in reversed(impl_matches[1:]):
                content = content[:match.start()] + content[match.end():]
        
        # 4. 移除重复的main函数 - 保留第一个
        main_pattern = r'#\[tokio::main\]\s*async fn main\(\)[^}]*\{[^}]*(?:\{[^}]*\}[^}]*)*\}'
        main_matches = list(re.finditer(main_pattern, content, re.DOTALL))
        
        if len(main_matches) > 1:
            print(f"🗑️  发现{len(main_matches)}个重复main函数, 保留第一个")
            # 移除除第一个之外的所有main函数
            for match in reversed(main_matches[1:]):
                content = content[:match.start()] + content[match.end():]
        
        # 5. 转换内部文档注释为普通注释
        content = re.sub(r'^\s*//!', '//', content, flags=re.MULTILINE)
        
        # 6. 确保colored导入正确
        if 'use colored::*;' not in content:
            # 在其他use语句后添加colored导入
            use_section_end = 0
            lines = content.split('\n')
            for i, line in enumerate(lines):
                if line.strip().startswith('use ') and line.strip().endswith(';'):
                    use_section_end = i
            
            if use_section_end > 0:
                lines.insert(use_section_end + 1, 'use colored::*;')
                content = '\n'.join(lines)
        
        # 写回文件
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        
        print(f"✅ 修复完成! 最终文件大小: {len(content)} 字符")
        return True
        
    except Exception as e:
        print(f"❌ 修复失败: {e}")
        return False

if __name__ == "__main__":
    success = surgical_fix()
    sys.exit(0 if success else 1) 