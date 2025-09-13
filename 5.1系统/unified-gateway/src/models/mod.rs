use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardRequest<T = serde_json::Value> {
    pub data: Option<T>,
    pub metadata: Option<RequestMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    pub request_id: String,
    pub timestamp: i64,
    pub version: String,
    pub client_info: Option<ClientInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub user_agent: String,
    pub ip_address: String,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardResponse<T = serde_json::Value> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ErrorInfo>,
    pub metadata: ResponseMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub request_id: String,
    pub timestamp: i64,
    pub execution_time_ms: u64,
    pub server_info: ServerInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub service: String,
    pub version: String,
    pub instance_id: String,
}

impl<T: Serialize> StandardResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata: ResponseMetadata {
                request_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().timestamp(),
                execution_time_ms: 0,
                server_info: ServerInfo {
                    service: "unified-gateway".to_string(),
                    version: "1.0.0".to_string(),
                    instance_id: std::env::var("INSTANCE_ID")
                        .unwrap_or_else(|_| "default".to_string()),
                },
            },
        }
    }

    pub fn error(code: String, message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ErrorInfo {
                code,
                message,
                details: None,
            }),
            metadata: ResponseMetadata {
                request_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().timestamp(),
                execution_time_ms: 0,
                server_info: ServerInfo {
                    service: "unified-gateway".to_string(),
                    version: "1.0.0".to_string(),
                    instance_id: std::env::var("INSTANCE_ID")
                        .unwrap_or_else(|_| "default".to_string()),
                },
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Internal server error: {0}")]
    InternalError(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Forbidden")]
    Forbidden,
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    
    #[error("No healthy instance available for service: {0}")]
    NoHealthyInstance(String),
    
    #[error("Proxy error: {0}")]
    ProxyError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

impl ApiError {
    pub fn error_code(&self) -> String {
        match self {
            ApiError::InternalError(_) => "INTERNAL_ERROR".to_string(),
            ApiError::InvalidRequest(_) => "INVALID_REQUEST".to_string(),
            ApiError::Unauthorized => "UNAUTHORIZED".to_string(),
            ApiError::Forbidden => "FORBIDDEN".to_string(),
            ApiError::NotFound(_) => "NOT_FOUND".to_string(),
            ApiError::ServiceNotFound(_) => "SERVICE_NOT_FOUND".to_string(),
            ApiError::NoHealthyInstance(_) => "NO_HEALTHY_INSTANCE".to_string(),
            ApiError::ProxyError(_) => "PROXY_ERROR".to_string(),
            ApiError::DatabaseError(_) => "DATABASE_ERROR".to_string(),
            ApiError::ValidationError(_) => "VALIDATION_ERROR".to_string(),
            ApiError::RateLimitExceeded => "RATE_LIMIT_EXCEEDED".to_string(),
        }
    }
    
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::ServiceNotFound(_) => StatusCode::BAD_GATEWAY,
            ApiError::NoHealthyInstance(_) => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::ProxyError(_) => StatusCode::BAD_GATEWAY,
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let response = StandardResponse::<()>::error(
            self.error_code(),
            self.to_string(),
        );
        
        (status, Json(response)).into_response()
    }
}