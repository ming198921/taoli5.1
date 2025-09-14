#!/bin/bash

# Qingxi K8s 部署脚本 - 应用所有修复
# 此脚本将部署修复后的配置和应用程序

set -e

echo "🚀 开始部署修复后的 Qingxi 应用..."

# 检查 kubectl 连接
if ! kubectl cluster-info &> /dev/null; then
    echo "❌ 无法连接到 K8s 集群，请检查 kubeconfig"
    exit 1
fi

echo "✅ K8s 集群连接正常"

# 确保命名空间存在
echo "📁 创建/确认命名空间..."
kubectl create namespace qingxi-market-data --dry-run=client -o yaml | kubectl apply -f -

# 应用更新的 ConfigMap
echo "⚙️ 应用修复后的 ConfigMap..."
kubectl apply -f k8s/configmap.yaml

# 验证 ConfigMap 创建成功
echo "🔍 验证 ConfigMap..."
kubectl get configmap qingxi-config -n qingxi-market-data -o yaml

# 如果 deployment 存在，重启它以应用新配置
if kubectl get deployment qingxi-market-data-service -n qingxi-market-data &> /dev/null; then
    echo "🔄 重启现有 deployment 以应用新配置..."
    kubectl rollout restart deployment/qingxi-market-data-service -n qingxi-market-data
    
    echo "⏳ 等待 deployment 完成..."
    kubectl rollout status deployment/qingxi-market-data-service -n qingxi-market-data --timeout=300s
else
    echo "📝 Deployment 不存在，需要首先创建 deployment"
    echo "请运行: kubectl apply -f k8s/deployment.yaml"
fi

# 检查 pod 状态
echo "🔍 检查 Pod 状态..."
kubectl get pods -n qingxi-market-data -l app=qingxi-market-data-service

# 显示最新的 pod 日志
echo "📋 显示最新 Pod 日志 (最后20行):"
LATEST_POD=$(kubectl get pods -n qingxi-market-data -l app=qingxi-market-data-service --sort-by=.metadata.creationTimestamp -o jsonpath='{.items[-1].metadata.name}' 2>/dev/null || echo "")

if [[ -n "$LATEST_POD" ]]; then
    echo "Pod: $LATEST_POD"
    kubectl logs "$LATEST_POD" -n qingxi-market-data --tail=20
else
    echo "⚠️ 未找到运行中的 Pod"
fi

echo ""
echo "🎉 部署脚本执行完成！"
echo ""
echo "📊 验证部署状态："
echo "1. 检查 Pod 状态: kubectl get pods -n qingxi-market-data"
echo "2. 查看详细日志: kubectl logs <pod-name> -n qingxi-market-data -f"
echo "3. 检查事件: kubectl get events -n qingxi-market-data --sort-by=.firstTimestamp"
echo ""
echo "✅ 修复的关键问题："
echo "  - 配置字段统一 (exchange_id)"
echo "  - AnomalyDetector 配置映射修复"
echo "  - ConfigMap 配置清理"
echo "  - Clippy 格式化修复"
echo "  - 语法错误修复"
echo ""
echo "🔥 期望结果: Pod 应该正常启动，不再出现 CrashLoopBackOff!"
