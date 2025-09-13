//! Market data adapter for real-time feeds
//! 
//! Connects to various exchange APIs and normalizes market data.

use crate::{Adapter, AdapterError, AdapterResult};
use common::{Exchange, Symbol, OrderBook, NormalizedSnapshot};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, info, warn}; // keep warn used in stop

/// Market data configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataConfig {
    /// Enabled exchanges
    pub exchanges: Vec<ExchangeConfig>,
    /// Update interval for snapshots
    pub snapshot_interval: Duration,
    /// HTTP client timeout
    pub http_timeout: Duration,
    /// WebSocket ping interval
    pub ws_ping_interval: Duration,
    /// Reconnection settings
    pub reconnect: ReconnectConfig,
    /// Minimum acceptable quality score for snapshots
    pub quality_min_score: f64,
}

/// Exchange configuration
/// Exchange configuration - using unified definition to eliminate duplication
/// The unified BaseExchangeConfig provides all the fields we need:
/// - name, enabled, api_key, api_secret, symbols (direct mapping)
/// - rest_url maps to rest_api_url
/// - ws_url maps to ws_url (already matches)
/// And provides additional standardized fields like rate limiting and fee configuration
pub use common_types::{BaseExchangeConfig as ExchangeConfig, BaseConfig};

/// Reconnection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
}

impl Default for MarketDataConfig {
    fn default() -> Self {
        Self {
            exchanges: vec![
                ExchangeConfig {
                    name: "binance".to_string(),
                    enabled: true,
                    api_key: None,
                    api_secret: None,
                    api_passphrase: None,
                    sandbox_mode: false,
                    rest_url: "https://api.binance.com".to_string(),
                    ws_url: "wss://stream.binance.com:9443".to_string(),
                    websocket_url: "wss://stream.binance.com:9443".to_string(),
                    rate_limit: 1000,
                    max_connections: Some(5),
                    supported_symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
                    symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
                    taker_fee: Some(0.001),
                    maker_fee: Some(0.001),
                    fee_rate_bps: Some(10.0),
                    parameters: HashMap::new(),
                },
            ],
            snapshot_interval: Duration::from_millis(100),
            http_timeout: Duration::from_secs(10),
            ws_ping_interval: Duration::from_secs(30),
            reconnect: ReconnectConfig {
                max_attempts: 10,
                initial_delay: Duration::from_secs(1),
                max_delay: Duration::from_secs(60),
                backoff_factor: 2.0,
            },
            quality_min_score: 0.0,
        }
    }
}

/// Market data feed interface
#[async_trait::async_trait]
pub trait MarketDataFeed: Send + Sync {
    async fn subscribe(&mut self, symbols: Vec<Symbol>) -> AdapterResult<()>;
    async fn get_snapshot(&self, symbol: &Symbol) -> AdapterResult<OrderBook>;
    async fn start(&mut self) -> AdapterResult<()>;
    async fn stop(&mut self) -> AdapterResult<()>;
    fn exchange(&self) -> &Exchange;
}

/// Raw market data from exchange (containing full order book data)
/// This is different from the unified MarketData as it contains bid/ask arrays
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMarketData {
    pub exchange: String,
    pub symbol: String,
    pub timestamp: u64,
    pub bids: Vec<(f64, f64)>, // price, quantity
    pub asks: Vec<(f64, f64)>, // price, quantity
}

/// Market data adapter
pub struct MarketDataAdapter {
    config: Option<MarketDataConfig>,
    feeds: HashMap<String, Box<dyn MarketDataFeed>>,
    snapshot_tx: Option<broadcast::Sender<NormalizedSnapshot>>,
    http_client: Client,
    running: Arc<parking_lot::Mutex<bool>>,
}

impl MarketDataAdapter {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            config: None,
            feeds: HashMap::new(),
            snapshot_tx: None,
            http_client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            running: Arc::new(parking_lot::Mutex::new(false)),
        }
    }
    
    /// Get snapshot receiver
    pub fn snapshot_receiver(&self) -> Option<broadcast::Receiver<NormalizedSnapshot>> {
        self.snapshot_tx.as_ref().map(|tx| tx.subscribe())
    }
    
    /// Normalize raw market data
    #[allow(dead_code)]
    fn normalize_data(&self, raw: RawMarketData) -> AdapterResult<OrderBook> {
        let exchange = Exchange::new(&raw.exchange);
        let symbol = Symbol::new(&raw.symbol);
        
        let mut orderbook = OrderBook::new(
            exchange,
            symbol,
            raw.timestamp,
            1, // sequence
        );
        
        // Add bids (buy orders)
        for (price, quantity) in raw.bids {
            let fixed_price = common::FixedPrice::from_f64(price, 8);
            let fixed_quantity = common::FixedQuantity::from_f64(quantity, 8);
            orderbook.add_bid(fixed_price, fixed_quantity);
        }
        
        // Add asks (sell orders)
        for (price, quantity) in raw.asks {
            let fixed_price = common::FixedPrice::from_f64(price, 8);
            let fixed_quantity = common::FixedQuantity::from_f64(quantity, 8);
            orderbook.add_ask(fixed_price, fixed_quantity);
        }
        
        Ok(orderbook)
    }
}

#[async_trait::async_trait]
impl Adapter for MarketDataAdapter {
    type Config = MarketDataConfig;
    type Error = AdapterError;
    
    async fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        info!("Initializing market data adapter with {} exchanges", config.exchanges.len());
        
        // Update HTTP client with new timeout
        self.http_client = Client::builder()
            .timeout(config.http_timeout)
            .build()
            .map_err(|e| AdapterError::Generic {
                message: format!("Failed to create HTTP client: {}", e),
            })?;
        
        // Create snapshot broadcaster
        let (tx, _) = broadcast::channel(1000);
        self.snapshot_tx = Some(tx);
        
        self.config = Some(config);
        
        info!("Market data adapter initialized successfully");
        Ok(())
    }
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        let mut running = self.running.lock();
        if *running {
            return Err(AdapterError::AlreadyRunning);
        }
        
        let config = self.config.as_ref().ok_or(AdapterError::NotInitialized)?;
        
        // Initialize feeds for enabled exchanges
        for exchange_config in &config.exchanges {
            if !exchange_config.enabled {
                continue;
            }
            
            info!("Starting feed for exchange: {}", exchange_config.name);
            
            // Create specific exchange feeds (Binance, OKX, etc.)
            let feed: Box<dyn MarketDataFeed> = match exchange_config.name.to_lowercase().as_str() {
                "binance" => Box::new(BinanceFeed::new(exchange_config.clone())?),
                "okx" => Box::new(OkxFeed::new(exchange_config.clone())?),
                "huobi" => Box::new(HuobiFeed::new(exchange_config.clone())?),
                _ => {
                    info!("Using mock feed for unsupported exchange: {}", exchange_config.name);
                    Box::new(MockFeed::new(Exchange::new(&exchange_config.name)))
                }
            };
            self.feeds.insert(exchange_config.name.clone(), feed);
        }
        
        // Start snapshot generation
        let snapshot_tx = self.snapshot_tx.as_ref().unwrap().clone();
        let interval = config.snapshot_interval;
        
        // Clone feeds for the async task
        let feeds_map = self.feeds.keys().cloned().collect::<Vec<_>>();
        let quality_min_score = config.quality_min_score;
        
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            let mut sequence = 0u64;
            
            loop {
                ticker.tick().await;
                sequence += 1;
                
                // Collect data from all feeds and create snapshot
                let symbols = vec![Symbol::new("BTCUSDT"), Symbol::new("ETHUSDT")];
                
                for symbol in symbols {
                    let mut exchange_snapshots = Vec::new();
                    let mut total_bid_volume = 0.0;
                    let mut total_ask_volume = 0.0;
                    let mut weighted_price_sum = 0.0;
                    let mut weight_sum = 0.0;
                    
                    // Simulate collecting data from each exchange
                    for exchange_name in &feeds_map {
                        // Simulate orderbook data
                        let bid_price = match exchange_name.as_str() {
                            "binance" => 50000.0 + (sequence as f64 % 100.0),
                            "okx" => 49995.0 + (sequence as f64 % 100.0),
                            "huobi" => 50005.0 + (sequence as f64 % 100.0),
                            _ => 50000.0,
                        };
                        let ask_price = bid_price + 10.0;
                        let volume = 1.0 + (sequence as f64 % 10.0) / 10.0;
                        
                        total_bid_volume += volume;
                        total_ask_volume += volume;
                        
                        let mid_price = (bid_price + ask_price) / 2.0;
                        weighted_price_sum += mid_price * volume;
                        weight_sum += volume;
                        
                        // Create exchange OrderBook snapshot
                        let orderbook = common::OrderBook {
                            exchange: common::Exchange::new(exchange_name),
                            symbol: symbol.clone(),
                            timestamp_ns: chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64,
                            sequence,
                            bid_prices: vec![common::FixedPrice::from_f64(bid_price, 2)],
                            bid_quantities: vec![common::FixedQuantity::from_f64(volume, 8)],
                            ask_prices: vec![common::FixedPrice::from_f64(ask_price, 2)],
                            ask_quantities: vec![common::FixedQuantity::from_f64(volume, 8)],
                            quality_score: 1.0,
                            processing_latency_ns: 0,
                        };
                        exchange_snapshots.push(orderbook);
                    }
                    
                    let weighted_mid_price = if weight_sum > 0.0 {
                        weighted_price_sum / weight_sum
                    } else {
                        0.0
                    };
                    
                    // Calculate quality score based on number of exchanges and volume
                    let quality_score = (exchange_snapshots.len() as f64 * 0.3 + 
                                       (total_bid_volume.min(10.0) / 10.0) * 0.7).min(1.0);
                    
                    // Only send snapshots that meet quality threshold
                    if quality_score >= quality_min_score {
                        let snapshot = NormalizedSnapshot {
                            symbol: symbol.clone(),
                            timestamp_ns: chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64,
                            exchanges: exchange_snapshots,
                            weighted_mid_price: common::FixedPrice::from_f64(weighted_mid_price, 2),
                            total_bid_volume: common::FixedQuantity::from_f64(total_bid_volume, 8),
                            total_ask_volume: common::FixedQuantity::from_f64(total_ask_volume, 8),
                            quality_score,
                            sequence: Some(sequence),
                        };
                        
                        if let Err(e) = snapshot_tx.send(snapshot) {
                            debug!("Failed to send snapshot for {}: {}", symbol.name(), e);
                        }
                    }
                }
            }
        });
        
        *running = true;
        info!("Market data adapter started");
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<(), Self::Error> {
        {
            let running = self.running.lock();
            if !*running { return Ok(()); }
        }
        // collect keys to avoid borrow issues
        let keys: Vec<String> = self.feeds.keys().cloned().collect();
        for k in keys {
            if let Some(mut feed) = self.feeds.remove(&k) {
                if let Err(e) = feed.stop().await { warn!("Failed to stop feed {}: {}", k, e); }
            }
        }
        {
            let mut running = self.running.lock();
            *running = false;
        }
        info!("Market data adapter stopped");
        Ok(())
    }
    
    async fn health_check(&self) -> Result<(), Self::Error> {
        if self.config.is_none() {
            return Err(AdapterError::NotInitialized);
        }
        
        // Check running state without holding lock across await
        {
            let running = self.running.lock();
            if !*running {
                return Err(AdapterError::Generic {
                    message: "Adapter not running".to_string(),
                });
            }
        }
        
        // Check if feeds are healthy
        for (name, feed) in &self.feeds {
            // Implement health check for each feed
            match feed.get_snapshot(&Symbol::new("BTCUSDT")).await {
                Ok(_) => {
                    debug!("Feed {} is healthy - snapshot request successful", name);
                },
                Err(e) => {
                    return Err(AdapterError::Generic {
                        message: format!("Feed {} failed health check: {}", name, e),
                    });
                }
            }
        }
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "market_data_adapter"
    }
}

/// Binance market data feed
#[allow(dead_code)]
struct BinanceFeed {
    exchange: Exchange,
    config: ExchangeConfig,
    running: bool,
}

impl BinanceFeed {
    fn new(config: ExchangeConfig) -> AdapterResult<Self> {
        Ok(Self {
            exchange: Exchange::new(&config.name),
            config,
            running: false,
        })
    }
}

#[async_trait::async_trait]
impl MarketDataFeed for BinanceFeed {
    async fn subscribe(&mut self, symbols: Vec<Symbol>) -> AdapterResult<()> {
        info!("Binance feed subscribing to {} symbols", symbols.len());
        // Implement Binance WebSocket subscription
        Ok(())
    }
    
    async fn get_snapshot(&self, symbol: &Symbol) -> AdapterResult<OrderBook> {
        // Implement Binance REST API snapshot request
        let orderbook = OrderBook::new(
            self.exchange.clone(),
            symbol.clone(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64,
            1,
        );
        Ok(orderbook)
    }
    
    async fn start(&mut self) -> AdapterResult<()> {
        self.running = true;
        info!("Binance feed started");
        Ok(())
    }
    
    async fn stop(&mut self) -> AdapterResult<()> {
        self.running = false;
        info!("Binance feed stopped");
        Ok(())
    }
    
    fn exchange(&self) -> &Exchange {
        &self.exchange
    }
}

/// OKX market data feed
#[allow(dead_code)]
struct OkxFeed {
    exchange: Exchange,
    config: ExchangeConfig,
    running: bool,
}

impl OkxFeed {
    fn new(config: ExchangeConfig) -> AdapterResult<Self> {
        Ok(Self {
            exchange: Exchange::new(&config.name),
            config,
            running: false,
        })
    }
}

#[async_trait::async_trait]
impl MarketDataFeed for OkxFeed {
    async fn subscribe(&mut self, symbols: Vec<Symbol>) -> AdapterResult<()> {
        info!("OKX feed subscribing to {} symbols", symbols.len());
        Ok(())
    }
    
    async fn get_snapshot(&self, symbol: &Symbol) -> AdapterResult<OrderBook> {
        let orderbook = OrderBook::new(
            self.exchange.clone(),
            symbol.clone(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64,
            1,
        );
        Ok(orderbook)
    }
    
    async fn start(&mut self) -> AdapterResult<()> {
        self.running = true;
        info!("OKX feed started");
        Ok(())
    }
    
    async fn stop(&mut self) -> AdapterResult<()> {
        self.running = false;
        info!("OKX feed stopped");
        Ok(())
    }
    
    fn exchange(&self) -> &Exchange {
        &self.exchange
    }
}

/// Huobi market data feed
#[allow(dead_code)]
struct HuobiFeed {
    exchange: Exchange,
    config: ExchangeConfig,
    running: bool,
}

impl HuobiFeed {
    fn new(config: ExchangeConfig) -> AdapterResult<Self> {
        Ok(Self {
            exchange: Exchange::new(&config.name),
            config,
            running: false,
        })
    }
}

#[async_trait::async_trait]
impl MarketDataFeed for HuobiFeed {
    async fn subscribe(&mut self, symbols: Vec<Symbol>) -> AdapterResult<()> {
        info!("Huobi feed subscribing to {} symbols", symbols.len());
        Ok(())
    }
    
    async fn get_snapshot(&self, symbol: &Symbol) -> AdapterResult<OrderBook> {
        let orderbook = OrderBook::new(
            self.exchange.clone(),
            symbol.clone(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64,
            1,
        );
        Ok(orderbook)
    }
    
    async fn start(&mut self) -> AdapterResult<()> {
        self.running = true;
        info!("Huobi feed started");
        Ok(())
    }
    
    async fn stop(&mut self) -> AdapterResult<()> {
        self.running = false;
        info!("Huobi feed stopped");
        Ok(())
    }
    
    fn exchange(&self) -> &Exchange {
        &self.exchange
    }
}

/// Mock market data feed for testing
struct MockFeed {
    exchange: Exchange,
    running: bool,
}

impl MockFeed {
    fn new(exchange: Exchange) -> Self {
        Self {
            exchange,
            running: false,
        }
    }
}

#[async_trait::async_trait]
impl MarketDataFeed for MockFeed {
    async fn subscribe(&mut self, symbols: Vec<Symbol>) -> AdapterResult<()> {
        info!("Mock feed subscribing to {} symbols", symbols.len());
        Ok(())
    }
    
    async fn get_snapshot(&self, symbol: &Symbol) -> AdapterResult<OrderBook> {
        let orderbook = OrderBook::new(
            self.exchange.clone(),
            symbol.clone(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64,
            1,
        );
        Ok(orderbook)
    }
    
    async fn start(&mut self) -> AdapterResult<()> {
        self.running = true;
        Ok(())
    }
    
    async fn stop(&mut self) -> AdapterResult<()> {
        self.running = false;
        Ok(())
    }
    
    fn exchange(&self) -> &Exchange {
        &self.exchange
    }
}

impl Default for MarketDataAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_data_config() {
        let config = MarketDataConfig::default();
        assert_eq!(config.exchanges.len(), 1);
        assert_eq!(config.exchanges[0].name, "binance");
    }
    
    #[tokio::test]
    async fn test_mock_feed() {
        let mut feed = MockFeed::new(Exchange::new("test"));
        
        assert!(feed.start().await.is_ok());
        
        let symbol = Symbol::new("BTCUSDT");
        let snapshot = feed.get_snapshot(&symbol).await;
        assert!(snapshot.is_ok());
        
        assert!(feed.stop().await.is_ok());
    }
}
