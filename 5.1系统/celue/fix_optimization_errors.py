#!/usr/bin/env python3
"""
修复优化后的编译错误
"""

def fix_compilation_errors():
    print("🔧 修复编译错误...")
    
    file_path = "src/bin/arbitrage_monitor.rs"
    with open(file_path, 'r') as f:
        content = f.read()
    
    # 1. 移除重复的Mutex导入
    content = content.replace('use std::sync::Mutex;', '// use std::sync::Mutex; // 已有tokio::sync::Mutex')
    
    # 2. 移除lazy_static相关代码
    content = content.replace('use lazy_static::lazy_static;', '// lazy_static removed for now')
    content = content.replace('use bumpalo::Bump;', '// bumpalo removed for now')
    
    # 3. 移除内存池相关代码（暂时简化）
    lines = content.split('\n')
    cleaned_lines = []
    skip_lines = False
    
    for line in lines:
        if 'lazy_static::lazy_static!' in line:
            skip_lines = True
            continue
        elif skip_lines and line.strip() == '}':
            skip_lines = False
            continue
        elif skip_lines:
            continue
        elif 'static ref MEMORY_POOL' in line:
            continue
        elif 'struct PoolAllocator' in line:
            skip_lines = True
            continue
        elif 'MEMORY_POOL.lock()' in line:
            # 替换为简单的实现
            cleaned_lines.append('        // Memory pool optimization placeholder')
            continue
        else:
            cleaned_lines.append(line)
    
    content = '\n'.join(cleaned_lines)
    
    # 4. 修复main函数的返回类型问题
    content = content.replace(
        'Ok(())',
        'Ok::<(), Box<dyn std::error::Error>>(())'
    )
    
    # 5. 修复线程池优化导致的语法问题
    content = content.replace(
        '''    rt.spawn(async {
    println!("🎯 启动超高频套利监控器");''',
        '''    println!("🎯 启动超高频套利监控器");'''
    )
    
    # 移除多余的runtime相关代码
    content = content.replace(
        '''    // 优化线程池配置
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(16)  // 16个工作线程
        .max_blocking_threads(32)  // 32个阻塞线程
        .enable_all()
        .build()
        .expect("Failed to create runtime");
    
    rt.spawn(async {''', 
        '    // 线程池优化 - 通过tokio::main配置'
    )
    
    content = content.replace(
        '''    }).await.expect("Runtime spawn failed");
    
    Ok::<(), Box<dyn std::error::Error>>(())''',
        '''    Ok::<(), Box<dyn std::error::Error>>(())'''
    )
    
    with open(file_path, 'w') as f:
        f.write(content)
    
    print("✅ 编译错误修复完成")

def add_lazy_static_dependency():
    """正确添加lazy_static依赖"""
    print("🔧 添加lazy_static依赖...")
    
    with open("Cargo.toml", 'r') as f:
        content = f.read()
    
    # 检查是否已经有lazy_static
    if 'lazy_static' not in content:
        content = content.replace(
            'bytemuck = { workspace = true }',
            '''bytemuck = { workspace = true }
lazy_static = { workspace = true }'''
        )
        
        # 在workspace dependencies中添加
        content = content.replace(
            'lazy_static = "1.4"  # 静态变量初始化',
            'lazy_static = "1.4"              # 静态变量初始化'
        )
    
    with open("Cargo.toml", 'w') as f:
        f.write(content)
    
    print("✅ 依赖添加完成")

def main():
    print("🚀 修复优化后的编译错误...")
    fix_compilation_errors()
    add_lazy_static_dependency()
    print("✅ 修复完成！")

if __name__ == "__main__":
    main() 
"""
修复优化后的编译错误
"""

def fix_compilation_errors():
    print("🔧 修复编译错误...")
    
    file_path = "src/bin/arbitrage_monitor.rs"
    with open(file_path, 'r') as f:
        content = f.read()
    
    # 1. 移除重复的Mutex导入
    content = content.replace('use std::sync::Mutex;', '// use std::sync::Mutex; // 已有tokio::sync::Mutex')
    
    # 2. 移除lazy_static相关代码
    content = content.replace('use lazy_static::lazy_static;', '// lazy_static removed for now')
    content = content.replace('use bumpalo::Bump;', '// bumpalo removed for now')
    
    # 3. 移除内存池相关代码（暂时简化）
    lines = content.split('\n')
    cleaned_lines = []
    skip_lines = False
    
    for line in lines:
        if 'lazy_static::lazy_static!' in line:
            skip_lines = True
            continue
        elif skip_lines and line.strip() == '}':
            skip_lines = False
            continue
        elif skip_lines:
            continue
        elif 'static ref MEMORY_POOL' in line:
            continue
        elif 'struct PoolAllocator' in line:
            skip_lines = True
            continue
        elif 'MEMORY_POOL.lock()' in line:
            # 替换为简单的实现
            cleaned_lines.append('        // Memory pool optimization placeholder')
            continue
        else:
            cleaned_lines.append(line)
    
    content = '\n'.join(cleaned_lines)
    
    # 4. 修复main函数的返回类型问题
    content = content.replace(
        'Ok(())',
        'Ok::<(), Box<dyn std::error::Error>>(())'
    )
    
    # 5. 修复线程池优化导致的语法问题
    content = content.replace(
        '''    rt.spawn(async {
    println!("🎯 启动超高频套利监控器");''',
        '''    println!("🎯 启动超高频套利监控器");'''
    )
    
    # 移除多余的runtime相关代码
    content = content.replace(
        '''    // 优化线程池配置
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(16)  // 16个工作线程
        .max_blocking_threads(32)  // 32个阻塞线程
        .enable_all()
        .build()
        .expect("Failed to create runtime");
    
    rt.spawn(async {''', 
        '    // 线程池优化 - 通过tokio::main配置'
    )
    
    content = content.replace(
        '''    }).await.expect("Runtime spawn failed");
    
    Ok::<(), Box<dyn std::error::Error>>(())''',
        '''    Ok::<(), Box<dyn std::error::Error>>(())'''
    )
    
    with open(file_path, 'w') as f:
        f.write(content)
    
    print("✅ 编译错误修复完成")

def add_lazy_static_dependency():
    """正确添加lazy_static依赖"""
    print("🔧 添加lazy_static依赖...")
    
    with open("Cargo.toml", 'r') as f:
        content = f.read()
    
    # 检查是否已经有lazy_static
    if 'lazy_static' not in content:
        content = content.replace(
            'bytemuck = { workspace = true }',
            '''bytemuck = { workspace = true }
lazy_static = { workspace = true }'''
        )
        
        # 在workspace dependencies中添加
        content = content.replace(
            'lazy_static = "1.4"  # 静态变量初始化',
            'lazy_static = "1.4"              # 静态变量初始化'
        )
    
    with open("Cargo.toml", 'w') as f:
        f.write(content)
    
    print("✅ 依赖添加完成")

def main():
    print("🚀 修复优化后的编译错误...")
    fix_compilation_errors()
    add_lazy_static_dependency()
    print("✅ 修复完成！")

if __name__ == "__main__":
    main() 