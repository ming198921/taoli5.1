import React from 'react';
import { Routes, Route } from 'react-router-dom';

const AIModelOverview = () => {
  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">训练管理</h3>
          <div className="text-2xl font-bold text-indigo-600">18个API</div>
        </div>
        
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">性能监控</h3>
          <div className="text-2xl font-bold text-blue-600">15个API</div>
        </div>
        
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">配置管理</h3>
          <div className="text-2xl font-bold text-green-600">15个API</div>
        </div>
      </div>

      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">AI模型管理API (48个)</h3>
        <div className="text-sm text-gray-600">
          模型训练、部署、监控、解释性分析 - 48个专业API接口
        </div>
      </div>
    </div>
  );
};

const AIModelModule = () => {
  return (
    <div className="min-h-screen bg-gray-50">
      <div className="bg-white shadow-sm border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-6">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">AI模型管理</h1>
              <p className="mt-1 text-sm text-gray-600">
                48个专业API接口 - 模型训练、部署、监控、解释性分析
              </p>
            </div>
            <div className="flex items-center space-x-2">
              <div className="w-2 h-2 bg-indigo-500 rounded-full animate-pulse"></div>
              <span className="text-sm text-gray-600">AI模型运行中</span>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <Routes>
          <Route path="/" element={<AIModelOverview />} />
        </Routes>
      </div>
    </div>
  );
};

export default AIModelModule; 