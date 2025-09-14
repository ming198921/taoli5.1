import React, { useEffect, useRef, useMemo } from 'react';
import * as echarts from 'echarts';
import { Card, Spin, Alert, Space, Tag, Tooltip } from 'antd';
import { InfoCircleOutlined } from '@ant-design/icons';

interface SankeyNode {
  name: string;
  value?: number;
  depth?: number;
  itemStyle?: {
    color?: string;
  };
  label?: {
    show?: boolean;
    position?: string;
  };
}

interface SankeyLink {
  source: string;
  target: string;
  value: number;
  label?: {
    show?: boolean;
    formatter?: string;
  };
  lineStyle?: {
    color?: string;
    opacity?: number;
  };
}

interface SankeyChartProps {
  title?: string;
  nodes: SankeyNode[];
  links: SankeyLink[];
  loading?: boolean;
  error?: string;
  height?: number;
  showValues?: boolean;
  colorScheme?: 'default' | 'profit' | 'risk' | 'custom';
  orientation?: 'horizontal' | 'vertical';
  className?: string;
}

export const SankeyChart: React.FC<SankeyChartProps> = ({
  title = '资金流向图',
  nodes,
  links,
  loading = false,
  error,
  height = 500,
  showValues = true,
  colorScheme = 'default',
  orientation = 'horizontal',
  className,
}) => {
  const chartRef = useRef<HTMLDivElement>(null);
  const chartInstance = useRef<echarts.ECharts>();

  // 颜色配置
  const colorSchemes = useMemo(() => ({
    default: ['#5470c6', '#91cc75', '#fac858', '#ee6666', '#73c0de', '#3ba272', '#fc8452', '#9a60b4'],
    profit: ['#52c41a', '#73d13d', '#95de64', '#b7eb8f', '#d9f7be', '#f6ffed'],
    risk: ['#ff4d4f', '#ff7875', '#ffadd2', '#ffc1cc', '#ffd4d4', '#fff2f0'],
    custom: ['#1890ff', '#722ed1', '#eb2f96', '#fa541c', '#faad14', '#13c2c2', '#52c41a', '#fa8c16'],
  }), []);

  // 处理节点数据，添加颜色和样式
  const processedNodes = useMemo(() => {
    const colors = colorSchemes[colorScheme];
    
    return nodes.map((node, index) => ({
      ...node,
      itemStyle: {
        color: colors[index % colors.length],
        ...node.itemStyle,
      },
      label: {
        show: true,
        position: orientation === 'horizontal' ? 'right' : 'bottom',
        formatter: showValues && node.value 
          ? `{b}\n{c|${node.value.toLocaleString()}}`
          : '{b}',
        rich: {
          c: {
            fontSize: 10,
            color: '#666',
            lineHeight: 20,
          },
        },
        ...node.label,
      },
    }));
  }, [nodes, colorScheme, colorSchemes, showValues, orientation]);

  // 处理连接数据，添加样式
  const processedLinks = useMemo(() => {
    const getFlowColor = (value: number, maxValue: number) => {
      const intensity = value / maxValue;
      if (colorScheme === 'profit') {
        return `rgba(82, 196, 26, ${0.3 + intensity * 0.7})`;
      } else if (colorScheme === 'risk') {
        return `rgba(255, 77, 79, ${0.3 + intensity * 0.7})`;
      }
      return `rgba(84, 112, 198, ${0.3 + intensity * 0.7})`;
    };

    const maxValue = Math.max(...links.map(link => link.value));

    return links.map(link => ({
      ...link,
      lineStyle: {
        color: getFlowColor(link.value, maxValue),
        curveness: 0.5,
        ...link.lineStyle,
      },
      label: {
        show: showValues,
        formatter: `{c|${link.value.toLocaleString()}}`,
        rich: {
          c: {
            fontSize: 10,
            color: '#333',
            backgroundColor: 'rgba(255, 255, 255, 0.8)',
            borderRadius: 3,
            padding: [2, 4],
          },
        },
        ...link.label,
      },
    }));
  }, [links, showValues, colorScheme]);

  // 图表配置
  const chartOptions = useMemo((): echarts.EChartsOption => {
    return {
      tooltip: {
        trigger: 'item',
        triggerOn: 'mousemove',
        backgroundColor: 'rgba(0, 0, 0, 0.8)',
        borderColor: '#333',
        textStyle: {
          color: '#fff',
          fontSize: 12,
        },
        formatter: function (params: any) {
          if (params.dataType === 'node') {
            return `
              <div style="margin-bottom: 4px; font-weight: bold;">${params.name}</div>
              ${params.value !== undefined ? 
                `<div>值: ${params.value.toLocaleString()}</div>` : 
                ''
              }
            `;
          } else if (params.dataType === 'edge') {
            return `
              <div style="margin-bottom: 4px;">
                <strong>${params.source}</strong> → <strong>${params.target}</strong>
              </div>
              <div>流量: ${params.value.toLocaleString()}</div>
            `;
          }
          return '';
        },
      },
      series: [
        {
          type: 'sankey',
          data: processedNodes,
          links: processedLinks,
          orient: orientation,
          layout: 'none',
          emphasis: {
            focus: 'adjacency',
          },
          levels: [
            {
              depth: 0,
              itemStyle: {
                color: '#fbb4ae',
              },
              lineStyle: {
                color: 'source',
                opacity: 0.6,
              },
            },
            {
              depth: 1,
              itemStyle: {
                color: '#b3cde3',
              },
              lineStyle: {
                color: 'source',
                opacity: 0.6,
              },
            },
            {
              depth: 2,
              itemStyle: {
                color: '#ccebc5',
              },
              lineStyle: {
                color: 'source',
                opacity: 0.6,
              },
            },
          ],
          lineStyle: {
            curveness: 0.5,
          },
          nodeWidth: 20,
          nodeGap: 8,
          layoutIterations: 32,
          label: {
            show: true,
            fontSize: 12,
            fontWeight: 'normal',
            color: '#333',
          },
        },
      ],
      animationDuration: 1000,
      animationEasing: 'cubicInOut',
    };
  }, [processedNodes, processedLinks, orientation]);

  // 初始化图表
  useEffect(() => {
    if (chartRef.current && !loading && !error) {
      chartInstance.current = echarts.init(chartRef.current);
      chartInstance.current.setOption(chartOptions, true);

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
  }, [chartOptions, loading, error]);

  // 计算统计信息
  const statistics = useMemo(() => {
    const totalValue = links.reduce((sum, link) => sum + link.value, 0);
    const nodeCount = nodes.length;
    const flowCount = links.length;
    const maxFlow = Math.max(...links.map(link => link.value));
    const minFlow = Math.min(...links.map(link => link.value));

    return {
      totalValue,
      nodeCount,
      flowCount,
      maxFlow,
      minFlow,
      avgFlow: totalValue / flowCount,
    };
  }, [nodes, links]);

  const renderStatistics = () => (
    <div className="mb-4 p-3 bg-gray-50 rounded-lg">
      <div className="flex flex-wrap gap-2">
        <Tag color="blue">节点数: {statistics.nodeCount}</Tag>
        <Tag color="green">连接数: {statistics.flowCount}</Tag>
        <Tag color="orange">总流量: {statistics.totalValue.toLocaleString()}</Tag>
        <Tag color="purple">最大流: {statistics.maxFlow.toLocaleString()}</Tag>
        <Tag color="cyan">最小流: {statistics.minFlow.toLocaleString()}</Tag>
        <Tag color="geekblue">平均流: {statistics.avgFlow.toFixed(0)}</Tag>
      </div>
    </div>
  );

  if (error) {
    return (
      <Card title={title} className={className}>
        <Alert
          message="Sankey图表加载失败"
          description={error}
          type="error"
          showIcon
        />
      </Card>
    );
  }

  return (
    <Card 
      title={
        <Space>
          {title}
          <Tooltip title="Sankey图显示资金在不同账户、交易所和策略之间的流动情况">
            <InfoCircleOutlined className="text-gray-400" />
          </Tooltip>
        </Space>
      }
      className={className}
    >
      {renderStatistics()}
      
      <Spin spinning={loading} tip="加载资金流向数据...">
        <div
          ref={chartRef}
          style={{ height: `${height}px`, width: '100%' }}
          className="chart-container"
        />
      </Spin>
    </Card>
  );
};

export default SankeyChart;