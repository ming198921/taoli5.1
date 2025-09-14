import React, { useState } from 'react';
import { Routes, Route, Link, useLocation } from 'react-router-dom';

const CleaningOverview = () => {
  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">清洗规则引擎</h3>
          <p className="text-sm text-gray-600 mb-4">管理和配置数据清洗规则</p>
          <div className="text-2xl font-bold text-blue-600">20个API</div>
        </div>
        
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">交易所配置</h3>
          <p className="text-sm text-gray-600 mb-4">交易所级别的精细化配置</p>
          <div className="text-2xl font-bold text-green-600">18个API</div>
        </div>
        
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">性能监控</h3>
          <p className="text-sm text-gray-600 mb-4">SIMD优化和质量监控</p>
          <div className="text-2xl font-bold text-purple-600">14个API</div>
        </div>
      </div>

      {/* API列表 */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">清洗配置API (52个)</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2">
          {[
            // 清洗规则引擎 (20个)
            'GET /api/cleaning/rules/list',
            'POST /api/cleaning/rules/create',
            'PUT /api/cleaning/rules/{id}',
            'DELETE /api/cleaning/rules/{id}',
            'POST /api/cleaning/rules/validate',
            'POST /api/cleaning/rules/test',
            'GET /api/cleaning/rules/{id}/stats',
            'POST /api/cleaning/rules/import',
            'POST /api/cleaning/rules/export',
            'POST /api/cleaning/rules/batch-update',
            'GET /api/cleaning/strategies/list',
            'POST /api/cleaning/strategies/create',
            'PUT /api/cleaning/strategies/{id}',
            'DELETE /api/cleaning/strategies/{id}',
            'POST /api/cleaning/strategies/activate',
            'POST /api/cleaning/strategies/deactivate',
            'GET /api/cleaning/strategies/{id}/performance',
            'POST /api/cleaning/strategies/optimize',
            'POST /api/cleaning/strategies/clone',
            'GET /api/cleaning/strategies/templates',
            
            // 交易所级别配置 (18个)
            'GET /api/cleaning/exchanges/list',
            'GET /api/cleaning/exchanges/{id}/config',
            'PUT /api/cleaning/exchanges/{id}/config',
            'POST /api/cleaning/exchanges/{id}/test',
            'GET /api/cleaning/exchanges/{id}/stats',
            'POST /api/cleaning/exchanges/{id}/reset',
            'GET /api/cleaning/symbols/list',
            'PUT /api/cleaning/symbols/{symbol}/config',
            'GET /api/cleaning/symbols/{symbol}/stats',
            'POST /api/cleaning/symbols/batch-config',
            'GET /api/cleaning/symbols/anomalies',
            'POST /api/cleaning/symbols/calibrate',
            'GET /api/cleaning/sources/list',
            'PUT /api/cleaning/sources/{id}/config',
            'GET /api/cleaning/sources/{id}/quality',
            'POST /api/cleaning/sources/{id}/benchmark',
            'GET /api/cleaning/sources/comparison',
            'POST /api/cleaning/sources/sync',
            
            // 性能监控 (14个)
            'GET /api/cleaning/performance/realtime',
            'GET /api/cleaning/performance/throughput',
            'GET /api/cleaning/performance/latency',
            'GET /api/cleaning/performance/errors',
            'GET /api/cleaning/performance/bottlenecks',
            'GET /api/cleaning/simd/status',
            'POST /api/cleaning/simd/enable',
            'POST /api/cleaning/simd/disable',
            'GET /api/cleaning/simd/performance',
            'POST /api/cleaning/simd/benchmark',
            'GET /api/cleaning/quality/metrics',
            'GET /api/cleaning/quality/trends',
            'POST /api/cleaning/quality/alerts',
            'GET /api/cleaning/quality/reports'
          ].map((api, index) => (
            <div key={index} className="flex items-center justify-between p-2 bg-gray-50 rounded text-xs">
              <span className="font-mono text-gray-700 truncate">{api}</span>
              <span className="text-green-600">✓</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

const CleaningModule = () => {
  const location = useLocation();

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="bg-white shadow-sm border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-6">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">清洗配置系统</h1>
              <p className="mt-1 text-sm text-gray-600">
                52个专业API接口 - 清洗规则引擎、交易所配置、SIMD优化
              </p>
            </div>
            <div className="flex items-center space-x-2">
              <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
              <span className="text-sm text-gray-600">清洗引擎运行中</span>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <Routes>
          <Route path="/" element={<CleaningOverview />} />
        </Routes>
      </div>
    </div>
  );
};

export default CleaningModule; 