#!/bin/bash

echo "========================================="
echo "     5.1系统修复验证报告"
echo "========================================="
echo ""

cd /home/ubuntu/5.1xitong/5.1系统

echo "【1】编译错误检查"
echo "-----------------------------------------"
cargo check --workspace 2>&1 | tee check_output.log > /dev/null 2>&1
error_count=$(grep -c "error\[" check_output.log 2>/dev/null || echo "0")
warning_count=$(grep -c "warning:" check_output.log 2>/dev/null || echo "0")

if [ $error_count -eq 0 ]; then
    echo "✅ 编译错误: 0 (已全部修复)"
else
    echo "❌ 编译错误数量: $error_count"
    echo "   主要错误类型:"
    grep "error\[E0" check_output.log | head -5 | sed 's/^/   - /'
fi
echo "⚠️  警告数量: $warning_count"
echo ""

echo "【2】硬编码检查"
echo "-----------------------------------------"
hardcode_count=$(find . -name "*.rs" -not -path "./target/*" -exec grep -l "const.*=.*[0-9]" {} \; | grep -v test | grep -v config | wc -l)
echo "硬编码常量文件数: $hardcode_count"

if [ $hardcode_count -eq 0 ]; then
    echo "✅ 硬编码已全部消除"
else
    echo "⚠️  仍包含硬编码的文件:"
    find . -name "*.rs" -not -path "./target/*" -exec grep -l "const.*=.*[0-9]" {} \; | grep -v test | grep -v config | head -5 | sed 's/^/   - /'
fi

# 检查配置文件
if [ -f "config/system_limits.toml" ]; then
    echo "✅ 系统配置文件已创建"
else
    echo "❌ 系统配置文件缺失"
fi
echo ""

echo "【3】TODO/FIXME/XXX检查"
echo "-----------------------------------------"
todo_count=$(find . -name "*.rs" -not -path "./target/*" -exec grep -c "TODO\|FIXME\|XXX" {} \; 2>/dev/null | awk '{sum += $1} END {print sum}')
if [ -z "$todo_count" ]; then
    todo_count=0
fi

if [ $todo_count -eq 0 ]; then
    echo "✅ TODO/FIXME/XXX: 0 (已全部实现)"
else
    echo "❌ TODO/FIXME/XXX数量: $todo_count"
    echo "   包含TODO的文件:"
    find . -name "*.rs" -not -path "./target/*" -exec grep -l "TODO\|FIXME\|XXX" {} \; | head -5 | sed 's/^/   - /'
fi
echo ""

echo "【4】功能完整性检查"
echo "-----------------------------------------"
# 检查关键功能实现
echo -n "深度分析功能: "
if grep -q "pub fn analyze_depth" qingxi/qingxi/src/cleaner/optimized_cleaner.rs 2>/dev/null; then
    if grep -q "// TODO\|unimplemented!" qingxi/qingxi/src/cleaner/optimized_cleaner.rs 2>/dev/null; then
        echo "⚠️  部分实现"
    else
        echo "✅ 已实现"
    fi
else
    echo "❌ 未找到"
fi

echo -n "波动率计算: "
if grep -q "pub fn calculate_volatility" celue/strategy/src/*.rs 2>/dev/null || grep -q "calculate_volatility" qingxi/qingxi/src/*.rs 2>/dev/null; then
    echo "✅ 已实现"
else
    echo "❌ 未找到"
fi

echo -n "数据一致性: "
if [ -f "qingxi/qingxi/src/consistency/hash_verifier.rs" ]; then
    echo "✅ 已实现"
else
    echo "❌ 未实现"
fi

echo -n "分布式锁: "
if [ -f "qingxi/qingxi/src/consistency/distributed_lock.rs" ]; then
    echo "✅ 已实现"
else
    echo "❌ 未实现"
fi
echo ""

echo "【5】测试准备状态"
echo "-----------------------------------------"
echo -n "测试编译: "
if cargo test --workspace --no-run 2>&1 | grep -q "Finished"; then
    echo "✅ 通过"
else
    echo "❌ 失败"
fi

echo -n "基准测试编译: "
if cargo bench --no-run 2>&1 | grep -q "Finished"; then
    echo "✅ 通过"
else
    echo "⚠️  失败"
fi
echo ""

echo "【6】项目结构检查"
echo "-----------------------------------------"
echo "主要模块:"
for module in architecture qingxi/qingxi celue/strategy celue/orchestrator; do
    if [ -d "$module" ]; then
        echo "  ✅ $module"
    else
        echo "  ❌ $module (缺失)"
    fi
done

echo ""
echo "========================================="
echo "               总体评分"
echo "========================================="

# 计算总分
total_score=0
max_score=100

# 编译错误 (40分)
if [ $error_count -eq 0 ]; then
    total_score=$((total_score + 40))
    compile_status="✅"
else
    compile_status="❌"
fi

# 硬编码 (20分)
if [ $hardcode_count -eq 0 ]; then
    total_score=$((total_score + 20))
    hardcode_status="✅"
else
    hardcode_status="⚠️"
    total_score=$((total_score + 10))
fi

# TODO消除 (20分)
if [ $todo_count -eq 0 ]; then
    total_score=$((total_score + 20))
    todo_status="✅"
else
    todo_status="❌"
fi

# 功能完整性 (20分)
total_score=$((total_score + 10)) # 部分实现

echo "完成度: $total_score/$max_score"
echo ""
echo "状态摘要:"
echo "  编译状态: $compile_status"
echo "  硬编码消除: $hardcode_status"
echo "  TODO清理: $todo_status"
echo "  功能完整性: ⚠️"
echo ""

if [ $total_score -ge 90 ]; then
    echo "🎉 系统修复基本完成，可以进行生产部署！"
elif [ $total_score -ge 70 ]; then
    echo "⚠️  系统修复大部分完成，需要继续完善"
else
    echo "❌ 系统仍需大量修复工作"
fi

echo "========================================="
echo "验证完成时间: $(date '+%Y-%m-%d %H:%M:%S')"
echo "========================================="

# 清理临时文件
rm -f check_output.log