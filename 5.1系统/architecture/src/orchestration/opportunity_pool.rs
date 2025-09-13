//! 全局套利机会池实现
//! 
//! 管理系统中检测到的所有套利机会

use crate::types::ArbitrageOpportunity;
use std::collections::{HashMap, VecDeque};
use chrono::Utc;

/// 全局套利机会池
pub struct GlobalOpportunityPool {
    opportunities: HashMap<String, ArbitrageOpportunity>,
    priority_queue: VecDeque<String>,
    max_size: usize,
}

impl GlobalOpportunityPool {
    /// 创建新的机会池
    pub fn new() -> Self {
        Self {
            opportunities: HashMap::new(),
            priority_queue: VecDeque::new(),
            max_size: 1000,
        }
    }
    
    /// 添加套利机会
    pub fn add_opportunity(&mut self, opportunity: ArbitrageOpportunity) {
        let id = opportunity.id.clone();
        
        // 如果已存在，先移除旧的
        if self.opportunities.contains_key(&id) {
            self.priority_queue.retain(|x| x != &id);
        }
        
        self.opportunities.insert(id.clone(), opportunity);
        self.priority_queue.push_back(id);
        
        // 维护队列大小
        if self.priority_queue.len() > self.max_size {
            if let Some(oldest_id) = self.priority_queue.pop_front() {
                self.opportunities.remove(&oldest_id);
            }
        }
    }
    
    /// 获取所有有效机会
    pub fn get_all(&self) -> Vec<ArbitrageOpportunity> {
        self.opportunities.values().cloned().collect()
    }
    
    /// 获取机会数量
    pub fn size(&self) -> usize {
        self.opportunities.len()
    }
    
    /// 获取活跃机会数量（未过期）
    pub fn active_count(&self) -> usize {
        self.opportunities.values()
            .filter(|opp| !opp.is_expired())
            .count()
    }
    
    /// 移除过期机会
    pub fn remove_expired(&mut self) {
        let now = Utc::now();
        let expired_ids: Vec<_> = self.opportunities.iter()
            .filter(|(_, opp)| opp.expires_at_datetime() < now)
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in expired_ids {
            self.opportunities.remove(&id);
            self.priority_queue.retain(|x| x != &id);
        }
    }
    
    /// 限制池大小
    pub fn limit_size(&mut self, max_size: usize) {
        self.max_size = max_size;
        
        while self.opportunities.len() > max_size {
            if let Some(oldest_id) = self.priority_queue.pop_front() {
                self.opportunities.remove(&oldest_id);
            } else {
                break;
            }
        }
    }
    
    /// 清空机会池
    pub fn clear(&mut self) {
        self.opportunities.clear();
        self.priority_queue.clear();
    }
    
    /// 获取特定机会
    pub fn get_opportunity(&self, id: &str) -> Option<&ArbitrageOpportunity> {
        self.opportunities.get(id)
    }
    
    /// 移除特定机会
    pub fn remove_opportunity(&mut self, id: &str) -> Option<ArbitrageOpportunity> {
        self.priority_queue.retain(|x| x != id);
        self.opportunities.remove(id)
    }
} 