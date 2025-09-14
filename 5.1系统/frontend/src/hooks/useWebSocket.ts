import { useEffect, useRef, useCallback } from 'react';
import { useAppDispatch } from '@/store/hooks';
import { setWebSocketConnectionStatus, addNotification } from '@/store/slices/appSlice';
import { wsManager } from '@/api/websocket';

interface UseWebSocketOptions {
  autoConnect?: boolean;
  onConnect?: () => void;
  onDisconnect?: () => void;
  onError?: (error: Error) => void;
}

export const useWebSocket = (options: UseWebSocketOptions = {}) => {
  const dispatch = useAppDispatch();
  const { autoConnect = true, onConnect, onDisconnect, onError } = options;
  const isInitialized = useRef(false);

  // 连接WebSocket
  const connect = useCallback(async () => {
    try {
      dispatch(setWebSocketConnectionStatus('connecting'));
      await wsManager.connect();
      
      dispatch(setWebSocketConnectionStatus('connected'));
      dispatch(addNotification({
        type: 'success',
        title: 'WebSocket连接成功',
        message: '实时数据连接已建立',
      }));
      
      onConnect?.();
    } catch (error) {
      console.error('WebSocket connection failed:', error);
      dispatch(setWebSocketConnectionStatus('disconnected'));
      dispatch(addNotification({
        type: 'error',
        title: 'WebSocket连接失败',
        message: '无法建立实时数据连接，请检查网络',
      }));
      
      onError?.(error as Error);
    }
  }, [dispatch, onConnect, onError]);

  // 断开WebSocket
  const disconnect = useCallback(() => {
    wsManager.disconnect();
    dispatch(setWebSocketConnectionStatus('disconnected'));
    onDisconnect?.();
  }, [dispatch, onDisconnect]);

  // 发送消息
  const sendMessage = useCallback((message: any) => {
    if (wsManager.isConnected()) {
      wsManager.send(message);
    } else {
      console.warn('WebSocket not connected, message not sent:', message);
    }
  }, []);

  // 订阅主题
  const subscribe = useCallback(<T = any>(topic: string, callback: (data: T) => void) => {
    wsManager.subscribe(topic, callback);
  }, []);

  // 取消订阅
  const unsubscribe = useCallback((topic: string) => {
    wsManager.unsubscribe(topic);
  }, []);

  // 获取连接状态
  const isConnected = useCallback(() => {
    return wsManager.isConnected();
  }, []);

  // 获取连接状态详情
  const getConnectionStatus = useCallback(() => {
    return wsManager.getConnectionStatus();
  }, []);

  // 初始化WebSocket连接
  useEffect(() => {
    if (!isInitialized.current && autoConnect) {
      isInitialized.current = true;
      
      // 检查认证状态
      const token = localStorage.getItem('auth_token');
      if (token) {
        connect();
      }
    }

    return () => {
      // 组件卸载时不自动断开连接，因为其他组件可能还在使用
    };
  }, [autoConnect, connect]);

  return {
    connect,
    disconnect,
    sendMessage,
    subscribe,
    unsubscribe,
    isConnected,
    getConnectionStatus,
  };
};