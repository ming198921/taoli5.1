#!/usr/bin/env python3

def fix_syntax():
    with open("src/bin/arbitrage_monitor.rs", 'r') as f:
        lines = f.readlines()
    
    # 找到并移除有问题的代码块
    cleaned_lines = []
    skip = False
    
    for i, line in enumerate(lines):
        if "// 全局内存池" in line:
            skip = True
            cleaned_lines.append("// Memory pool optimization removed for compilation\n")
            continue
        elif skip and line.strip() == "use colored::*;":
            skip = False
            cleaned_lines.append(line)
            continue
        elif skip:
            continue
        else:
            cleaned_lines.append(line)
    
    with open("src/bin/arbitrage_monitor.rs", 'w') as f:
        f.writelines(cleaned_lines)
    
    print("✅ 语法错误修复完成")

if __name__ == "__main__":
    fix_syntax() 

def fix_syntax():
    with open("src/bin/arbitrage_monitor.rs", 'r') as f:
        lines = f.readlines()
    
    # 找到并移除有问题的代码块
    cleaned_lines = []
    skip = False
    
    for i, line in enumerate(lines):
        if "// 全局内存池" in line:
            skip = True
            cleaned_lines.append("// Memory pool optimization removed for compilation\n")
            continue
        elif skip and line.strip() == "use colored::*;":
            skip = False
            cleaned_lines.append(line)
            continue
        elif skip:
            continue
        else:
            cleaned_lines.append(line)
    
    with open("src/bin/arbitrage_monitor.rs", 'w') as f:
        f.writelines(cleaned_lines)
    
    print("✅ 语法错误修复完成")

if __name__ == "__main__":
    fix_syntax() 