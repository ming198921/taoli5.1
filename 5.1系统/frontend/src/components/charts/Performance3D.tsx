import React, { useEffect, useRef, useMemo } from 'react';
import * as echarts from 'echarts';
import 'echarts-gl';
import { Card, Spin, Alert, Select, Space, Button, Slider, Switch } from 'antd';
import { RotateLeftOutlined, ReloadOutlined, SettingOutlined } from '@ant-design/icons';

interface Performance3DData {
  strategy: string;
  timestamp: string;
  profit: number;
  risk: number;
  volume: number;
  sharpe: number;
  maxDrawdown: number;
  winRate: number;
}

interface Performance3DProps {
  title?: string;
  data: Performance3DData[];
  loading?: boolean;
  error?: string;
  height?: number;
  viewType?: '3d_scatter' | '3d_surface' | '3d_bar' | '3d_line';
  xAxis?: 'profit' | 'risk' | 'volume' | 'sharpe';
  yAxis?: 'profit' | 'risk' | 'volume' | 'sharpe';
  zAxis?: 'profit' | 'risk' | 'volume' | 'sharpe';
  colorBy?: 'profit' | 'risk' | 'sharpe' | 'winRate';
  animation?: boolean;
  className?: string;
  onDataPointClick?: (dataPoint: Performance3DData) => void;
}

export const Performance3D: React.FC<Performance3DProps> = ({
  title = '3D性能分析',
  data,
  loading = false,
  error,
  height = 600,
  viewType = '3d_scatter',
  xAxis = 'risk',
  yAxis = 'profit',
  zAxis = 'volume',
  colorBy = 'sharpe',
  animation = true,
  className,
  onDataPointClick,
}) => {
  const chartRef = useRef<HTMLDivElement>(null);
  const chartInstance = useRef<echarts.ECharts>();
  const [autoRotate, setAutoRotate] = React.useState(false);
  const [viewAngle, setViewAngle] = React.useState({ alpha: 40, beta: 40 });

  // 数据预处理
  const processedData = useMemo(() => {
    return data.map(item => {
      const point = {
        name: item.strategy,
        value: [
          item[xAxis],
          item[yAxis], 
          item[zAxis],
          item[colorBy], // 颜色值
        ],
        rawData: item,
      };
      return point;
    });
  }, [data, xAxis, yAxis, zAxis, colorBy]);

  // 获取轴标签
  const getAxisLabel = (axis: string) => {
    const labels: Record<string, string> = {
      profit: '收益率 (%)',
      risk: '风险值',
      volume: '交易量',
      sharpe: '夏普比率',
      maxDrawdown: '最大回撤 (%)',
      winRate: '胜率 (%)',
    };
    return labels[axis] || axis;
  };

  // 获取颜色映射
  const getColorMapping = (value: number, min: number, max: number) => {
    const ratio = (value - min) / (max - min);
    if (colorBy === 'profit') {
      return ratio > 0.5 ? 'rgb(76, 175, 80)' : 'rgb(244, 67, 54)';
    } else if (colorBy === 'risk') {
      return `rgb(${255 * ratio}, ${255 * (1 - ratio)}, 50)`;
    } else if (colorBy === 'sharpe') {
      return `rgb(${50}, ${255 * ratio}, ${255 * (1 - ratio)})`;
    } else {
      return `rgb(${100 + 155 * ratio}, ${150}, ${200 - 100 * ratio})`;
    }
  };

  // 图表配置
  const chartOptions = useMemo((): echarts.EChartsOption => {
    const colorValues = processedData.map(d => d.value[3]);
    const minColor = Math.min(...colorValues);
    const maxColor = Math.max(...colorValues);

    const baseOption = {
      tooltip: {
        trigger: 'item',
        backgroundColor: 'rgba(0, 0, 0, 0.8)',
        borderColor: '#333',
        textStyle: {
          color: '#fff',
        },
        formatter: function (params: any) {
          const data = params.data?.rawData;
          if (!data) return '';
          
          return `
            <div style="margin-bottom: 8px; font-weight: bold; font-size: 14px;">${data.strategy}</div>
            <div style="margin-bottom: 4px;">时间: ${new Date(data.timestamp).toLocaleString()}</div>
            <div style="margin-bottom: 4px;">收益率: ${data.profit.toFixed(2)}%</div>
            <div style="margin-bottom: 4px;">风险值: ${data.risk.toFixed(4)}</div>
            <div style="margin-bottom: 4px;">交易量: ${data.volume.toLocaleString()}</div>
            <div style="margin-bottom: 4px;">夏普比率: ${data.sharpe.toFixed(2)}</div>
            <div style="margin-bottom: 4px;">最大回撤: ${data.maxDrawdown.toFixed(2)}%</div>
            <div>胜率: ${data.winRate.toFixed(1)}%</div>
          `;
        },
      },
      visualMap: {
        min: minColor,
        max: maxColor,
        dimension: 3,
        inRange: {
          color: colorBy === 'profit' 
            ? ['#ff4757', '#ffa502', '#2ed573']
            : colorBy === 'risk'
            ? ['#70a1ff', '#5352ed', '#ff3838'] 
            : ['#3742fa', '#2f3542', '#ff6348'],
        },
        textStyle: {
          color: '#666',
        },
        left: 'left',
        top: 'bottom',
        text: [`高${getAxisLabel(colorBy)}`, `低${getAxisLabel(colorBy)}`],
        calculable: true,
        realtime: true,
      },
      grid3D: {
        viewControl: {
          projection: 'perspective',
          autoRotate: autoRotate,
          autoRotateDirection: 'cw',
          autoRotateSpeed: 10,
          damping: 0.8,
          rotateSensitivity: 1,
          zoomSensitivity: 1,
          panSensitivity: 1,
          alpha: viewAngle.alpha,
          beta: viewAngle.beta,
          distance: 200,
        },
        boxWidth: 100,
        boxHeight: 100,
        boxDepth: 100,
        light: {
          main: {
            intensity: 1.2,
            shadow: true,
            shadowQuality: 'high',
            alpha: 30,
            beta: 40,
          },
          ambient: {
            intensity: 0.3,
          },
        },
        environment: 'auto',
        groundPlane: {
          show: true,
          color: '#f8f9fa',
        },
        postEffect: {
          enable: true,
          SSAO: {
            enable: true,
            quality: 'medium',
            radius: 2,
          },
        },
      },
      xAxis3D: {
        name: getAxisLabel(xAxis),
        nameTextStyle: {
          fontSize: 14,
          color: '#666',
        },
        axisLabel: {
          fontSize: 12,
          color: '#666',
        },
      },
      yAxis3D: {
        name: getAxisLabel(yAxis),
        nameTextStyle: {
          fontSize: 14,
          color: '#666',
        },
        axisLabel: {
          fontSize: 12,
          color: '#666',
        },
      },
      zAxis3D: {
        name: getAxisLabel(zAxis),
        nameTextStyle: {
          fontSize: 14,
          color: '#666',
        },
        axisLabel: {
          fontSize: 12,
          color: '#666',
        },
      },
      animation: animation,
      animationDuration: 1000,
      animationEasing: 'cubicOut',
    };

    // 根据视图类型配置系列
    switch (viewType) {
      case '3d_scatter':
        return {
          ...baseOption,
          series: [
            {
              type: 'scatter3D',
              data: processedData,
              itemStyle: {
                opacity: 0.8,
              },
              emphasis: {
                itemStyle: {
                  opacity: 1,
                },
              },
            },
          ],
        };

      case '3d_surface':
        // 为曲面图转换数据格式
        const surfaceData = [];
        const xRange = [...new Set(processedData.map(d => d.value[0]))].sort((a, b) => a - b);
        const yRange = [...new Set(processedData.map(d => d.value[1]))].sort((a, b) => a - b);
        
        for (let i = 0; i < xRange.length; i++) {
          const row = [];
          for (let j = 0; j < yRange.length; j++) {
            const point = processedData.find(d => d.value[0] === xRange[i] && d.value[1] === yRange[j]);
            row.push(point ? point.value[2] : 0);
          }
          surfaceData.push(row);
        }

        return {
          ...baseOption,
          series: [
            {
              type: 'surface',
              data: surfaceData,
              itemStyle: {
                opacity: 0.8,
              },
              shading: 'realistic',
              realisticMaterial: {
                roughness: 0.8,
                metalness: 0,
              },
            },
          ],
        };

      case '3d_bar':
        return {
          ...baseOption,
          series: [
            {
              type: 'bar3D',
              data: processedData.map(d => ({
                ...d,
                value: [d.value[0], d.value[1], d.value[2]],
              })),
              itemStyle: {
                opacity: 0.8,
              },
              emphasis: {
                itemStyle: {
                  opacity: 1,
                },
              },
            },
          ],
        };

      case '3d_line':
        return {
          ...baseOption,
          series: [
            {
              type: 'line3D',
              data: processedData,
              lineStyle: {
                width: 4,
                opacity: 0.8,
              },
            },
          ],
        };

      default:
        return baseOption;
    }
  }, [processedData, viewType, xAxis, yAxis, zAxis, colorBy, autoRotate, viewAngle, animation]);

  // 初始化图表
  useEffect(() => {
    if (chartRef.current && !loading && !error) {
      chartInstance.current = echarts.init(chartRef.current, undefined, {
        renderer: 'canvas',
      });

      chartInstance.current.setOption(chartOptions, true);

      // 监听点击事件
      chartInstance.current.on('click', (params) => {
        if (params.data?.rawData) {
          onDataPointClick?.(params.data.rawData);
        }
      });

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
  }, [chartOptions, loading, error, onDataPointClick]);

  const handleReset = () => {
    setViewAngle({ alpha: 40, beta: 40 });
    if (chartInstance.current) {
      chartInstance.current.setOption(chartOptions, true);
    }
  };

  const renderControls = () => (
    <Space className="mb-4" wrap>
      <Select
        value={viewType}
        onChange={(value) => setViewType(value)}
        style={{ width: 120 }}
      >
        <Select.Option value="3d_scatter">散点图</Select.Option>
        <Select.Option value="3d_surface">曲面图</Select.Option>
        <Select.Option value="3d_bar">柱状图</Select.Option>
        <Select.Option value="3d_line">线图</Select.Option>
      </Select>

      <Select
        value={xAxis}
        onChange={(value) => setXAxis(value)}
        style={{ width: 100 }}
        placeholder="X轴"
      >
        <Select.Option value="profit">收益</Select.Option>
        <Select.Option value="risk">风险</Select.Option>
        <Select.Option value="volume">交易量</Select.Option>
        <Select.Option value="sharpe">夏普</Select.Option>
      </Select>

      <Select
        value={yAxis}
        onChange={(value) => setYAxis(value)}
        style={{ width: 100 }}
        placeholder="Y轴"
      >
        <Select.Option value="profit">收益</Select.Option>
        <Select.Option value="risk">风险</Select.Option>
        <Select.Option value="volume">交易量</Select.Option>
        <Select.Option value="sharpe">夏普</Select.Option>
      </Select>

      <Select
        value={zAxis}
        onChange={(value) => setZAxis(value)}
        style={{ width: 100 }}
        placeholder="Z轴"
      >
        <Select.Option value="profit">收益</Select.Option>
        <Select.Option value="risk">风险</Select.Option>
        <Select.Option value="volume">交易量</Select.Option>
        <Select.Option value="sharpe">夏普</Select.Option>
      </Select>

      <Select
        value={colorBy}
        onChange={(value) => setColorBy(value)}
        style={{ width: 100 }}
        placeholder="颜色"
      >
        <Select.Option value="profit">收益</Select.Option>
        <Select.Option value="risk">风险</Select.Option>
        <Select.Option value="sharpe">夏普</Select.Option>
        <Select.Option value="winRate">胜率</Select.Option>
      </Select>

      <Space>
        <span className="text-sm text-gray-600">自动旋转:</span>
        <Switch size="small" checked={autoRotate} onChange={setAutoRotate} />
      </Space>

      <Button size="small" icon={<RotateLeftOutlined />} onClick={handleReset}>
        重置视角
      </Button>
    </Space>
  );

  if (error) {
    return (
      <Card title={title} className={className}>
        <Alert
          message="3D性能图表加载失败"
          description={error}
          type="error"
          showIcon
        />
      </Card>
    );
  }

  return (
    <Card 
      title={title}
      className={className}
      extra={
        <Button size="small" icon={<SettingOutlined />}>
          高级设置
        </Button>
      }
    >
      {renderControls()}
      
      <Spin spinning={loading} tip="加载3D性能数据...">
        <div
          ref={chartRef}
          style={{ height: `${height}px`, width: '100%' }}
          className="chart-container bg-gray-50 rounded-lg"
        />
      </Spin>

      <div className="mt-4 text-xs text-gray-500">
        <div>提示: 拖拽旋转视角，滚轮缩放，右键平移</div>
        <div>数据点总数: {data.length} | 当前视图: {viewType}</div>
      </div>
    </Card>
  );
};

export default Performance3D;