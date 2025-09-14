import type { ThemeConfig } from 'antd';

// Ant Design 主题配置
export const theme: ThemeConfig = {
  token: {
    // 主色彩
    colorPrimary: '#1890ff',
    colorSuccess: '#52c41a',
    colorWarning: '#faad14',
    colorError: '#ff4d4f',
    colorInfo: '#1890ff',
    
    // 文本颜色
    colorTextBase: '#000000d9',
    colorTextSecondary: '#00000073',
    colorTextTertiary: '#00000040',
    colorTextQuaternary: '#00000026',
    
    // 背景颜色
    colorBgBase: '#ffffff',
    colorBgContainer: '#ffffff',
    colorBgElevated: '#ffffff',
    colorBgLayout: '#f5f5f5',
    colorBgSpotlight: '#ffffff',
    
    // 边框颜色
    colorBorder: '#d9d9d9',
    colorBorderSecondary: '#f0f0f0',
    
    // 填充颜色
    colorFill: '#f5f5f5',
    colorFillSecondary: '#fafafa',
    colorFillTertiary: '#f5f5f5',
    colorFillQuaternary: '#f0f0f0',
    
    // 尺寸配置
    borderRadius: 6,
    borderRadiusLG: 8,
    borderRadiusSM: 4,
    borderRadiusXS: 2,
    
    // 字体配置
    fontSize: 14,
    fontSizeLG: 16,
    fontSizeSM: 12,
    fontSizeXL: 20,
    fontSizeHeading1: 38,
    fontSizeHeading2: 30,
    fontSizeHeading3: 24,
    fontSizeHeading4: 20,
    fontSizeHeading5: 16,
    
    // 行高
    lineHeight: 1.5714285714285714,
    lineHeightLG: 1.5,
    lineHeightSM: 1.6666666666666667,
    
    // 间距配置
    controlHeight: 32,
    controlHeightSM: 24,
    controlHeightXS: 16,
    controlHeightLG: 40,
    
    // 运动配置
    motionDurationFast: '0.1s',
    motionDurationMid: '0.2s',
    motionDurationSlow: '0.3s',
    
    // 阴影配置
    boxShadow: '0 6px 16px 0 rgba(0, 0, 0, 0.08), 0 3px 6px -4px rgba(0, 0, 0, 0.12), 0 9px 28px 8px rgba(0, 0, 0, 0.05)',
    boxShadowSecondary: '0 6px 16px 0 rgba(0, 0, 0, 0.08), 0 3px 6px -4px rgba(0, 0, 0, 0.12), 0 9px 28px 8px rgba(0, 0, 0, 0.05)',
  },
  
  // 组件配置
  components: {
    // Layout 布局
    Layout: {
      headerBg: '#001529',
      headerColor: 'rgba(255, 255, 255, 0.85)',
      headerHeight: 64,
      siderBg: '#001529',
      bodyBg: '#f0f2f5',
      footerBg: '#f0f2f5',
    },
    
    // Menu 菜单
    Menu: {
      itemBg: 'transparent',
      itemColor: 'rgba(255, 255, 255, 0.85)',
      itemHoverBg: 'rgba(255, 255, 255, 0.08)',
      itemHoverColor: '#ffffff',
      itemSelectedBg: '#1890ff',
      itemSelectedColor: '#ffffff',
      subMenuItemBg: '#000c17',
      groupTitleColor: 'rgba(255, 255, 255, 0.67)',
    },
    
    // Button 按钮
    Button: {
      controlHeight: 32,
      controlHeightLG: 40,
      controlHeightSM: 24,
      primaryColor: '#ffffff',
      colorBgContainer: '#ffffff',
      algorithm: true,
    },
    
    // Table 表格
    Table: {
      headerBg: '#fafafa',
      headerColor: '#000000d9',
      headerSplitColor: '#f0f0f0',
      rowHoverBg: '#f5f5f5',
      borderColor: '#f0f0f0',
      fontSize: 14,
      cellPaddingBlock: 16,
      cellPaddingInline: 16,
    },
    
    // Card 卡片
    Card: {
      headerBg: 'transparent',
      headerFontSize: 16,
      headerFontSizeSM: 14,
      headerHeight: 56,
      headerHeightSM: 36,
    },
    
    // Form 表单
    Form: {
      labelFontSize: 14,
      labelColor: '#000000d9',
      labelColonMarginInlineStart: 2,
      labelColonMarginInlineEnd: 8,
      itemMarginBottom: 24,
    },
    
    // Input 输入框
    Input: {
      controlHeight: 32,
      controlHeightLG: 40,
      controlHeightSM: 24,
      borderRadius: 6,
      colorBorder: '#d9d9d9',
      colorBorderHover: '#40a9ff',
      colorPrimaryBorder: '#1890ff',
      colorPrimaryBorderHover: '#40a9ff',
    },
    
    // Select 选择器
    Select: {
      controlHeight: 32,
      controlHeightLG: 40,
      controlHeightSM: 24,
      borderRadius: 6,
      colorBorder: '#d9d9d9',
      colorBorderHover: '#40a9ff',
      colorPrimaryBorder: '#1890ff',
    },
    
    // DatePicker 日期选择器
    DatePicker: {
      controlHeight: 32,
      controlHeightLG: 40,
      controlHeightSM: 24,
      borderRadius: 6,
    },
    
    // Modal 对话框
    Modal: {
      titleFontSize: 16,
      contentBg: '#ffffff',
      headerBg: '#ffffff',
      footerBg: 'transparent',
    },
    
    // Drawer 抽屉
    Drawer: {
      colorBgElevated: '#ffffff',
      colorBgMask: 'rgba(0, 0, 0, 0.45)',
    },
    
    // Message 消息提示
    Message: {
      contentBg: '#ffffff',
      contentPadding: '10px 16px',
    },
    
    // Notification 通知提醒
    Notification: {
      width: 384,
      paddingContentHorizontal: 24,
    },
    
    // Statistic 统计数值
    Statistic: {
      titleFontSize: 14,
      contentFontSize: 24,
      fontFamily: '"Helvetica Neue", Helvetica, "PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", Arial, sans-serif',
    },
    
    // Progress 进度条
    Progress: {
      defaultColor: '#1890ff',
      remainingColor: 'rgba(0, 0, 0, 0.06)',
      circleTextColor: '#000000d9',
      lineBorderRadius: 100,
    },
    
    // Badge 徽标数
    Badge: {
      indicatorHeight: 20,
      indicatorHeightSM: 16,
      dotSize: 6,
    },
    
    // Tag 标签
    Tag: {
      defaultBg: '#fafafa',
      defaultColor: '#000000d9',
      fontSize: 12,
      lineHeight: 1.5,
    },
    
    // Alert 警告提示
    Alert: {
      defaultPadding: '8px 15px',
      withDescriptionPadding: '15px 15px 15px 64px',
    },
    
    // Spin 加载中
    Spin: {
      contentHeight: 400,
    },
    
    // Skeleton 骨架屏
    Skeleton: {
      color: 'rgba(190, 190, 190, 0.2)',
      colorGradientEnd: 'rgba(190, 190, 190, 0.4)',
    },
  },
  
  // 算法配置
  algorithm: [],
};

// 暗色主题配置
export const darkTheme: ThemeConfig = {
  ...theme,
  token: {
    ...theme.token,
    colorTextBase: '#ffffff',
    colorTextSecondary: 'rgba(255, 255, 255, 0.65)',
    colorTextTertiary: 'rgba(255, 255, 255, 0.45)',
    colorTextQuaternary: 'rgba(255, 255, 255, 0.25)',
    
    colorBgBase: '#141414',
    colorBgContainer: '#1f1f1f',
    colorBgElevated: '#1f1f1f',
    colorBgLayout: '#000000',
    colorBgSpotlight: '#424242',
    
    colorBorder: '#434343',
    colorBorderSecondary: '#303030',
    
    colorFill: 'rgba(255, 255, 255, 0.08)',
    colorFillSecondary: 'rgba(255, 255, 255, 0.04)',
    colorFillTertiary: 'rgba(255, 255, 255, 0.04)',
    colorFillQuaternary: 'rgba(255, 255, 255, 0.02)',
  },
  
  components: {
    ...theme.components,
    Layout: {
      ...theme.components?.Layout,
      headerBg: '#141414',
      siderBg: '#141414',
      bodyBg: '#000000',
      footerBg: '#141414',
    },
    
    Table: {
      ...theme.components?.Table,
      headerBg: '#1f1f1f',
      headerColor: '#ffffff',
      rowHoverBg: 'rgba(255, 255, 255, 0.04)',
      borderColor: '#303030',
    },
  },
};

// 主题切换工具函数
export const getThemeConfig = (isDark: boolean): ThemeConfig => {
  return isDark ? darkTheme : theme;
};