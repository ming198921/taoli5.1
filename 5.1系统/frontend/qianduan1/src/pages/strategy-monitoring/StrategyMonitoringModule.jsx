import React from 'react';
import { Routes, Route } from 'react-router-dom';

const StrategyMonitoringOverview = () => {
  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">实时监控</h3>
          <p className="text-sm text-gray-600 mb-4">策略状态、性能、错误监控</p>
          <div className="text-2xl font-bold text-purple-600">15个API</div>
        </div>
        
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">生命周期管理</h3>
          <p className="text-sm text-gray-600 mb-4">启停、依赖、启动顺序管理</p>
          <div className="text-2xl font-bold text-blue-600">12个API</div>
        </div>
        
        <div className="bg-white p-6 rounded-lg border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">调试工具</h3>
          <p className="text-sm text-gray-600 mb-4">断点、热重载、变量监控</p>
          <div className="text-2xl font-bold text-green-600">11个API</div>
        </div>
      </div>

      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">策略监控API (38个)</h3>
        <div className="text-sm text-gray-600 mb-4">
          策略状态监控、生命周期管理、调试工具 - 38个专业API接口
        </div>
      </div>
    </div>
  );
};

const StrategyMonitoringModule = () => {
  return (
    <div className="min-h-screen bg-gray-50">
      <div className="bg-white shadow-sm border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-6">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">策略监控系统</h1>
              <p className="mt-1 text-sm text-gray-600">
                38个专业API接口 - 策略状态监控、生命周期管理、调试工具
              </p>
            </div>
            <div className="flex items-center space-x-2">
              <div className="w-2 h-2 bg-purple-500 rounded-full animate-pulse"></div>
              <span className="text-sm text-gray-600">策略监控中</span>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <Routes>
          <Route path="/" element={<StrategyMonitoringOverview />} />
        </Routes>
      </div>
    </div>
  );
};

export default StrategyMonitoringModule; 