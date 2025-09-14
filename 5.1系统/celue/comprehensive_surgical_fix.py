#!/usr/bin/env python3
"""
综合外科手术修复脚本 - 彻底清理arbitrage_monitor.rs
"""
import re

def comprehensive_surgical_fix():
    print("🏥 开始综合外科手术修复...")
    
    # 1. 重新从simple版本开始
    with open("src/bin/arbitrage_monitor_simple.rs", 'r') as f:
        content = f.read()
    
    print("✅ 1. 重新获取干净的代码基础")
    
    # 2. 移除所有重复的结构体定义，只保留第一个
    structs_to_dedupe = [
        'CelueMarketData', 'PricePoint', 'ArbitrageOpportunity', 
        'ArbitrageType', 'ArbitrageStats'
    ]
    
    for struct_name in structs_to_dedupe:
        pattern = rf'#\[derive.*?\]\s*pub struct {struct_name}\s*\{{[^}}]*\}}'
        matches = list(re.finditer(pattern, content, re.DOTALL))
        if len(matches) > 1:
            # 保留第一个，删除其他的
            for match in reversed(matches[1:]):
                content = content[:match.start()] + content[match.end():]
            print(f"✅ 去重 {struct_name} 结构体定义")
    
    # 3. 移除重复的impl块，只保留第一个
    impl_pattern = r'impl\s+ArbitrageMonitor\s*\{[^}]*(?:\{[^}]*\}[^}]*)*\}'
    impl_matches = list(re.finditer(impl_pattern, content, re.DOTALL))
    if len(impl_matches) > 1:
        # 保留第一个最完整的impl块
        for match in reversed(impl_matches[1:]):
            content = content[:match.start()] + content[match.end():]
        print("✅ 去重 impl ArbitrageMonitor 块")
    
    # 4. 去重use语句
    use_statements = []
    lines = content.split('\n')
    cleaned_lines = []
    
    for line in lines:
        if line.strip().startswith('use ') and line.strip().endswith(';'):
            use_stmt = line.strip()
            if use_stmt not in use_statements:
                use_statements.append(use_stmt)
                cleaned_lines.append(line)
            else:
                print(f"✅ 去重: {use_stmt}")
        else:
            cleaned_lines.append(line)
    
    content = '\n'.join(cleaned_lines)
    
    # 5. 修复API调用问题
    api_fixes = [
        # SIMDFixedPointProcessor构造函数参数
        ('SIMDFixedPointProcessor::new()', 'SIMDFixedPointProcessor::new(2048)'),
        
        # 修复clone调用 - 使用Arc::clone
        ('let processor = self.simd_processor.clone();', 
         'let processor = Arc::clone(&self.simd_processor);'),
        
        # 修复calculate_profit_batch_optimal调用，添加第三个参数
        ('processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices)',
         'processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices, &buy_prices)'),
        
        # 修复async_nats Message字段
        ('&message.data', '&message.payload'),
        
        # 修复println!格式化参数
        ('println!("   [{}] {} | {} | {} <-> {} | {}"',
         'println!("   [{}] {} | {} | {}"'),
    ]
    
    for old, new in api_fixes:
        if old in content:
            content = content.replace(old, new)
            print(f"✅ API修复: {old[:30]}...")
    
    # 6. 确保colored导入正确
    if 'use colored::*;' not in content:
        # 在其他use语句后添加colored导入
        use_section_end = content.find('\nconst')
        if use_section_end != -1:
            content = content[:use_section_end] + '\nuse colored::*;' + content[use_section_end:]
            print("✅ 添加colored导入")
    
    # 7. 修复重复的main函数
    main_pattern = r'#\[tokio::main\]\s*async fn main\(\)\s*->\s*Result<\(\), Box<dyn std::error::Error>>\s*\{[^}]*(?:\{[^}]*\}[^}]*)*\}'
    main_matches = list(re.finditer(main_pattern, content, re.DOTALL))
    if len(main_matches) > 1:
        # 保留第一个main函数
        for match in reversed(main_matches[1:]):
            content = content[:match.start()] + content[match.end():]
        print("✅ 去重 main 函数")
    
    # 8. 写入修复后的文件
    with open("src/bin/arbitrage_monitor.rs", 'w') as f:
        f.write(content)
    
    print("🎉 综合外科手术修复完成！")

if __name__ == "__main__":
    comprehensive_surgical_fix() 
"""
综合外科手术修复脚本 - 彻底清理arbitrage_monitor.rs
"""
import re

def comprehensive_surgical_fix():
    print("🏥 开始综合外科手术修复...")
    
    # 1. 重新从simple版本开始
    with open("src/bin/arbitrage_monitor_simple.rs", 'r') as f:
        content = f.read()
    
    print("✅ 1. 重新获取干净的代码基础")
    
    # 2. 移除所有重复的结构体定义，只保留第一个
    structs_to_dedupe = [
        'CelueMarketData', 'PricePoint', 'ArbitrageOpportunity', 
        'ArbitrageType', 'ArbitrageStats'
    ]
    
    for struct_name in structs_to_dedupe:
        pattern = rf'#\[derive.*?\]\s*pub struct {struct_name}\s*\{{[^}}]*\}}'
        matches = list(re.finditer(pattern, content, re.DOTALL))
        if len(matches) > 1:
            # 保留第一个，删除其他的
            for match in reversed(matches[1:]):
                content = content[:match.start()] + content[match.end():]
            print(f"✅ 去重 {struct_name} 结构体定义")
    
    # 3. 移除重复的impl块，只保留第一个
    impl_pattern = r'impl\s+ArbitrageMonitor\s*\{[^}]*(?:\{[^}]*\}[^}]*)*\}'
    impl_matches = list(re.finditer(impl_pattern, content, re.DOTALL))
    if len(impl_matches) > 1:
        # 保留第一个最完整的impl块
        for match in reversed(impl_matches[1:]):
            content = content[:match.start()] + content[match.end():]
        print("✅ 去重 impl ArbitrageMonitor 块")
    
    # 4. 去重use语句
    use_statements = []
    lines = content.split('\n')
    cleaned_lines = []
    
    for line in lines:
        if line.strip().startswith('use ') and line.strip().endswith(';'):
            use_stmt = line.strip()
            if use_stmt not in use_statements:
                use_statements.append(use_stmt)
                cleaned_lines.append(line)
            else:
                print(f"✅ 去重: {use_stmt}")
        else:
            cleaned_lines.append(line)
    
    content = '\n'.join(cleaned_lines)
    
    # 5. 修复API调用问题
    api_fixes = [
        # SIMDFixedPointProcessor构造函数参数
        ('SIMDFixedPointProcessor::new()', 'SIMDFixedPointProcessor::new(2048)'),
        
        # 修复clone调用 - 使用Arc::clone
        ('let processor = self.simd_processor.clone();', 
         'let processor = Arc::clone(&self.simd_processor);'),
        
        # 修复calculate_profit_batch_optimal调用，添加第三个参数
        ('processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices)',
         'processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices, &buy_prices)'),
        
        # 修复async_nats Message字段
        ('&message.data', '&message.payload'),
        
        # 修复println!格式化参数
        ('println!("   [{}] {} | {} | {} <-> {} | {}"',
         'println!("   [{}] {} | {} | {}"'),
    ]
    
    for old, new in api_fixes:
        if old in content:
            content = content.replace(old, new)
            print(f"✅ API修复: {old[:30]}...")
    
    # 6. 确保colored导入正确
    if 'use colored::*;' not in content:
        # 在其他use语句后添加colored导入
        use_section_end = content.find('\nconst')
        if use_section_end != -1:
            content = content[:use_section_end] + '\nuse colored::*;' + content[use_section_end:]
            print("✅ 添加colored导入")
    
    # 7. 修复重复的main函数
    main_pattern = r'#\[tokio::main\]\s*async fn main\(\)\s*->\s*Result<\(\), Box<dyn std::error::Error>>\s*\{[^}]*(?:\{[^}]*\}[^}]*)*\}'
    main_matches = list(re.finditer(main_pattern, content, re.DOTALL))
    if len(main_matches) > 1:
        # 保留第一个main函数
        for match in reversed(main_matches[1:]):
            content = content[:match.start()] + content[match.end():]
        print("✅ 去重 main 函数")
    
    # 8. 写入修复后的文件
    with open("src/bin/arbitrage_monitor.rs", 'w') as f:
        f.write(content)
    
    print("🎉 综合外科手术修复完成！")

if __name__ == "__main__":
    comprehensive_surgical_fix() 