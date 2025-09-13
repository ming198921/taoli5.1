#!/bin/bash

cd /home/ubuntu/celue/strategy/src

echo "🔧 修复深度分析模块的 += 操作符问题..."

# 替换 FixedQuantity += 操作
sed -i 's/*price_levels\.entry(price)\.or_insert(FixedQuantity::from_raw(0, quantity\.scale())) += quantity;/{ let existing = price_levels.entry(price).or_insert(FixedQuantity::from_raw(0, quantity.scale())); *existing = *existing + quantity; }/g' depth_analysis.rs

# 替换 cumulative_quantity += 操作
sed -i 's/cumulative_quantity += consumed_quantity;/cumulative_quantity = cumulative_quantity + consumed_quantity;/g' depth_analysis.rs

# 替换 cumulative_cost += 操作
sed -i 's/cumulative_cost += FixedPrice::from_f64(level_cost\.to_f64(), 6);/cumulative_cost = cumulative_cost + FixedPrice::from_f64(level_cost.to_f64(), 6);/g' depth_analysis.rs

echo "✅ 深度分析模块修复完成" 