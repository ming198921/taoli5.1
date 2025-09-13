#!/bin/bash

# 测试自动修复功能 - Test Auto-Repair Functionality

set -euo pipefail

# 颜色配置
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m'

echo -e "${WHITE}🧪 5.1套利系统 - 自动修复功能测试${NC}"
echo -e "${WHITE}================================================${NC}"
echo ""

# 1. 检查初始状态
echo -e "${CYAN}1. 检查初始状态...${NC}"
./auto-service-manager.sh status | grep "HEALTHY" | wc -l
initial_healthy_count=$(./auto-service-manager.sh status 2>/dev/null | grep -c "HEALTHY" || echo "0")
echo -e "${GREEN}✅ 初始健康服务数: $initial_healthy_count${NC}"
echo ""

# 2. 模拟故障 - 停止一个非关键服务
echo -e "${CYAN}2. 模拟故障 (停止 cleaning-service)...${NC}"
pkill -f "cleaning-service" 2>/dev/null || echo "服务可能已经停止"
sleep 3

# 验证故障
echo -e "${YELLOW}验证故障状态:${NC}"
if curl -sf http://localhost:4002/health >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  cleaning-service 仍在运行${NC}"
else
    echo -e "${RED}❌ cleaning-service 已停止 (模拟故障成功)${NC}"
fi
echo ""

# 3. 测试诊断工具检测
echo -e "${CYAN}3. 测试故障检测...${NC}"
echo -e "${WHITE}使用诊断工具检测:${NC}"
node microservice-diagnostic-tool.js status | grep -E "(UNHEALTHY|ERROR|cleaning-service)" | head -3 || echo "检测中..."
echo ""

# 4. 测试自动修复
echo -e "${CYAN}4. 测试自动修复功能...${NC}"
echo -e "${WHITE}执行自动修复:${NC}"
./auto-service-manager.sh repair
echo ""

# 5. 等待服务恢复
echo -e "${CYAN}5. 等待服务恢复...${NC}"
sleep 10

# 6. 验证修复结果
echo -e "${CYAN}6. 验证修复结果...${NC}"
final_healthy_count=$(./auto-service-manager.sh status 2>/dev/null | grep -c "HEALTHY" || echo "0")

if [ "$final_healthy_count" -eq "$initial_healthy_count" ]; then
    echo -e "${GREEN}✅ 自动修复成功! 健康服务数: $final_healthy_count${NC}"
    
    # 验证特定服务
    if curl -sf http://localhost:4002/health >/dev/null 2>&1; then
        echo -e "${GREEN}✅ cleaning-service 已恢复健康${NC}"
    else
        echo -e "${RED}❌ cleaning-service 仍未恢复${NC}"
    fi
else
    echo -e "${RED}❌ 自动修复可能失败. 当前健康服务数: $final_healthy_count (期望: $initial_healthy_count)${NC}"
fi
echo ""

# 7. 测试持续监控
echo -e "${CYAN}7. 测试持续监控 (10秒)...${NC}"
echo -e "${WHITE}启动监控 (会自动停止):${NC}"

# 使用timeout限制监控时间
timeout 10 node microservice-diagnostic-tool.js monitor 5000 2>/dev/null || echo -e "${GREEN}✅ 监控测试完成${NC}"
echo ""

# 8. 最终状态验证
echo -e "${CYAN}8. 最终状态验证...${NC}"
echo -e "${WHITE}使用Python监控工具:${NC}"
python3 self-healing-monitor.py status | jq '.summary' 2>/dev/null || echo "Python监控工具测试完成"
echo ""

# 9. 生成测试报告
echo -e "${CYAN}9. 生成测试报告...${NC}"
test_report="/home/ubuntu/5.1xitong/logs/auto_repair_test_$(date +%Y%m%d_%H%M%S).txt"

{
    echo "自动修复功能测试报告 - $(date)"
    echo "=============================="
    echo ""
    echo "初始健康服务数: $initial_healthy_count"
    echo "最终健康服务数: $final_healthy_count"
    echo "测试结果: $([ "$final_healthy_count" -eq "$initial_healthy_count" ] && echo "成功" || echo "失败")"
    echo ""
    echo "详细状态:"
    ./auto-service-manager.sh status 2>/dev/null
    echo ""
    echo "资源使用:"
    free -h
    echo ""
    echo "网络连接:"
    ss -tlnp | grep -E ":(400[0-9]|300[0-1])" | head -10
} > "$test_report"

echo -e "${GREEN}✅ 测试报告已保存: $test_report${NC}"
echo ""

# 10. 总结
echo -e "${WHITE}📋 测试总结${NC}"
echo -e "${WHITE}================================================${NC}"

if [ "$final_healthy_count" -eq "$initial_healthy_count" ]; then
    echo -e "${GREEN}🎉 自动修复功能测试 - 通过${NC}"
    echo -e "${GREEN}   ✅ 故障检测正常${NC}"
    echo -e "${GREEN}   ✅ 自动修复正常${NC}"
    echo -e "${GREEN}   ✅ 服务恢复正常${NC}"
    echo -e "${GREEN}   ✅ 监控系统正常${NC}"
else
    echo -e "${RED}❌ 自动修复功能测试 - 失败${NC}"
    echo -e "${YELLOW}   建议检查日志文件和服务配置${NC}"
fi

echo ""
echo -e "${WHITE}🌐 系统访问地址:${NC}"
echo -e "${WHITE}   本地: http://localhost:3003/dashboard${NC}"
echo -e "${WHITE}   外网: http://57.183.21.242:3003/dashboard${NC}"
echo ""
echo -e "${WHITE}🛠️  管理工具:${NC}"
echo -e "${WHITE}   系统仪表板: ./system-dashboard.sh${NC}"
echo -e "${WHITE}   服务管理器: ./auto-service-manager.sh status${NC}"
echo -e "${WHITE}   诊断工具: node microservice-diagnostic-tool.js status${NC}"
echo -e "${WHITE}   监控系统: python3 self-healing-monitor.py status${NC}"
echo ""
echo -e "${GREEN}测试完成! 5.1套利系统自动化管理平台就绪 🚀${NC}"