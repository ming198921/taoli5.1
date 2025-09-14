import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import 'antd/dist/reset.css';

console.log('5.1套利系统启动中...');

const rootElement = document.getElementById('root');

if (rootElement) {
  console.log('✅ 找到root元素，开始渲染...');
  const root = ReactDOM.createRoot(rootElement);
  root.render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
  console.log('✅ React应用已挂载');
} else {
  console.error('❌ 未找到root元素');
}
if (rootElement) {
  console.log('✅ 找到root元素，开始渲染...');
  const root = ReactDOM.createRoot(rootElement);
  root.render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
  console.log('✅ React应用已挂载');
} else {
  console.error('❌ 未找到root元素');
}