#!/usr/bin/env python3
import os
import re

def clean_file_by_removing_duplicates(file_path, line_cutoff=None):
    """通过删除指定行号之后的内容来清理文件"""
    if not os.path.exists(file_path):
        return
    
    with open(file_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    if line_cutoff:
        lines = lines[:line_cutoff]
        # 确保文件以合适的结尾
        if not lines[-1].strip().endswith('}'):
            lines.append('}\n')
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.writelines(lines)
    
    print(f"已清理文件: {file_path} (保留前{line_cutoff}行)")

def add_missing_imports():
    """添加缺失的imports"""
    
    # 修复market_state.rs的HashMap导入
    market_state_path = 'src/strategy/market_state.rs'
    if os.path.exists(market_state_path):
        with open(market_state_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        if 'use std::collections::HashMap;' not in content:
            content = content.replace(
                'use std::sync::Arc;',
                'use std::collections::HashMap;\nuse std::sync::Arc;'
            )
        
        with open(market_state_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"已添加HashMap导入: {market_state_path}")

def fix_missing_structs():
    """添加缺失的结构体定义"""
    
    # 在scheduler.rs中添加缺失的结构体
    scheduler_path = 'src/strategy/scheduler.rs'
    if os.path.exists(scheduler_path):
        with open(scheduler_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # 在文件末尾添加缺失的结构体
        missing_structs = '''
/// 调度统计信息
#[derive(Debug, Clone)]
pub struct SchedulingStatistics {
    pub total_tasks_scheduled: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub average_execution_time_ms: f64,
    pub current_queue_size: usize,
}

impl Default for SchedulingStatistics {
    fn default() -> Self {
        Self {
            total_tasks_scheduled: 0,
            tasks_completed: 0,
            tasks_failed: 0,
            average_execution_time_ms: 0.0,
            current_queue_size: 0,
        }
    }
}

impl StrategySchedulerClone {
    async fn execute_task(&self, _task: SchedulingTask) {
        // 实现任务执行逻辑
    }
}
'''
        
        if 'pub struct SchedulingStatistics' not in content:
            content = content.replace(
                '// End of scheduler module',
                missing_structs + '\n// End of scheduler module'
            )
        
        with open(scheduler_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"已添加缺失结构体: {scheduler_path}")
    
    # 在failure_detector.rs中添加缺失的结构体
    failure_detector_path = 'src/strategy/failure_detector.rs'
    if os.path.exists(failure_detector_path):
        with open(failure_detector_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        failure_stats_struct = '''
/// 故障统计信息
#[derive(Debug, Clone)]
pub struct FailureStatistics {
    pub total_failures: u64,
    pub failures_by_strategy: std::collections::HashMap<String, u64>,
    pub recovery_success_rate: f64,
    pub average_recovery_time_ms: f64,
}

impl Default for FailureStatistics {
    fn default() -> Self {
        Self {
            total_failures: 0,
            failures_by_strategy: std::collections::HashMap::new(),
            recovery_success_rate: 0.0,
            average_recovery_time_ms: 0.0,
        }
    }
}
'''
        
        if 'pub struct FailureStatistics' not in content:
            # 在文件末尾添加
            content += '\n' + failure_stats_struct
        
        with open(failure_detector_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"已添加FailureStatistics: {failure_detector_path}")

def main():
    """主函数 - 执行所有清理操作"""
    
    # 彻底清理有重复定义的文件
    files_to_clean = [
        ('src/strategy/config_manager.rs', 530),  # 只保留前530行
        ('src/strategy/market_state.rs', 665),    # 只保留前665行
    ]
    
    for file_path, cutoff in files_to_clean:
        clean_file_by_removing_duplicates(file_path, cutoff)
    
    # 添加缺失的imports
    add_missing_imports()
    
    # 添加缺失的结构体定义
    fix_missing_structs()
    
    print("所有清理工作完成!")

if __name__ == "__main__":
    main() 
import os
import re

def clean_file_by_removing_duplicates(file_path, line_cutoff=None):
    """通过删除指定行号之后的内容来清理文件"""
    if not os.path.exists(file_path):
        return
    
    with open(file_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    if line_cutoff:
        lines = lines[:line_cutoff]
        # 确保文件以合适的结尾
        if not lines[-1].strip().endswith('}'):
            lines.append('}\n')
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.writelines(lines)
    
    print(f"已清理文件: {file_path} (保留前{line_cutoff}行)")

def add_missing_imports():
    """添加缺失的imports"""
    
    # 修复market_state.rs的HashMap导入
    market_state_path = 'src/strategy/market_state.rs'
    if os.path.exists(market_state_path):
        with open(market_state_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        if 'use std::collections::HashMap;' not in content:
            content = content.replace(
                'use std::sync::Arc;',
                'use std::collections::HashMap;\nuse std::sync::Arc;'
            )
        
        with open(market_state_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"已添加HashMap导入: {market_state_path}")

def fix_missing_structs():
    """添加缺失的结构体定义"""
    
    # 在scheduler.rs中添加缺失的结构体
    scheduler_path = 'src/strategy/scheduler.rs'
    if os.path.exists(scheduler_path):
        with open(scheduler_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # 在文件末尾添加缺失的结构体
        missing_structs = '''
/// 调度统计信息
#[derive(Debug, Clone)]
pub struct SchedulingStatistics {
    pub total_tasks_scheduled: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub average_execution_time_ms: f64,
    pub current_queue_size: usize,
}

impl Default for SchedulingStatistics {
    fn default() -> Self {
        Self {
            total_tasks_scheduled: 0,
            tasks_completed: 0,
            tasks_failed: 0,
            average_execution_time_ms: 0.0,
            current_queue_size: 0,
        }
    }
}

impl StrategySchedulerClone {
    async fn execute_task(&self, _task: SchedulingTask) {
        // 实现任务执行逻辑
    }
}
'''
        
        if 'pub struct SchedulingStatistics' not in content:
            content = content.replace(
                '// End of scheduler module',
                missing_structs + '\n// End of scheduler module'
            )
        
        with open(scheduler_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"已添加缺失结构体: {scheduler_path}")
    
    # 在failure_detector.rs中添加缺失的结构体
    failure_detector_path = 'src/strategy/failure_detector.rs'
    if os.path.exists(failure_detector_path):
        with open(failure_detector_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        failure_stats_struct = '''
/// 故障统计信息
#[derive(Debug, Clone)]
pub struct FailureStatistics {
    pub total_failures: u64,
    pub failures_by_strategy: std::collections::HashMap<String, u64>,
    pub recovery_success_rate: f64,
    pub average_recovery_time_ms: f64,
}

impl Default for FailureStatistics {
    fn default() -> Self {
        Self {
            total_failures: 0,
            failures_by_strategy: std::collections::HashMap::new(),
            recovery_success_rate: 0.0,
            average_recovery_time_ms: 0.0,
        }
    }
}
'''
        
        if 'pub struct FailureStatistics' not in content:
            # 在文件末尾添加
            content += '\n' + failure_stats_struct
        
        with open(failure_detector_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"已添加FailureStatistics: {failure_detector_path}")

def main():
    """主函数 - 执行所有清理操作"""
    
    # 彻底清理有重复定义的文件
    files_to_clean = [
        ('src/strategy/config_manager.rs', 530),  # 只保留前530行
        ('src/strategy/market_state.rs', 665),    # 只保留前665行
    ]
    
    for file_path, cutoff in files_to_clean:
        clean_file_by_removing_duplicates(file_path, cutoff)
    
    # 添加缺失的imports
    add_missing_imports()
    
    # 添加缺失的结构体定义
    fix_missing_structs()
    
    print("所有清理工作完成!")

if __name__ == "__main__":
    main() 