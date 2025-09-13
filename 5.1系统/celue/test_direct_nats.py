#!/usr/bin/env python3
"""
直接测试NATS数据接收，验证QingXi是否真的在发布数据
"""

import asyncio
import json
import sys
import time

try:
    import nats
except ImportError:
    print("请安装nats-py: pip install nats-py")
    sys.exit(1)

async def test_qingxi_nats():
    print("🔍 直接测试QingXi NATS数据流...")
    
    try:
        # 连接到NATS服务器
        nc = await nats.connect("nats://127.0.0.1:4222")
        print("✅ 成功连接到NATS服务器")
        
        message_count = 0
        
        async def message_handler(msg):
            nonlocal message_count
            message_count += 1
            
            print(f"\n📦 消息 #{message_count}")
            print(f"   主题: {msg.subject}")
            print(f"   大小: {len(msg.data)} bytes")
            print(f"   时间: {time.strftime('%H:%M:%S')}")
            
            try:
                # 尝试解析为JSON
                data = json.loads(msg.data.decode('utf-8'))
                print(f"   交易所: {data.get('exchange', 'unknown')}")
                print(f"   币种: {data.get('symbol', 'unknown')}")
                print(f"   买单数量: {len(data.get('bids', []))}")
                print(f"   卖单数量: {len(data.get('asks', []))}")
                if data.get('bids'):
                    print(f"   最佳买价: {data['bids'][0][0]}")
                if data.get('asks'):
                    print(f"   最佳卖价: {data['asks'][0][0]}")
                print(f"   质量分数: {data.get('quality_score', 'unknown')}")
            except Exception as e:
                print(f"   解析错误: {e}")
                # 显示原始数据的前200个字符
                raw_data = msg.data.decode('utf-8', errors='ignore')
                print(f"   原始数据: {raw_data[:200]}...")
            
            print("-" * 60)
        
        # 测试多种主题模式
        patterns = [
            "qx.v5.md.clean.*.*.*",
            "qx.v5.md.clean.>",
            "qx.v5.md.clean.huobi.>",
            "qx.v5.md.clean.binance.>",
        ]
        
        for pattern in patterns:
            print(f"📡 订阅主题: {pattern}")
            await nc.subscribe(pattern, cb=message_handler)
        
        print("⏳ 等待消息 (30秒)...")
        await asyncio.sleep(30)
        
        print(f"\n📊 总共接收到 {message_count} 条消息")
        
        if message_count == 0:
            print("❌ 没有接收到任何消息！")
            print("\n🔧 可能的问题:")
            print("  1. QingXi使用JetStream发布，普通NATS订阅无法接收")
            print("  2. 主题模式不匹配")
            print("  3. NATS服务器配置问题")
        else:
            print("✅ 成功接收到QingXi数据!")
            
        await nc.close()
        
    except Exception as e:
        print(f"❌ 错误: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    asyncio.run(test_qingxi_nats()) 
"""
直接测试NATS数据接收，验证QingXi是否真的在发布数据
"""

import asyncio
import json
import sys
import time

try:
    import nats
except ImportError:
    print("请安装nats-py: pip install nats-py")
    sys.exit(1)

async def test_qingxi_nats():
    print("🔍 直接测试QingXi NATS数据流...")
    
    try:
        # 连接到NATS服务器
        nc = await nats.connect("nats://127.0.0.1:4222")
        print("✅ 成功连接到NATS服务器")
        
        message_count = 0
        
        async def message_handler(msg):
            nonlocal message_count
            message_count += 1
            
            print(f"\n📦 消息 #{message_count}")
            print(f"   主题: {msg.subject}")
            print(f"   大小: {len(msg.data)} bytes")
            print(f"   时间: {time.strftime('%H:%M:%S')}")
            
            try:
                # 尝试解析为JSON
                data = json.loads(msg.data.decode('utf-8'))
                print(f"   交易所: {data.get('exchange', 'unknown')}")
                print(f"   币种: {data.get('symbol', 'unknown')}")
                print(f"   买单数量: {len(data.get('bids', []))}")
                print(f"   卖单数量: {len(data.get('asks', []))}")
                if data.get('bids'):
                    print(f"   最佳买价: {data['bids'][0][0]}")
                if data.get('asks'):
                    print(f"   最佳卖价: {data['asks'][0][0]}")
                print(f"   质量分数: {data.get('quality_score', 'unknown')}")
            except Exception as e:
                print(f"   解析错误: {e}")
                # 显示原始数据的前200个字符
                raw_data = msg.data.decode('utf-8', errors='ignore')
                print(f"   原始数据: {raw_data[:200]}...")
            
            print("-" * 60)
        
        # 测试多种主题模式
        patterns = [
            "qx.v5.md.clean.*.*.*",
            "qx.v5.md.clean.>",
            "qx.v5.md.clean.huobi.>",
            "qx.v5.md.clean.binance.>",
        ]
        
        for pattern in patterns:
            print(f"📡 订阅主题: {pattern}")
            await nc.subscribe(pattern, cb=message_handler)
        
        print("⏳ 等待消息 (30秒)...")
        await asyncio.sleep(30)
        
        print(f"\n📊 总共接收到 {message_count} 条消息")
        
        if message_count == 0:
            print("❌ 没有接收到任何消息！")
            print("\n🔧 可能的问题:")
            print("  1. QingXi使用JetStream发布，普通NATS订阅无法接收")
            print("  2. 主题模式不匹配")
            print("  3. NATS服务器配置问题")
        else:
            print("✅ 成功接收到QingXi数据!")
            
        await nc.close()
        
    except Exception as e:
        print(f"❌ 错误: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    asyncio.run(test_qingxi_nats()) 
"""
直接测试NATS数据接收，验证QingXi是否真的在发布数据
"""

import asyncio
import json
import sys
import time

try:
    import nats
except ImportError:
    print("请安装nats-py: pip install nats-py")
    sys.exit(1)

async def test_qingxi_nats():
    print("🔍 直接测试QingXi NATS数据流...")
    
    try:
        # 连接到NATS服务器
        nc = await nats.connect("nats://127.0.0.1:4222")
        print("✅ 成功连接到NATS服务器")
        
        message_count = 0
        
        async def message_handler(msg):
            nonlocal message_count
            message_count += 1
            
            print(f"\n📦 消息 #{message_count}")
            print(f"   主题: {msg.subject}")
            print(f"   大小: {len(msg.data)} bytes")
            print(f"   时间: {time.strftime('%H:%M:%S')}")
            
            try:
                # 尝试解析为JSON
                data = json.loads(msg.data.decode('utf-8'))
                print(f"   交易所: {data.get('exchange', 'unknown')}")
                print(f"   币种: {data.get('symbol', 'unknown')}")
                print(f"   买单数量: {len(data.get('bids', []))}")
                print(f"   卖单数量: {len(data.get('asks', []))}")
                if data.get('bids'):
                    print(f"   最佳买价: {data['bids'][0][0]}")
                if data.get('asks'):
                    print(f"   最佳卖价: {data['asks'][0][0]}")
                print(f"   质量分数: {data.get('quality_score', 'unknown')}")
            except Exception as e:
                print(f"   解析错误: {e}")
                # 显示原始数据的前200个字符
                raw_data = msg.data.decode('utf-8', errors='ignore')
                print(f"   原始数据: {raw_data[:200]}...")
            
            print("-" * 60)
        
        # 测试多种主题模式
        patterns = [
            "qx.v5.md.clean.*.*.*",
            "qx.v5.md.clean.>",
            "qx.v5.md.clean.huobi.>",
            "qx.v5.md.clean.binance.>",
        ]
        
        for pattern in patterns:
            print(f"📡 订阅主题: {pattern}")
            await nc.subscribe(pattern, cb=message_handler)
        
        print("⏳ 等待消息 (30秒)...")
        await asyncio.sleep(30)
        
        print(f"\n📊 总共接收到 {message_count} 条消息")
        
        if message_count == 0:
            print("❌ 没有接收到任何消息！")
            print("\n🔧 可能的问题:")
            print("  1. QingXi使用JetStream发布，普通NATS订阅无法接收")
            print("  2. 主题模式不匹配")
            print("  3. NATS服务器配置问题")
        else:
            print("✅ 成功接收到QingXi数据!")
            
        await nc.close()
        
    except Exception as e:
        print(f"❌ 错误: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    asyncio.run(test_qingxi_nats()) 
 
 
 