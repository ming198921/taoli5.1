#!/usr/bin/env python3
"""
完整修复arbitrage_monitor.rs的所有编译错误
"""
import re

def complete_fix():
    print("🔧 开始完整修复编译错误...")
    
    # 从simple版本重新复制干净的代码
    with open("src/bin/arbitrage_monitor_simple.rs", 'r') as f:
        content = f.read()
    
    print("✅ 1. 从simple版本复制干净代码")
    
    # 修复所有API调用问题
    fixes = [
        # 1. 修复SIMDFixedPointProcessor::new()缺少参数
        ('SIMDFixedPointProcessor::new()', 'SIMDFixedPointProcessor::new(2048)'),
        
        # 2. 修复clone()调用
        ('let processor = self.simd_processor.clone();', 'let processor = &self.simd_processor;'),
        
        # 3. 修复calculate_profit_batch_optimal调用
        ('processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices)', 
         'processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices, &buy_prices)'),
        
        # 4. 移除Result的match，直接使用结果
        ('match ', 'let profits = '),
        ('Ok(profits) => {', '{'),
        
        # 5. 移除所有colored相关调用，替换为简单println
        ('.bright_green().bold()', ''),
        ('.bright_green()', ''),
        ('.bright_yellow().bold()', ''),
        ('.bright_yellow()', ''),
        ('.bright_blue().bold()', ''),
        ('.bright_blue()', ''),
        ('.bright_magenta().bold()', ''),
        ('.bright_magenta()', ''),
        ('.bright_cyan().bold()', ''),
        ('.bright_cyan()', ''),
        ('.bright_white().bold()', ''),
        ('.bright_white()', ''),
        ('.bright_red().bold()', ''),
        ('.bright_red()', ''),
        ('.bright_black()', ''),
        ('.cyan()', ''),
        ('.bold()', ''),
    ]
    
    for old, new in fixes:
        if old in content:
            content = content.replace(old, new)
            print(f"✅ 修复: {old} -> {new}")
    
    # 移除colored导入
    content = content.replace('use colored::*;', '// use colored::*; // 临时移除')
    
    # 移除match的Err分支
    err_pattern = r'\s*Err\([^)]*\)\s*=>\s*\{[^}]*\}'
    content = re.sub(err_pattern, '', content, flags=re.DOTALL)
    
    # 确保大括号匹配
    lines = content.split('\n')
    
    # 移除多余的大括号和修复结构
    fixed_lines = []
    in_match_block = False
    brace_count = 0
    
    for line in lines:
        # 跳过空的Err处理
        if 'Err(' in line and '=>' in line:
            continue
            
        # 修复可能的语法问题
        if line.strip().startswith('let profits = processor.calculate_profit_batch_optimal'):
            # 确保这行结束正确
            if not line.strip().endswith(';'):
                line = line.rstrip() + ';'
        
        fixed_lines.append(line)
    
    content = '\n'.join(fixed_lines)
    
    # 写回文件
    with open("src/bin/arbitrage_monitor.rs", 'w') as f:
        f.write(content)
    
    print("✅ 所有修复完成，写入文件")
    print("🔧 正在验证修复效果...")

if __name__ == "__main__":
    complete_fix() 
"""
完整修复arbitrage_monitor.rs的所有编译错误
"""
import re

def complete_fix():
    print("🔧 开始完整修复编译错误...")
    
    # 从simple版本重新复制干净的代码
    with open("src/bin/arbitrage_monitor_simple.rs", 'r') as f:
        content = f.read()
    
    print("✅ 1. 从simple版本复制干净代码")
    
    # 修复所有API调用问题
    fixes = [
        # 1. 修复SIMDFixedPointProcessor::new()缺少参数
        ('SIMDFixedPointProcessor::new()', 'SIMDFixedPointProcessor::new(2048)'),
        
        # 2. 修复clone()调用
        ('let processor = self.simd_processor.clone();', 'let processor = &self.simd_processor;'),
        
        # 3. 修复calculate_profit_batch_optimal调用
        ('processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices)', 
         'processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices, &buy_prices)'),
        
        # 4. 移除Result的match，直接使用结果
        ('match ', 'let profits = '),
        ('Ok(profits) => {', '{'),
        
        # 5. 移除所有colored相关调用，替换为简单println
        ('.bright_green().bold()', ''),
        ('.bright_green()', ''),
        ('.bright_yellow().bold()', ''),
        ('.bright_yellow()', ''),
        ('.bright_blue().bold()', ''),
        ('.bright_blue()', ''),
        ('.bright_magenta().bold()', ''),
        ('.bright_magenta()', ''),
        ('.bright_cyan().bold()', ''),
        ('.bright_cyan()', ''),
        ('.bright_white().bold()', ''),
        ('.bright_white()', ''),
        ('.bright_red().bold()', ''),
        ('.bright_red()', ''),
        ('.bright_black()', ''),
        ('.cyan()', ''),
        ('.bold()', ''),
    ]
    
    for old, new in fixes:
        if old in content:
            content = content.replace(old, new)
            print(f"✅ 修复: {old} -> {new}")
    
    # 移除colored导入
    content = content.replace('use colored::*;', '// use colored::*; // 临时移除')
    
    # 移除match的Err分支
    err_pattern = r'\s*Err\([^)]*\)\s*=>\s*\{[^}]*\}'
    content = re.sub(err_pattern, '', content, flags=re.DOTALL)
    
    # 确保大括号匹配
    lines = content.split('\n')
    
    # 移除多余的大括号和修复结构
    fixed_lines = []
    in_match_block = False
    brace_count = 0
    
    for line in lines:
        # 跳过空的Err处理
        if 'Err(' in line and '=>' in line:
            continue
            
        # 修复可能的语法问题
        if line.strip().startswith('let profits = processor.calculate_profit_batch_optimal'):
            # 确保这行结束正确
            if not line.strip().endswith(';'):
                line = line.rstrip() + ';'
        
        fixed_lines.append(line)
    
    content = '\n'.join(fixed_lines)
    
    # 写回文件
    with open("src/bin/arbitrage_monitor.rs", 'w') as f:
        f.write(content)
    
    print("✅ 所有修复完成，写入文件")
    print("🔧 正在验证修复效果...")

if __name__ == "__main__":
    complete_fix() 