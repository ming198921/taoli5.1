#!/usr/bin/env python3
"""
快速修复arbitrage_monitor.rs编译错误
"""

def fix_arbitrage_monitor():
    file_path = "src/bin/arbitrage_monitor.rs"
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    print(f"🔧 修复 {file_path}")
    
    # 1. 修复SIMDFixedPointProcessor::new()缺少参数
    content = content.replace(
        'SIMDFixedPointProcessor::new()',
        'SIMDFixedPointProcessor::new(OPTIMAL_BATCH_SIZE)'
    )
    
    # 2. 修复clone()调用
    content = content.replace(
        'let processor = self.simd_processor.clone();',
        'let processor = &self.simd_processor;'
    )
    
    # 3. 修复calculate_profit_batch_optimal调用（缺少第三个参数）
    content = content.replace(
        'match processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices)',
        'let profits = processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices, &buy_prices); // 第三个参数暂时用buy_prices'
    )
    
    # 4. 移除match语句，直接使用结果
    content = content.replace(
        'let profits = processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices, &buy_prices); // 第三个参数暂时用buy_prices\n                Ok(profits) => {',
        'let profits = processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices, &buy_prices); // 第三个参数暂时用buy_prices\n                {'
    )
    
    # 5. 移除Err分支
    import re
    content = re.sub(r'\s*Err\(e\) => \{[^}]*eprintln![^}]*\}\s*\}', '', content, flags=re.DOTALL)
    
    # 6. 修复colored方法调用 - 将字符串方法调用改为Colorize trait调用
    # 修复String类型的colored调用
    content = content.replace('.bright_cyan()', '.as_str().bright_cyan()')
    content = content.replace('.bright_green().bold()', '.as_str().bright_green().bold()')
    content = content.replace('.bright_yellow().bold()', '.as_str().bright_yellow().bold()')
    content = content.replace('.bright_white()', '.as_str().bright_white()')
    content = content.replace('.cyan()', '.as_str().cyan()')
    
    # 写回文件
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)
    
    print("✅ 修复完成")

if __name__ == "__main__":
    fix_arbitrage_monitor() 
"""
快速修复arbitrage_monitor.rs编译错误
"""

def fix_arbitrage_monitor():
    file_path = "src/bin/arbitrage_monitor.rs"
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    print(f"🔧 修复 {file_path}")
    
    # 1. 修复SIMDFixedPointProcessor::new()缺少参数
    content = content.replace(
        'SIMDFixedPointProcessor::new()',
        'SIMDFixedPointProcessor::new(OPTIMAL_BATCH_SIZE)'
    )
    
    # 2. 修复clone()调用
    content = content.replace(
        'let processor = self.simd_processor.clone();',
        'let processor = &self.simd_processor;'
    )
    
    # 3. 修复calculate_profit_batch_optimal调用（缺少第三个参数）
    content = content.replace(
        'match processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices)',
        'let profits = processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices, &buy_prices); // 第三个参数暂时用buy_prices'
    )
    
    # 4. 移除match语句，直接使用结果
    content = content.replace(
        'let profits = processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices, &buy_prices); // 第三个参数暂时用buy_prices\n                Ok(profits) => {',
        'let profits = processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices, &buy_prices); // 第三个参数暂时用buy_prices\n                {'
    )
    
    # 5. 移除Err分支
    import re
    content = re.sub(r'\s*Err\(e\) => \{[^}]*eprintln![^}]*\}\s*\}', '', content, flags=re.DOTALL)
    
    # 6. 修复colored方法调用 - 将字符串方法调用改为Colorize trait调用
    # 修复String类型的colored调用
    content = content.replace('.bright_cyan()', '.as_str().bright_cyan()')
    content = content.replace('.bright_green().bold()', '.as_str().bright_green().bold()')
    content = content.replace('.bright_yellow().bold()', '.as_str().bright_yellow().bold()')
    content = content.replace('.bright_white()', '.as_str().bright_white()')
    content = content.replace('.cyan()', '.as_str().cyan()')
    
    # 写回文件
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)
    
    print("✅ 修复完成")

if __name__ == "__main__":
    fix_arbitrage_monitor() 