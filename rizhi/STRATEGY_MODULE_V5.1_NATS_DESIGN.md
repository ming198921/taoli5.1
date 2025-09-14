# QingXi 5.1 策略模块开发文档（NATS 传输版，严格对齐5.1总纲）

---

- 适用版本：QingXi 5.1 增强版
- 总纲约束：本开发文档严格遵循《5.1++开发文档》的核心原则与范围，仅支持“跨交易所套利”“三角套利”两类策略；性能目标为亚毫秒级检测路径；全链路可观测、可追溯、可回滚；配置中心统一管理、热加载；运维具备审批与审计。
- 数据传输：未来统一通过 NATS（建议 JetStream）在模块间传输，本文给出主题规范与消息契约。
- 实现语言与热路径原则：Rust；策略静态注册与静态分发（避免动态插件/trait object 热路径）；热路径不经过 NATS，采用内存 SPSC 环/共享内存；NATS 用于跨服务复制、影子/仿真与审计控制。

---

## 1. 模块边界与目标

- 目标
  - 以插件化策略引擎实现两类核心策略：跨交易所套利、三角套利。
  - 策略检测/执行与风控、资金、监控深度联动；检测路径不阻塞，执行路径可降级。
  - 通过 NATS 接入清洗一致性模块输出的“标准化深度50档订单簿/成交”等数据。
  - 提供 Shadow/仿真/实盘三态运行，具备一键回滚与灰度发布能力。

- 非目标
  - 不包含统计套利、做市等非总纲范围的策略。
  - 不直接实现交易所下单（交由交易模块/执行适配器），但输出规范化执行意图。

---

## 2. 总体架构与分层

- Strategies（策略实现层，静态注册）
  - 静态注册 + 静态分发（避免动态插件与 trait object 热路径）；两类策略分别实现；策略开关为编译期/运行期参数控制。

- Strategy Orchestrator（策略编排器）
  - 从 NATS 订阅行情与上下文事件（非热路径）；
  - 驱动市场状态判定与 min_profit 自适应；
  - 对各策略进行优先级调度、熔断与回退；
  - 输出信号到执行层或回到控制面。

- Adapters（适配层）
  - NATS 适配器（订阅/发布、JetStream 消费者）；
  - 风控适配器（阈值/审批/限频/黑白名单）；
  - 资金适配器（余额/限额/撮合约束）；
  - 精度与费率适配器（按交易所/币对动态热加载）。

- Observability（可观测性）
  - Prometheus 指标、tracing 分布式追踪、审计事件（与 `AuditDataType::ArbitrageSignal` 对齐）。

## 2.1 与清洗一致性模块/数据分发契合
- 热路径接口：直接消费 `data_distribution::CleanedMarketData` 与/或 `MarketDataSnapshot`，不经任何序列化/反序列化；
- 对接方式：优先通过同进程调用或共享内存环接入 `DataDistributor::send_to_strategy` 的下游；如需跨服务复制，仅在侧路通过 NATS 复制；
- 零拷贝映射：订单簿采用“自适应深度”的 SOA（price[i], qty[i]）视图，深度由上游 `orderbook_depth_limit`/每源 `channel` 动态驱动（如 20/50/100/200）；策略仅持引用/切片，不做复制；对象池按阶梯（64/128/256）预分配，向上兼容；
- 顺序与重放：按 `sequence` 严格单调，旧序列直接丢弃；`timestamp_ns` 用于审计与迟到检测；
- 质量门控：依据 `quality_score` 与一致性模块输出的 `QualityMetrics` 限制低质量数据进入检测；
- 严禁：在该接口层使用 JSON/serde/simd-json；仅在侧路复制/审计使用。

## 2.2 部署模型与微服务边界
- 微服务边界保持一致：策略与执行虽可同进程/同机部署以满足≤100µs，但接口契约与回执（ACK）与跨机形态完全一致；
- 三种部署：同进程（首选）、同机共享内存（次选）、跨机NATS核心（备选）；
- 回压联动：与清洗/分发的有界队列阈值对齐（参考 `central_manager.event_buffer_size`），策略侧优先“近端优先+丢旧保新+影子降级”。

## 2.3 符号规范与时间同步
- 符号规范：所有输入快照的 `symbol` 必须为“清洗一致性模块”的规范化符号（canonical symbol），别名映射由上游维护；
- Tick/Step/Precision：按交易所/符号维度由上游推送，策略仅只读；
- 时间同步：比较跨所延迟与时序一致性依赖稳定时钟（Chrony/PTP）；策略内部仅使用单调时钟/稳定TSC计时；
- 时钟漂移防护：跨所组合判定时要求参与快照 `timestamp_ns` 之间的差值 ≤ `staleness_max_ns`（可配置）。

---

## 3. NATS 主题规范与消息契约

- 注意：策略检测热路径不经过 NATS；NATS 仅用于跨服务复制/影子仿真/审计与控制面。
- 建议开启 JetStream：为关键主题配置持久化、保留策略与回放能力。

- 主题命名（subject）
  - 行情输入（清洗一致性→策略）：
    - `qx.v5.md.clean.{exchange}.{symbol}.ob50` 深度50档订单簿快照/增量
    - `qx.v5.md.clean.{exchange}.{symbol}.trades` 成交
    - `qx.v5.md.cross.{symbol}.snapshot` 跨所聚合快照（可选）
  - 上下文输入：
    - `qx.v5.ctx.fee.{exchange}` 费率/精度参数更新
    - `qx.v5.ctx.health.{component}` API/模块健康度事件
    - `qx.v5.ctx.state.market` 市场状态变更（常规/谨慎/极端）
  - 策略输出：
    - `qx.v5.strategy.signal` 标准化套利信号（影子/仿真/实盘均可使用）
    - `qx.v5.strategy.exec` 标准化执行意图（供执行模块消费）
    - `qx.v5.exec.ack` 执行回执（accepted/rejected/partial）
    - `qx.v5.strategy.audit` 审计流（备查，不参与实时）
  - 控制与运维：
    - `qx.v5.ctrl.strategy.cmd` 控制命令（启停策略/阈值覆盖/影子模式等）
    - `qx.v5.ctrl.strategy.reply` 命令执行回执

- 消息契约（JSON，强约束，版本化）
  - OrderBook 50 档（输入）
    ```json
    {
      "v": 1,
      "type": "orderbook50",
      "exchange": "binance",
      "symbol": "BTCUSDT",
      "timestamp_ns": 1720000000000000000,
      "sequence": 123456,
      "bids": [[price, qty], ...],
      "asks": [[price, qty], ...],
      "quality_score": 0.99,
      "processing_latency_ns": 800
    }
    ```
  - 策略信号（输出）
    ```json
    {
      "v": 1,
      "signal_id": "uuid",
      "mode": "shadow|paper|live",
      "strategy": "inter_exchange|triangular",
      "symbols": ["BTCUSDT"],
      "legs": [
        {"exchange": "binance", "side": "buy", "price": 60000.1, "qty": 0.5},
        {"exchange": "okx",     "side": "sell", "price": 60010.3, "qty": 0.5}
      ],
      "expected_profit": 5.2,
      "expected_profit_pct": 0.008,
      "min_profit_threshold": 0.006,
      "slippage_budget_pct": 0.002,
      "ttl_ms": 150,
      "created_ts_ns": 1720000000000000000,
      "idempotency_key": "uuid",
      "trace_id": "trace-xxx",
      "tags": {"market_state": "regular", "p99_latency_ns": 900000}
    }
    ```

- 压缩与序列化
  - 默认 JSON；可选 `application/zstd+json`（大体量时启用）。
  - 严格要求 `idempotency_key` 与 `trace_id`，审计与幂等。
  - 数值精度：热路径内部一律采用“定点整数”表示（如 price/qty 为 i64 + per-symbol scale），外部 JSON 仍用浮点表示但保留到交易所精度；入库/审计保留原始精度，不做二次四舍五入。

---

## 4. 核心接口（Rust 伪代码，对齐5.1）

- 策略插件 Trait
  ```rust
  pub trait ArbitrageStrategy: Send + Sync {
      fn name(&self) -> &'static str;
      fn kind(&self) -> StrategyKind; // InterExchange | Triangular
      // CPU-bound, 同步、无分配、可内联
      fn detect(
          &self,
          ctx: &StrategyContext,
          input: &NormalizedSnapshot,
      ) -> Option<ArbitrageOpportunity>;
      // 执行可能涉及IO，保持异步
      async fn execute(
          &self,
          ctx: &StrategyContext,
          opp: &ArbitrageOpportunity,
      ) -> Result<ExecutionResult, StrategyError>;
  }
  ```

- 策略编排器 Orchestrator
  ```rust
  pub struct StrategyOrchestrator { /* NATS consumers, priority queue, state, caches */ }
  impl StrategyOrchestrator {
      pub async fn start(&self) -> Result<(), OrchestratorError> { /* subscribe, run loop */ }
      pub async fn stop(&self) -> Result<(), OrchestratorError> { /* drain, close */ }
  }
  ```

- 策略上下文 Context（只读主链路 + 原子热更新）
  ```rust
  pub struct StrategyContext {
      pub min_profit_cache: Arc<MinProfitModel>,
      pub market_state: AtomicMarketState,          // regular|cautious|extreme
      pub fee_precision_repo: Arc<FeePrecisionRepo>,
      pub health_snapshot: Arc<HealthSnapshot>,
      pub nats: Arc<NatsBus>,                       // pub/sub helpers
      pub clocks: Arc<Clocks>,                      // time sync utilities
      pub metrics: Arc<StrategyMetrics>,
      pub risk: Arc<RiskAdapter>,
      pub funds: Arc<FundsAdapter>,
  }
  ```

- 机会/执行意图
  ```rust
  pub struct ArbitrageOpportunity { /* legs, expected_profit, budgets, ttl, trace_id, etc. */ }
  pub struct ExecutionResult { pub accepted: bool, pub reason: Option<String>, pub order_ids: Vec<String> }
  ```

---

## 5. 运行时流程（亚毫秒检测主链路）

1) Intake：清洗→策略同节点内内存 SPSC/共享内存直通（不经过 NATS）→ 标准化 `NormalizedSnapshot`；
2) Context 更新：费率/精度/健康度/市场状态通过独立消费者异步刷新，采用 RCU/版本号原子切换；
3) 市场状态 + min_profit：
   - 市场状态 = 多指标加权（波动率、深度、成交、API健康度）→ 常规/谨慎/极端；
   - `min_profit` = 基础阈值 × 状态权重 × 学习反馈（对齐5.1 2.5节）；
4) 检测：在同核分片上以同步、无锁、无分配的 `detect` 快速判定；必要时并行在多核上对不同符号分片运行；
5) 风控快速预筛（策略侧最小子集）：黑白名单、单笔/总额限额、频率限流、基本风险敞口（同步常量时间检查）；注意：权威风控与复杂约束由独立风控服务负责，策略仅做前置筛查避免无效信号；
6) 输出：本地执行路径直达执行模块；并行发布影子/审计到 NATS（非热路径）；
7) 审计：异步写入 `strategy.audit`，并落盘/对象存储（与 `AuditData` 对齐）。

### 5.1 执行交付与确认路径（Strategy→Order Executor）
- 首选（同进程）: 策略与执行在同一进程/线程池内，调用执行接口（零拷贝/零IPC）。
- 次选（同节点）: 通过共享内存无锁环（SPSC/MPMC，`memmap2` + 自旋/事件通知）传递 `ExecutionIntent`；ACK 走同环返还；
- 备选（跨节点）: 使用 NATS 核心（非 JetStream）传输执行意图 `qx.v5.strategy.exec`，同步接收 ACK `qx.v5.exec.ack`；开启直连、禁用持久化、专用 subject、最小载荷（bincode/定长结构）。
- SLO 约束：
  - 同进程/同节点：执行交付≤50µs；
  - 跨节点：传输≤300µs（取决于网络RTT），策略检测SLO不受影响，整体端到端SLO单独评估；
- 幂等与截止：所有意图携带 `idempotency_key` 与 `deadline_ns`；过期意图丢弃并审计；
- 回执：执行模块必须在 `deadline_ns` 前通过本地环或 `qx.v5.exec.ack` 返回 `accepted|rejected|partial` 与原因码；


---

## 6. 性能目标与工程约束

- 目标
  - 策略检测路径：平均 ≤ 100µs，P99 ≤ 200µs（单币种单次机会判定，且不损失精度）；
  - 同节点内（清洗→检测）端到端：平均 ≤ 300µs，P99 ≤ 600µs；
  - 跨服务经 NATS 的复制/审计链路不在热路径，延迟单独评估；
  - 4 交易所 × 每所 ≥30 币种 × 深度50 档并发负载保持稳定。

- 工程实践
  - 零拷贝、SIMD、结构体紧凑布局（避免堆分配与Box/Arc克隆）；
  - 热路径无锁：使用无锁 SPSC/MPMC 环形缓冲（每符号/每核心分片）；避免 RwLock；
  - CPU 亲和与分片：I/O 核与计算核分离；每核心绑定固定符号分片；NUMA 本地化与缓存行填充；
  - 内存：预分配 + 对象池 + slab allocator；
  - NATS JetStream：仅用于非热路径（复制/影子/审计/控制）；`pull` 批量、合理 `ack_wait`；
  - 背压：有界队列+优先近端策略；超阈值降级为影子模式或暂停低优先级符号。

### 6.1 热路径禁令清单（确保≤100µs）
- 禁止：动态内存分配（含 Vec push/HashMap 扩容）、trait object 动态分发、任何日志/字符串格式化、chrono/date 处理、阻塞/自旋锁、跨核迁移、跨 NUMA 访问、serde/simd-json 编码解码、NATS/网络调用。
- 时间源：仅使用单调时钟（u64 纳秒）或稳定 TSC；禁止系统时间与时区换算。
- 分支：避免不可预测分支；采用掩码/branchless 方式。
- 内联：检测函数 `detect` 标记 `#[inline(always)]`，并启用 `-C target-cpu=native -C lto=fat -C codegen-units=1`。

### 6.2 准确度与确定性约束（不牺牲精度）
- 定点整数：价格/数量统一为定点整数（i64/i128 + 每符号 scale）；内部累加使用 i128，溢出使用饱和算术，输出阶段按交易所精度舍入。
- 费用/滑点：检测中严格扣除费率、滑点预算、最小下单量与步进；不得使用启发式近似；
- 可成交量：对 50 档逐级累加验证目标数量的可成交性，并计算精确加权成本/收益；
- 价格精度与步进：所有比较均在定点域进行；输出前统一按交易所 tick size/step size 校正；
- 决策确定性：同利润并列时，按固定优先级（交易所→符号→时间戳→序号）稳定决定；
- 审计可重放：记录输入快照哈希、阈值版本、费率版本与决策路径摘要，实现 bit-for-bit 重放。

### 6.3 算法与数据结构建议（50档、4所）
- 固定容量数组：每符号维护 bids[50]/asks[50] 的紧凑结构（SOA：价格数组、数量数组分离），缓存行对齐；
- 增量更新：清洗模块提供已排序增量，策略模块仅 O(k) 局部更新，无全表重排；
- 交叉对比：4 所双向 12 组合，使用向量化/手动展开扫描，常量时间上界；
- 三角套利：3 腿价格链乘除在定点域完成，预先校正精度与费率；
- 分片并行：按符号哈希到核心，核心内 SPSC/MPMC；禁止跨核处理同一符号；
- 预取与对齐：手动 `prefetch_read` 上下两级缓存，结构体填充至 64B 对齐避免伪共享。

### 6.4 基准与观测
- 微基准：使用 `criterion` 离线模式或自研 harness；统计均值/P99/P999；
- 采样观测：热路径仅用 `Relaxed` 原子计数器；直方图/摘要在侧路聚合；
- 编译参数：`RUSTFLAGS="-C target-cpu=native -C lto=fat -C codegen-units=1 -C target-feature=+avx2,+fma"`；可选 PGO。

---

## 7. 配置与热加载（TOML 草案）

```toml
[strategy]
enabled = ["inter_exchange", "triangular"]
mode = "shadow"            # shadow|paper|live
min_profit_base_pct = 0.005 # 0.5%
min_profit_dynamic = true
slippage_budget_pct = 0.002
max_parallel_detection = 4
signal_ttl_ms = 150

[strategy.weights]
profit = 0.5
liquidity = 0.2
ttl = 0.1
historical_fill = 0.2

[strategy.nats]
servers = ["nats://nats:4222"]
jetstream = true
subjects_md_ob = "qx.v5.md.clean.*.*.ob50"
subject_signal = "qx.v5.strategy.signal"
subject_exec = "qx.v5.strategy.exec"
subject_ctrl_cmd = "qx.v5.ctrl.strategy.cmd"
subject_ctrl_reply = "qx.v5.ctrl.strategy.reply"

[strategy.market_state]
regular_weight = 1.0
cautious_weight = 1.4
extreme_weight = 2.5
hysteresis_window_sec = 300

[strategy.risk]
max_notional_per_signal = 50000
max_open_signals = 50
rate_limit_per_symbol_per_sec = 10
``` 

- 配置源：etcd/Consul/ConfigMap 均可；
- 热加载：`notify`/版本号热切；失败回退到上一份有效快照；
- 参数变更全量审计与审批（对齐5.1 运维与审计要求）。

---

## 8. 可观测性与审计

- 指标（Prometheus）
  - `strategy_detect_latency_ns_bucket{strategy=...,symbol=...}`
  - `strategy_signals_total{strategy=...,mode=...}`
  - `strategy_opportunities_filtered_total{reason=...}`
  - `strategy_min_profit_current{state=...}`
  - `nats_consumer_lag{subject=...}`
  - `risk_block_total{reason=...}`

- 日志与追踪
  - `tracing` 结构化日志 + OTLP；关键链路附 `trace_id`；
  - 影子/仿真/实盘标识必须输出。

- 审计
  - 与 `AuditDataType::ArbitrageSignal` 对齐，记录信号、上下文参数、决策路径摘要；
  - 审计不可篡改（WORM 存储/对象存储 + Hash 链）。

---

## 9. 测试与验收（与5.1总纲对齐）

- 单元测试
  - min_profit 自适应边界；市场状态切换的迟滞；精度与费率加载；
  - 机会检测正确性（多所、多币、不同深度/滑点）。

- 集成测试
  - NATS 模拟 4 所 × 30 币 × ob50 并发；
  - P50/P95/P99 延迟统计达标；
  - 风控/限流/黑白名单联动；
  - 影子→仿真→实盘切换正确性与回滚。

- 压力与故障注入
  - NATS 消费积压、消息乱序/重复、部分交易所断连；
  - 市场极端波动（大跳变、深度骤降、成交突发）；
  - API 健康度恶化自动进入谨慎/极端状态；
  - 验收阈值：检测均值<1ms，P99<3ms；端到端 P99<5ms；信号正确率 ≥ 99.9%。

---

## 10. 失败模式与降级策略

- 数据缺失或延迟：回退使用最近有效快照（带 TTL），或跳过该符号；
- 交易所异常：熔断该所的腿，降级为影子模式；
- 风险超限：丢弃或缩量并告警；
- 消费积压：优先处理近端消息，旧消息直接丢弃或压缩处理；
- 参数异常：回滚到上一版本配置；
- 监控异常：告警聚合与抑制，避免雪崩。

---

## 11. 上线与回滚

- 灰度：按交易所/币种/模式（shadow→paper→live）逐步放量；
- 快速回滚：任意异常 1 键切回影子模式；
- 审批：策略启停、阈值覆盖、模式切换均需审批与审计；
- 变更窗口：非极端行情优先，设定熔断保护。

---

## 12. 交付物与对接

- 代码结构建议（与仓库对齐，不强行变更现有架构）：
  - `strategy/traits.rs`：`ArbitrageStrategy`、`StrategyContext`、`ArbitrageOpportunity` 等；
  - `strategy/orchestrator.rs`：编排与调度；
  - `strategy/plugins/inter_exchange.rs`、`strategy/plugins/triangular.rs`；
  - `strategy/adapters/{nats,risk,funds,fees}.rs`；
  - `strategy/min_profit.rs`、`strategy/market_state.rs`；
  - `strategy/metrics.rs`、`strategy/audit.rs`；
  - `configs/strategy.toml`（上文草案）。

- 与现有模块的契合点
  - 输入契合 `data_distribution::CleanedMarketData`/`MarketDataSnapshot`；
  - 审计契合 `AuditDataType::ArbitrageSignal`；
  - 健康度、清洗一致性模块维持现有职责；策略模块不回写清洗层。

---

## 13. 附录：最小可用实现（MVP）范围

- NATS 直连（无 JetStream）+ 单策略（跨所套利）+ 影子模式；
- 固定 min_profit + 手动市场状态；
- 指标与审计基础能力；
- 4 所 × 10 币 × ob50 的 POC 性能证明（均值<1ms，P99<3ms）。

---

本开发文档确保不偏离《5.1++开发文档》的总纲：策略范围限定、性能目标为≤100µs（检测路径）、min_profit 与市场状态自适应、风控联动、NATS 非热路径统一传输、全链路可观测、审计与审批回滚齐备。后续实现阶段可按“最小可用→功能强化→极端鲁棒→自动化回归”路线推进。 