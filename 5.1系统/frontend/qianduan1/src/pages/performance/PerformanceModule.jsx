import React from 'react';
import { Routes, Route } from 'react-router-dom';

const PerformanceOverview = () => {
  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">CPU调优</h3>
          <div className="text-2xl font-bold text-yellow-600">18个API</div>
        </div>
        
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">内存调优</h3>
          <div className="text-2xl font-bold text-blue-600">16个API</div>
        </div>
        
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">网络调优</h3>
          <div className="text-2xl font-bold text-green-600">17个API</div>
        </div>
        
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">存储调优</h3>
          <div className="text-2xl font-bold text-purple-600">16个API</div>
        </div>
      </div>

      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">性能调优API (67个)</h3>
        <div className="text-sm text-gray-600">
          CPU/内存/网络/存储全方位性能调优 - 67个专业API接口
        </div>
      </div>
    </div>
  );
};

const PerformanceModule = () => {
  return (
    <div className="min-h-screen bg-gray-50">
      <div className="bg-white shadow-sm border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-6">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">性能调优系统</h1>
              <p className="mt-1 text-sm text-gray-600">
                67个专业API接口 - CPU/内存/网络/存储全方位性能调优
              </p>
            </div>
            <div className="flex items-center space-x-2">
              <div className="w-2 h-2 bg-yellow-500 rounded-full animate-pulse"></div>
              <span className="text-sm text-gray-600">性能优化中</span>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <Routes>
          <Route path="/" element={<PerformanceOverview />} />
        </Routes>
      </div>
    </div>
  );
};

export default PerformanceModule; 