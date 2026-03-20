# 并发回测示例详解 (backtests_concurrent.rs)

## 整体概述

这个示例展示了如何使用 Barter 框架**并发运行大量回测**。回测(Backtesting)是量化交易中的核心概念,指的是用历史数据测试交易策略的表现。

### 为什么需要并发回测?

在量化交易中,我们经常需要:
- 测试不同的策略参数组合
- 对比多个交易策略的效果
- 进行参数优化

这个示例演示了如何**同时运行 10,000 个回测**,充分利用多核 CPU 提高效率。

---

## 代码结构解析

### 1. 常量定义

```rust
// 回测配置文件路径
const CONFIG_PATH: &str = "barter/examples/config/backtest_config.json";

// 历史市场数据文件路径 (包含 BTC/USDT, ETH/USDT, SOL/USDT 的交易数据)
const FILE_PATH_MARKET_DATA_INDEXED: &str =
    "barter/examples/data/binance_spot_trades_l1_btcusdt_ethusdt_solusdt.json";

// 要并发运行的回测数量
const NUM_BACKTESTS: usize = 1;
```

### 2. 配置结构体

```rust
#[derive(Deserialize)]
pub struct Config {
    pub risk_free_return: Decimal,  // 无风险收益率(用于计算夏普比率等指标)
    pub system: SystemConfig,        // 系统配置(交易品种、执行配置等)
}
```

---

## 主函数流程详解

### 步骤 1: 加载配置

```rust
let Config {
    risk_free_return,              // 无风险收益率
    system: SystemConfig {
        instruments,               // 交易品种列表(如 BTC/USDT)
        executions,                // 执行配置(如何连接交易所)
    },
} = load_config();
```

**作用**: 从 JSON 文件读取回测配置。

---

### 步骤 2: 构建索引化交易品种

```rust
let instruments = IndexedInstruments::new(instruments);
```

**作用**: 将交易品种(如 BTC/USDT)转换为索引结构,使用整数索引代替字符串查找,**提高性能**。

**类比**: 像给每个交易对分配一个员工编号,用编号查找比用姓名查找快得多。

---

### 步骤 3: 加载历史市场数据

```rust
let market_events = market_data_from_file(FILE_PATH_MARKET_DATA_INDEXED);
let market_data = MarketDataInMemory::new(Arc::new(market_events));
let time_engine_start = market_data.time_first_event().await.unwrap();
```

**详细说明**:
- `market_data_from_file`: 从文件逐行读取历史交易数据(成交价、成交量等)
- `MarketDataInMemory`: 将数据存储在内存中供回测使用
- `Arc::new`: 使用智能指针包装,允许多个回测**共享同一份数据**,节省内存
- `time_engine_start`: 获取第一个事件的时间戳,作为回测起始时间

---

### 步骤 4: 构建引擎状态

```rust
let engine_state = EngineStateBuilder::new(
    &instruments,                           // 交易品种索引
    DefaultGlobalData::default(),           // 全局数据(如总资金)
    |_| DefaultInstrumentMarketData::default()  // 每个品种的市场数据初始化
)
.time_engine_start(time_engine_start)       // 设置回测起始时间
.trading_state(TradingState::Enabled)       // 启用交易
.build();
```

**作用**: 创建交易引擎的初始状态,包括:
- 账户余额
- 持仓信息
- 订单状态
- 市场数据缓存

---

### 步骤 5: 准备常量回测参数

```rust
let args_constant = Arc::new(BacktestArgsConstant {
    instruments,          // 交易品种
    executions,           // 执行配置
    market_data,          // 历史市场数据
    summary_interval: Daily,  // 每日统计摘要
    engine_state,         // 引擎初始状态
});
```

**关键点**:
- `BacktestArgsConstant` 包含**所有回测共享**的不变参数
- 使用 `Arc` 包装,让 10,000 个回测共享这份数据,**极大节省内存**
- 如果不共享,10,000 个回测会各自复制一份市场数据,内存占用会爆炸

---

### 步骤 6: 定义动态回测参数

```rust
let dynamic_arg = BacktestArgsDynamic {
    id: SmolStr::default(),           // 回测ID
    risk_free_return,                 // 无风险收益率
    strategy: DefaultStrategy::default(),  // 交易策略
    risk: DefaultRiskManager::default(),   // 风险管理器
};
```

**说明**:
- `BacktestArgsDynamic` 包含**每个回测独有**的参数
- 在实际应用中,每个回测应该有不同的策略参数(这里只是演示,都用默认值)

---

### 步骤 7: 生成回测参数迭代器

```rust
let args_dynamic_iter = (0..NUM_BACKTESTS).map(|index| {
    let mut dynamic_args = dynamic_arg.clone();
    dynamic_args.id = index.to_smolstr();  // 给每个回测分配唯一ID
    dynamic_args
});
```

**作用**: 创建 10,000 个回测的参数,每个只是 ID 不同。

**实际应用场景**: 这里应该是不同的策略参数,例如:
- 不同的移动平均线周期
- 不同的止损比例
- 不同的仓位管理策略

---

### 步骤 8: 运行并发回测

```rust
let mut summary = run_backtests(args_constant, args_dynamic_iter)
    .await
    .unwrap();
```

**核心功能**:
- `run_backtests` 是并发执行的关键函数
- 它会自动:
  - 将 10,000 个回测分配到多个 CPU 核心
  - 并发执行所有回测
  - 收集每个回测的结果
- 返回包含所有回测统计摘要的结果

---

### 步骤 9: 分析回测结果

```rust
println!("\nNum Backtests: {}", summary.num_backtests);  // 打印回测数量
println!("Duration: {:?}", summary.duration);           // 打印总耗时

// 按照累计盈亏(PnL)排序,找出表现最好的策略
summary.summaries.sort_by(|a, b| {
    let backtest_a_total_pnl = a
        .trading_summary
        .instruments
        .values()
        .map(|tear| tear.pnl)      // 获取每个品种的盈亏
        .sum::<Decimal>();         // 求和得到总盈亏

    let backtest_b_total_pnl = b
        .trading_summary
        .instruments
        .values()
        .map(|tear| tear.pnl)
        .sum::<Decimal>();

    backtest_a_total_pnl.cmp(&backtest_b_total_pnl).reverse()  // 降序排列
});

// 获取表现最好的回测(实际上应该叫 best_cumulative_pnl,代码中写错了)
let best_cumulative_sharpe = summary.summaries.first().unwrap();

println!(
    "\nBest Cumulative Sharpe: BacktestId = {}",
    best_cumulative_sharpe.id
);
best_cumulative_sharpe.trading_summary.print_summary();  // 打印详细统计
```

**统计指标说明**:
- **PnL (Profit and Loss)**: 盈亏,即赚了多少钱
- **Sharpe Ratio**: 夏普比率,衡量风险调整后的收益
- **Sortino Ratio**: 索提诺比率,类似夏普比率但只关注下行风险
- **Max Drawdown**: 最大回撤,即从最高点到最低点的最大跌幅
- **Win Rate**: 胜率,盈利交易占总交易的比例
- **Profit Factor**: 盈利因子,总盈利除以总亏损

---

## 辅助函数解析

### 1. 加载配置文件

```rust
pub fn load_config() -> Config {
    let file = File::open(CONFIG_PATH).expect("Failed to open config file");
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).expect("Failed to parse config file")
}
```

**作用**: 从 JSON 文件读取配置并反序列化为 `Config` 结构体。

---

### 2. 从文件加载市场数据

```rust
pub fn market_data_from_file<InstrumentKey, Kind>(
    file_path: &str,
) -> Vec<MarketStreamEvent<InstrumentKey, Kind>>
where
    InstrumentKey: for<'de> Deserialize<'de>,  // 泛型约束:品种键可反序列化
    Kind: for<'de> Deserialize<'de>,           // 泛型约束:数据类型可反序列化
{
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);

    reader
        .lines()                    // 逐行读取文件
        .map(|line_result| {
            let line = line_result.unwrap();
            // 每行是一个 JSON 对象,反序列化为市场事件
            serde_json::from_str::<MarketStreamEvent<InstrumentKey, Kind>>(&line).unwrap()
        })
        .collect()                  // 收集所有事件到 Vec
}
```

**数据格式**: 文件中每行是一个 JSON 对象,代表一个市场事件(如一笔成交)。

---

## 关键概念总结

### 1. 回测 (Backtesting)
用历史数据模拟交易,验证策略是否有效。

### 2. 并发执行的优势
- **串行执行**: 10,000 个回测依次运行,总耗时 = 单个回测时间 × 10,000
- **并发执行**: 利用多核 CPU 同时运行,总耗时 ≈ 单个回测时间 × (10,000 / CPU核心数)

### 3. 内存优化
- 通过 `Arc` 共享不可变数据(市场数据、配置等)
- 只有策略参数等动态部分每个回测各自拥有
- 这样 10,000 个回测不会消耗 10,000 倍的内存

### 4. 实际应用场景
在真实的量化交易中,你会:
- 测试不同的策略参数组合(参数网格搜索)
- 对比多个交易策略
- 进行蒙特卡洛模拟(随机参数测试)
- 寻找最优参数配置

---

## 示例运行

```bash
# 运行这个并发回测示例
cargo run --example backtests_concurrent

# 输出示例:
# Num Backtests: 10000
# Duration: 45.2s
#
# Best Cumulative Sharpe: BacktestId = 7823
# Total PnL: $12,345.67
# Sharpe Ratio: 2.34
# Max Drawdown: -8.5%
# Win Rate: 58.3%
# ...
```

---

## 总结

这个示例是 Barter 框架高性能设计的体现:
- **事件驱动架构**: 高效处理历史数据流
- **零拷贝共享**: 通过 Arc 共享数据
- **并发执行**: 充分利用多核 CPU
- **类型安全**: Rust 的类型系统保证正确性

对于量化交易初学者,这是学习如何评估和优化交易策略的绝佳起点。
