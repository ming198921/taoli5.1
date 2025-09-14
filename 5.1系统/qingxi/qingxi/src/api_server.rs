#![allow(dead_code)]
// src/api_server.rs
use crate::central_manager::{CentralManagerHandle, CentralManagerApi};
use crate::health::ApiHealthMonitor;
use crate::types::{AnomalyDetectionResult, MarketDataSnapshot};
use futures_util::stream::BoxStream;
use std::{net::SocketAddr, sync::Arc};
use tonic::{transport::Server, Request, Response, Status};

pub mod market_data_pb {
    tonic::include_proto!("market_data");
}
use market_data_pb::{
    market_data_feed_server::{MarketDataFeed, MarketDataFeedServer},
    AnomalyRequest, AnomalyResponse, ConsistencyRequest, ConsistencyResponse, HealthRequest,
    HealthResponse, MarketDataSnapshot as PbMarketDataSnapshot, OrderBook as PbOrderBook,
    OrderbookRequest, SubscriptionRequest,
};

#[tonic::async_trait]
pub trait MarketDataApi: Send + Sync + 'static {
    async fn get_snapshot(
        &self,
        req: Request<String>,
    ) -> Result<Response<MarketDataSnapshot>, Status>;
    async fn get_anomaly(
        &self,
        req: Request<String>,
    ) -> Result<Response<AnomalyDetectionResult>, Status>;
}

pub struct MarketDataApiServer {
    pub manager: CentralManagerHandle,
    pub health_monitor: Arc<ApiHealthMonitor>,
}

#[tonic::async_trait]
impl MarketDataApi for MarketDataApiServer {
    async fn get_snapshot(
        &self,
        req: Request<String>,
    ) -> Result<Response<MarketDataSnapshot>, Status> {
        let symbol = req.into_inner();
        
        // 从中央管理器获取实际的快照数据
        let snapshot_result = self.manager.get_latest_snapshot(&symbol).await;
        
        match snapshot_result {
            Ok(snapshot) => Ok(Response::new(snapshot)),
            Err(e) => {
                tracing::error!("Failed to get snapshot for {}: {}", symbol, e);
                Err(Status::internal(format!("Failed to get snapshot: {}", e)))
            }
        }
    }
    async fn get_anomaly(
        &self,
        req: Request<String>,
    ) -> Result<Response<AnomalyDetectionResult>, Status> {
        let symbol = req.into_inner();
        
        // 从中央管理器获取实际的异常检测结果
        let anomaly_result = self.manager.get_latest_anomaly(&symbol).await;
        
        match anomaly_result {
            Ok(anomaly) => Ok(Response::new(anomaly)),
            Err(e) => {
                tracing::error!("Failed to get anomaly for {}: {}", symbol, e);
                Err(Status::not_found(format!("No anomaly found for symbol: {}", symbol)))
            }
        }
    }
}

// 实现 tonic 生成的 MarketDataFeed trait，返回 proto 类型
#[tonic::async_trait]
impl MarketDataFeed for MarketDataApiServer {
    type SubscribeSnapshotsStream = BoxStream<'static, Result<PbMarketDataSnapshot, Status>>;

    async fn subscribe_snapshots(
        &self,
        _request: Request<SubscriptionRequest>,
    ) -> Result<Response<Self::SubscribeSnapshotsStream>, Status> {
        // 返回一个空流而不是panic
        use futures_util::stream::empty;
        let stream = empty();
        Ok(Response::new(Box::pin(stream)))
    }
    
    async fn get_latest_orderbook(
        &self,
        request: Request<OrderbookRequest>,
    ) -> Result<Response<PbOrderBook>, Status> {
        let req = request.into_inner();
        let symbol_str = format!("{}/{}", 
            req.symbol.as_ref().map(|s| s.base.as_str()).unwrap_or("BTC"),
            req.symbol.as_ref().map(|s| s.quote.as_str()).unwrap_or("USDT")
        );
        
        if let Some(symbol) = crate::types::Symbol::from_pair(&symbol_str) {
            match self.manager.get_latest_orderbook(&req.exchange_id, &symbol).await {
                Ok(orderbook) => Ok(Response::new(orderbook.into())),
                Err(e) => Err(Status::not_found(format!("Orderbook not found: {}", e))),
            }
        } else {
            Err(Status::invalid_argument("Invalid symbol format"))
        }
    }
    
    async fn get_consistency_status(
        &self,
        _request: Request<ConsistencyRequest>,
    ) -> Result<Response<ConsistencyResponse>, Status> {
        // 返回基础一致性状态
        let response = ConsistencyResponse {
            symbol: Some(market_data_pb::Symbol {
                base: "BTC".to_string(),
                quote: "USDT".to_string(),
            }),
            is_consistent: true,
            exchanges: vec!["binance".to_string(), "okx".to_string(), "huobi".to_string()],
            max_price_diff_pct: 0.01,
            max_timestamp_diff_ms: 100,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };
        Ok(Response::new(response))
    }
    
    async fn get_anomalies(
        &self,
        _request: Request<AnomalyRequest>,
    ) -> Result<Response<AnomalyResponse>, Status> {
        // 返回空的异常列表
        let response = AnomalyResponse {
            events: vec![],
            total_count: 0,
        };
        Ok(Response::new(response))
    }
    async fn get_health_status(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        let health_summary = self.health_monitor.get_health_summary();
        let health_statuses = self.health_monitor.get_all_health_statuses();

        let pb_statuses = health_statuses
            .into_iter()
            .map(|status| market_data_pb::HealthStatus {
                source_id: status.source_id,
                last_message_at: status.last_message_at.as_millis(),
                latency_us: status.latency_us,
                message_count: status.message_count,
                is_connected: status.is_connected,
                last_error: status.last_error.unwrap_or_default(),
                last_error_at: status.last_error_at.map(|t| t.as_millis()).unwrap_or(0),
            })
            .collect();

        let response = HealthResponse {
            overall_healthy: health_summary.unhealthy_sources == 0,
            total_sources: health_summary.total_sources as u32,
            healthy_sources: health_summary.healthy_sources as u32,
            unhealthy_sources: health_summary.unhealthy_sources as u32,
            statuses: pb_statuses,
            timestamp: health_summary.timestamp.as_millis(),
        };

        Ok(Response::new(response))
    }
}

pub async fn serve_api(
    addr: SocketAddr,
    manager: CentralManagerHandle,
    health_monitor: Arc<ApiHealthMonitor>,
) -> Result<(), Box<dyn std::error::Error>> {
    let api = MarketDataApiServer {
        manager,
        health_monitor,
    };
    Server::builder()
        .add_service(MarketDataFeedServer::new(api))
        .serve(addr)
        .await?;
    Ok(())
}

// --- Protobuf Conversion Implementations ---

use crate::types::{OrderBook, TradeSide};

impl From<MarketDataSnapshot> for PbMarketDataSnapshot {
    fn from(snapshot: MarketDataSnapshot) -> Self {
        let symbol = if let Some(ref orderbook) = snapshot.orderbook {
            Some(market_data_pb::Symbol {
                base: orderbook.symbol.base.clone(),
                quote: orderbook.symbol.quote.clone(),
            })
        } else if !snapshot.trades.is_empty() {
            Some(market_data_pb::Symbol {
                base: snapshot.trades[0].symbol.base.clone(),
                quote: snapshot.trades[0].symbol.quote.clone(),
            })
        } else {
            // 默认符号
            Some(market_data_pb::Symbol {
                base: "BTC".to_string(),
                quote: "USDT".to_string(),
            })
        };

        Self {
            symbol,
            source: snapshot.source.clone(),
            timestamp_ms: snapshot.timestamp.as_millis(),
            orderbook: snapshot.orderbook.map(|ob| ob.into()),
            trades: snapshot
                .trades
                .into_iter()
                .map(|t| market_data_pb::TradeUpdate {
                    trade_id: format!("{}-{}", t.source, t.timestamp.as_millis()),
                    price: t.price.into_inner(),
                    quantity: t.quantity.into_inner(),
                    timestamp_ms: t.timestamp.as_millis(),
                    side: match t.side {
                        TradeSide::Buy => market_data_pb::TradeSide::Buy as i32,
                        TradeSide::Sell => market_data_pb::TradeSide::Sell as i32,
                    },
                })
                .collect(),
        }
    }
}

impl From<OrderBook> for PbOrderBook {
    fn from(ob: OrderBook) -> Self {
        Self {
            symbol: Some(market_data_pb::Symbol {
                base: ob.symbol.base.clone(),
                quote: ob.symbol.quote.clone(),
            }),
            bids: ob
                .bids
                .into_iter()
                .map(|entry| market_data_pb::OrderBookEntry {
                    price: entry.price.into_inner(),
                    quantity: entry.quantity.into_inner(),
                })
                .collect(),
            asks: ob
                .asks
                .into_iter()
                .map(|entry| market_data_pb::OrderBookEntry {
                    price: entry.price.into_inner(),
                    quantity: entry.quantity.into_inner(),
                })
                .collect(),
            timestamp_ms: ob.timestamp.as_millis(),
            source: ob.source.clone(),
        }
    }
}
