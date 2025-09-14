export default function TestApp() {
  console.log('TestApp rendering');
  
  return (
    <div style={{ 
      padding: '50px', 
      fontSize: '20px', 
      color: '#333',
      background: '#f0f0f0',
      minHeight: '100vh'
    }}>
      <h1>5.1套利系统 - 测试页面</h1>
      <p>如果你能看到这个页面，说明React应用已经成功运行！</p>
      <p>当前时间: {new Date().toLocaleString()}</p>
      <div style={{ marginTop: '20px', padding: '20px', background: '#fff', border: '1px solid #ddd' }}>
        <h3>系统状态检查:</h3>
        <p>✅ React 应用已加载</p>
        <p>✅ JavaScript 执行正常</p>
        <p>✅ 页面渲染成功</p>
      </div>
    </div>
  );
}