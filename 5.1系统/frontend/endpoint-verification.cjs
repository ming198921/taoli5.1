#!/usr/bin/env node

const axios = require('axios');

// Key failing endpoints and their expected service routing
const endpointTests = [
  // AI Model Service endpoints (port 4006)
  {
    name: 'AI Models List',
    gateway: 'http://localhost:3000/api/models',
    direct: 'http://localhost:4006/api/ml/models',
    expected: 'AI service should respond with models array'
  },
  {
    name: 'Training Jobs',
    gateway: 'http://localhost:3000/api/training/jobs', 
    direct: 'http://localhost:4006/api/ml/training/jobs',
    expected: 'AI service should respond with training jobs'
  },
  {
    name: 'Datasets',
    gateway: 'http://localhost:3000/api/datasets',
    direct: 'http://localhost:4006/api/ml/training/datasets',
    expected: 'AI service should respond with datasets'
  },
  
  // Data Cleaning Service endpoints (port 4002)
  {
    name: 'Cleaning Exchanges List',
    gateway: 'http://localhost:3000/api/cleaning/exchanges/list',
    direct: 'http://localhost:4002/api/cleaning/exchanges/list',
    expected: 'Cleaning service should respond with exchanges'
  },
  {
    name: 'Cleaning Quality Metrics', 
    gateway: 'http://localhost:3000/api/cleaning/quality/metrics',
    direct: 'http://localhost:4002/api/cleaning/quality/metrics',
    expected: 'Cleaning service should respond with quality metrics'
  },
  {
    name: 'Cleaning Status',
    gateway: 'http://localhost:3000/api/cleaning/status',
    direct: 'http://localhost:4002/api/cleaning/status',
    expected: 'Cleaning service should respond with status'
  },
  
  // Logging Service endpoints (port 4001)
  {
    name: 'Log Stream Pause',
    gateway: 'http://localhost:3000/api/logs/stream/pause',
    direct: 'http://localhost:4001/api/logs/stream/pause',
    expected: 'Logging service should handle stream pause'
  },
  {
    name: 'Log Stream Resume',
    gateway: 'http://localhost:3000/api/logs/stream/resume', 
    direct: 'http://localhost:4001/api/logs/stream/resume',
    expected: 'Logging service should handle stream resume'
  },
  
  // Strategy Service endpoints (port 4003)
  {
    name: 'Strategy Common Enable',
    gateway: 'http://localhost:3000/api/hotreload/strategy_common/enable',
    direct: 'http://localhost:4003/api/hotreload/strategy_common/enable',
    expected: 'Strategy service should handle hotreload enable'
  },
  {
    name: 'Strategy Common Disable', 
    gateway: 'http://localhost:3000/api/hotreload/strategy_common/disable',
    direct: 'http://localhost:4003/api/hotreload/strategy_common/disable',
    expected: 'Strategy service should handle hotreload disable'
  }
];

async function testEndpoint(endpoint) {
  console.log(`\n🔍 Testing: ${endpoint.name}`);
  console.log(`   Expected: ${endpoint.expected}`);
  
  // Test direct service call
  try {
    const directResponse = await axios.get(endpoint.direct, { timeout: 5000 });
    console.log(`   ✅ Direct call: SUCCESS (${directResponse.status})`);
    if (directResponse.data?.data) {
      const dataType = Array.isArray(directResponse.data.data) ? 'array' : 'object';
      const dataCount = Array.isArray(directResponse.data.data) ? directResponse.data.data.length : 'N/A';
      console.log(`   📊 Data: ${dataType} (${dataCount} items)`);
    }
  } catch (error) {
    console.log(`   ❌ Direct call: FAILED (${error.response?.status || error.message})`);
  }
  
  // Test gateway routing
  try {
    const gatewayResponse = await axios.get(endpoint.gateway, { timeout: 5000 });
    console.log(`   ✅ Gateway call: SUCCESS (${gatewayResponse.status})`);
    if (gatewayResponse.data?.data) {
      const dataType = Array.isArray(gatewayResponse.data.data) ? 'array' : 'object';
      const dataCount = Array.isArray(gatewayResponse.data.data) ? gatewayResponse.data.data.length : 'N/A';
      console.log(`   📊 Data: ${dataType} (${dataCount} items)`);
    }
  } catch (error) {
    console.log(`   ❌ Gateway call: FAILED (${error.response?.status || error.message})`);
    console.log(`   🔧 Route issue: Gateway not routing ${endpoint.gateway} to service`);
  }
}

async function main() {
  console.log('🚀 Endpoint Verification Report');
  console.log('=====================================\n');
  
  for (const endpoint of endpointTests) {
    await testEndpoint(endpoint);
    await new Promise(resolve => setTimeout(resolve, 200)); // Small delay between tests
  }
  
  console.log('\n📋 Summary:');
  console.log('- Direct service calls should work if services are running');
  console.log('- Gateway call failures indicate routing configuration issues');
  console.log('- The unified gateway needs route mappings for each service');
}

main().catch(console.error);