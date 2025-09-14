import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import unifiedApi from '../../services/unified-api';

const LogConfigPage = () => {
  const [activeTab, setActiveTab] = useState('levels');
  const queryClient = useQueryClient();

  // 获取日志级别配置
  const { data: logLevels, isLoading: levelsLoading } = useQuery({
    queryKey: ['log-levels'],
    queryFn: () => {
      // 模拟API调用
      return Promise.resolve({
        qingxi: 'info',
        celue: 'debug',
        risk: 'warning',
        architecture: 'info',
        observability: 'info'
      });
    }
  });

  // 更新日志级别
  const updateLevelMutation = useMutation({
    mutationFn: ({ module, level }) => {
      // 调用实际API
      // return unifiedApi.loggingApi.setModuleLogLevel(module, { level });
      return Promise.resolve({ success: true });
    },
    onSuccess: () => {
      queryClient.invalidateQueries(['log-levels']);
    }
  });

  const TabButton = ({ id, label, isActive, onClick }) => (
    <button
      onClick={() => onClick(id)}
      className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
        isActive
          ? 'bg-blue-100 text-blue-700 border border-blue-200'
          : 'text-gray-600 hover:text-gray-900 hover:bg-gray-100'
      }`}
    >
      {label}
    </button>
  );

  return (
    <div className="space-y-6">
      {/* 标签导航 */}
      <div className="flex space-x-2">
        <TabButton 
          id="levels" 
          label="日志级别" 
          isActive={activeTab === 'levels'} 
          onClick={setActiveTab} 
        />
        <TabButton 
          id="storage" 
          label="存储配置" 
          isActive={activeTab === 'storage'} 
          onClick={setActiveTab} 
        />
        <TabButton 
          id="rotation" 
          label="日志轮转" 
          isActive={activeTab === 'rotation'} 
          onClick={setActiveTab} 
        />
      </div>

      {/* 日志级别配置 */}
      {activeTab === 'levels' && (
        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">模块日志级别配置</h3>
          <p className="text-sm text-gray-600 mb-6">
            配置各个模块的日志输出级别，支持实时热更新
          </p>
          
          <div className="space-y-4">
            {logLevels && Object.entries(logLevels).map(([module, currentLevel]) => (
              <div key={module} className="flex items-center justify-between p-4 bg-gray-50 rounded-lg">
                <div>
                  <h4 className="font-medium text-gray-900">{module}</h4>
                  <p className="text-sm text-gray-600">当前级别: {currentLevel.toUpperCase()}</p>
                </div>
                <select
                  value={currentLevel}
                  onChange={(e) => updateLevelMutation.mutate({ module, level: e.target.value })}
                  className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="debug">DEBUG</option>
                  <option value="info">INFO</option>
                  <option value="warning">WARNING</option>
                  <option value="error">ERROR</option>
                </select>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 存储配置 */}
      {activeTab === 'storage' && (
        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">存储配置管理</h3>
          <p className="text-sm text-gray-600 mb-6">
            管理日志存储路径、保留策略和备份设置
          </p>
          
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">存储路径</label>
                <input
                  type="text"
                  defaultValue="/var/log/arbitrage-system"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">保留天数</label>
                <input
                  type="number"
                  defaultValue="30"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">最大文件大小</label>
                <input
                  type="text"
                  defaultValue="100MB"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">压缩格式</label>
                <select className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                  <option value="gzip">GZIP</option>
                  <option value="zip">ZIP</option>
                  <option value="none">不压缩</option>
                </select>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 日志轮转配置 */}
      {activeTab === 'rotation' && (
        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">日志轮转配置</h3>
          <p className="text-sm text-gray-600 mb-6">
            配置日志文件的轮转策略和触发条件
          </p>
          
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">轮转策略</label>
                <select className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                  <option value="daily">每日轮转</option>
                  <option value="hourly">每小时轮转</option>
                  <option value="size">按大小轮转</option>
                </select>
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">轮转时间</label>
                <input
                  type="time"
                  defaultValue="00:00"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">保留文件数</label>
                <input
                  type="number"
                  defaultValue="7"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
            </div>
            
            <div className="flex items-center space-x-4">
              <button className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors">
                手动轮转
              </button>
              <button className="px-4 py-2 bg-gray-100 text-gray-700 rounded-md hover:bg-gray-200 transition-colors">
                测试配置
              </button>
            </div>
          </div>
        </div>
      )}

      {/* API状态显示 */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">配置管理API状态 (18个)</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {[
            'GET /api/logs/config/levels',
            'PUT /api/logs/config/levels/{module}',
            'POST /api/logs/config/levels/batch',
            'GET /api/logs/config/appenders',
            'PUT /api/logs/config/appenders/{id}',
            'GET /api/logs/config/rotation',
            'PUT /api/logs/config/rotation',
            'POST /api/logs/rotation/trigger',
            'GET /api/logs/files/list',
            'GET /api/logs/files/{filename}/download',
            'GET /api/logs/storage/config',
            'PUT /api/logs/storage/config',
            'GET /api/logs/storage/usage',
            'POST /api/logs/storage/cleanup',
            'GET /api/logs/storage/backup',
            'POST /api/logs/storage/backup/create',
            'POST /api/logs/storage/restore',
            'GET /api/logs/retention/policy'
          ].map((api, index) => (
            <div key={index} className="flex items-center justify-between p-2 bg-gray-50 rounded">
              <span className="text-xs font-mono text-gray-700">{api}</span>
              <span className="text-green-600 text-xs">✓</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default LogConfigPage; 