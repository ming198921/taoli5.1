#!/usr/bin/env node

const axios = require('axios');

async function testServiceHealth() {
  const services = [
    { name: 'logging-service', port: 4001 },
    { name: 'cleaning-service', port: 4002 },
    { name: 'strategy-service', port: 4003 },
    { name: 'performance-service', port: 4004 },
    { name: 'trading-service', port: 4005 },
    { name: 'ai-model-service', port: 4006 },
    { name: 'config-service', port: 4007 }
  ];

  console.log('🔍 测试所有服务健康状态...\n');
  
  let healthyCount = 0;
  let totalApiCount = 0;
  const serviceStats = [];

  for (const { name, port } of services) {
    try {
      const response = await axios.get(`http://localhost:${port}/health`, { 
        timeout: 5000
      });
      
      if (response.status === 200 && response.data) {
        healthyCount++;
        const apiCount = response.data.data?.apis_count || 0;
        totalApiCount += apiCount;
        
        serviceStats.push({
          name,
          status: 'healthy',
          apis: apiCount,
          uptime: response.data.data?.service || 'unknown'
        });
        
        console.log(`✅ ${name}: 健康 (${apiCount} APIs)`);
      }
    } catch (error) {
      serviceStats.push({
        name,
        status: 'error',
        apis: 0,
        error: error.message
      });
      console.log(`❌ ${name}: 异常 - ${error.message}`);
    }
  }

  console.log('\n📊 统计结果:');
  console.log(`- 健康服务: ${healthyCount}/7`);
  console.log(`- 总API数量: ${totalApiCount} (期望: 387)`);
  console.log(`- 健康度: ${((healthyCount/7) * 100).toFixed(1)}%`);

  if (healthyCount === 7) {
    console.log('\n🎉 所有服务运行正常！');
    return true;
  } else {
    console.log('\n⚠️  存在异常服务，需要修复');
    return false;
  }
}

testServiceHealth().catch(console.error);