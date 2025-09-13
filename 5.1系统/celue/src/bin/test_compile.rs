fn main() {
    println!("Testing strategy module compilation...");
    
    // 测试策略模块是否可以正确导入
    use celue::strategy::*;
    use celue::performance::*;
    
    println!("✅ Strategy module compiled successfully!");
} 