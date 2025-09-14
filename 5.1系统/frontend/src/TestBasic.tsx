import React, { useEffect } from 'react';

function TestBasic() {
  useEffect(() => {
    // 隐藏加载屏幕
    const hideLoadingScreen = () => {
      const loadingScreen = document.getElementById('loading-screen');
      if (loadingScreen) {
        loadingScreen.classList.add('hidden');
        setTimeout(() => {
          loadingScreen.style.display = 'none';
        }, 300);
      }
    };
    
    hideLoadingScreen();
  }, []);

  return (
    <div style={{ padding: '20px', textAlign: 'center' }}>
      <h1>5.1套利系统测试页面</h1>
      <p>如果你看到这个页面，说明React基础功能正常</p>
      <button onClick={() => alert('点击测试成功!')}>
        点击测试
      </button>
    </div>
  );
}

export default TestBasic;