/**
 * 5.1套利系统 - 工具函数库
 */

import dayjs from 'dayjs';
import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import { 
  SYSTEM_STATUS, 
  MODULE_STATUS, 
  RISK_LEVELS, 
  TIME_FORMATS,
  COLORS 
} from './constants.js';

// ===== 样式工具函数 =====

/**
 * 合并 Tailwind CSS 类名
 */
export const cn = (...inputs) => {
  return twMerge(clsx(inputs));
};

/**
 * 根据状态获取对应的颜色
 */
export const getStatusColor = (status) => {
  const colorMap = {
    [SYSTEM_STATUS.RUNNING]: COLORS.SUCCESS,
    [SYSTEM_STATUS.STOPPED]: COLORS.ERROR,
    [SYSTEM_STATUS.STARTING]: COLORS.WARNING,
    [SYSTEM_STATUS.STOPPING]: COLORS.WARNING,
    [SYSTEM_STATUS.ERROR]: COLORS.ERROR,
    [SYSTEM_STATUS.UNKNOWN]: '#d9d9d9',
    
    [MODULE_STATUS.HEALTHY]: COLORS.SUCCESS,
    [MODULE_STATUS.UNHEALTHY]: COLORS.ERROR,
    [MODULE_STATUS.WARNING]: COLORS.WARNING,
    [MODULE_STATUS.UNKNOWN]: '#d9d9d9',
    
    [RISK_LEVELS.LOW]: COLORS.SUCCESS,
    [RISK_LEVELS.MEDIUM]: COLORS.WARNING,
    [RISK_LEVELS.HIGH]: COLORS.ERROR,
    [RISK_LEVELS.CRITICAL]: '#ff0000',
  };
  
  return colorMap[status] || '#d9d9d9';
};

/**
 * 根据状态获取对应的 Tailwind 类名
 */
export const getStatusClass = (status) => {
  const classMap = {
    [SYSTEM_STATUS.RUNNING]: 'text-green-600 bg-green-50 border-green-200',
    [SYSTEM_STATUS.STOPPED]: 'text-red-600 bg-red-50 border-red-200',
    [SYSTEM_STATUS.STARTING]: 'text-yellow-600 bg-yellow-50 border-yellow-200',
    [SYSTEM_STATUS.STOPPING]: 'text-yellow-600 bg-yellow-50 border-yellow-200',
    [SYSTEM_STATUS.ERROR]: 'text-red-600 bg-red-50 border-red-200',
    [SYSTEM_STATUS.UNKNOWN]: 'text-gray-600 bg-gray-50 border-gray-200',
    
    [MODULE_STATUS.HEALTHY]: 'text-green-600 bg-green-50',
    [MODULE_STATUS.UNHEALTHY]: 'text-red-600 bg-red-50',
    [MODULE_STATUS.WARNING]: 'text-yellow-600 bg-yellow-50',
    [MODULE_STATUS.UNKNOWN]: 'text-gray-600 bg-gray-50',
    
    [RISK_LEVELS.LOW]: 'text-green-600 bg-green-50',
    [RISK_LEVELS.MEDIUM]: 'text-yellow-600 bg-yellow-50',
    [RISK_LEVELS.HIGH]: 'text-red-600 bg-red-50',
    [RISK_LEVELS.CRITICAL]: 'text-red-700 bg-red-100',
  };
  
  return classMap[status] || 'text-gray-600 bg-gray-50';
};

// ===== 时间工具函数 =====

/**
 * 格式化时间
 */
export const formatTime = (timestamp, format = TIME_FORMATS.DATETIME) => {
  if (!timestamp) return '-';
  return dayjs(timestamp).format(format);
};

/**
 * 获取相对时间
 */
export const getRelativeTime = (timestamp) => {
  if (!timestamp) return '-';
  return dayjs(timestamp).fromNow();
};

/**
 * 计算运行时间（秒转换为可读格式）
 */
export const formatUptime = (seconds) => {
  if (!seconds || seconds < 0) return '0秒';
  
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);
  
  const parts = [];
  if (days > 0) parts.push(`${days}天`);
  if (hours > 0) parts.push(`${hours}小时`);
  if (minutes > 0) parts.push(`${minutes}分钟`);
  if (secs > 0 || parts.length === 0) parts.push(`${secs}秒`);
  
  return parts.join(' ');
};

/**
 * 计算时间差（毫秒）
 */
export const getTimeDiff = (startTime, endTime = new Date()) => {
  return dayjs(endTime).diff(dayjs(startTime));
};

// ===== 数值工具函数 =====

/**
 * 格式化数字（添加千分位分隔符）
 */
export const formatNumber = (num, decimals = 2) => {
  if (num === null || num === undefined || isNaN(num)) return '-';
  return Number(num).toLocaleString('zh-CN', {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals
  });
};

/**
 * 格式化货币
 */
export const formatCurrency = (amount, currency = '¥', decimals = 2) => {
  if (amount === null || amount === undefined || isNaN(amount)) return '-';
  const formattedAmount = formatNumber(amount, decimals);
  return `${currency}${formattedAmount}`;
};

/**
 * 格式化百分比
 */
export const formatPercentage = (value, decimals = 2) => {
  if (value === null || value === undefined || isNaN(value)) return '-';
  return `${formatNumber(value, decimals)}%`;
};

export const formatNumber2 = (num, decimals = 2) => {
  if (num === null || num === undefined || isNaN(num)) return '-';
  return Number(num).toLocaleString('zh-CN', {
    minimumFractionDigits: 0,
    maximumFractionDigits: decimals,
  });
};

/**
 * 格式化百分比
 */
export const formatPercent = (value, decimals = 1) => {
  if (value === null || value === undefined || isNaN(value)) return '-';
  return `${Number(value).toFixed(decimals)}%`;
};

/**
 * 格式化字节大小
 */
export const formatBytes = (bytes, decimals = 1) => {
  if (bytes === 0 || !bytes) return '0 B';
  
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
};

/**
 * 格式化延迟时间（毫秒）
 */
export const formatLatency = (ms) => {
  if (ms === null || ms === undefined || isNaN(ms)) return '-';
  return `${Number(ms).toFixed(0)}ms`;
};

// ===== 数据处理工具函数 =====

/**
 * 深拷贝对象
 */
export const deepClone = (obj) => {
  if (obj === null || typeof obj !== 'object') return obj;
  if (obj instanceof Date) return new Date(obj.getTime());
  if (obj instanceof Array) return obj.map(item => deepClone(item));
  if (typeof obj === 'object') {
    const clonedObj = {};
    for (const key in obj) {
      if (obj.hasOwnProperty(key)) {
        clonedObj[key] = deepClone(obj[key]);
      }
    }
    return clonedObj;
  }
};

/**
 * 防抖函数
 */
export const debounce = (func, wait, immediate = false) => {
  let timeout;
  return function executedFunction(...args) {
    const later = () => {
      timeout = null;
      if (!immediate) func(...args);
    };
    const callNow = immediate && !timeout;
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
    if (callNow) func(...args);
  };
};

/**
 * 节流函数
 */
export const throttle = (func, limit) => {
  let inThrottle;
  return function (...args) {
    if (!inThrottle) {
      func.apply(this, args);
      inThrottle = true;
      setTimeout(() => (inThrottle = false), limit);
    }
  };
};

/**
 * 生成唯一ID
 */
export const generateId = (prefix = 'id') => {
  return `${prefix}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
};

// ===== 验证工具函数 =====

/**
 * 验证是否为空值
 */
export const isEmpty = (value) => {
  if (value === null || value === undefined) return true;
  if (typeof value === 'string') return value.trim() === '';
  if (Array.isArray(value)) return value.length === 0;
  if (typeof value === 'object') return Object.keys(value).length === 0;
  return false;
};

/**
 * 验证是否为有效的JSON
 */
export const isValidJson = (str) => {
  try {
    JSON.parse(str);
    return true;
  } catch (e) {
    return false;
  }
};

/**
 * 验证是否为数字
 */
export const isNumber = (value) => {
  return !isNaN(parseFloat(value)) && isFinite(value);
};

// ===== URL工具函数 =====

/**
 * 构建查询字符串
 */
export const buildQueryString = (params) => {
  const query = new URLSearchParams();
  Object.keys(params).forEach(key => {
    if (params[key] !== null && params[key] !== undefined) {
      query.append(key, params[key]);
    }
  });
  return query.toString();
};

/**
 * 解析查询字符串
 */
export const parseQueryString = (queryString) => {
  const params = new URLSearchParams(queryString);
  const result = {};
  for (const [key, value] of params.entries()) {
    result[key] = value;
  }
  return result;
};

// ===== 错误处理工具函数 =====

/**
 * 安全的JSON解析
 */
export const safeJsonParse = (str, defaultValue = null) => {
  try {
    return JSON.parse(str);
  } catch (error) {
    console.warn('JSON解析失败:', error);
    return defaultValue;
  }
};

/**
 * 安全的属性访问
 */
export const safeGet = (obj, path, defaultValue = null) => {
  try {
    return path.split('.').reduce((current, key) => current[key], obj) ?? defaultValue;
  } catch (error) {
    return defaultValue;
  }
};

// ===== 本地存储工具函数 =====

/**
 * 设置本地存储
 */
export const setStorage = (key, value) => {
  try {
    localStorage.setItem(key, JSON.stringify(value));
    return true;
  } catch (error) {
    console.error('设置本地存储失败:', error);
    return false;
  }
};

/**
 * 获取本地存储
 */
export const getStorage = (key, defaultValue = null) => {
  try {
    const item = localStorage.getItem(key);
    return item ? JSON.parse(item) : defaultValue;
  } catch (error) {
    console.error('获取本地存储失败:', error);
    return defaultValue;
  }
};

/**
 * 删除本地存储
 */
export const removeStorage = (key) => {
  try {
    localStorage.removeItem(key);
    return true;
  } catch (error) {
    console.error('删除本地存储失败:', error);
    return false;
  }
};

// ===== 数组工具函数 =====

/**
 * 数组去重
 */
export const uniqueArray = (arr, key = null) => {
  if (!Array.isArray(arr)) return [];
  
  if (key) {
    const seen = new Set();
    return arr.filter(item => {
      const value = item[key];
      if (seen.has(value)) {
        return false;
      }
      seen.add(value);
      return true;
    });
  }
  
  return [...new Set(arr)];
};

/**
 * 数组分页
 */
export const paginateArray = (arr, page = 1, pageSize = 10) => {
  if (!Array.isArray(arr)) return { data: [], total: 0 };
  
  const startIndex = (page - 1) * pageSize;
  const endIndex = startIndex + pageSize;
  
  return {
    data: arr.slice(startIndex, endIndex),
    total: arr.length,
    page,
    pageSize,
    totalPages: Math.ceil(arr.length / pageSize),
  };
};

// ===== 导出所有工具函数 =====
export default {
  // 样式工具
  cn,
  getStatusColor,
  getStatusClass,
  
  // 时间工具
  formatTime,
  getRelativeTime,
  formatUptime,
  getTimeDiff,
  
  // 数值工具
  formatNumber,
  formatPercent,
  formatBytes,
  formatLatency,
  
  // 数据处理
  deepClone,
  debounce,
  throttle,
  generateId,
  
  // 验证工具
  isEmpty,
  isValidJson,
  isNumber,
  
  // URL工具
  buildQueryString,
  parseQueryString,
  
  // 错误处理
  safeJsonParse,
  safeGet,
  
  // 本地存储
  setStorage,
  getStorage,
  removeStorage,
  
  // 数组工具
  uniqueArray,
  paginateArray,
};