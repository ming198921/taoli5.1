#!/usr/bin/env python3
import http.server
import socketserver
import json
import urllib.request
import os
from datetime import datetime

PORT = 8080
API_BASE = "http://localhost:50061"

class APIProxyHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path.startswith('/api/'):
            # 代理API请求
            try:
                api_url = API_BASE + self.path
                with urllib.request.urlopen(api_url) as response:
                    data = response.read()
                    self.send_response(200)
                    self.send_header('Content-Type', 'application/json')
                    self.send_header('Access-Control-Allow-Origin', '*')
                    self.end_headers()
                    self.wfile.write(data)
            except Exception as e:
                # 如果API服务器不可用，返回模拟数据
                self.send_response(200)
                self.send_header('Content-Type', 'application/json')
                self.send_header('Access-Control-Allow-Origin', '*')
                self.end_headers()
                
                mock_data = self.get_mock_data(self.path)
                self.wfile.write(json.dumps(mock_data).encode())
        else:
            # 服务静态文件
            super().do_GET()
    
    def get_mock_data(self, path):
        timestamp = int(datetime.now().timestamp() * 1000)
        
        if path == '/api/v1/health':
            return {
                "status": "ok",
                "timestamp": timestamp,
                "uptime": 3600,
                "version": "1.0.0"
            }
        elif path == '/api/v1/exchanges':
            return {
                "exchanges": ["binance", "okx", "huobi", "bybit", "gateio"],
                "count": 5,
                "timestamp": timestamp
            }
        elif path == '/api/v1/symbols':
            return {
                "symbols": ["BTC/USDT", "ETH/USDT", "BNB/USDT", "SOL/USDT", "ADA/USDT"],
                "count": 5,
                "timestamp": timestamp
            }
        elif path == '/api/v1/stats':
            return {
                "total_messages": 125430,
                "messages_per_second": 45.2,
                "active_connections": 4,
                "uptime_hours": 72.5,
                "memory_usage_mb": 256.8,
                "timestamp": timestamp
            }
        elif path.startswith('/api/v1/orderbook'):
            return {
                "symbol": "BTC/USDT",
                "exchange": "binance",
                "bids": [
                    ["67450.00", "0.15420"],
                    ["67449.50", "0.28340"],
                    ["67449.00", "0.45670"],
                    ["67448.50", "0.12890"],
                    ["67448.00", "0.67890"]
                ],
                "asks": [
                    ["67450.50", "0.23450"],
                    ["67451.00", "0.34560"],
                    ["67451.50", "0.45670"],
                    ["67452.00", "0.56780"],
                    ["67452.50", "0.67890"]
                ],
                "timestamp": timestamp
            }
        else:
            return {"error": "Not found"}

if __name__ == "__main__":
    os.chdir('/home/devbox/project/qingxi_clean_8bd559a/qingxi/frontend')
    with socketserver.TCPServer(("", PORT), APIProxyHandler) as httpd:
        print(f"前端服务器启动在 http://localhost:{PORT}")
        print(f"API代理服务: {API_BASE}")
        httpd.serve_forever()
