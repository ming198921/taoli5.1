use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::RwLock;
use tracing::{info, debug};

use super::{ServiceInstance, ServiceRegistry};

#[derive(Clone)]
pub struct LoadBalancer {
    registry: Arc<ServiceRegistry>,
    round_robin_counters: Arc<RwLock<std::collections::HashMap<String, AtomicUsize>>>,
    algorithm: LoadBalancingAlgorithm,
}

#[derive(Debug, Clone)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,
    LeastConnections,
    Weighted,
    Random,
}

impl LoadBalancer {
    pub async fn new(registry: Arc<ServiceRegistry>) -> anyhow::Result<Self> {
        Ok(Self {
            registry,
            round_robin_counters: Arc::new(RwLock::new(std::collections::HashMap::new())),
            algorithm: LoadBalancingAlgorithm::RoundRobin,
        })
    }

    pub async fn select_instance(&self, service_name: &str) -> Option<ServiceInstance> {
        let instances = self.registry.get_healthy_instances(service_name).await;
        
        if instances.is_empty() {
            return None;
        }

        match self.algorithm {
            LoadBalancingAlgorithm::RoundRobin => {
                self.round_robin_select(service_name, instances).await
            },
            LoadBalancingAlgorithm::Random => {
                self.random_select(instances)
            },
            LoadBalancingAlgorithm::Weighted => {
                self.weighted_select(instances)
            },
            LoadBalancingAlgorithm::LeastConnections => {
                // 简化实现，暂时使用轮询
                self.round_robin_select(service_name, instances).await
            },
        }
    }

    async fn round_robin_select(
        &self, 
        service_name: &str, 
        instances: Vec<ServiceInstance>
    ) -> Option<ServiceInstance> {
        let mut counters = self.round_robin_counters.write().await;
        let counter = counters
            .entry(service_name.to_string())
            .or_insert_with(|| AtomicUsize::new(0));
        
        let index = counter.fetch_add(1, Ordering::SeqCst) % instances.len();
        
        debug!("Round-robin selected instance at index {} for service {}", 
               index, service_name);
        
        instances.get(index).cloned()
    }

    fn random_select(&self, instances: Vec<ServiceInstance>) -> Option<ServiceInstance> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..instances.len());
        instances.get(index).cloned()
    }

    fn weighted_select(&self, instances: Vec<ServiceInstance>) -> Option<ServiceInstance> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let total_weight: u32 = instances.iter().map(|i| i.weight).sum();
        let mut random_weight = rng.gen_range(0..total_weight);
        
        for instance in instances {
            if random_weight < instance.weight {
                return Some(instance);
            }
            random_weight -= instance.weight;
        }
        
        None
    }

    pub fn set_algorithm(&mut self, algorithm: LoadBalancingAlgorithm) {
        self.algorithm = algorithm;
        info!("Load balancing algorithm changed to {:?}", self.algorithm);
    }
}