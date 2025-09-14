import { useNavigate, useLocation } from 'react-router-dom';
import { cn } from '../utils/helpers.js';

const Navigation = () => {
  const navigate = useNavigate();
  const location = useLocation();
  
  const navigationItems = [
    { path: '/system', label: '系统控制' },
    { path: '/qingxi', label: '清洗模块' },
    { path: '/celue', label: '策略模块' },
    { path: '/risk', label: '风险管理' },
    { path: '/architecture', label: '架构监控' },
    { path: '/observability', label: '可观测性' },
  ];

  return (
    <div className="bg-white shadow-sm border-b mb-6">
      <div className="max-w-7xl mx-auto px-6">
        <div className="flex items-center justify-between h-16">
          <div>
            <h1 className="text-xl font-semibold text-gray-900">5.1套利系统</h1>
          </div>
          <nav className="flex space-x-1">
            {navigationItems.map((item) => (
              <button
                key={item.path}
                onClick={() => navigate(item.path)}
                className={cn(
                  'px-4 py-2 text-sm font-medium rounded-lg transition-colors',
                  location.pathname === item.path
                    ? 'text-white bg-blue-600'
                    : 'text-gray-700 hover:text-gray-900 hover:bg-gray-100'
                )}
              >
                {item.label}
              </button>
            ))}
          </nav>
        </div>
      </div>
    </div>
  );
};

export default Navigation;