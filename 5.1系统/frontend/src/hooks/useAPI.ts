import { useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useAppDispatch } from '@/store/hooks';
import { setApiConnectionStatus, addNotification } from '@/store/slices/appSlice';
import { apiClient } from '@/api/client';
import { wsManager } from '@/api/websocket';

// 通用API hook选项
interface UseAPIOptions<T = any> {
  onSuccess?: (data: T) => void;
  onError?: (error: Error) => void;
  enabled?: boolean;
  staleTime?: number;
  cacheTime?: number;
  refetchInterval?: number;
  retry?: number | boolean;
}

// 使用API查询的hook
export const useAPIQuery = <T = any>(
  queryKey: string | string[],
  queryFn: () => Promise<T>,
  options: UseAPIOptions<T> = {}
) => {
  const dispatch = useAppDispatch();
  
  return useQuery({
    queryKey: Array.isArray(queryKey) ? queryKey : [queryKey],
    queryFn: async () => {
      try {
        dispatch(setApiConnectionStatus('connected'));
        const result = await queryFn();
        options.onSuccess?.(result);
        return result;
      } catch (error) {
        dispatch(setApiConnectionStatus('disconnected'));
        dispatch(addNotification({
          type: 'error',
          title: 'API请求失败',
          message: error instanceof Error ? error.message : '网络请求错误',
        }));
        options.onError?.(error as Error);
        throw error;
      }
    },
    enabled: options.enabled !== false,
    staleTime: options.staleTime ?? 5 * 60 * 1000, // 5分钟
    cacheTime: options.cacheTime ?? 30 * 60 * 1000, // 30分钟
    refetchInterval: options.refetchInterval,
    retry: options.retry ?? 3,
    onError: (error: Error) => {
      console.error('Query error:', error);
    },
  });
};

// 使用API变更的hook
export const useAPIMutation = <T = any, V = any>(
  mutationFn: (variables: V) => Promise<T>,
  options: UseAPIOptions<T> & {
    invalidateQueries?: string | string[];
    showSuccessNotification?: boolean;
    successMessage?: string;
  } = {}
) => {
  const dispatch = useAppDispatch();
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async (variables: V) => {
      try {
        dispatch(setApiConnectionStatus('connected'));
        const result = await mutationFn(variables);
        
        if (options.showSuccessNotification !== false) {
          dispatch(addNotification({
            type: 'success',
            title: '操作成功',
            message: options.successMessage || '操作已完成',
          }));
        }
        
        options.onSuccess?.(result);
        return result;
      } catch (error) {
        dispatch(setApiConnectionStatus('disconnected'));
        dispatch(addNotification({
          type: 'error',
          title: '操作失败',
          message: error instanceof Error ? error.message : '操作执行失败',
        }));
        options.onError?.(error as Error);
        throw error;
      }
    },
    onSuccess: (data, variables) => {
      // 使相关查询失效
      if (options.invalidateQueries) {
        const keys = Array.isArray(options.invalidateQueries) 
          ? options.invalidateQueries 
          : [options.invalidateQueries];
        
        keys.forEach(key => {
          queryClient.invalidateQueries(Array.isArray(key) ? key : [key]);
        });
      }
    },
    onError: (error: Error) => {
      console.error('Mutation error:', error);
    },
  });
};

// 健康检查hook
export const useHealthCheck = (interval: number = 60000) => {
  const dispatch = useAppDispatch();
  
  return useAPIQuery(
    ['health-check'],
    async () => {
      const isHealthy = await apiClient.healthCheck();
      return { healthy: isHealthy, timestamp: new Date().toISOString() };
    },
    {
      refetchInterval: interval,
      staleTime: interval / 2,
      onSuccess: (data) => {
        dispatch(setApiConnectionStatus(data.healthy ? 'connected' : 'disconnected'));
      },
      onError: () => {
        dispatch(setApiConnectionStatus('disconnected'));
      },
    }
  );
};

// 系统版本信息hook
export const useSystemVersion = () => {
  return useAPIQuery(
    ['system-version'],
    () => apiClient.getVersion(),
    {
      staleTime: Infinity, // 版本信息不会改变，除非重新部署
      cacheTime: Infinity,
    }
  );
};

// 批量请求hook
export const useBatchAPI = <T = any>() => {
  const dispatch = useAppDispatch();
  
  return useAPIMutation(
    async (requests: Array<{
      method: 'get' | 'post' | 'put' | 'delete' | 'patch';
      url: string;
      data?: any;
      config?: any;
    }>) => {
      return apiClient.batch<T>(requests);
    },
    {
      onSuccess: () => {
        dispatch(addNotification({
          type: 'success',
          title: '批量操作完成',
          message: '所有请求已成功执行',
        }));
      },
    }
  );
};

// 文件上传hook
export const useFileUpload = () => {
  return useAPIMutation(
    async ({ url, file, config }: { url: string; file: File; config?: any }) => {
      return apiClient.upload(url, file, config);
    },
    {
      successMessage: '文件上传成功',
    }
  );
};

// 文件下载hook
export const useFileDownload = () => {
  return useAPIMutation(
    async ({ url, filename, config }: { url: string; filename?: string; config?: any }) => {
      return apiClient.download(url, filename, config);
    },
    {
      successMessage: '文件下载已开始',
    }
  );
};

// 轮询hook
export const usePolling = <T = any>(
  queryKey: string | string[],
  queryFn: () => Promise<T>,
  interval: number = 5000,
  options: UseAPIOptions<T> = {}
) => {
  return useAPIQuery(queryKey, queryFn, {
    ...options,
    refetchInterval: interval,
    refetchIntervalInBackground: true,
  });
};

// 实时数据hook (结合WebSocket)
export const useRealTimeData = <T = any>(
  topic: string,
  initialData?: T,
  options: {
    onUpdate?: (data: T) => void;
    enabled?: boolean;
  } = {}
) => {
  const queryClient = useQueryClient();
  const queryKey = ['realtime', topic];
  
  // 订阅WebSocket消息
  useEffect(() => {
    if (options.enabled !== false) {
      wsManager.subscribe<T>(topic, (data) => {
        queryClient.setQueryData(queryKey, data);
        options.onUpdate?.(data);
      });
    }
    
    return () => {
      wsManager.unsubscribe(topic);
    };
  }, [topic, options.enabled, options.onUpdate]);
  
  return useQuery({
    queryKey,
    queryFn: () => initialData || null,
    enabled: false, // 不自动获取，只通过WebSocket更新
    staleTime: Infinity,
  });
};