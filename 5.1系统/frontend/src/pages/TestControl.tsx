// 系统控制测试页面
// 专门用于测试前端控制后端的功能

import React, { useState, useEffect } from 'react';
import { Card, Button, Space, message, Alert, Typography, Badge, Divider } from 'antd';
import { 
  PlayCircleOutlined, 
  PauseCircleOutlined, 
  ReloadOutlined,
  ApiOutlined,
  CheckCircleOutlined,
  CloseCircleOutlined
} from '@ant-design/icons';
import systemControl, { DeploymentType } from '@/services/systemControl';

const { Title, Text, Paragraph } = Typography;

export const TestControl: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const [backendStatus, setBackendStatus] = useState<'unknown' | 'running' | 'stopped'>('unknown');
  const [apiConnected, setApiConnected] = useState(false);
  const [testResults, setTestResults] = useState<string[]>([]);
  const [deploymentType, setDeploymentType] = useState<DeploymentType>(DeploymentType.DIRECT);

  useEffect(() => {
    setDeploymentType(systemControl.getDeploymentType());
    checkBackendStatus();
  }, []);

  // 添加测试结果
  const addTestResult = (message: string, success: boolean = true) => {
    const timestamp = new Date().toLocaleTimeString();
    const result = `[${timestamp}] ${success ? '✅' : '❌'} ${message}`;
    setTestResults(prev => [...prev.slice(-9), result]); // 只保留最近10条
  };

  // 检查后端状态
  const checkBackendStatus = async () => {
    try {
      const response = await fetch('http://localhost:8080/health');
      if (response.ok) {
        setBackendStatus('running');
        setApiConnected(true);
        addTestResult('后端API连接正常');
      } else {
        setBackendStatus('stopped');
        setApiConnected(false);
        addTestResult('后端API响应异常', false);
      }
    } catch (error) {
      setBackendStatus('stopped');
      setApiConnected(false);
      addTestResult('无法连接到后端API', false);
    }
  };

  // 测试启动系统
  const testStartSystem = async () => {
    setLoading(true);
    addTestResult('正在尝试启动系统...');
    
    try {
      const result = await systemControl.startModule('system');
      
      if (result.success) {
        message.success('启动命令发送成功');
        addTestResult(`启动成功: ${result.message}`);
        
        // 等待一段时间后检查状态
        setTimeout(async () => {
          await checkBackendStatus();
        }, 3000);
      } else {
        message.error('启动失败');
        addTestResult(`启动失败: ${result.message}`, false);
      }
    } catch (error) {
      message.error('启动过程中发生错误');
      addTestResult(`启动错误: ${error}`, false);
    } finally {
      setLoading(false);
    }
  };

  // 测试停止系统
  const testStopSystem = async () => {
    setLoading(true);
    addTestResult('正在尝试停止系统...');
    
    try {
      const result = await systemControl.stopModule('system');
      
      if (result.success) {
        message.success('停止命令发送成功');
        addTestResult(`停止成功: ${result.message}`);
        
        // 等待一段时间后检查状态
        setTimeout(async () => {
          await checkBackendStatus();
        }, 3000);
      } else {
        message.error('停止失败');
        addTestResult(`停止失败: ${result.message}`, false);
      }
    } catch (error) {
      message.error('停止过程中发生错误');
      addTestResult(`停止错误: ${error}`, false);
    } finally {
      setLoading(false);
    }
  };

  // 测试重启系统
  const testRestartSystem = async () => {
    setLoading(true);
    addTestResult('正在尝试重启系统...');
    
    try {
      const result = await systemControl.restartModule('system');
      
      if (result.success) {
        message.success('重启命令发送成功');
        addTestResult(`重启成功: ${result.message}`);
        
        // 等待一段时间后检查状态
        setTimeout(async () => {
          await checkBackendStatus();
        }, 5000);
      } else {
        message.error('重启失败');
        addTestResult(`重启失败: ${result.message}`, false);
      }
    } catch (error) {
      message.error('重启过程中发生错误');
      addTestResult(`重启错误: ${error}`, false);
    } finally {
      setLoading(false);
    }
  };

  // 测试获取状态
  const testGetStatus = async () => {
    addTestResult('正在获取系统状态...');
    
    try {
      const status = await systemControl.getModuleStatus('system');
      addTestResult(`状态获取成功: ${status.status} (健康度: ${status.health})`);
    } catch (error) {
      addTestResult(`状态获取失败: ${error}`, false);
    }
  };

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <Title level={2}>
        系统控制功能测试
      </Title>
      
      <Paragraph>
        这个页面专门用于测试前端控制后端的功能。当前部署类型: <Badge count={deploymentType} />
      </Paragraph>

      <Alert
        message="测试说明"
        description={
          <div>
            <p>1. 首先检查后端连接状态</p>
            <p>2. 测试启动、停止、重启功能</p>
            <p>3. 观察测试结果和后端响应</p>
            <p>4. 确认前端可以完全控制后端系统</p>
          </div>
        }
        type="info"
        showIcon
        className="mb-4"
      />

      {/* 当前状态 */}
      <Card title="当前系统状态" className="mb-4">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="text-center">
            <div className="text-lg mb-2">
              {apiConnected ? (
                <CheckCircleOutlined style={{ color: '#52c41a' }} />
              ) : (
                <CloseCircleOutlined style={{ color: '#ff4d4f' }} />
              )}
            </div>
            <div>API连接</div>
            <Text type="secondary">{apiConnected ? '已连接' : '未连接'}</Text>
          </div>
          
          <div className="text-center">
            <div className="text-lg mb-2">
              <Badge 
                status={backendStatus === 'running' ? 'success' : backendStatus === 'stopped' ? 'error' : 'default'}
              />
            </div>
            <div>后端状态</div>
            <Text type="secondary">
              {backendStatus === 'running' ? '运行中' : 
               backendStatus === 'stopped' ? '已停止' : '未知'}
            </Text>
          </div>
          
          <div className="text-center">
            <div className="text-lg mb-2">
              <ApiOutlined />
            </div>
            <div>部署模式</div>
            <Text type="secondary">{deploymentType}</Text>
          </div>
        </div>
        
        <Divider />
        
        <div className="text-center">
          <Button 
            icon={<ReloadOutlined />} 
            onClick={checkBackendStatus}
          >
            刷新状态
          </Button>
        </div>
      </Card>

      {/* 控制按钮 */}
      <Card title="系统控制测试" className="mb-4">
        <Space size="large" wrap>
          <Button
            type="primary"
            size="large"
            icon={<PlayCircleOutlined />}
            loading={loading}
            onClick={testStartSystem}
          >
            测试启动系统
          </Button>
          
          <Button
            danger
            size="large"
            icon={<PauseCircleOutlined />}
            loading={loading}
            onClick={testStopSystem}
          >
            测试停止系统
          </Button>
          
          <Button
            size="large"
            icon={<ReloadOutlined />}
            loading={loading}
            onClick={testRestartSystem}
          >
            测试重启系统
          </Button>
          
          <Button
            size="large"
            icon={<ApiOutlined />}
            onClick={testGetStatus}
          >
            获取状态
          </Button>
        </Space>
      </Card>

      {/* 测试结果 */}
      <Card title="测试结果日志">
        <div style={{ 
          height: '300px', 
          overflowY: 'auto', 
          backgroundColor: '#001529', 
          padding: '12px', 
          borderRadius: '6px',
          fontFamily: 'Monaco, Consolas, "Lucida Console", monospace'
        }}>
          {testResults.length > 0 ? (
            testResults.map((result, index) => (
              <div key={index} style={{ 
                color: result.includes('✅') ? '#52c41a' : 
                       result.includes('❌') ? '#ff4d4f' : '#d9d9d9',
                fontSize: '12px',
                lineHeight: '1.4',
                marginBottom: '2px',
                whiteSpace: 'pre-wrap'
              }}>
                {result}
              </div>
            ))
          ) : (
            <div style={{ color: '#d9d9d9', textAlign: 'center', marginTop: '50px' }}>
              暂无测试结果...
              <br />
              <small>点击上面的按钮开始测试</small>
            </div>
          )}
        </div>
        
        <div className="mt-3 text-center">
          <Button 
            size="small"
            onClick={() => setTestResults([])}
          >
            清空日志
          </Button>
        </div>
      </Card>
    </div>
  );
};

export default TestControl;