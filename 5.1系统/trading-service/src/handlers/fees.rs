use axum::{
    extract::{State, Path, Query},
    Json,
};
use serde::{Deserialize, Serialize};
use crate::{AppState, models::{StandardResponse, TradingFee}};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct FeeQuery {
    exchange: Option<String>,
    symbol: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExchangeFeeInfo {
    pub exchange: String,
    pub base_maker_fee: f64,
    pub base_taker_fee: f64,
    pub vip_levels: Vec<VipLevel>,
    pub symbol_specific_fees: HashMap<String, SymbolFee>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct VipLevel {
    pub level: u8,
    pub maker_fee: f64,
    pub taker_fee: f64,
    pub requirements: String,
}

#[derive(Debug, Serialize)]
pub struct SymbolFee {
    pub symbol: String,
    pub maker_fee: f64,
    pub taker_fee: f64,
    pub withdrawal_fee: f64,
    pub deposit_fee: f64,
    pub min_withdrawal: f64,
}

#[derive(Debug, Serialize)]
pub struct FeeCalculation {
    pub trade_amount: f64,
    pub maker_fee_amount: f64,
    pub taker_fee_amount: f64,
    pub net_profit_maker: f64,
    pub net_profit_taker: f64,
    pub breakeven_fee_rate: f64,
}

// GET /api/fees/exchanges - 获取所有交易所费率信息
pub async fn get_all_exchange_fees(
    State(_state): State<AppState>,
) -> Result<Json<StandardResponse<HashMap<String, ExchangeFeeInfo>>>, axum::http::StatusCode> {
    let mut fees = HashMap::new();
    
    // Binance 费率信息
    fees.insert("binance".to_string(), ExchangeFeeInfo {
        exchange: "binance".to_string(),
        base_maker_fee: 0.001,
        base_taker_fee: 0.001,
        vip_levels: vec![
            VipLevel { level: 0, maker_fee: 0.001, taker_fee: 0.001, requirements: "< $50,000 30-day volume".to_string() },
            VipLevel { level: 1, maker_fee: 0.0009, taker_fee: 0.001, requirements: "> $50,000 30-day volume".to_string() },
            VipLevel { level: 2, maker_fee: 0.0008, taker_fee: 0.001, requirements: "> $500,000 30-day volume".to_string() },
            VipLevel { level: 3, maker_fee: 0.0007, taker_fee: 0.0009, requirements: "> $5,000,000 30-day volume".to_string() },
        ],
        symbol_specific_fees: HashMap::new(),
        last_updated: Utc::now(),
    });

    // OKX 费率信息
    fees.insert("okx".to_string(), ExchangeFeeInfo {
        exchange: "okx".to_string(),
        base_maker_fee: 0.0008,
        base_taker_fee: 0.001,
        vip_levels: vec![
            VipLevel { level: 0, maker_fee: 0.0008, taker_fee: 0.001, requirements: "< $100,000 30-day volume".to_string() },
            VipLevel { level: 1, maker_fee: 0.0007, taker_fee: 0.0009, requirements: "> $100,000 30-day volume".to_string() },
            VipLevel { level: 2, maker_fee: 0.0006, taker_fee: 0.0008, requirements: "> $1,000,000 30-day volume".to_string() },
        ],
        symbol_specific_fees: HashMap::new(),
        last_updated: Utc::now(),
    });

    // Huobi 费率信息
    fees.insert("huobi".to_string(), ExchangeFeeInfo {
        exchange: "huobi".to_string(),
        base_maker_fee: 0.002,
        base_taker_fee: 0.002,
        vip_levels: vec![
            VipLevel { level: 0, maker_fee: 0.002, taker_fee: 0.002, requirements: "< $50,000 30-day volume".to_string() },
            VipLevel { level: 1, maker_fee: 0.0018, taker_fee: 0.002, requirements: "> $50,000 30-day volume".to_string() },
        ],
        symbol_specific_fees: HashMap::new(),
        last_updated: Utc::now(),
    });

    // Bybit 费率信息
    fees.insert("bybit".to_string(), ExchangeFeeInfo {
        exchange: "bybit".to_string(),
        base_maker_fee: 0.001,
        base_taker_fee: 0.001,
        vip_levels: vec![
            VipLevel { level: 0, maker_fee: 0.001, taker_fee: 0.001, requirements: "< $50,000 30-day volume".to_string() },
            VipLevel { level: 1, maker_fee: 0.0008, taker_fee: 0.001, requirements: "> $50,000 30-day volume".to_string() },
        ],
        symbol_specific_fees: HashMap::new(),
        last_updated: Utc::now(),
    });

    Ok(Json(StandardResponse::success(fees)))
}

// GET /api/fees/exchanges/{exchange} - 获取指定交易所费率信息
pub async fn get_exchange_fees(
    State(_state): State<AppState>,
    Path(exchange): Path<String>,
) -> Result<Json<StandardResponse<ExchangeFeeInfo>>, axum::http::StatusCode> {
    let fee_info = match exchange.as_str() {
        "binance" => ExchangeFeeInfo {
            exchange: "binance".to_string(),
            base_maker_fee: 0.001,
            base_taker_fee: 0.001,
            vip_levels: vec![
                VipLevel { level: 0, maker_fee: 0.001, taker_fee: 0.001, requirements: "< $50,000 30-day volume".to_string() },
                VipLevel { level: 1, maker_fee: 0.0009, taker_fee: 0.001, requirements: "> $50,000 30-day volume".to_string() },
            ],
            symbol_specific_fees: HashMap::new(),
            last_updated: Utc::now(),
        },
        "okx" => ExchangeFeeInfo {
            exchange: "okx".to_string(),
            base_maker_fee: 0.0008,
            base_taker_fee: 0.001,
            vip_levels: vec![
                VipLevel { level: 0, maker_fee: 0.0008, taker_fee: 0.001, requirements: "< $100,000 30-day volume".to_string() },
            ],
            symbol_specific_fees: HashMap::new(),
            last_updated: Utc::now(),
        },
        "huobi" => ExchangeFeeInfo {
            exchange: "huobi".to_string(),
            base_maker_fee: 0.002,
            base_taker_fee: 0.002,
            vip_levels: vec![
                VipLevel { level: 0, maker_fee: 0.002, taker_fee: 0.002, requirements: "< $50,000 30-day volume".to_string() },
            ],
            symbol_specific_fees: HashMap::new(),
            last_updated: Utc::now(),
        },
        "bybit" => ExchangeFeeInfo {
            exchange: "bybit".to_string(),
            base_maker_fee: 0.001,
            base_taker_fee: 0.001,
            vip_levels: vec![
                VipLevel { level: 0, maker_fee: 0.001, taker_fee: 0.001, requirements: "< $50,000 30-day volume".to_string() },
            ],
            symbol_specific_fees: HashMap::new(),
            last_updated: Utc::now(),
        },
        _ => return Err(axum::http::StatusCode::NOT_FOUND),
    };

    Ok(Json(StandardResponse::success(fee_info)))
}

// GET /api/fees/symbols/{symbol} - 获取指定交易对在所有交易所的费率
pub async fn get_symbol_fees(
    State(_state): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<Json<StandardResponse<Vec<TradingFee>>>, axum::http::StatusCode> {
    let exchanges = vec!["binance", "okx", "huobi", "bybit"];
    let mut symbol_fees = Vec::new();

    for exchange in exchanges {
        let (maker_fee, taker_fee) = match exchange {
            "binance" => (0.001, 0.001),
            "okx" => (0.0008, 0.001),
            "huobi" => (0.002, 0.002),
            "bybit" => (0.001, 0.001),
            _ => (0.001, 0.001),
        };

        symbol_fees.push(TradingFee {
            exchange: exchange.to_string(),
            symbol: symbol.clone(),
            maker_fee,
            taker_fee,
            withdrawal_fee: 0.0005, // 默认提现手续费
            deposit_fee: 0.0,       // 默认充值手续费
            last_updated: Utc::now(),
        });
    }

    Ok(Json(StandardResponse::success(symbol_fees)))
}

// POST /api/fees/calculate - 计算交易费用
pub async fn calculate_trading_fees(
    State(_state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<StandardResponse<FeeCalculation>>, axum::http::StatusCode> {
    let trade_amount = request["trade_amount"].as_f64().unwrap_or(0.0);
    let exchange = request["exchange"].as_str().unwrap_or("binance");
    let symbol = request["symbol"].as_str().unwrap_or("BTCUSDT");

    let (maker_fee_rate, taker_fee_rate) = match exchange {
        "binance" => (0.001, 0.001),
        "okx" => (0.0008, 0.001),
        "huobi" => (0.002, 0.002),
        "bybit" => (0.001, 0.001),
        _ => (0.001, 0.001),
    };

    let maker_fee_amount = trade_amount * maker_fee_rate;
    let taker_fee_amount = trade_amount * taker_fee_rate;

    // 计算净利润（假设价格差为1%）
    let assumed_profit_rate = 0.01;
    let gross_profit = trade_amount * assumed_profit_rate;
    let net_profit_maker = gross_profit - maker_fee_amount;
    let net_profit_taker = gross_profit - taker_fee_amount;

    // 计算盈亏平衡费率
    let breakeven_fee_rate = assumed_profit_rate / 2.0; // 假设买卖各一次

    let calculation = FeeCalculation {
        trade_amount,
        maker_fee_amount,
        taker_fee_amount,
        net_profit_maker,
        net_profit_taker,
        breakeven_fee_rate,
    };

    Ok(Json(StandardResponse::success(calculation)))
}

// GET /api/fees/comparison - 费率比较
pub async fn compare_fees(
    State(_state): State<AppState>,
    Query(params): Query<FeeQuery>,
) -> Result<Json<StandardResponse<serde_json::Value>>, axum::http::StatusCode> {
    let exchanges = vec!["binance", "okx", "huobi", "bybit"];
    let mut comparison = Vec::new();

    for exchange in exchanges {
        let (maker_fee, taker_fee) = match exchange {
            "binance" => (0.001, 0.001),
            "okx" => (0.0008, 0.001),
            "huobi" => (0.002, 0.002),
            "bybit" => (0.001, 0.001),
            _ => (0.001, 0.001),
        };

        comparison.push(serde_json::json!({
            "exchange": exchange,
            "maker_fee": maker_fee,
            "taker_fee": taker_fee,
            "average_fee": (maker_fee + taker_fee) / 2.0,
            "competitiveness_score": 1.0 - ((maker_fee + taker_fee) / 2.0) / 0.002 // 相对于最高费率的得分
        }));
    }

    // 按平均费率排序
    comparison.sort_by(|a, b| {
        a["average_fee"].as_f64().unwrap()
            .partial_cmp(&b["average_fee"].as_f64().unwrap())
            .unwrap()
    });

    let result = serde_json::json!({
        "comparison": comparison,
        "lowest_fees": &comparison[0],
        "highest_fees": &comparison[comparison.len() - 1],
        "analysis_time": Utc::now()
    });

    Ok(Json(StandardResponse::success(result)))
}

// PUT /api/fees/exchanges/{exchange} - 更新交易所费率信息（用于动态费率获取）
pub async fn update_exchange_fees(
    State(_state): State<AppState>,
    Path(exchange): Path<String>,
    Json(_fee_data): Json<serde_json::Value>,
) -> Result<Json<StandardResponse<String>>, axum::http::StatusCode> {
    // 这里应该调用交易所API获取最新费率
    // 目前返回模拟结果
    Ok(Json(StandardResponse::success(
        format!("{} 交易所费率已更新", exchange)
    )))
}

// POST /api/fees/refresh - 刷新所有交易所费率
pub async fn refresh_all_fees(
    State(_state): State<AppState>,
) -> Result<Json<StandardResponse<serde_json::Value>>, axum::http::StatusCode> {
    // 模拟刷新所有交易所费率
    let result = serde_json::json!({
        "updated_exchanges": ["binance", "okx", "huobi", "bybit"],
        "updated_at": Utc::now(),
        "status": "success"
    });

    Ok(Json(StandardResponse::success(result)))
}

// GET /api/fees/arbitrage-analysis - 套利费用分析
pub async fn analyze_arbitrage_fees(
    State(_state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<StandardResponse<serde_json::Value>>, axum::http::StatusCode> {
    let symbol = params.get("symbol").unwrap_or(&"BTCUSDT".to_string()).clone();
    let amount = params.get("amount")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(10000.0);

    // 计算不同交易所组合的套利成本
    let exchanges = vec![
        ("binance", 0.001, 0.001),
        ("okx", 0.0008, 0.001),
        ("huobi", 0.002, 0.002),
        ("bybit", 0.001, 0.001),
    ];

    let mut arbitrage_costs = Vec::new();

    for i in 0..exchanges.len() {
        for j in i + 1..exchanges.len() {
            let (ex1, maker1, taker1) = exchanges[i];
            let (ex2, maker2, taker2) = exchanges[j];

            // 计算双向套利成本
            let cost_1_to_2 = amount * (taker1 + taker2); // 假设都是taker订单
            let cost_2_to_1 = amount * (taker2 + taker1);
            let min_cost = cost_1_to_2.min(cost_2_to_1);

            // 计算盈亏平衡点
            let breakeven_spread = (taker1 + taker2) * 100.0; // 转换为百分比

            arbitrage_costs.push(serde_json::json!({
                "pair": format!("{} <-> {}", ex1, ex2),
                "buy_exchange": ex1,
                "sell_exchange": ex2,
                "total_fee_cost": min_cost,
                "fee_percentage": min_cost / amount * 100.0,
                "breakeven_spread_percent": breakeven_spread,
                "recommendation": if breakeven_spread < 0.5 { "推荐" } else { "谨慎" }
            }));
        }
    }

    // 按费用成本排序
    arbitrage_costs.sort_by(|a, b| {
        a["total_fee_cost"].as_f64().unwrap()
            .partial_cmp(&b["total_fee_cost"].as_f64().unwrap())
            .unwrap()
    });

    let result = serde_json::json!({
        "symbol": symbol,
        "analysis_amount": amount,
        "arbitrage_opportunities": arbitrage_costs,
        "best_pair": arbitrage_costs.first(),
        "worst_pair": arbitrage_costs.last(),
        "analyzed_at": Utc::now()
    });

    Ok(Json(StandardResponse::success(result)))
}