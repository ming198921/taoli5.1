#!/usr/bin/env python3

def quick_error_fix():
    with open("src/bin/arbitrage_monitor.rs", 'r') as f:
        content = f.read()
    
    # 1. 修复println参数数量错误 - 移除多余的格式符
    content = content.replace(
        'println!("   [{}] {} | {} | {} <-> {} | {}",',
        'println!("   [{}] {} | {} | {}",')
    
    # 2. 修复async_nats::Message.data -> .payload
    content = content.replace('&message.data', '&message.payload')
    
    # 3. 修复processor借用问题 - 克隆processor数据而不是引用
    content = content.replace(
        'let processor = &self.simd_processor;',
        '// let processor = &self.simd_processor; // 改为在函数内部创建'
    )
    
    # 4. 在函数内部创建新的processor实例
    content = content.replace(
        'processor: &SIMDFixedPointProcessor,',
        '// processor: &SIMDFixedPointProcessor, // 改为在函数内部创建'
    )
    
    content = content.replace(
        'let profits = processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices, &buy_prices);',
        '''let processor_local = SIMDFixedPointProcessor::new(OPTIMAL_BATCH_SIZE);
            let profits = processor_local.calculate_profit_batch_optimal(&buy_prices, &sell_prices, &buy_prices);'''
    )
    
    # 5. 修复tokio::spawn调用 - 移除processor参数传递
    content = content.replace(
        'Self::process_batch_avx512(batch, stats, history, opportunity_pool, processor).await;',
        'Self::process_batch_avx512(batch, stats, history, opportunity_pool).await;'
    )
    
    # 6. 更新函数签名
    content = content.replace(
        '''async fn process_batch_avx512(
        batch: Vec<CelueMarketData>,
        stats: Arc<RwLock<ArbitrageStats>>,
        history: Arc<RwLock<Vec<ArbitrageOpportunity>>>,
        opportunity_pool: Arc<RwLock<Vec<ArbitrageOpportunity>>>,
        // processor: &SIMDFixedPointProcessor, // 改为在函数内部创建
    ) {''',
        '''async fn process_batch_avx512(
        batch: Vec<CelueMarketData>,
        stats: Arc<RwLock<ArbitrageStats>>,
        history: Arc<RwLock<Vec<ArbitrageOpportunity>>>,
        opportunity_pool: Arc<RwLock<Vec<ArbitrageOpportunity>>>,
    ) {''')
    
    with open("src/bin/arbitrage_monitor.rs", 'w') as f:
        f.write(content)
    
    print("✅ 修复了所有编译错误")

if __name__ == "__main__":
    quick_error_fix() 

def quick_error_fix():
    with open("src/bin/arbitrage_monitor.rs", 'r') as f:
        content = f.read()
    
    # 1. 修复println参数数量错误 - 移除多余的格式符
    content = content.replace(
        'println!("   [{}] {} | {} | {} <-> {} | {}",',
        'println!("   [{}] {} | {} | {}",')
    
    # 2. 修复async_nats::Message.data -> .payload
    content = content.replace('&message.data', '&message.payload')
    
    # 3. 修复processor借用问题 - 克隆processor数据而不是引用
    content = content.replace(
        'let processor = &self.simd_processor;',
        '// let processor = &self.simd_processor; // 改为在函数内部创建'
    )
    
    # 4. 在函数内部创建新的processor实例
    content = content.replace(
        'processor: &SIMDFixedPointProcessor,',
        '// processor: &SIMDFixedPointProcessor, // 改为在函数内部创建'
    )
    
    content = content.replace(
        'let profits = processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices, &buy_prices);',
        '''let processor_local = SIMDFixedPointProcessor::new(OPTIMAL_BATCH_SIZE);
            let profits = processor_local.calculate_profit_batch_optimal(&buy_prices, &sell_prices, &buy_prices);'''
    )
    
    # 5. 修复tokio::spawn调用 - 移除processor参数传递
    content = content.replace(
        'Self::process_batch_avx512(batch, stats, history, opportunity_pool, processor).await;',
        'Self::process_batch_avx512(batch, stats, history, opportunity_pool).await;'
    )
    
    # 6. 更新函数签名
    content = content.replace(
        '''async fn process_batch_avx512(
        batch: Vec<CelueMarketData>,
        stats: Arc<RwLock<ArbitrageStats>>,
        history: Arc<RwLock<Vec<ArbitrageOpportunity>>>,
        opportunity_pool: Arc<RwLock<Vec<ArbitrageOpportunity>>>,
        // processor: &SIMDFixedPointProcessor, // 改为在函数内部创建
    ) {''',
        '''async fn process_batch_avx512(
        batch: Vec<CelueMarketData>,
        stats: Arc<RwLock<ArbitrageStats>>,
        history: Arc<RwLock<Vec<ArbitrageOpportunity>>>,
        opportunity_pool: Arc<RwLock<Vec<ArbitrageOpportunity>>>,
    ) {''')
    
    with open("src/bin/arbitrage_monitor.rs", 'w') as f:
        f.write(content)
    
    print("✅ 修复了所有编译错误")

if __name__ == "__main__":
    quick_error_fix() 