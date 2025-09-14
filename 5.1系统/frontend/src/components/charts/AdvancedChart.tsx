import React, { useEffect, useRef, useMemo } from 'react';
import * as echarts from 'echarts';
import { Card, Spin, Alert, Select, DatePicker, Button, Space } from 'antd';
import { ReloadOutlined, DownloadOutlined, FullscreenOutlined } from '@ant-design/icons';
import type { EChartsOption } from 'echarts';

interface AdvancedChartProps {
  title?: string;
  data: any[];
  chartType: 'line' | 'area' | 'bar' | 'scatter' | 'candlestick' | 'gauge' | 'radar' | 'heatmap';
  loading?: boolean;
  error?: string;
  height?: number;
  options?: Partial<EChartsOption>;
  onRefresh?: () => void;
  onExport?: (format: 'png' | 'svg' | 'pdf') => void;
  realtime?: boolean;
  className?: string;
}

export const AdvancedChart: React.FC<AdvancedChartProps> = ({
  title,
  data,
  chartType,
  loading = false,
  error,
  height = 400,
  options = {},
  onRefresh,
  onExport,
  realtime = false,
  className,
}) => {
  const chartRef = useRef<HTMLDivElement>(null);
  const chartInstance = useRef<echarts.ECharts>();

  // 生成默认配置
  const defaultOptions = useMemo((): EChartsOption => {
    const baseOption: EChartsOption = {
      tooltip: {
        trigger: 'axis',
        backgroundColor: 'rgba(0, 0, 0, 0.8)',
        borderColor: '#333',
        textStyle: {
          color: '#fff',
          fontSize: 12,
        },
        formatter: (params: any) => {
          if (!Array.isArray(params)) params = [params];
          let tooltip = `<div style="margin-bottom: 4px">${params[0].axisValue}</div>`;
          params.forEach((param: any) => {
            tooltip += `
              <div style="margin-bottom: 2px">
                <span style="display:inline-block;margin-right:4px;border-radius:10px;width:10px;height:10px;background-color:${param.color}"></span>
                <span style="font-size:12px;color:#666;font-weight:400;margin-left:2px">${param.seriesName}</span>
                <span style="float:right;margin-left:20px;font-size:12px;color:#fff;font-weight:900">${param.value}</span>
              </div>
            `;
          });
          return tooltip;
        },
      },
      legend: {
        type: 'scroll',
        orient: 'horizontal',
        left: 'center',
        bottom: 0,
        textStyle: {
          color: '#666',
          fontSize: 12,
        },
      },
      grid: {
        left: '3%',
        right: '4%',
        bottom: '10%',
        top: '15%',
        containLabel: true,
      },
      toolbox: {
        feature: {
          saveAsImage: {
            name: `chart_${new Date().getTime()}`,
            title: '保存为图片',
          },
          dataZoom: {
            title: {
              zoom: '区域缩放',
              back: '区域缩放还原',
            },
          },
          restore: {
            title: '还原',
          },
        },
        right: '2%',
        top: '2%',
      },
      dataZoom: [
        {
          type: 'inside',
          start: 0,
          end: 100,
        },
        {
          start: 0,
          end: 100,
          height: 30,
          bottom: 30,
        },
      ],
    };

    // 根据图表类型配置特定选项
    switch (chartType) {
      case 'line':
        return {
          ...baseOption,
          xAxis: {
            type: 'category',
            boundaryGap: false,
            data: data.map(d => d.time || d.name || d.category),
            axisLine: {
              lineStyle: { color: '#ddd' },
            },
            axisLabel: {
              color: '#666',
            },
          },
          yAxis: {
            type: 'value',
            axisLine: {
              lineStyle: { color: '#ddd' },
            },
            axisLabel: {
              color: '#666',
            },
            splitLine: {
              lineStyle: { color: '#f0f0f0' },
            },
          },
          series: [
            {
              name: '数值',
              type: 'line',
              smooth: true,
              symbol: 'circle',
              symbolSize: 4,
              lineStyle: {
                width: 2,
                color: '#1890ff',
              },
              areaStyle: {
                color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                  { offset: 0, color: 'rgba(24, 144, 255, 0.3)' },
                  { offset: 1, color: 'rgba(24, 144, 255, 0.1)' },
                ]),
              },
              data: data.map(d => d.value),
            },
          ],
        };

      case 'candlestick':
        return {
          ...baseOption,
          xAxis: {
            type: 'category',
            data: data.map(d => d.time),
            boundaryGap: true,
            axisLine: { lineStyle: { color: '#ddd' } },
            axisLabel: { color: '#666' },
          },
          yAxis: {
            type: 'value',
            scale: true,
            axisLine: { lineStyle: { color: '#ddd' } },
            axisLabel: { color: '#666' },
            splitLine: { lineStyle: { color: '#f0f0f0' } },
          },
          series: [
            {
              name: 'K线',
              type: 'candlestick',
              data: data.map(d => [d.open, d.close, d.low, d.high]),
              itemStyle: {
                color: '#26a69a',
                color0: '#ef5350',
                borderColor: '#26a69a',
                borderColor0: '#ef5350',
              },
            },
          ],
        };

      case 'gauge':
        return {
          ...baseOption,
          series: [
            {
              name: '仪表盘',
              type: 'gauge',
              center: ['50%', '60%'],
              startAngle: 200,
              endAngle: -40,
              min: 0,
              max: 100,
              splitNumber: 12,
              itemStyle: {
                color: '#1890ff',
              },
              progress: {
                show: true,
                roundCap: true,
                width: 18,
              },
              pointer: {
                icon: 'path://M2090.36389,615.30999 L2090.36389,615.30999 C2091.48372,615.30999 2092.40383,616.194028 2092.44859,617.312956 L2096.90698,728.755929 C2097.05155,732.369577 2094.2393,735.416212 2090.62566,735.56078 C2090.53845,735.564269 2090.45117,735.566014 2090.36389,735.566014 L2090.36389,735.566014 C2086.74736,735.566014 2083.81557,732.63423 2083.81557,729.017692 C2083.81557,728.930412 2083.81732,728.84314 2083.82081,728.755929 L2088.2792,617.312956 C2088.32396,616.194028 2089.24407,615.30999 2090.36389,615.30999 Z',
                length: '75%',
                width: 16,
                offsetCenter: [0, '5%'],
              },
              axisLine: {
                roundCap: true,
                lineStyle: {
                  width: 18,
                },
              },
              axisTick: {
                splitNumber: 2,
                lineStyle: {
                  width: 2,
                  color: '#999',
                },
              },
              splitLine: {
                length: 12,
                lineStyle: {
                  width: 3,
                  color: '#999',
                },
              },
              axisLabel: {
                distance: 30,
                color: '#999',
                fontSize: 20,
              },
              title: {
                show: false,
              },
              detail: {
                backgroundColor: '#fff',
                borderColor: '#999',
                borderWidth: 2,
                width: '60%',
                lineHeight: 40,
                height: 40,
                borderRadius: 8,
                offsetCenter: [0, '35%'],
                valueAnimation: true,
                formatter: function (value: number) {
                  return '{value|' + value.toFixed(1) + '}{unit|%}';
                },
                rich: {
                  value: {
                    fontSize: 50,
                    fontWeight: 'bolder',
                    color: '#777',
                  },
                  unit: {
                    fontSize: 20,
                    color: '#999',
                    padding: [0, 0, -20, 10],
                  },
                },
              },
              data: [
                {
                  value: data[0]?.value || 0,
                },
              ],
            },
          ],
        };

      case 'radar':
        const indicators = data[0]?.indicators || [];
        return {
          ...baseOption,
          radar: {
            indicator: indicators,
            center: ['50%', '50%'],
            radius: '60%',
          },
          series: [
            {
              name: '雷达图',
              type: 'radar',
              data: data.map(d => ({
                value: d.values,
                name: d.name,
              })),
              areaStyle: {
                opacity: 0.1,
              },
            },
          ],
        };

      default:
        return baseOption;
    }
  }, [data, chartType]);

  // 合并用户配置
  const finalOptions = useMemo(() => {
    return echarts.util.merge(defaultOptions, options, true);
  }, [defaultOptions, options]);

  // 初始化图表
  useEffect(() => {
    if (chartRef.current && !loading && !error) {
      chartInstance.current = echarts.init(chartRef.current, undefined, {
        renderer: 'canvas',
        useDirtyRect: true,
      });

      // 设置图表配置
      chartInstance.current.setOption(finalOptions, true);

      // 窗口大小变化时重新调整图表
      const handleResize = () => {
        chartInstance.current?.resize();
      };

      window.addEventListener('resize', handleResize);

      return () => {
        window.removeEventListener('resize', handleResize);
        chartInstance.current?.dispose();
      };
    }
  }, [finalOptions, loading, error]);

  // 实时更新数据
  useEffect(() => {
    if (realtime && chartInstance.current && data.length > 0) {
      chartInstance.current.setOption(finalOptions, false);
    }
  }, [data, finalOptions, realtime]);

  const handleExport = (format: 'png' | 'svg' | 'pdf') => {
    if (chartInstance.current) {
      const url = chartInstance.current.getDataURL({
        type: format,
        pixelRatio: 2,
        backgroundColor: '#fff',
      });
      
      const link = document.createElement('a');
      link.download = `chart_${new Date().getTime()}.${format}`;
      link.href = url;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
    }
    onExport?.(format);
  };

  const renderToolbar = () => (
    <Space className="mb-2">
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
      
      {onExport && (
        <Select
          size="small"
          placeholder="导出"
          style={{ width: 80 }}
          onChange={handleExport}
          suffixIcon={<DownloadOutlined />}
        >
          <Select.Option value="png">PNG</Select.Option>
          <Select.Option value="svg">SVG</Select.Option>
          <Select.Option value="pdf">PDF</Select.Option>
        </Select>
      )}
      
      <Button
        size="small"
        icon={<FullscreenOutlined />}
        onClick={() => {
          if (chartInstance.current) {
            const option = chartInstance.current.getOption();
            // 全屏显示逻辑
            console.log('切换全屏模式', option);
          }
        }}
      >
        全屏
      </Button>
    </Space>
  );

  if (error) {
    return (
      <Card title={title} className={className}>
        <Alert
          message="图表加载失败"
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
      title={title}
      className={className}
      extra={renderToolbar()}
    >
      <Spin spinning={loading} tip="加载图表数据...">
        <div
          ref={chartRef}
          style={{ height: `${height}px`, width: '100%' }}
          className="chart-container"
        />
      </Spin>
      
      {realtime && (
        <div className="absolute top-2 right-2">
          <div className="flex items-center space-x-1 text-xs text-gray-500">
            <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
            <span>实时更新</span>
          </div>
        </div>
      )}
    </Card>
  );
};

export default AdvancedChart;