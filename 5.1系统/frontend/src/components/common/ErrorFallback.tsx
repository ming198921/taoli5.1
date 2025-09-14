import React from 'react';
import { Button, Result } from 'antd';
import { ReloadOutlined, BugOutlined } from '@ant-design/icons';

interface ErrorFallbackProps {
  error: Error;
  resetError: () => void;
}

export const ErrorFallback: React.FC<ErrorFallbackProps> = ({ error, resetError }) => {
  const isDevelopment = import.meta.env.DEV;

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 p-4">
      <Result
        status="error"
        icon={<BugOutlined style={{ fontSize: '4rem', color: '#ff4d4f' }} />}
        title="系统发生错误"
        subTitle={
          <div className="space-y-2">
            <p className="text-gray-600">
              抱歉，应用程序遇到了一个意外错误。我们已经记录了这个问题。
            </p>
            {isDevelopment && (
              <details className="mt-4 p-3 bg-red-50 border border-red-200 rounded text-left">
                <summary className="cursor-pointer font-medium text-red-700 mb-2">
                  错误详情 (开发环境)
                </summary>
                <div className="text-sm text-red-600 whitespace-pre-wrap">
                  <strong>错误信息:</strong><br />
                  {error.message}
                  <br /><br />
                  <strong>堆栈跟踪:</strong><br />
                  {error.stack}
                </div>
              </details>
            )}
          </div>
        }
        extra={[
          <Button key="refresh" type="primary" icon={<ReloadOutlined />} onClick={resetError}>
            重试
          </Button>,
          <Button key="reload" onClick={() => window.location.reload()}>
            刷新页面
          </Button>,
        ]}
      />
    </div>
  );
};