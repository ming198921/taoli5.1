/// 服务代理模块 - 连接各个后端服务
pub mod auth_service;
pub mod system_service;
pub mod qingxi_service;
pub mod dashboard_service;
pub mod monitoring_service;
pub mod service_registry;

// 重新导出服务特征  
pub use crate::api_gateway::{
    AuthServiceTrait, 
    SystemServiceTrait, 
    QingxiServiceTrait, 
    DashboardServiceTrait
};

// 监控服务使用自己的特征定义
pub use monitoring_service::MonitoringService as MonitoringServiceTrait;

// 重新导出服务实现
pub use auth_service::AuthService;
pub use system_service::SystemService;
pub use qingxi_service::QingxiService;
pub use dashboard_service::DashboardService;
pub use monitoring_service::ProductionMonitoringService;
pub use service_registry::ServiceRegistry;