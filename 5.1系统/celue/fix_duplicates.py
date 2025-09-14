#!/usr/bin/env python3
"""
精准清理重复代码的手术刀式脚本
保持代码架构完整性，只移除重复定义
"""

import re
import sys

def fix_arbitrage_monitor():
    """精准修复arbitrage_monitor.rs的重复定义"""
    file_path = "src/bin/arbitrage_monitor.rs"
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        print(f"🔍 分析文件: {file_path}")
        print(f"📊 原文件大小: {len(content)} 字符")
        
        # 1. 添加colored import（如果缺失）
        if "use colored::*;" not in content:
            # 找到imports区域并添加
            import_pattern = r"(use std::sync::Arc;\n)"
            if re.search(import_pattern, content):
                content = re.sub(import_pattern, r"\1use colored::*;\n", content)
                print("✅ 添加了colored导入")
        
        # 2. 精准识别重复的impl ArbitrageMonitor块
        impl_pattern = r"impl ArbitrageMonitor \{[^{}]*(?:\{[^{}]*\}[^{}]*)*\}"
        impl_matches = list(re.finditer(impl_pattern, content, re.DOTALL))
        
        print(f"🔍 发现 {len(impl_matches)} 个impl块")
        
        if len(impl_matches) > 1:
            # 保留第一个最完整的impl块，删除重复的
            kept_impl = impl_matches[0]
            print(f"✅ 保留第一个impl块 (位置: {kept_impl.start()}-{kept_impl.end()})")
            
            # 从后往前删除重复的impl块，避免位置偏移
            for i in range(len(impl_matches) - 1, 0, -1):
                impl_match = impl_matches[i]
                print(f"🗑️ 删除重复impl块 {i} (位置: {impl_match.start()}-{impl_match.end()})")
                content = content[:impl_match.start()] + content[impl_match.end():]
        
        # 3. 清理重复的结构体定义
        struct_pattern = r"#\[derive.*?\]\s*pub struct ArbitrageMonitor \{[^{}]*\}"
        struct_matches = list(re.finditer(struct_pattern, content, re.DOTALL))
        
        if len(struct_matches) > 1:
            print(f"🔍 发现 {len(struct_matches)} 个重复结构体定义")
            # 保留第一个，删除其他
            for i in range(len(struct_matches) - 1, 0, -1):
                struct_match = struct_matches[i]
                print(f"🗑️ 删除重复结构体定义 {i}")
                content = content[:struct_match.start()] + content[struct_match.end():]
        
        # 4. 清理重复的main函数
        main_pattern = r"#\[tokio::main\]\s*async fn main\(\)[^{}]*\{[^{}]*(?:\{[^{}]*\}[^{}]*)*\}"
        main_matches = list(re.finditer(main_pattern, content, re.DOTALL))
        
        if len(main_matches) > 1:
            print(f"🔍 发现 {len(main_matches)} 个重复main函数")
            # 保留最后一个，删除其他
            for i in range(len(main_matches) - 1):
                main_match = main_matches[i]
                print(f"🗑️ 删除重复main函数 {i}")
                content = content[:main_match.start()] + content[main_match.end():]
        
        # 5. 清理多余的空行
        content = re.sub(r'\n\n\n+', '\n\n', content)
        
        print(f"📊 清理后文件大小: {len(content)} 字符")
        
        # 写回文件
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        
        print("✅ 文件清理完成")
        return True
        
    except Exception as e:
        print(f"❌ 错误: {e}")
        return False

def fix_cargo_toml():
    """确保Cargo.toml包含colored依赖"""
    file_path = "Cargo.toml"
    
    try:
        with open(file_path, 'r') as f:
            content = f.read()
        
        if 'colored = "2.0"' not in content:
            # 在dependencies区域添加colored
            if '[dependencies]' in content:
                content = re.sub(
                    r'(\[dependencies\]\n)',
                    r'\1colored = "2.0"\n',
                    content
                )
                print("✅ 添加colored依赖到Cargo.toml")
                
                with open(file_path, 'w') as f:
                    f.write(content)
        else:
            print("✅ colored依赖已存在")
            
        return True
        
    except Exception as e:
        print(f"❌ Cargo.toml修复失败: {e}")
        return False

def main():
    """主函数"""
    print("🔧 开始精准清理重复代码...")
    
    success = True
    
    # 1. 修复Cargo.toml
    if not fix_cargo_toml():
        success = False
    
    # 2. 修复arbitrage_monitor.rs
    if not fix_arbitrage_monitor():
        success = False
    
    if success:
        print("🎉 所有重复代码已精准清理完成！")
        print("💡 建议运行: cargo check 验证修复效果")
    else:
        print("❌ 清理过程中遇到问题，请检查错误信息")
    
    return 0 if success else 1

if __name__ == "__main__":
    sys.exit(main()) 