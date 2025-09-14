import React, { useState, useEffect } from 'react';
import { Form, Input, Button, Card, Alert, Checkbox, Divider, Typography, Space } from 'antd';
import { UserOutlined, LockOutlined, SafetyOutlined } from '@ant-design/icons';
import { useAppDispatch, useAppSelector } from '@/store/hooks';
import { loginUser, verifyTwoFactor, clearAuthError } from '@/store/slices/authSlice';
import { useNavigate } from 'react-router-dom';

const { Title, Text } = Typography;

interface LoginFormData {
  username: string;
  password: string;
  remember: boolean;
  captcha?: string;
}

interface TwoFactorFormData {
  code: string;
}

export const LoginPage: React.FC = () => {
  const dispatch = useAppDispatch();
  const navigate = useNavigate();
  const [form] = Form.useForm();
  const [twoFactorForm] = Form.useForm();
  
  const { isLoading, isAuthenticated, twoFactorRequired, twoFactorToken, isLocked, lockExpiry } = 
    useAppSelector(state => state.auth);
  
  const [error, setError] = useState<string | null>(null);
  const [lockTimeRemaining, setLockTimeRemaining] = useState<number>(0);

  // 如果已登录，重定向到仪表板
  useEffect(() => {
    if (isAuthenticated) {
      navigate('/dashboard', { replace: true });
    }
  }, [isAuthenticated, navigate]);

  // 处理账户锁定倒计时
  useEffect(() => {
    if (isLocked && lockExpiry) {
      const updateTimer = () => {
        const now = new Date().getTime();
        const lockEnd = new Date(lockExpiry).getTime();
        const remaining = Math.max(0, lockEnd - now);
        
        setLockTimeRemaining(remaining);
        
        if (remaining <= 0) {
          setError(null);
        }
      };
      
      updateTimer();
      const interval = setInterval(updateTimer, 1000);
      
      return () => clearInterval(interval);
    }
  }, [isLocked, lockExpiry]);

  // 处理普通登录
  const handleLogin = async (values: LoginFormData) => {
    try {
      setError(null);
      await dispatch(loginUser(values)).unwrap();
      
      if (!twoFactorRequired) {
        navigate('/dashboard');
      }
    } catch (error: any) {
      setError(error.message || '登录失败，请重试');
    }
  };

  // 处理二次验证
  const handleTwoFactorVerify = async (values: TwoFactorFormData) => {
    if (!twoFactorToken) return;
    
    try {
      setError(null);
      await dispatch(verifyTwoFactor({
        token: twoFactorToken,
        code: values.code,
      })).unwrap();
      
      navigate('/dashboard');
    } catch (error: any) {
      setError(error.message || '验证失败，请重试');
    }
  };

  // 格式化锁定剩余时间
  const formatLockTime = (ms: number): string => {
    const minutes = Math.floor(ms / 60000);
    const seconds = Math.floor((ms % 60000) / 1000);
    return `${minutes}分${seconds}秒`;
  };

  // 系统信息
  const systemInfo = {
    version: '5.1.0',
    buildTime: '2024-09-02',
    environment: import.meta.env.MODE,
  };

  return (
    <div className="min-h-screen flex">
      {/* 左侧背景图 */}
      <div className="hidden lg:block lg:w-1/2 bg-gradient-to-br from-blue-600 via-purple-600 to-blue-800 relative">
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="text-center text-white p-8">
            <div className="mb-8">
              <div className="w-32 h-32 mx-auto mb-6 rounded-full bg-white bg-opacity-20 flex items-center justify-center">
                <span className="text-4xl font-bold">5.1</span>
              </div>
              <Title level={1} className="text-white mb-4">
                高频套利系统
              </Title>
              <Text className="text-xl text-blue-100">
                专业的加密货币套利交易管理平台
              </Text>
            </div>
            
            <div className="space-y-4 text-blue-100">
              <div className="flex items-center justify-center space-x-2">
                <SafetyOutlined />
                <span>企业级安全保障</span>
              </div>
              <div className="flex items-center justify-center space-x-2">
                <UserOutlined />
                <span>专业团队支持</span>
              </div>
              <div className="flex items-center justify-center space-x-2">
                <LockOutlined />
                <span>数据隐私保护</span>
              </div>
            </div>
          </div>
        </div>
        
        {/* 装饰性元素 */}
        <div className="absolute top-10 left-10 w-20 h-20 rounded-full bg-white bg-opacity-10 animate-pulse"></div>
        <div className="absolute bottom-20 right-20 w-16 h-16 rounded-full bg-white bg-opacity-10 animate-pulse animation-delay-1000"></div>
        <div className="absolute top-1/3 right-10 w-12 h-12 rounded-full bg-white bg-opacity-10 animate-pulse animation-delay-2000"></div>
      </div>

      {/* 右侧登录表单 */}
      <div className="w-full lg:w-1/2 flex items-center justify-center p-8 bg-gray-50">
        <Card className="w-full max-w-md shadow-xl">
          <div className="text-center mb-8">
            <Title level={2} className="mb-2">
              {twoFactorRequired ? '二次验证' : '系统登录'}
            </Title>
            <Text type="secondary">
              {twoFactorRequired 
                ? '请输入您的验证码' 
                : '登录您的5.1套利系统账户'
              }
            </Text>
          </div>

          {/* 错误提示 */}
          {error && (
            <Alert
              message={error}
              type="error"
              showIcon
              closable
              className="mb-4"
              onClose={() => setError(null)}
            />
          )}

          {/* 账户锁定提示 */}
          {isLocked && lockTimeRemaining > 0 && (
            <Alert
              message="账户已被锁定"
              description={`请在 ${formatLockTime(lockTimeRemaining)} 后重试`}
              type="warning"
              showIcon
              className="mb-4"
            />
          )}

          {/* 二次验证表单 */}
          {twoFactorRequired ? (
            <Form
              form={twoFactorForm}
              name="two-factor"
              onFinish={handleTwoFactorVerify}
              size="large"
            >
              <Form.Item
                name="code"
                rules={[
                  { required: true, message: '请输入验证码' },
                  { len: 6, message: '验证码应为6位数字' },
                ]}
              >
                <Input
                  prefix={<SafetyOutlined />}
                  placeholder="请输入6位验证码"
                  maxLength={6}
                  className="text-center text-2xl tracking-widest"
                />
              </Form.Item>

              <Form.Item>
                <Button
                  type="primary"
                  htmlType="submit"
                  loading={isLoading}
                  block
                  size="large"
                >
                  验证登录
                </Button>
              </Form.Item>
            </Form>
          ) : (
            /* 普通登录表单 */
            <Form
              form={form}
              name="login"
              onFinish={handleLogin}
              initialValues={{ remember: true }}
              size="large"
            >
              <Form.Item
                name="username"
                rules={[
                  { required: true, message: '请输入用户名' },
                  { min: 3, message: '用户名至少3个字符' },
                ]}
              >
                <Input
                  prefix={<UserOutlined />}
                  placeholder="用户名"
                  disabled={isLocked && lockTimeRemaining > 0}
                />
              </Form.Item>

              <Form.Item
                name="password"
                rules={[
                  { required: true, message: '请输入密码' },
                  { min: 6, message: '密码至少6个字符' },
                ]}
              >
                <Input.Password
                  prefix={<LockOutlined />}
                  placeholder="密码"
                  disabled={isLocked && lockTimeRemaining > 0}
                />
              </Form.Item>

              <Form.Item>
                <div className="flex justify-between items-center">
                  <Form.Item name="remember" valuePropName="checked" noStyle>
                    <Checkbox>记住我</Checkbox>
                  </Form.Item>
                  <Button type="link" className="p-0">
                    忘记密码？
                  </Button>
                </div>
              </Form.Item>

              <Form.Item>
                <Button
                  type="primary"
                  htmlType="submit"
                  loading={isLoading}
                  disabled={isLocked && lockTimeRemaining > 0}
                  block
                  size="large"
                >
                  登录
                </Button>
              </Form.Item>
            </Form>
          )}

          <Divider />

          {/* 系统信息 */}
          <div className="text-center text-gray-500 text-xs">
            <Space direction="vertical" size="small">
              <div>版本 {systemInfo.version} | 环境 {systemInfo.environment}</div>
              <div>© 2024 5.1套利系统. 保留所有权利.</div>
            </Space>
          </div>
        </Card>
      </div>
    </div>
  );
};