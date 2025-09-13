use super::*;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tokio::time::sleep;

impl TestExecutor {
    /// 端到端业务流程详细测试
    pub async fn test_end_to_end_workflows_detailed(&self) -> Vec<TestResult> {
        info!("🔄 执行端到端业务流程详细测试...");
        
        let mut results = Vec::new();

        // 1. 完整套利交易流程测试
        results.extend(self.test_complete_arbitrage_workflow().await);
        
        // 2. 策略配置到执行流程测试
        results.extend(self.test_strategy_configuration_workflow().await);
        
        // 3. 风险监控和响应流程测试
        results.extend(self.test_risk_monitoring_workflow().await);
        
        // 4. 数据清洗和分析流程测试
        results.extend(self.test_data_processing_workflow().await);

        results
    }

    /// 完整套利交易流程测试
    async fn test_complete_arbitrage_workflow(&self) -> Vec<TestResult> {
        let mut results = Vec::new();
        
        // 1. 市场数据获取
        results.push(self.test_market_data_acquisition().await);
        
        // 2. 套利机会识别
        results.push(self.test_arbitrage_opportunity_detection().await);
        
        // 3. 订单执行
        results.push(self.test_order_execution_workflow().await);
        
        // 4. 头寸监控
        results.push(self.test_position_monitoring_workflow().await);
        
        // 5. 盈亏计算
        results.push(self.test_pnl_calculation_workflow().await);

        results
    }

    /// 市场数据获取测试
    async fn test_market_data_acquisition(&self) -> TestResult {
        let start = Instant::now();
        
        let mut workflow_score = 0.0;
        let mut issues = Vec::new();
        let mut total_response_size = 0;

        // 步骤1: 获取交易对列表
        match self.client.get("http://localhost:4005/api/markets/symbols").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 20.0;
                    if let Ok(text) = response.text().await {
                        total_response_size += text.len();
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if json.get("data").and_then(|d| d.as_array()).is_some() {
                                workflow_score += 10.0;
                            }
                        }
                    }
                } else {
                    issues.push("获取交易对列表失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("连接交易服务失败: {}", e));
            }
        }

        // 短暂延迟模拟实际流程
        sleep(Duration::from_millis(100)).await;

        // 步骤2: 获取实时价格数据
        for symbol in ["BTC/USDT", "ETH/USDT", "BNB/USDT"] {
            let url = format!("http://localhost:4005/api/markets/ticker?symbol={}", symbol);
            match self.client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        workflow_score += 8.0;
                        if let Ok(text) = response.text().await {
                            total_response_size += text.len();
                            if let Ok(json) = serde_json::from_str::<Value>(&text) {
                                if json.get("price").is_some() && json.get("volume").is_some() {
                                    workflow_score += 2.0;
                                }
                            }
                        }
                    } else {
                        issues.push(format!("获取{}价格失败", symbol));
                    }
                }
                Err(e) => {
                    issues.push(format!("获取{}价格连接失败: {}", symbol, e));
                }
            }
        }

        // 步骤3: 获取深度数据
        match self.client.get("http://localhost:4005/api/markets/depth?symbol=BTC/USDT&limit=20").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 15.0;
                    if let Ok(text) = response.text().await {
                        total_response_size += text.len();
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if json.get("bids").and_then(|b| b.as_array()).is_some() &&
                               json.get("asks").and_then(|a| a.as_array()).is_some() {
                                workflow_score += 15.0;
                            }
                        }
                    }
                } else {
                    issues.push("获取深度数据失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("获取深度数据连接失败: {}", e));
            }
        }

        TestResult {
            api_name: "市场数据获取流程".to_string(),
            category: "arbitrage_workflow".to_string(),
            method: "GET".to_string(),
            endpoint: "multiple_endpoints".to_string(),
            success: workflow_score >= 70.0,
            response_time: start.elapsed(),
            status_code: Some(200),
            error_message: if issues.is_empty() { None } else { Some(issues.join(", ")) },
            data_integrity_score: workflow_score,
            control_capability_score: workflow_score * 0.8,
            response_size_bytes: total_response_size,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 套利机会识别测试
    async fn test_arbitrage_opportunity_detection(&self) -> TestResult {
        let start = Instant::now();
        
        let mut workflow_score = 0.0;
        let mut issues = Vec::new();

        // 步骤1: 提交价格数据进行套利分析
        let price_data = json!({
            "prices": [
                {
                    "exchange": "binance",
                    "symbol": "BTC/USDT",
                    "price": 45000.0,
                    "timestamp": chrono::Utc::now()
                },
                {
                    "exchange": "okx", 
                    "symbol": "BTC/USDT",
                    "price": 45050.0,
                    "timestamp": chrono::Utc::now()
                }
            ]
        });

        match self.client.post("http://localhost:4003/api/arbitrage/analyze")
            .json(&price_data).send().await {
            Ok(response) => {
                let status_code = response.status();
                let response_text = response.text().await.unwrap_or_default();
                if status_code.is_success() {
                    workflow_score += 40.0;
                    
                    if let Ok(json) = serde_json::from_str::<Value>(&response_text) {
                        if let Some(opportunities) = json.get("opportunities") {
                            if opportunities.is_array() {
                                workflow_score += 30.0;
                                
                                // 检查套利机会的完整性
                                if let Some(ops) = opportunities.as_array() {
                                    for op in ops {
                                        if op.get("profit_percent").is_some() && 
                                           op.get("volume").is_some() &&
                                           op.get("exchanges").is_some() {
                                            workflow_score += 10.0;
                                            break;
                                        }
                                    }
                                }
                            }
                        } else {
                            issues.push("缺少套利机会数据".to_string());
                        }
                    } else {
                        issues.push("响应格式错误".to_string());
                    }
                } else {
                    issues.push("套利分析请求失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("套利分析连接失败: {}", e));
            }
        }

        // 步骤2: 验证套利策略配置
        let strategy_validation = json!({
            "strategy_id": "test_arbitrage_001",
            "min_profit_percent": 0.1,
            "max_volume": 10000.0
        });

        match self.client.post("http://localhost:4003/api/strategies/validate")
            .json(&strategy_validation).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 20.0;
                } else {
                    issues.push("策略验证失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("策略验证连接失败: {}", e));
            }
        }

        TestResult {
            api_name: "套利机会识别流程".to_string(),
            category: "arbitrage_workflow".to_string(),
            method: "POST".to_string(),
            endpoint: "/api/arbitrage/analyze".to_string(),
            success: workflow_score >= 70.0,
            response_time: start.elapsed(),
            status_code: Some(200),
            error_message: if issues.is_empty() { None } else { Some(issues.join(", ")) },
            data_integrity_score: workflow_score,
            control_capability_score: workflow_score * 0.9,
            response_size_bytes: 0,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 订单执行工作流测试
    async fn test_order_execution_workflow(&self) -> TestResult {
        let start = Instant::now();
        
        let mut workflow_score = 0.0;
        let mut issues = Vec::new();

        // 步骤1: 创建买单
        let buy_order = json!({
            "symbol": "BTC/USDT",
            "side": "buy",
            "amount": 0.01,
            "price": 45000.0,
            "order_type": "limit",
            "exchange": "binance"
        });

        let mut buy_order_id = None;

        match self.client.post("http://localhost:4005/api/orders/create")
            .json(&buy_order).send().await {
            Ok(response) => {
                let status_code = response.status();
                let response_text = response.text().await.unwrap_or_default();
                if status_code.is_success() {
                    workflow_score += 25.0;
                    
                    if let Ok(json) = serde_json::from_str::<Value>(&response_text) {
                        if let Some(order_id) = json.get("order_id") {
                            buy_order_id = order_id.as_str().map(|s| s.to_string());
                            workflow_score += 15.0;
                        }
                    }
                } else {
                    issues.push("创建买单失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("创建买单连接失败: {}", e));
            }
        }

        // 短暂延迟模拟订单处理时间
        sleep(Duration::from_millis(200)).await;

        // 步骤2: 创建卖单
        let sell_order = json!({
            "symbol": "BTC/USDT",
            "side": "sell",
            "amount": 0.01,
            "price": 45050.0,
            "order_type": "limit",
            "exchange": "okx"
        });

        let mut sell_order_id = None;

        match self.client.post("http://localhost:4005/api/orders/create")
            .json(&sell_order).send().await {
            Ok(response) => {
                let status_code = response.status();
                let response_text = response.text().await.unwrap_or_default();
                if status_code.is_success() {
                    workflow_score += 25.0;
                    
                    if let Ok(json) = serde_json::from_str::<Value>(&response_text) {
                        if let Some(order_id) = json.get("order_id") {
                            sell_order_id = order_id.as_str().map(|s| s.to_string());
                            workflow_score += 15.0;
                        }
                    }
                } else {
                    issues.push("创建卖单失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("创建卖单连接失败: {}", e));
            }
        }

        // 步骤3: 检查订单状态
        if let Some(order_id) = &buy_order_id {
            let url = format!("http://localhost:4005/api/orders/status?order_id={}", order_id);
            match self.client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        workflow_score += 10.0;
                        
                        if let Ok(text) = response.text().await {
                            if let Ok(json) = serde_json::from_str::<Value>(&text) {
                                if json.get("status").is_some() {
                                    workflow_score += 10.0;
                                }
                            }
                        }
                    } else {
                        issues.push("检查买单状态失败".to_string());
                    }
                }
                Err(e) => {
                    issues.push(format!("检查买单状态连接失败: {}", e));
                }
            }
        }

        TestResult {
            api_name: "订单执行流程".to_string(),
            category: "arbitrage_workflow".to_string(),
            method: "POST".to_string(),
            endpoint: "/api/orders/create".to_string(),
            success: workflow_score >= 70.0,
            response_time: start.elapsed(),
            status_code: Some(200),
            error_message: if issues.is_empty() { None } else { Some(issues.join(", ")) },
            data_integrity_score: workflow_score,
            control_capability_score: workflow_score * 0.95,
            response_size_bytes: 0,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 头寸监控工作流测试
    async fn test_position_monitoring_workflow(&self) -> TestResult {
        let start = Instant::now();
        
        let mut workflow_score = 0.0;
        let mut issues = Vec::new();

        // 步骤1: 获取当前头寸
        match self.client.get("http://localhost:4005/api/positions/current").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 30.0;
                    
                    if let Ok(text) = response.text().await {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if let Some(positions) = json.get("positions") {
                                if positions.is_array() {
                                    workflow_score += 20.0;
                                }
                            }
                        }
                    }
                } else {
                    issues.push("获取头寸失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("获取头寸连接失败: {}", e));
            }
        }

        // 步骤2: 监控头寸风险
        let risk_check = json!({
            "positions": [
                {
                    "symbol": "BTC/USDT",
                    "size": 0.01,
                    "side": "long",
                    "entry_price": 45000.0
                }
            ]
        });

        match self.client.post("http://localhost:4005/api/risk/check")
            .json(&risk_check).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 25.0;
                    
                    if let Ok(text) = response.text().await {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if json.get("risk_level").is_some() {
                                workflow_score += 15.0;
                            }
                        }
                    }
                } else {
                    issues.push("风险检查失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("风险检查连接失败: {}", e));
            }
        }

        // 步骤3: 设置止损订单
        let stop_loss_order = json!({
            "symbol": "BTC/USDT",
            "side": "sell",
            "amount": 0.01,
            "stop_price": 44000.0,
            "order_type": "stop_loss"
        });

        match self.client.post("http://localhost:4005/api/orders/stop_loss")
            .json(&stop_loss_order).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 10.0;
                } else {
                    issues.push("设置止损失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("设置止损连接失败: {}", e));
            }
        }

        TestResult {
            api_name: "头寸监控流程".to_string(),
            category: "arbitrage_workflow".to_string(),
            method: "GET".to_string(),
            endpoint: "/api/positions/current".to_string(),
            success: workflow_score >= 70.0,
            response_time: start.elapsed(),
            status_code: Some(200),
            error_message: if issues.is_empty() { None } else { Some(issues.join(", ")) },
            data_integrity_score: workflow_score,
            control_capability_score: workflow_score * 0.85,
            response_size_bytes: 0,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 盈亏计算工作流测试
    async fn test_pnl_calculation_workflow(&self) -> TestResult {
        let start = Instant::now();
        
        let mut workflow_score = 0.0;
        let mut issues = Vec::new();

        // 步骤1: 获取交易历史
        match self.client.get("http://localhost:4005/api/trades/history?limit=100").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 25.0;
                    
                    if let Ok(text) = response.text().await {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if let Some(trades) = json.get("trades") {
                                if trades.is_array() {
                                    workflow_score += 15.0;
                                }
                            }
                        }
                    }
                } else {
                    issues.push("获取交易历史失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("获取交易历史连接失败: {}", e));
            }
        }

        // 步骤2: 计算实时盈亏
        let pnl_request = json!({
            "positions": [
                {
                    "symbol": "BTC/USDT",
                    "size": 0.01,
                    "side": "long",
                    "entry_price": 45000.0,
                    "current_price": 45050.0
                }
            ]
        });

        match self.client.post("http://localhost:4005/api/pnl/calculate")
            .json(&pnl_request).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 30.0;
                    
                    if let Ok(text) = response.text().await {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if json.get("unrealized_pnl").is_some() && 
                               json.get("realized_pnl").is_some() {
                                workflow_score += 20.0;
                            }
                        }
                    }
                } else {
                    issues.push("计算盈亏失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("计算盈亏连接失败: {}", e));
            }
        }

        // 步骤3: 生成盈亏报告
        match self.client.get("http://localhost:4005/api/reports/pnl/daily").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 10.0;
                } else {
                    issues.push("生成盈亏报告失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("生成盈亏报告连接失败: {}", e));
            }
        }

        TestResult {
            api_name: "盈亏计算流程".to_string(),
            category: "arbitrage_workflow".to_string(),
            method: "POST".to_string(),
            endpoint: "/api/pnl/calculate".to_string(),
            success: workflow_score >= 70.0,
            response_time: start.elapsed(),
            status_code: Some(200),
            error_message: if issues.is_empty() { None } else { Some(issues.join(", ")) },
            data_integrity_score: workflow_score,
            control_capability_score: workflow_score * 0.9,
            response_size_bytes: 0,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 策略配置到执行流程测试
    async fn test_strategy_configuration_workflow(&self) -> Vec<TestResult> {
        let mut results = Vec::new();
        
        results.push(self.test_strategy_setup_workflow().await);
        results.push(self.test_strategy_deployment_workflow().await);
        results.push(self.test_strategy_monitoring_workflow().await);

        results
    }

    /// 策略设置工作流测试
    async fn test_strategy_setup_workflow(&self) -> TestResult {
        let start = Instant::now();
        
        let mut workflow_score = 0.0;
        let mut issues = Vec::new();

        // 步骤1: 创建策略配置
        let strategy_config = json!({
            "name": "高频套利策略V1",
            "type": "arbitrage",
            "parameters": {
                "min_profit_threshold": 0.1,
                "max_position_size": 10000.0,
                "timeout_seconds": 30,
                "exchanges": ["binance", "okx"],
                "symbols": ["BTC/USDT", "ETH/USDT"]
            },
            "risk_limits": {
                "max_daily_loss": 1000.0,
                "max_drawdown": 0.05
            }
        });

        match self.client.post("http://localhost:4003/api/strategies/create")
            .json(&strategy_config).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 30.0;
                    
                    if let Ok(text) = response.text().await {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if json.get("strategy_id").is_some() {
                                workflow_score += 20.0;
                            }
                        }
                    }
                } else {
                    issues.push("创建策略失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("创建策略连接失败: {}", e));
            }
        }

        // 步骤2: 验证策略配置
        match self.client.post("http://localhost:4003/api/strategies/validate")
            .json(&strategy_config).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 25.0;
                    
                    if let Ok(text) = response.text().await {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if json.get("validation_result").and_then(|v| v.get("valid")).and_then(|v| v.as_bool()).unwrap_or(false) {
                                workflow_score += 25.0;
                            } else {
                                issues.push("策略配置验证失败".to_string());
                            }
                        }
                    }
                } else {
                    issues.push("策略验证失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("策略验证连接失败: {}", e));
            }
        }

        TestResult {
            api_name: "策略设置流程".to_string(),
            category: "strategy_workflow".to_string(),
            method: "POST".to_string(),
            endpoint: "/api/strategies/create".to_string(),
            success: workflow_score >= 70.0,
            response_time: start.elapsed(),
            status_code: Some(200),
            error_message: if issues.is_empty() { None } else { Some(issues.join(", ")) },
            data_integrity_score: workflow_score,
            control_capability_score: workflow_score * 0.9,
            response_size_bytes: 0,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 策略部署工作流测试
    async fn test_strategy_deployment_workflow(&self) -> TestResult {
        let start = Instant::now();
        
        let mut workflow_score = 0.0;
        let mut issues = Vec::new();

        // 步骤1: 启动策略
        let start_config = json!({
            "strategy_id": "test_strategy_001",
            "mode": "live"
        });

        match self.client.post("http://localhost:4003/api/strategies/start")
            .json(&start_config).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 40.0;
                } else {
                    issues.push("启动策略失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("启动策略连接失败: {}", e));
            }
        }

        // 短暂延迟等待策略启动
        sleep(Duration::from_millis(300)).await;

        // 步骤2: 检查策略状态
        match self.client.get("http://localhost:4003/api/strategies/status/test_strategy_001").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 30.0;
                    
                    if let Ok(text) = response.text().await {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if let Some(status) = json.get("status") {
                                if status.as_str() == Some("running") {
                                    workflow_score += 30.0;
                                }
                            }
                        }
                    }
                } else {
                    issues.push("检查策略状态失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("检查策略状态连接失败: {}", e));
            }
        }

        TestResult {
            api_name: "策略部署流程".to_string(),
            category: "strategy_workflow".to_string(),
            method: "POST".to_string(),
            endpoint: "/api/strategies/start".to_string(),
            success: workflow_score >= 70.0,
            response_time: start.elapsed(),
            status_code: Some(200),
            error_message: if issues.is_empty() { None } else { Some(issues.join(", ")) },
            data_integrity_score: workflow_score,
            control_capability_score: workflow_score * 0.95,
            response_size_bytes: 0,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 策略监控工作流测试
    async fn test_strategy_monitoring_workflow(&self) -> TestResult {
        let start = Instant::now();
        
        let mut workflow_score = 0.0;
        let mut issues = Vec::new();

        // 步骤1: 获取策略性能指标
        match self.client.get("http://localhost:4003/api/strategies/metrics/test_strategy_001").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 35.0;
                    
                    if let Ok(text) = response.text().await {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if json.get("profit_loss").is_some() && 
                               json.get("total_trades").is_some() &&
                               json.get("win_rate").is_some() {
                                workflow_score += 25.0;
                            }
                        }
                    }
                } else {
                    issues.push("获取策略指标失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("获取策略指标连接失败: {}", e));
            }
        }

        // 步骤2: 检查策略告警
        match self.client.get("http://localhost:4001/api/alerts/strategy/test_strategy_001").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 20.0;
                    
                    if let Ok(text) = response.text().await {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if json.get("alerts").is_some() {
                                workflow_score += 10.0;
                            }
                        }
                    }
                } else {
                    issues.push("检查策略告警失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("检查策略告警连接失败: {}", e));
            }
        }

        // 步骤3: 生成策略报告
        match self.client.get("http://localhost:4003/api/strategies/reports/test_strategy_001?period=1d").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 10.0;
                } else {
                    issues.push("生成策略报告失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("生成策略报告连接失败: {}", e));
            }
        }

        TestResult {
            api_name: "策略监控流程".to_string(),
            category: "strategy_workflow".to_string(),
            method: "GET".to_string(),
            endpoint: "/api/strategies/metrics".to_string(),
            success: workflow_score >= 70.0,
            response_time: start.elapsed(),
            status_code: Some(200),
            error_message: if issues.is_empty() { None } else { Some(issues.join(", ")) },
            data_integrity_score: workflow_score,
            control_capability_score: workflow_score * 0.8,
            response_size_bytes: 0,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 风险监控和响应流程测试
    async fn test_risk_monitoring_workflow(&self) -> Vec<TestResult> {
        let mut results = Vec::new();
        
        results.push(self.test_risk_detection_workflow().await);
        results.push(self.test_risk_response_workflow().await);

        results
    }

    /// 风险检测工作流测试
    async fn test_risk_detection_workflow(&self) -> TestResult {
        let start = Instant::now();
        
        let mut workflow_score = 0.0;
        let mut issues = Vec::new();

        // 步骤1: 实时风险监控
        match self.client.get("http://localhost:4005/api/risk/realtime").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 30.0;
                    
                    if let Ok(text) = response.text().await {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if json.get("risk_metrics").is_some() {
                                workflow_score += 20.0;
                            }
                        }
                    }
                } else {
                    issues.push("实时风险监控失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("实时风险监控连接失败: {}", e));
            }
        }

        // 步骤2: 风险阈值检查
        let risk_data = json!({
            "portfolio_value": 100000.0,
            "daily_pnl": -1500.0,
            "max_drawdown": 0.06,
            "var_95": 2000.0
        });

        match self.client.post("http://localhost:4005/api/risk/threshold_check")
            .json(&risk_data).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 25.0;
                    
                    if let Ok(text) = response.text().await {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if json.get("violations").is_some() {
                                workflow_score += 15.0;
                            }
                        }
                    }
                } else {
                    issues.push("风险阈值检查失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("风险阈值检查连接失败: {}", e));
            }
        }

        // 步骤3: 触发风险告警
        match self.client.get("http://localhost:4001/api/alerts/risk/active").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 10.0;
                } else {
                    issues.push("获取风险告警失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("获取风险告警连接失败: {}", e));
            }
        }

        TestResult {
            api_name: "风险检测流程".to_string(),
            category: "risk_workflow".to_string(),
            method: "GET".to_string(),
            endpoint: "/api/risk/realtime".to_string(),
            success: workflow_score >= 70.0,
            response_time: start.elapsed(),
            status_code: Some(200),
            error_message: if issues.is_empty() { None } else { Some(issues.join(", ")) },
            data_integrity_score: workflow_score,
            control_capability_score: workflow_score * 0.9,
            response_size_bytes: 0,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 风险响应工作流测试
    async fn test_risk_response_workflow(&self) -> TestResult {
        let start = Instant::now();
        
        let mut workflow_score = 0.0;
        let mut issues = Vec::new();

        // 步骤1: 自动风险减仓
        let reduce_positions = json!({
            "risk_level": "high",
            "reduction_percentage": 0.5,
            "positions": ["BTC/USDT", "ETH/USDT"]
        });

        match self.client.post("http://localhost:4005/api/risk/reduce_positions")
            .json(&reduce_positions).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 40.0;
                } else {
                    issues.push("自动减仓失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("自动减仓连接失败: {}", e));
            }
        }

        // 步骤2: 暂停高风险策略
        let pause_strategies = json!({
            "strategy_ids": ["test_strategy_001"],
            "reason": "high_risk_detected"
        });

        match self.client.post("http://localhost:4003/api/strategies/pause")
            .json(&pause_strategies).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 30.0;
                } else {
                    issues.push("暂停策略失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("暂停策略连接失败: {}", e));
            }
        }

        // 步骤3: 发送风险通知
        let risk_notification = json!({
            "level": "critical",
            "message": "系统检测到高风险，已执行自动减仓",
            "channels": ["email", "sms", "webhook"]
        });

        match self.client.post("http://localhost:4001/api/notifications/send")
            .json(&risk_notification).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 30.0;
                } else {
                    issues.push("发送风险通知失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("发送风险通知连接失败: {}", e));
            }
        }

        TestResult {
            api_name: "风险响应流程".to_string(),
            category: "risk_workflow".to_string(),
            method: "POST".to_string(),
            endpoint: "/api/risk/reduce_positions".to_string(),
            success: workflow_score >= 70.0,
            response_time: start.elapsed(),
            status_code: Some(200),
            error_message: if issues.is_empty() { None } else { Some(issues.join(", ")) },
            data_integrity_score: workflow_score,
            control_capability_score: workflow_score,
            response_size_bytes: 0,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 数据清洗和分析流程测试
    async fn test_data_processing_workflow(&self) -> Vec<TestResult> {
        let mut results = Vec::new();
        
        results.push(self.test_data_ingestion_workflow().await);
        results.push(self.test_data_cleaning_workflow().await);
        results.push(self.test_data_analysis_workflow().await);

        results
    }

    /// 数据摄取工作流测试
    async fn test_data_ingestion_workflow(&self) -> TestResult {
        let start = Instant::now();
        
        let mut workflow_score = 0.0;
        let mut issues = Vec::new();

        // 步骤1: 配置数据源
        let data_source_config = json!({
            "sources": [
                {
                    "name": "binance_websocket",
                    "type": "websocket",
                    "url": "wss://stream.binance.com:9443/ws/btcusdt@ticker"
                },
                {
                    "name": "trading_logs",
                    "type": "log_file",
                    "path": "/var/log/trading.log"
                }
            ]
        });

        match self.client.post("http://localhost:4002/api/sources/configure")
            .json(&data_source_config).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 30.0;
                } else {
                    issues.push("配置数据源失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("配置数据源连接失败: {}", e));
            }
        }

        // 步骤2: 启动数据采集
        match self.client.post("http://localhost:4002/api/ingestion/start").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 35.0;
                } else {
                    issues.push("启动数据采集失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("启动数据采集连接失败: {}", e));
            }
        }

        // 步骤3: 检查数据摄取状态
        match self.client.get("http://localhost:4002/api/ingestion/status").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 25.0;
                    
                    if let Ok(text) = response.text().await {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if json.get("active_sources").is_some() {
                                workflow_score += 10.0;
                            }
                        }
                    }
                } else {
                    issues.push("检查摄取状态失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("检查摄取状态连接失败: {}", e));
            }
        }

        TestResult {
            api_name: "数据摄取流程".to_string(),
            category: "data_workflow".to_string(),
            method: "POST".to_string(),
            endpoint: "/api/sources/configure".to_string(),
            success: workflow_score >= 70.0,
            response_time: start.elapsed(),
            status_code: Some(200),
            error_message: if issues.is_empty() { None } else { Some(issues.join(", ")) },
            data_integrity_score: workflow_score,
            control_capability_score: workflow_score * 0.8,
            response_size_bytes: 0,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 数据清洗工作流测试
    async fn test_data_cleaning_workflow(&self) -> TestResult {
        let start = Instant::now();
        
        let mut workflow_score = 0.0;
        let mut issues = Vec::new();

        // 步骤1: 应用清洗规则
        let cleaning_rules = json!({
            "rules": [
                {
                    "name": "remove_duplicates",
                    "type": "deduplication",
                    "fields": ["timestamp", "symbol", "price"]
                },
                {
                    "name": "price_validation",
                    "type": "validation",
                    "min_price": 0.001,
                    "max_price": 100000.0
                }
            ]
        });

        match self.client.post("http://localhost:4002/api/cleaning/apply")
            .json(&cleaning_rules).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 40.0;
                } else {
                    issues.push("应用清洗规则失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("应用清洗规则连接失败: {}", e));
            }
        }

        // 步骤2: 数据质量检查
        match self.client.get("http://localhost:4002/api/quality/check").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 30.0;
                    
                    if let Ok(text) = response.text().await {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if json.get("quality_score").is_some() {
                                workflow_score += 20.0;
                            }
                        }
                    }
                } else {
                    issues.push("数据质量检查失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("数据质量检查连接失败: {}", e));
            }
        }

        // 步骤3: 生成清洗报告
        match self.client.get("http://localhost:4002/api/cleaning/report").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 10.0;
                } else {
                    issues.push("生成清洗报告失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("生成清洗报告连接失败: {}", e));
            }
        }

        TestResult {
            api_name: "数据清洗流程".to_string(),
            category: "data_workflow".to_string(),
            method: "POST".to_string(),
            endpoint: "/api/cleaning/apply".to_string(),
            success: workflow_score >= 70.0,
            response_time: start.elapsed(),
            status_code: Some(200),
            error_message: if issues.is_empty() { None } else { Some(issues.join(", ")) },
            data_integrity_score: workflow_score,
            control_capability_score: workflow_score * 0.85,
            response_size_bytes: 0,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 数据分析工作流测试
    async fn test_data_analysis_workflow(&self) -> TestResult {
        let start = Instant::now();
        
        let mut workflow_score = 0.0;
        let mut issues = Vec::new();

        // 步骤1: 运行AI模型分析
        let analysis_request = json!({
            "model": "price_prediction_v1",
            "data_source": "cleaned_market_data",
            "parameters": {
                "lookback_hours": 24,
                "prediction_horizon": 1
            }
        });

        match self.client.post("http://localhost:4006/api/models/analyze")
            .json(&analysis_request).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 40.0;
                    
                    if let Ok(text) = response.text().await {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if json.get("predictions").is_some() {
                                workflow_score += 25.0;
                            }
                        }
                    }
                } else {
                    issues.push("AI模型分析失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("AI模型分析连接失败: {}", e));
            }
        }

        // 步骤2: 生成分析报告
        match self.client.get("http://localhost:4006/api/analysis/report?period=1h").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 20.0;
                } else {
                    issues.push("生成分析报告失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("生成分析报告连接失败: {}", e));
            }
        }

        // 步骤3: 更新交易信号
        match self.client.post("http://localhost:4003/api/signals/update").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    workflow_score += 15.0;
                } else {
                    issues.push("更新交易信号失败".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("更新交易信号连接失败: {}", e));
            }
        }

        TestResult {
            api_name: "数据分析流程".to_string(),
            category: "data_workflow".to_string(),
            method: "POST".to_string(),
            endpoint: "/api/models/analyze".to_string(),
            success: workflow_score >= 70.0,
            response_time: start.elapsed(),
            status_code: Some(200),
            error_message: if issues.is_empty() { None } else { Some(issues.join(", ")) },
            data_integrity_score: workflow_score,
            control_capability_score: workflow_score * 0.9,
            response_size_bytes: 0,
            timestamp: chrono::Utc::now(),
        }
    }
}