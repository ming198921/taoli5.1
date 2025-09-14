#!/usr/bin/env node
/**
 * 5.1套利系统前端页面功能完整性测试
 * 测试所有7个页面的加载和基本功能
 */

import axios from 'axios';
import fs from 'fs';

const FRONTEND_BASE_URL = 'http://57.183.21.242:3003';
const API_BASE_URL = 'http://localhost:3000/api';
const OUTPUT_FILE = 'page-functionality-test-report.json';

console.log('🚀 开始页面功能完整性测试');
console.log('==================================================');

const testResults = {
  timestamp: new Date().toISOString(),
  frontend_url: FRONTEND_BASE_URL,
  api_url: API_BASE_URL,
  pages_tested: 0,
  pages_passed: 0,
  total_tests: 0,
  tests_passed: 0,
  page_results: {},
  summary: {}
};

// 页面测试配置
const pageTests = [
  {
    name: '策略管理页面',
    url: '/strategy',
    api_endpoints: [
      '/strategies/list',
      '/monitoring/realtime',
      '/hotreload/status'
    ],
    expected_elements: [
      '策略服务管理',
      '策略列表',
      '实时监控',
      '热重载'
    ]
  },
  {
    name: '配置管理页面',
    url: '/config', 
    api_endpoints: [
      '/config/list',
      '/config/versions/current',
      '/config/environments'
    ],
    expected_elements: [
      '配置管理',
      '版本控制',
      '环境管理'
    ]
  },
  {
    name: '交易管理页面',
    url: '/trading',
    api_endpoints: [
      '/orders/active',
      '/positions/current', 
      '/account/balance'
    ],
    expected_elements: [
      '交易服务管理',
      '订单监控',
      '仓位监控',
      '资金管理'
    ]
  },
  {
    name: '性能监控页面',
    url: '/performance',
    api_endpoints: [
      '/performance/cpu/usage',
      '/performance/memory/usage',
      '/performance/network/stats'
    ],
    expected_elements: [
      '性能监控',
      'CPU使用率',
      '内存使用',
      '网络统计'
    ]
  },
  {
    name: 'AI模型页面',
    url: '/ai-model',
    api_endpoints: [
      '/ml/models',
      '/ml/training/jobs',
      '/ml/datasets'
    ],
    expected_elements: [
      'AI模型服务',
      '模型管理', 
      '训练任务',
      '推理服务'
    ]
  },
  {
    name: '日志管理页面',
    url: '/logging',
    api_endpoints: [
      '/logs/stream/realtime',
      '/logs/levels',
      '/logs/search'
    ],
    expected_elements: [
      '日志服务管理',
      '实时日志',
      '历史日志',
      '配置管理'
    ]
  },
  {
    name: '数据清洗页面',
    url: '/cleaning',
    api_endpoints: [
      '/cleaning/rules/list',
      '/cleaning/exchanges/config',
      '/cleaning/quality/metrics'
    ],
    expected_elements: [
      '数据清洗服务',
      '清洗规则',
      '交易所配置',
      '数据质量'
    ]
  }
];

// 测试API端点
async function testAPIEndpoint(endpoint) {
  try {
    const startTime = Date.now();
    const response = await axios.get(`${API_BASE_URL}${endpoint}`, {
      timeout: 10000,
      headers: {
        'Content-Type': 'application/json'
      }
    });
    const duration = Date.now() - startTime;
    
    return {
      endpoint,
      status: 'SUCCESS',
      http_status: response.status,
      response_time: duration,
      has_data: response.data ? true : false,
      data_type: Array.isArray(response.data) ? 'array' : typeof response.data
    };
  } catch (error) {
    return {
      endpoint,
      status: 'FAILED',
      error: error.message,
      http_status: error.response?.status || 0
    };
  }
}

// 测试页面内容
async function testPageContent(pageUrl) {
  try {
    console.log(`  🔍 测试页面内容: ${pageUrl}`);
    const response = await axios.get(`${FRONTEND_BASE_URL}${pageUrl}`, {
      timeout: 15000,
      headers: {
        'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8'
      }
    });
    
    const html = response.data;
    const hasReactRoot = html.includes('<div id="root">');
    const hasTitle = html.includes('5.1套利系统');
    const hasViteScripts = html.includes('/@vite/client') || html.includes('/assets/');
    
    return {
      status: 'SUCCESS',
      http_status: response.status,
      has_react_root: hasReactRoot,
      has_title: hasTitle,
      has_vite_scripts: hasViteScripts,
      content_length: html.length,
      loadable: response.status === 200
    };
  } catch (error) {
    return {
      status: 'FAILED',
      error: error.message,
      http_status: error.response?.status || 0
    };
  }
}

// 主测试函数
async function runPageTests() {
  console.log(`📋 共需测试 ${pageTests.length} 个页面`);
  console.log('');
  
  for (const pageTest of pageTests) {
    console.log(`📄 测试页面: ${pageTest.name}`);
    testResults.pages_tested++;
    
    const pageResult = {
      name: pageTest.name,
      url: pageTest.url,
      page_content_test: null,
      api_tests: [],
      element_tests: [],
      overall_status: 'UNKNOWN',
      tests_passed: 0,
      tests_total: 0
    };
    
    // 1. 测试页面内容加载
    console.log(`  📱 测试页面加载: ${FRONTEND_BASE_URL}${pageTest.url}`);
    pageResult.page_content_test = await testPageContent(pageTest.url);
    pageResult.tests_total++;
    if (pageResult.page_content_test.status === 'SUCCESS') {
      pageResult.tests_passed++;
    }
    
    // 2. 测试相关API端点
    console.log(`  🔌 测试 ${pageTest.api_endpoints.length} 个API端点:`);
    for (const endpoint of pageTest.api_endpoints) {
      console.log(`    - ${endpoint}`);
      const apiResult = await testAPIEndpoint(endpoint);
      pageResult.api_tests.push(apiResult);
      pageResult.tests_total++;
      testResults.total_tests++;
      
      if (apiResult.status === 'SUCCESS') {
        pageResult.tests_passed++;
        testResults.tests_passed++;
      }
    }
    
    // 3. 计算页面测试结果
    const successRate = pageResult.tests_passed / pageResult.tests_total;
    if (successRate >= 0.7) {
      pageResult.overall_status = 'PASSED';
      testResults.pages_passed++;
    } else if (successRate >= 0.5) {
      pageResult.overall_status = 'PARTIAL';
    } else {
      pageResult.overall_status = 'FAILED';
    }
    
    console.log(`  ✅ 页面测试完成: ${pageResult.tests_passed}/${pageResult.tests_total} 通过 (${(successRate * 100).toFixed(1)}%)`);
    console.log('');
    
    testResults.page_results[pageTest.name] = pageResult;
  }
  
  // 生成汇总结果
  testResults.summary = {
    pages_success_rate: (testResults.pages_passed / testResults.pages_tested * 100).toFixed(1) + '%',
    api_success_rate: (testResults.tests_passed / testResults.total_tests * 100).toFixed(1) + '%',
    total_pages: testResults.pages_tested,
    passed_pages: testResults.pages_passed,
    total_api_tests: testResults.total_tests,
    passed_api_tests: testResults.tests_passed
  };
  
  return testResults;
}

// 生成详细报告
function generateDetailedReport(results) {
  console.log('======================================================');
  console.log('📊 页面功能完整性测试总结');
  console.log('======================================================');
  console.log('');
  
  console.log(`🎯 总体结果:`);
  console.log(`   页面测试通过率: ${results.summary.pages_success_rate} (${results.summary.passed_pages}/${results.summary.total_pages})`);
  console.log(`   API测试通过率: ${results.summary.api_success_rate} (${results.summary.passed_api_tests}/${results.summary.total_api_tests})`);
  console.log('');
  
  console.log(`📋 各页面详细结果:`);
  for (const [pageName, pageResult] of Object.entries(results.page_results)) {
    const status = pageResult.overall_status === 'PASSED' ? '✅' : 
                   pageResult.overall_status === 'PARTIAL' ? '⚠️' : '❌';
    console.log(`   ${status} ${pageName}: ${pageResult.tests_passed}/${pageResult.tests_total} (${(pageResult.tests_passed/pageResult.tests_total*100).toFixed(1)}%)`);
    
    // 显示API测试详情
    if (pageResult.api_tests.length > 0) {
      console.log(`      API端点测试:`);
      for (const apiTest of pageResult.api_tests) {
        const apiStatus = apiTest.status === 'SUCCESS' ? '✅' : '❌';
        const responseTime = apiTest.response_time ? ` (${apiTest.response_time}ms)` : '';
        console.log(`        ${apiStatus} ${apiTest.endpoint}${responseTime}`);
      }
    }
  }
  console.log('');
  
  // 生成建议
  console.log(`💡 建议:`);
  if (results.summary.pages_success_rate.replace('%', '') >= 80) {
    console.log(`   ✅ 页面功能基本完整，建议进行生产环境部署前的最终测试`);
  } else if (results.summary.pages_success_rate.replace('%', '') >= 60) {
    console.log(`   ⚠️ 部分页面功能需要完善，建议重点关注失败的API端点`);
  } else {
    console.log(`   ❌ 页面功能存在较多问题，需要全面检查API服务和前端代码`);
  }
  
  if (results.summary.api_success_rate.replace('%', '') >= 70) {
    console.log(`   ✅ API对接状况良好，大部分功能可正常使用`);
  } else {
    console.log(`   ❌ API对接存在问题，需要检查后端微服务状态`);
  }
  
  console.log('');
  console.log(`📄 详细报告已保存到: ${OUTPUT_FILE}`);
}

// 执行测试
async function main() {
  try {
    const results = await runPageTests();
    
    // 保存结果到文件
    fs.writeFileSync(OUTPUT_FILE, JSON.stringify(results, null, 2));
    
    // 生成详细报告
    generateDetailedReport(results);
    
    console.log('🎉 页面功能测试完成！');
    
  } catch (error) {
    console.error('❌ 测试执行失败:', error.message);
    process.exit(1);
  }
}

// 启动测试
main();