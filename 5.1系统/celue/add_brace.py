#!/usr/bin/env python3

def add_missing_brace():
    with open("src/bin/arbitrage_monitor.rs", 'r') as f:
        lines = f.readlines()
    
    # 在第260行前插入一个大括号
    for i, line in enumerate(lines):
        if line.strip() == "fn convert_to_price_point(data: &CelueMarketData) -> PricePoint {":
            # 在这行前插入一个大括号
            lines.insert(i, "}\n")
            break
    
    with open("src/bin/arbitrage_monitor.rs", 'w') as f:
        f.writelines(lines)
    
    print("✅ 添加了缺失的大括号")

if __name__ == "__main__":
    add_missing_brace() 

def add_missing_brace():
    with open("src/bin/arbitrage_monitor.rs", 'r') as f:
        lines = f.readlines()
    
    # 在第260行前插入一个大括号
    for i, line in enumerate(lines):
        if line.strip() == "fn convert_to_price_point(data: &CelueMarketData) -> PricePoint {":
            # 在这行前插入一个大括号
            lines.insert(i, "}\n")
            break
    
    with open("src/bin/arbitrage_monitor.rs", 'w') as f:
        f.writelines(lines)
    
    print("✅ 添加了缺失的大括号")

if __name__ == "__main__":
    add_missing_brace() 