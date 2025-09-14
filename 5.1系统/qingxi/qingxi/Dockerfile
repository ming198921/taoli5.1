# qingxi/Dockerfile - 生产环境版本

# --- 构建阶段 ---
FROM rust:latest AS builder
WORKDIR /app

# 安装protobuf编译器
RUN apt-get update && \
    apt-get install -y protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

# 复制项目文件
COPY qingxi/Cargo.toml ./Cargo.toml
COPY qingxi/Cargo.lock ./Cargo.lock
COPY qingxi/src ./src
COPY qingxi/configs ./configs
COPY qingxi/proto ./proto
COPY qingxi/build.rs ./build.rs
COPY qingxi/examples ./examples
COPY qingxi/tests ./tests

# 构建生产版本
RUN cargo build --release

# --- 运行阶段 ---
FROM debian:bookworm-slim
WORKDIR /app

# 安装运行时依赖和curl
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 复制二进制文件
COPY --from=builder /app/target/release/market_data_module ./market_data_module
COPY --from=builder /app/target/release/http_api_demo ./http_api_demo
COPY --from=builder /app/target/release/market_collector_demo ./market_collector_demo
COPY --from=builder /app/target/release/market_data_feed_demo ./market_data_feed_demo
COPY --from=builder /app/target/release/config_validator ./config_validator

# 复制配置文件
COPY qingxi/configs ./configs

# 设置环境变量
ENV RUST_LOG=info
ENV QINGXI_CONFIG_PATH=/app/configs/qingxi.toml

# 暴露端口
EXPOSE 50051
EXPOSE 50061

# 创建非root用户
RUN useradd --create-home --shell /bin/bash qingxi
RUN chown -R qingxi:qingxi /app
USER qingxi

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:50053/health/live || exit 1

# 启动命令
CMD ["./market_data_module"]