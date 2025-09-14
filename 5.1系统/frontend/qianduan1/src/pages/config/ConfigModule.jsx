import React from 'react';
import { Routes, Route } from 'react-router-dom';

const ConfigOverview = () => {
  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">系统配置</h3>
          <div className="text-2xl font-bold text-gray-600">24个API</div>
        </div>
        
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">业务配置</h3>
          <div className="text-2xl font-bold text-blue-600">25个API</div>
        </div>
        
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">管理工具</h3>
          <div className="text-2xl font-bold text-green-600">23个API</div>
        </div>
        
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">监控告警</h3>
          <div className="text-2xl font-bold text-purple-600">24个API</div>
        </div>
      </div>

      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">配置管理API (96个)</h3>
        <div className="text-sm text-gray-600">
          配置热更新、版本管理、监控告警 - 96个专业API接口
        </div>
      </div>
    </div>
  );
};

const ConfigModule = () => {
  return (
    <div className="min-h-screen bg-gray-50">
      <div className="bg-white shadow-sm border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-6">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">配置管理系统</h1>
              <p className="mt-1 text-sm text-gray-600">
                96个专业API接口 - 配置热更新、版本管理、监控告警
              </p>
            </div>
            <div className="flex items-center space-x-2">
              <div className="w-2 h-2 bg-gray-500 rounded-full animate-pulse"></div>
              <span className="text-sm text-gray-600">配置管理中</span>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <Routes>
          <Route path="/" element={<ConfigOverview />} />
        </Routes>
      </div>
    </div>
  );
};

export default ConfigModule; 