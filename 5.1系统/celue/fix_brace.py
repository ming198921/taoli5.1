#!/usr/bin/env python3

def fix_brace():
    with open("src/bin/arbitrage_monitor.rs", 'r') as f:
        content = f.read()
    
    # 找到问题行并修复
    lines = content.split('\n')
    for i, line in enumerate(lines):
        if 'calculate_profit_batch_optimal(&buy_prices, &sell_prices, &buy_prices); // 第三个参数暂时用buy_prices {' in line:
            # 移除行尾的 {
            lines[i] = line.replace(' {', '')
            # 下一行从 Ok(profits) => { 改为简单的 {
            if i + 1 < len(lines) and 'Ok(profits) => {' in lines[i + 1]:
                lines[i + 1] = '            {'
            break
    
    # 移除多余的 Err 分支
    fixed_lines = []
    skip_err_block = False
    brace_count = 0
    
    for line in lines:
        if 'Err(e) => {' in line:
            skip_err_block = True
            brace_count = 1
            continue
        
        if skip_err_block:
            brace_count += line.count('{') - line.count('}')
            if brace_count <= 0:
                skip_err_block = False
            continue
        
        fixed_lines.append(line)
    
    with open("src/bin/arbitrage_monitor.rs", 'w') as f:
        f.write('\n'.join(fixed_lines))
    
    print("✅ 大括号问题已修复")

if __name__ == "__main__":
    fix_brace() 

def fix_brace():
    with open("src/bin/arbitrage_monitor.rs", 'r') as f:
        content = f.read()
    
    # 找到问题行并修复
    lines = content.split('\n')
    for i, line in enumerate(lines):
        if 'calculate_profit_batch_optimal(&buy_prices, &sell_prices, &buy_prices); // 第三个参数暂时用buy_prices {' in line:
            # 移除行尾的 {
            lines[i] = line.replace(' {', '')
            # 下一行从 Ok(profits) => { 改为简单的 {
            if i + 1 < len(lines) and 'Ok(profits) => {' in lines[i + 1]:
                lines[i + 1] = '            {'
            break
    
    # 移除多余的 Err 分支
    fixed_lines = []
    skip_err_block = False
    brace_count = 0
    
    for line in lines:
        if 'Err(e) => {' in line:
            skip_err_block = True
            brace_count = 1
            continue
        
        if skip_err_block:
            brace_count += line.count('{') - line.count('}')
            if brace_count <= 0:
                skip_err_block = False
            continue
        
        fixed_lines.append(line)
    
    with open("src/bin/arbitrage_monitor.rs", 'w') as f:
        f.write('\n'.join(fixed_lines))
    
    print("✅ 大括号问题已修复")

if __name__ == "__main__":
    fix_brace() 