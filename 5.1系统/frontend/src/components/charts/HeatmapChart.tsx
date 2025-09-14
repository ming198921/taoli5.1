import React, { useEffect, useRef, useMemo, useState } from 'react';
import * as echarts from 'echarts';
import { Card, Spin, Alert, Select, Space, Button, Tag, Slider, Switch } from 'antd';
import { ReloadOutlined, SettingOutlined, FullscreenOutlined } from '@ant-design/icons';

interface HeatmapData {
  x: string | number;
  y: string | number;
  value: number;
  rawData?: any;
}

interface HeatmapChartProps {
  title?: string;
  data: HeatmapData[];
  loading?: boolean;
  error?: string;
  height?: number;
  xAxisData?: (string | number)[];
  yAxisData?: (string | number)[];
  colorScheme?: 'default' | 'profit_loss' | 'risk_level' | 'volume' | 'custom';
  showLabels?: boolean;
  cellSize?: [number, number] | 'auto';
  realtime?: boolean;
  animation?: boolean;
  className?: string;
  onCellClick?: (data: HeatmapData) => void;
  onRefresh?: () => void;
}

export const HeatmapChart: React.FC<HeatmapChartProps> = ({
  title = '实时热力图',
  data,
  loading = false,
  error,
  height = 400,
  xAxisData = [],
  yAxisData = [],
  colorScheme = 'default',
  showLabels = true,
  cellSize = 'auto',
  realtime = false,
  animation = true,
  className,
  onCellClick,
  onRefresh,
}) => {
  const chartRef = useRef<HTMLDivElement>(null);
  const chartInstance = useRef<echarts.ECharts>();
  const [intensity, setIntensity] = useState(50);
  const [autoScale, setAutoScale] = useState(true);

  // 处理数据并生成轴数据
  const processedData = useMemo(() => {
    // 如果没有提供轴数据，从数据中提取
    const xValues = xAxisData.length > 0 ? xAxisData : [...new Set(data.map(d => d.x))].sort();
    const yValues = yAxisData.length > 0 ? yAxisData : [...new Set(data.map(d => d.y))].sort();

    // 转换数据格式为 ECharts 需要的格式
    const heatmapData = data.map(item => [
      xValues.indexOf(item.x),
      yValues.indexOf(item.y),
      item.value,
      item.rawData || item,
    ]);

    return {
      data: heatmapData,
      xAxisData: xValues,
      yAxisData: yValues,
    };
  }, [data, xAxisData, yAxisData]);

  // 颜色方案配置
  const colorSchemes = useMemo(() => ({
    default: [
      '#313695', '#4575b4', '#74add1', '#abd9e9', '#e0f3f8',
      '#ffffcc', '#fee090', '#fdae61', '#f46d43', '#d73027', '#a50026'
    ],
    profit_loss: [
      '#d73027', '#f46d43', '#fdae61', '#fee090', '#ffffcc',
      '#e0f3f8', '#abd9e9', '#74add1', '#4575b4', '#313695'
    ],
    risk_level: [
      '#1a9850', '#66bd63', '#a6d96a', '#d9ef8b', '#ffffbf',
      '#fee08b', '#fdae61', '#f46d43', '#d73027', '#a50026'
    ],
    volume: [
      '#f7fbff', '#deebf7', '#c6dbef', '#9ecae1', '#6baed6',
      '#4292c6', '#2171b5', '#08519c', '#08306b'
    ],
    custom: [
      '#ff0000', '#ff4500', '#ffa500', '#ffff00', '#9acd32',
      '#00ff00', '#00ffff', '#0000ff', '#4b0082', '#8b00ff'
    ],
  }), []);

  // 计算数值范围和统计信息
  const statistics = useMemo(() => {
    const values = data.map(d => d.value);
    const min = Math.min(...values);
    const max = Math.max(...values);
    const mean = values.reduce((sum, v) => sum + v, 0) / values.length;
    const variance = values.reduce((sum, v) => sum + Math.pow(v - mean, 2), 0) / values.length;
    const stdDev = Math.sqrt(variance);

    return {
      min,
      max,
      mean,
      stdDev,
      range: max - min,
      count: values.length,
    };
  }, [data]);

  // 图表配置
  const chartOptions = useMemo((): echarts.EChartsOption => {
    const colors = colorSchemes[colorScheme];
    const minValue = autoScale ? statistics.min : 0;
    const maxValue = autoScale ? statistics.max : 100;

    return {
      tooltip: {
        trigger: 'item',
        backgroundColor: 'rgba(0, 0, 0, 0.8)',
        borderColor: '#333',
        textStyle: {
          color: '#fff',
          fontSize: 12,
        },
        formatter: function (params: any) {
          const [xIndex, yIndex, value, rawData] = params.data;
          const xLabel = processedData.xAxisData[xIndex];
          const yLabel = processedData.yAxisData[yIndex];

          return `
            <div style="margin-bottom: 8px; font-weight: bold;">${yLabel} × ${xLabel}</div>
            <div style="margin-bottom: 4px;">数值: ${value.toFixed(4)}</div>
            ${rawData?.timestamp ? `<div>时间: ${new Date(rawData.timestamp).toLocaleTimeString()}</div>` : ''}
          `;
        },
      },
      grid: {
        left: '10%',
        right: '10%',
        top: '10%',
        bottom: '15%',
      },
      xAxis: {
        type: 'category',
        data: processedData.xAxisData,
        splitArea: {
          show: true,
        },
        axisLabel: {
          fontSize: 12,
          color: '#666',
          interval: 0,
          rotate: processedData.xAxisData.length > 10 ? 45 : 0,
        },
        axisTick: {
          show: false,
        },
        axisLine: {
          show: false,
        },
      },
      yAxis: {
        type: 'category',
        data: processedData.yAxisData,
        splitArea: {
          show: true,
        },
        axisLabel: {
          fontSize: 12,
          color: '#666',
        },
        axisTick: {
          show: false,
        },
        axisLine: {
          show: false,
        },
      },
      visualMap: {
        min: minValue,
        max: maxValue,
        calculable: true,
        realtime: true,
        orient: 'horizontal',
        left: 'center',
        bottom: '5%',
        inRange: {
          color: colors,
        },
        textStyle: {
          color: '#666',
          fontSize: 10,
        },
        formatter: function (value: number) {
          return value.toFixed(2);
        },
      },
      series: [
        {
          name: '热力数据',
          type: 'heatmap',
          data: processedData.data,
          label: {
            show: showLabels && processedData.data.length < 100, // 数据点太多时隐藏标签
            fontSize: 10,
            color: '#333',
            formatter: function (params: any) {
              return params.data[2].toFixed(2);
            },
          },
          emphasis: {
            itemStyle: {
              shadowBlur: 10,
              shadowColor: 'rgba(0, 0, 0, 0.5)',
            },
          },
          itemStyle: {
            borderColor: '#fff',
            borderWidth: 1,
          },
          progressive: 1000,
          animation: animation,
          animationDuration: 1000,
          animationEasing: 'cubicInOut',
        },
      ],
      animationDuration: animation ? 1000 : 0,
      animationEasing: 'cubicInOut',
    };
  }, [processedData, colorScheme, colorSchemes, showLabels, animation, statistics, autoScale]);

  // 初始化图表
  useEffect(() => {
    if (chartRef.current && !loading && !error) {
      chartInstance.current = echarts.init(chartRef.current, undefined, {
        renderer: 'canvas',
        useDirtyRect: true,
      });

      chartInstance.current.setOption(chartOptions, true);

      // 监听点击事件
      chartInstance.current.on('click', (params) => {
        if (params.data && onCellClick) {
          const [xIndex, yIndex, value, rawData] = params.data;
          const xLabel = processedData.xAxisData[xIndex];
          const yLabel = processedData.yAxisData[yIndex];
          
          onCellClick({
            x: xLabel,
            y: yLabel,
            value,
            rawData,
          });
        }
      });

      // 窗口大小变化时重新调整图表
      const handleResize = () => {
        chartInstance.current?.resize();
      };

      window.addEventListener('resize', handleResize);

      return () => {
        window.removeEventListener('resize', handleResize);
        chartInstance.current?.off('click');
        window.removeEventListener('resize', handleResize);
        chartInstance.current?.dispose();
      };
    }
  }, [chartOptions, loading, error, onCellClick, processedData]);

  // 实时更新
  useEffect(() => {
    if (realtime && chartInstance.current) {
      chartInstance.current.setOption(chartOptions, false);
    }
  }, [data, chartOptions, realtime]);

  const renderControls = () => (
    <Space className="mb-4" wrap>
      <Select
        value={colorScheme}
        onChange={(value) => setColorScheme(value)}
        style={{ width: 120 }}
        size="small"
      >
        <Select.Option value="default">默认</Select.Option>
        <Select.Option value="profit_loss">盈亏</Select.Option>
        <Select.Option value="risk_level">风险等级</Select.Option>
        <Select.Option value="volume">交易量</Select.Option>
        <Select.Option value="custom">自定义</Select.Option>
      </Select>

      <Space>
        <span className="text-xs text-gray-600">显示数值:</span>
        <Switch
          size="small"
          checked={showLabels}
          onChange={setShowLabels}
        />
      </Space>

      <Space>
        <span className="text-xs text-gray-600">自动缩放:</span>
        <Switch
          size="small"
          checked={autoScale}
          onChange={setAutoScale}
        />
      </Space>

      <Space>
        <span className="text-xs text-gray-600">动画:</span>
        <Switch
          size="small"
          checked={animation}
          onChange={setAnimation}
        />
      </Space>

      {onRefresh && (
        <Button
          size="small"
          icon={<ReloadOutlined />}
          onClick={onRefresh}
          loading={loading}
        >
          刷新
        </Button>
      )}
    </Space>
  );

  const renderStatistics = () => (
    <div className="mb-2">
      <Space wrap>
        <Tag color="blue">数据点: {statistics.count}</Tag>
        <Tag color="green">最小值: {statistics.min.toFixed(4)}</Tag>
        <Tag color="orange">最大值: {statistics.max.toFixed(4)}</Tag>
        <Tag color="purple">均值: {statistics.mean.toFixed(4)}</Tag>
        <Tag color="cyan">标准差: {statistics.stdDev.toFixed(4)}</Tag>
      </Space>
    </div>
  );

  if (error) {
    return (
      <Card title={title} className={className}>
        <Alert
          message="热力图加载失败"
          description={error}
          type="error"
          showIcon
          action={
            onRefresh && (
              <Button size="small" onClick={onRefresh}>
                重试
              </Button>
            )
          }
        />
      </Card>
    );
  }

  return (
    <Card
      title={
        <Space>
          {title}
          {realtime && (
            <Tag color="green" size="small">
              <span className="inline-block w-2 h-2 bg-green-400 rounded-full animate-pulse mr-1"></span>
              实时
            </Tag>
          )}
        </Space>
      }
      className={className}
      extra={
        <Button size="small" icon={<FullscreenOutlined />}>
          全屏
        </Button>
      }
    >
      {renderControls()}
      {renderStatistics()}
      
      <Spin spinning={loading} tip="加载热力图数据...">
        <div
          ref={chartRef}
          style={{ height: `${height}px`, width: '100%' }}
          className="chart-container"
        />
      </Spin>

      <div className="mt-2 text-xs text-gray-500 text-center">
        {processedData.xAxisData.length} × {processedData.yAxisData.length} 矩阵
        {realtime && ' | 实时更新'}
      </div>
    </Card>
  );
};

export default HeatmapChart;