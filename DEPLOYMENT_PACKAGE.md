# 5.1套利系统完整部署包

## 系统架构概览
- **根目录**: `/home/ubuntu/5.1xitong/`
- **核心系统**: `/home/ubuntu/5.1xitong/5.1系统/`
- **QingXi数据模块**: `5.1系统/qingxi/`
- **CeLue策略模块**: `5.1系统/celue/`
- **统一网关**: `5.1系统/unified-gateway/`

## 必须包含的核心文件和目录

### 1. 根目录核心文件
```
/Cargo.toml                        # 根workspace配置
/Cargo.lock                        # 锁定依赖版本
/README.md                         # 系统说明
/QUICK_START_GUIDE.md             # 快速启动指南
/requirements.txt                  # Python依赖
/.gitignore                        # Git忽略规则
```

### 2. Rust核心模块
```
/ultra_low_latency_order_system.rs # 超低延迟订单系统
/5.1系统/                          # 主系统目录
├── Cargo.toml                     # 主workspace配置
├── src/                           # 主系统源码
├── common_types/                  # 通用类型定义
├── qingxi/                        # QingXi数据处理
├── celue/                         # CeLue策略模块
├── unified-gateway/               # 统一网关
└── architecture/                  # 系统架构
```

### 3. 配置和启动脚本
```
/configs/                          # 系统配置
/start-gateway.sh                  # 网关启动
/system-dashboard.sh               # 系统监控
/auto-service-manager.sh           # 服务管理
/check_system_status.sh            # 状态检查
```

### 4. 核心Python脚本
```
/arbitrage-cli-controller.py       # CLI控制器
/enhanced_triangular_arbitrage.py  # 三角套利
/exchange_latency_test.py          # 延迟测试
/test-all-apis.js                  # API测试
```

## 兼容性修复配置

### Socket兼容性处理
在上传前需要修复socket相关代码，创建条件编译版本：

```rust
// ultra_low_latency_order_system.rs 修复方案
#[cfg(all(target_os = "linux", feature = "socket2"))]
use socket2::Socket;
#[cfg(target_os = "linux")]
use std::os::unix::io::AsRawFd;

// 条件编译的socket优化
#[cfg(all(target_os = "linux", feature = "socket2"))]
{
    // 高级socket优化代码
}
#[cfg(not(all(target_os = "linux", feature = "socket2")))]
{
    // 基础优化后备方案
    stream.set_nodelay(true)?;
}
```

### Cargo特性配置
```toml
[features]
default = ["socket2", "tcp_quickack"]
socket2 = []
tcp_quickack = []
full_optimization = ["socket2", "tcp_quickack"]
```

## 部署后验证检查列表

1. **编译检查**: `cargo check --workspace`
2. **功能测试**: `cargo test --workspace`
3. **服务启动**: `./start-gateway.sh`
4. **API测试**: `node test-all-apis.js`
5. **延迟测试**: `python3 exchange_latency_test.py`

## 新服务器必备软件
- Rust 1.70+ (建议nightly)
- Python 3.10+
- Node.js 18+
- Docker & Docker Compose
- Redis (可选)
- 系统依赖: build-essential, pkg-config, libssl-dev

## 快速部署命令
```bash
# 1. 克隆仓库
git clone https://github.com/ming198921/taoli5.1.git
cd taoli5.1

# 2. 安装依赖
cargo build --release

# 3. Python依赖
pip3 install -r requirements.txt

# 4. 启动系统
./start-gateway.sh
```

## 故障排除
- 如果socket编译错误: 使用 `cargo build --no-default-features`
- 如果模块找不到: 检查workspace配置和路径依赖
- 如果端口冲突: 修改配置文件中的端口设置