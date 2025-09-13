#!/usr/bin/env python3

import asyncio
import json
import sys

try:
    import nats
    from nats.errors import ConnectionClosedError, TimeoutError
except ImportError:
    print("请安装nats-py: pip install nats-py")
    sys.exit(1)

async def test_nats_connection():
    print("🔍 测试NATS连接和数据流...")
    
    try:
        # 连接到NATS服务器
        nc = await nats.connect("nats://127.0.0.1:4222")
        print("✅ 成功连接到NATS服务器")
        
        # 创建一个计数器
        message_count = 0
        
        async def message_handler(msg):
            nonlocal message_count
            message_count += 1
            
            print(f"\n📦 消息 #{message_count}")
            print(f"   主题: {msg.subject}")
            print(f"   大小: {len(msg.data)} bytes")
            
            try:
                # 尝试解析为JSON
                data = json.loads(msg.data.decode('utf-8'))
                print(f"   JSON数据: {json.dumps(data, indent=2, ensure_ascii=False)}")
            except:
                # 如果不是JSON，显示原始数据的前100个字符
                raw_data = msg.data.decode('utf-8', errors='ignore')
                print(f"   原始数据: {raw_data[:100]}...")
            
            print("-" * 50)
        
        # 订阅QingXi的主题
        print("📡 订阅主题: qx.v5.md.clean.*.*.*")
        sub1 = await nc.subscribe("qx.v5.md.clean.*.*.*", cb=message_handler)
        
        # 同时订阅桥接器的输出主题
        print("📡 订阅主题: market.data.normalized.*.*")
        sub2 = await nc.subscribe("market.data.normalized.*.*", cb=message_handler)
        
        print("⏳ 等待消息 (30秒)...")
        
        # 等待30秒
        await asyncio.sleep(30)
        
        print(f"\n📊 总共接收到 {message_count} 条消息")
        
        if message_count == 0:
            print("❌ 没有接收到任何消息")
            print("可能的原因:")
            print("  1. QingXi系统没有发布数据")
            print("  2. NATS主题名称不匹配")
            print("  3. JetStream配置问题")
        else:
            print("✅ 数据流正常")
            
        await nc.close()
        
    except Exception as e:
        print(f"❌ 错误: {e}")

if __name__ == "__main__":
    asyncio.run(test_nats_connection()) 

import asyncio
import json
import sys

try:
    import nats
    from nats.errors import ConnectionClosedError, TimeoutError
except ImportError:
    print("请安装nats-py: pip install nats-py")
    sys.exit(1)

async def test_nats_connection():
    print("🔍 测试NATS连接和数据流...")
    
    try:
        # 连接到NATS服务器
        nc = await nats.connect("nats://127.0.0.1:4222")
        print("✅ 成功连接到NATS服务器")
        
        # 创建一个计数器
        message_count = 0
        
        async def message_handler(msg):
            nonlocal message_count
            message_count += 1
            
            print(f"\n📦 消息 #{message_count}")
            print(f"   主题: {msg.subject}")
            print(f"   大小: {len(msg.data)} bytes")
            
            try:
                # 尝试解析为JSON
                data = json.loads(msg.data.decode('utf-8'))
                print(f"   JSON数据: {json.dumps(data, indent=2, ensure_ascii=False)}")
            except:
                # 如果不是JSON，显示原始数据的前100个字符
                raw_data = msg.data.decode('utf-8', errors='ignore')
                print(f"   原始数据: {raw_data[:100]}...")
            
            print("-" * 50)
        
        # 订阅QingXi的主题
        print("📡 订阅主题: qx.v5.md.clean.*.*.*")
        sub1 = await nc.subscribe("qx.v5.md.clean.*.*.*", cb=message_handler)
        
        # 同时订阅桥接器的输出主题
        print("📡 订阅主题: market.data.normalized.*.*")
        sub2 = await nc.subscribe("market.data.normalized.*.*", cb=message_handler)
        
        print("⏳ 等待消息 (30秒)...")
        
        # 等待30秒
        await asyncio.sleep(30)
        
        print(f"\n📊 总共接收到 {message_count} 条消息")
        
        if message_count == 0:
            print("❌ 没有接收到任何消息")
            print("可能的原因:")
            print("  1. QingXi系统没有发布数据")
            print("  2. NATS主题名称不匹配")
            print("  3. JetStream配置问题")
        else:
            print("✅ 数据流正常")
            
        await nc.close()
        
    except Exception as e:
        print(f"❌ 错误: {e}")

if __name__ == "__main__":
    asyncio.run(test_nats_connection()) 

import asyncio
import json
import sys

try:
    import nats
    from nats.errors import ConnectionClosedError, TimeoutError
except ImportError:
    print("请安装nats-py: pip install nats-py")
    sys.exit(1)

async def test_nats_connection():
    print("🔍 测试NATS连接和数据流...")
    
    try:
        # 连接到NATS服务器
        nc = await nats.connect("nats://127.0.0.1:4222")
        print("✅ 成功连接到NATS服务器")
        
        # 创建一个计数器
        message_count = 0
        
        async def message_handler(msg):
            nonlocal message_count
            message_count += 1
            
            print(f"\n📦 消息 #{message_count}")
            print(f"   主题: {msg.subject}")
            print(f"   大小: {len(msg.data)} bytes")
            
            try:
                # 尝试解析为JSON
                data = json.loads(msg.data.decode('utf-8'))
                print(f"   JSON数据: {json.dumps(data, indent=2, ensure_ascii=False)}")
            except:
                # 如果不是JSON，显示原始数据的前100个字符
                raw_data = msg.data.decode('utf-8', errors='ignore')
                print(f"   原始数据: {raw_data[:100]}...")
            
            print("-" * 50)
        
        # 订阅QingXi的主题
        print("📡 订阅主题: qx.v5.md.clean.*.*.*")
        sub1 = await nc.subscribe("qx.v5.md.clean.*.*.*", cb=message_handler)
        
        # 同时订阅桥接器的输出主题
        print("📡 订阅主题: market.data.normalized.*.*")
        sub2 = await nc.subscribe("market.data.normalized.*.*", cb=message_handler)
        
        print("⏳ 等待消息 (30秒)...")
        
        # 等待30秒
        await asyncio.sleep(30)
        
        print(f"\n📊 总共接收到 {message_count} 条消息")
        
        if message_count == 0:
            print("❌ 没有接收到任何消息")
            print("可能的原因:")
            print("  1. QingXi系统没有发布数据")
            print("  2. NATS主题名称不匹配")
            print("  3. JetStream配置问题")
        else:
            print("✅ 数据流正常")
            
        await nc.close()
        
    except Exception as e:
        print(f"❌ 错误: {e}")

if __name__ == "__main__":
    asyncio.run(test_nats_connection()) 
 
 
 