#![allow(dead_code)]
// src/memory/mod.rs
// Qingxi V3.0 内存模块入口

pub mod advanced_allocator;
pub mod zero_allocation_engine;

pub use advanced_allocator::{
    QingxiMemoryManager, 
    QINGXI_MEMORY, 
    AlignedMarketData, 
    AlignedOrderBook,
    MemoryHealthReport,
    benchmark_memory_performance
};

pub use zero_allocation_engine::{
    ZeroAllocationEngine,
    ZeroAllocationConfig,
    init_zero_allocation_system,
    ZERO_ALLOCATION_ENGINE
};
