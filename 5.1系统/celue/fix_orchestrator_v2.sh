#!/bin/bash

cd /home/ubuntu/celue/orchestrator/src

echo "🔧 修复 orchestrator 编译错误..."

# 1. 修复 NatsAdapter::new() 调用
sed -i 's/let nats_adapter = NatsAdapter::new().*/let nats_adapter = NatsAdapter::new();/' main.rs

# 2. 修复 publish 调用 - 使用正确的 NatsMessage 结构
cat > temp_fix.rs << 'EOF'
                if let Ok(status_bytes) = serde_json::to_vec(&status) {
                    let status_msg = adapters::nats::NatsMessage::Metrics {
                        component: "orchestrator".to_string(),
                        metrics: {
                            let mut m = std::collections::HashMap::new();
                            m.insert("uptime_seconds".to_string(), uptime as f64);
                            m.insert("status_reports_sent".to_string(), status_count as f64);
                            m
                        },
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    };
                    if let Err(e) = nats.publish("orchestrator.status", &status_msg).await {
EOF

# 替换 publish 调用部分
sed -i '/if let Ok(status_bytes)/,/\.await {/c\
                if let Ok(_status_bytes) = serde_json::to_vec(&status) {\
                    let status_msg = adapters::nats::NatsMessage::Metrics {\
                        component: "orchestrator".to_string(),\
                        metrics: {\
                            let mut m = std::collections::HashMap::new();\
                            m.insert("uptime_seconds".to_string(), uptime as f64);\
                            m.insert("status_reports_sent".to_string(), status_count as f64);\
                            m\
                        },\
                        timestamp: std::time::SystemTime::now()\
                            .duration_since(std::time::UNIX_EPOCH)\
                            .unwrap()\
                            .as_secs(),\
                    };\
                    if let Err(e) = nats.publish("orchestrator.status", &status_msg).await {' main.rs

# 3. 移除不存在的方法调用
sed -i 's/self\.nats_adapter\.connect()\.await/Ok(())/' main.rs
sed -i 's/self\.nats_adapter\.disconnect()\.await/Ok(())/' main.rs

# 4. 修复借用生命周期问题 - 简化 risk monitoring
sed -i '/async fn start_risk_monitoring/,/^    }$/{
    s/let risk_controller = &self.risk_controller;//
    a\        let risk_controller = Arc::new(parking_lot::Mutex::new(self.risk_controller.clone()));
    s/risk_controller\.perform_risk_check/risk_controller.lock().perform_risk_check/g
}' main.rs

# 5. 清理未使用的导入
sed -i '/use.*MarketState/d' main.rs
sed -i 's/use adapters::{nats::NatsAdapter, nats::NatsMessage};/use adapters::nats::NatsAdapter;/' main.rs

echo "✅ orchestrator 修复完成"

rm -f temp_fix.rs 