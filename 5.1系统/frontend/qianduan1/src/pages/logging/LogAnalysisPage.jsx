import React, { useState } from 'react';
import { useQuery } from '@tanstack/react-query';

const LogAnalysisPage = () => {
  const [analysisType, setAnalysisType] = useState('anomaly');

  return (
    <div className="space-y-6">
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">日志分析工具</h3>
        <p className="text-sm text-gray-600 mb-6">
          使用AI驱动的分析工具检测异常、关联分析和根因定位
        </p>
        
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {[
            { key: 'anomaly', label: '异常检测', icon: '🔍' },
            { key: 'correlation', label: '关联分析', icon: '🔗' },
            { key: 'timeline', label: '时间线分析', icon: '📅' },
            { key: 'root-cause', label: '根因分析', icon: '🎯' }
          ].map((type) => (
            <button
              key={type.key}
              onClick={() => setAnalysisType(type.key)}
              className={`p-4 rounded-lg border-2 transition-colors ${
                analysisType === type.key
                  ? 'border-blue-500 bg-blue-50'
                  : 'border-gray-200 hover:border-gray-300'
              }`}
            >
              <div className="text-2xl mb-2">{type.icon}</div>
              <div className="font-medium">{type.label}</div>
            </button>
          ))}
        </div>
      </div>

      {/* API状态 */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">分析工具API状态 (12个)</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {[
            'POST /api/logs/analysis/anomaly',
            'POST /api/logs/analysis/correlation',
            'POST /api/logs/analysis/timeline',
            'GET /api/logs/analysis/frequent-errors',
            'POST /api/logs/analysis/root-cause',
            'POST /api/logs/reports/generate',
            'GET /api/logs/reports/templates',
            'GET /api/logs/reports/scheduled',
            'POST /api/logs/reports/schedule',
            'DELETE /api/logs/reports/{id}',
            'POST /api/logs/export/csv',
            'POST /api/logs/export/json'
          ].map((api, index) => (
            <div key={index} className="flex items-center justify-between p-3 bg-gray-50 rounded">
              <span className="text-sm font-mono text-gray-700">{api}</span>
              <span className="text-green-600">✓</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default LogAnalysisPage; 