#![allow(dead_code)]
//! # 交易所发现和符号获取模块
//!
//! 提供从交易所REST API获取所有可用交易对的功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};

/// 交易对信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingPair {
    pub symbol: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub status: String,
    pub min_qty: Option<String>,
    pub max_qty: Option<String>,
    pub step_size: Option<String>,
    pub min_price: Option<String>,
    pub max_price: Option<String>,
    pub tick_size: Option<String>,
}

/// 交易所信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeInfo {
    pub exchange_id: String,
    pub name: String,
    pub status: String,
    pub supported: bool,
    pub has_testnet: bool,
    pub rate_limits: Vec<RateLimit>,
    pub trading_pairs: Vec<TradingPair>,
}

/// 速率限制信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub rate_limit_type: String,
    pub interval: String,
    pub interval_num: u32,
    pub limit: u32,
}

/// 交易所发现器
pub struct ExchangeDiscovery {
    supported_exchanges: HashMap<String, ExchangeMetadata>,
}

#[derive(Debug, Clone)]
struct ExchangeMetadata {
    name: String,
    rest_api_base: String,
    testnet_api_base: Option<String>,
    symbols_endpoint: String,
    exchange_info_endpoint: String,
}

impl ExchangeDiscovery {
    pub fn new() -> Self {
        let mut supported_exchanges = HashMap::new();

        // Binance
        supported_exchanges.insert("binance".to_string(), ExchangeMetadata {
            name: "Binance".to_string(),
            rest_api_base: "https://api.binance.com".to_string(),
            testnet_api_base: Some("https://testnet.binance.vision".to_string()),
            symbols_endpoint: "/api/v3/exchangeInfo".to_string(),
            exchange_info_endpoint: "/api/v3/exchangeInfo".to_string(),
        });

        // OKX
        supported_exchanges.insert("okx".to_string(), ExchangeMetadata {
            name: "OKX".to_string(),
            rest_api_base: "https://www.okx.com".to_string(),
            testnet_api_base: None,
            symbols_endpoint: "/api/v5/public/instruments?instType=SPOT".to_string(),
            exchange_info_endpoint: "/api/v5/public/status".to_string(),
        });

        // Huobi
        supported_exchanges.insert("huobi".to_string(), ExchangeMetadata {
            name: "Huobi".to_string(),
            rest_api_base: "https://api.huobi.pro".to_string(),
            testnet_api_base: None,
            symbols_endpoint: "/v1/common/symbols".to_string(),
            exchange_info_endpoint: "/v1/common/currencys".to_string(),
        });

        // Bybit
        supported_exchanges.insert("bybit".to_string(), ExchangeMetadata {
            name: "Bybit".to_string(),
            rest_api_base: "https://api.bybit.com".to_string(),
            testnet_api_base: Some("https://api-testnet.bybit.com".to_string()),
            symbols_endpoint: "/v5/market/instruments-info?category=spot".to_string(),
            exchange_info_endpoint: "/v5/market/instruments-info?category=spot".to_string(),
        });

        Self { supported_exchanges }
    }

    /// 获取所有支持的交易所列表
    pub fn get_supported_exchanges(&self) -> Vec<ExchangeInfo> {
        self.supported_exchanges.iter().map(|(id, metadata)| {
            ExchangeInfo {
                exchange_id: id.clone(),
                name: metadata.name.clone(),
                status: "available".to_string(),
                supported: true,
                has_testnet: metadata.testnet_api_base.is_some(),
                rate_limits: vec![], // 将在获取详细信息时填充
                trading_pairs: vec![], // 将在获取符号时填充
            }
        }).collect()
    }

    /// 从特定交易所获取所有交易对
    pub async fn fetch_trading_pairs(&self, exchange_id: &str, testnet: bool) -> Result<Vec<TradingPair>, String> {
        let metadata = self.supported_exchanges.get(exchange_id)
            .ok_or_else(|| format!("Unsupported exchange: {}", exchange_id))?;

        let base_url = if testnet && metadata.testnet_api_base.is_some() {
            metadata.testnet_api_base.as_ref().expect("Testnet API base not configured")
        } else {
            &metadata.rest_api_base
        };

        let url = format!("{}{}", base_url, metadata.symbols_endpoint);
        
        match exchange_id {
            "binance" => self.fetch_binance_symbols(&url).await,
            "okx" => self.fetch_okx_symbols(&url).await,
            "huobi" => self.fetch_huobi_symbols(&url).await,
            "bybit" => self.fetch_bybit_symbols(&url).await,
            _ => Err(format!("Symbol fetching not implemented for {}", exchange_id)),
        }
    }

    /// 获取Binance交易对
    async fn fetch_binance_symbols(&self, url: &str) -> Result<Vec<TradingPair>, String> {
        let client = reqwest::Client::new();
        let response = client.get(url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()));
        }

        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let symbols = json["symbols"].as_array()
            .ok_or("Invalid response format")?;

        let mut trading_pairs = Vec::new();
        for symbol in symbols {
            if let (Some(symbol_name), Some(base), Some(quote), Some(status)) = (
                symbol["symbol"].as_str(),
                symbol["baseAsset"].as_str(),
                symbol["quoteAsset"].as_str(),
                symbol["status"].as_str(),
            ) {
                trading_pairs.push(TradingPair {
                    symbol: symbol_name.to_string(),
                    base_asset: base.to_string(),
                    quote_asset: quote.to_string(),
                    status: status.to_string(),
                    min_qty: symbol["filters"][2]["minQty"].as_str().map(|s| s.to_string()),
                    max_qty: symbol["filters"][2]["maxQty"].as_str().map(|s| s.to_string()),
                    step_size: symbol["filters"][2]["stepSize"].as_str().map(|s| s.to_string()),
                    min_price: symbol["filters"][0]["minPrice"].as_str().map(|s| s.to_string()),
                    max_price: symbol["filters"][0]["maxPrice"].as_str().map(|s| s.to_string()),
                    tick_size: symbol["filters"][0]["tickSize"].as_str().map(|s| s.to_string()),
                });
            }
        }

        info!("Fetched {} trading pairs from Binance", trading_pairs.len());
        Ok(trading_pairs)
    }

    /// 获取OKX交易对
    async fn fetch_okx_symbols(&self, url: &str) -> Result<Vec<TradingPair>, String> {
        let client = reqwest::Client::new();
        let response = client.get(url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()));
        }

        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let data = json["data"].as_array()
            .ok_or("Invalid response format")?;

        let mut trading_pairs = Vec::new();
        for instrument in data {
            if let (Some(inst_id), Some(base_ccy), Some(quote_ccy), Some(state)) = (
                instrument["instId"].as_str(),
                instrument["baseCcy"].as_str(),
                instrument["quoteCcy"].as_str(),
                instrument["state"].as_str(),
            ) {
                trading_pairs.push(TradingPair {
                    symbol: inst_id.to_string(),
                    base_asset: base_ccy.to_string(),
                    quote_asset: quote_ccy.to_string(),
                    status: state.to_string(),
                    min_qty: instrument["minSz"].as_str().map(|s| s.to_string()),
                    max_qty: instrument["maxSz"].as_str().map(|s| s.to_string()),
                    step_size: instrument["lotSz"].as_str().map(|s| s.to_string()),
                    min_price: None,
                    max_price: None,
                    tick_size: instrument["tickSz"].as_str().map(|s| s.to_string()),
                });
            }
        }

        info!("Fetched {} trading pairs from OKX", trading_pairs.len());
        Ok(trading_pairs)
    }

    /// 获取Huobi交易对
    async fn fetch_huobi_symbols(&self, url: &str) -> Result<Vec<TradingPair>, String> {
        let client = reqwest::Client::new();
        let response = client.get(url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()));
        }

        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let data = json["data"].as_array()
            .ok_or("Invalid response format")?;

        let mut trading_pairs = Vec::new();
        for symbol in data {
            if let (Some(symbol_name), Some(base), Some(quote), Some(state)) = (
                symbol["symbol"].as_str(),
                symbol["base-currency"].as_str(),
                symbol["quote-currency"].as_str(),
                symbol["state"].as_str(),
            ) {
                trading_pairs.push(TradingPair {
                    symbol: symbol_name.to_string(),
                    base_asset: base.to_string(),
                    quote_asset: quote.to_string(),
                    status: state.to_string(),
                    min_qty: symbol["amount-precision"].as_u64().map(|p| format!("1e-{}", p)),
                    max_qty: None,
                    step_size: symbol["amount-precision"].as_u64().map(|p| format!("1e-{}", p)),
                    min_price: symbol["price-precision"].as_u64().map(|p| format!("1e-{}", p)),
                    max_price: None,
                    tick_size: symbol["price-precision"].as_u64().map(|p| format!("1e-{}", p)),
                });
            }
        }

        info!("Fetched {} trading pairs from Huobi", trading_pairs.len());
        Ok(trading_pairs)
    }

    /// 获取Bybit交易对
    async fn fetch_bybit_symbols(&self, url: &str) -> Result<Vec<TradingPair>, String> {
        let client = reqwest::Client::new();
        let response = client.get(url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()));
        }

        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let list = json["result"]["list"].as_array()
            .ok_or("Invalid response format")?;

        let mut trading_pairs = Vec::new();
        for instrument in list {
            if let (Some(symbol), Some(base_coin), Some(quote_coin), Some(status)) = (
                instrument["symbol"].as_str(),
                instrument["baseCoin"].as_str(),
                instrument["quoteCoin"].as_str(),
                instrument["status"].as_str(),
            ) {
                trading_pairs.push(TradingPair {
                    symbol: symbol.to_string(),
                    base_asset: base_coin.to_string(),
                    quote_asset: quote_coin.to_string(),
                    status: status.to_string(),
                    min_qty: instrument["lotSizeFilter"]["minOrderQty"].as_str().map(|s| s.to_string()),
                    max_qty: instrument["lotSizeFilter"]["maxOrderQty"].as_str().map(|s| s.to_string()),
                    step_size: instrument["lotSizeFilter"]["qtyStep"].as_str().map(|s| s.to_string()),
                    min_price: instrument["priceFilter"]["minPrice"].as_str().map(|s| s.to_string()),
                    max_price: instrument["priceFilter"]["maxPrice"].as_str().map(|s| s.to_string()),
                    tick_size: instrument["priceFilter"]["tickSize"].as_str().map(|s| s.to_string()),
                });
            }
        }

        info!("Fetched {} trading pairs from Bybit", trading_pairs.len());
        Ok(trading_pairs)
    }

    /// 获取特定交易所的完整信息（包含交易对）
    pub async fn get_exchange_info(&self, exchange_id: &str, testnet: bool) -> Result<ExchangeInfo, String> {
        let mut exchange_info = self.get_supported_exchanges()
            .into_iter()
            .find(|e| e.exchange_id == exchange_id)
            .ok_or_else(|| format!("Exchange {} not found", exchange_id))?;

        // 获取交易对
        match self.fetch_trading_pairs(exchange_id, testnet).await {
            Ok(pairs) => {
                exchange_info.trading_pairs = pairs;
                exchange_info.status = "online".to_string();
            },
            Err(e) => {
                warn!("Failed to fetch trading pairs for {}: {}", exchange_id, e);
                exchange_info.status = "error".to_string();
            }
        }

        Ok(exchange_info)
    }

    /// 按Quote资产过滤交易对
    pub fn filter_by_quote_asset(&self, pairs: &[TradingPair], quote_asset: &str) -> Vec<TradingPair> {
        pairs.iter()
            .filter(|pair| pair.quote_asset.to_uppercase() == quote_asset.to_uppercase())
            .cloned()
            .collect()
    }

    /// 搜索交易对
    pub fn search_trading_pairs(&self, pairs: &[TradingPair], query: &str) -> Vec<TradingPair> {
        let query_upper = query.to_uppercase();
        pairs.iter()
            .filter(|pair| {
                pair.symbol.to_uppercase().contains(&query_upper) ||
                pair.base_asset.to_uppercase().contains(&query_upper) ||
                pair.quote_asset.to_uppercase().contains(&query_upper)
            })
            .cloned()
            .collect()
    }
}

impl Default for ExchangeDiscovery {
    fn default() -> Self {
        Self::new()
    }
}
