// ============================================================================
// 并发回测示例 - 演示如何使用 Barter 框架高效运行大量回测
// ============================================================================
//
// 本示例展示了 Barter 的核心优势:
// - 并发执行 10,000 个回测,充分利用多核 CPU
// - 通过 Arc 共享市场数据,避免内存浪费
// - 自动收集和分析回测结果

use barter::{
    backtest::{
        BacktestArgsConstant, // 所有回测共享的常量参数(市场数据、配置等)
        BacktestArgsDynamic,  // 每个回测独有的动态参数(策略、风险管理等)
        market_data::{BacktestMarketData, MarketDataInMemory}, // 内存中的市场数据
        run_backtests,        // 并发执行多个回测的核心函数
    },
    engine::state::{
        EngineState,                                   // 交易引擎状态(账户余额、持仓、订单等)
        builder::EngineStateBuilder,                   // 状态构建器
        global::DefaultGlobalData,                     // 默认全局数据(如总资金)
        instrument::data::DefaultInstrumentMarketData, // 默认品种市场数据
        trading::TradingState,                         // 交易状态(启用/禁用)
    },
    risk::DefaultRiskManager,     // 默认风险管理器
    statistic::time::Daily,       // 每日统计时间间隔
    strategy::DefaultStrategy,    // 默认交易策略
    system::config::SystemConfig, // 系统配置
};
use barter_data::streams::consumer::MarketStreamEvent; // 市场数据事件
use barter_instrument::index::IndexedInstruments; // 索引化的交易品种
use rust_decimal::Decimal; // 高精度小数(用于金融计算)
use serde::Deserialize; // JSON 反序列化
use smol_str::{SmolStr, ToSmolStr}; // 优化的短字符串
use std::{
    fs::File,
    io::{BufRead, BufReader},
    sync::Arc, // 原子引用计数,用于多线程共享数据
};

// 回测配置文件路径
const CONFIG_PATH: &str = "barter/examples/config/backtest_config.json";

// 历史市场数据文件路径 (包含币安现货 BTC/USDT, ETH/USDT, SOL/USDT 的 L1 交易数据)
const FILE_PATH_MARKET_DATA_INDEXED: &str =
    "barter/examples/data/binance_spot_trades_l1_btcusdt_ethusdt_solusdt.json";

// 并发运行的回测数量 (本示例中所有回测使用相同策略,仅作性能演示)
// 实际应用中应该测试不同的策略参数组合
const NUM_BACKTESTS: usize = 1;

/// 回测配置结构体
#[derive(Deserialize)]
pub struct Config {
    /// 无风险收益率 - 用于计算夏普比率等风险调整后收益指标
    /// 通常使用国债收益率作为无风险利率
    pub risk_free_return: Decimal,

    /// 系统配置 - 包含交易品种、执行配置等
    pub system: SystemConfig,
}

/// 并发回测主函数
///
/// 执行流程:
/// 1. 加载配置和历史市场数据
/// 2. 构建回测所需的引擎状态
/// 3. 准备常量和动态参数
/// 4. 并发运行 10,000 个回测
/// 5. 分析结果并找出最佳策略
#[tokio::main]
async fn main() {
    // ========== 步骤 1: 初始化日志系统 ==========
    // 用于记录回测过程中的调试信息和错误
    barter::logging::init_logging();

    // ========== 步骤 2: 加载配置文件 ==========
    // 从 JSON 文件读取无风险收益率、交易品种、执行配置等
    let Config {
        risk_free_return, // 无风险收益率(用于计算夏普比率)
        system:
            SystemConfig {
                instruments, // 交易品种列表 (如 ["BTC/USDT", "ETH/USDT"])
                executions,  // 执行配置(如何连接到交易所)
            },
    } = load_config();

    // ========== 步骤 3: 构建索引化交易品种 ==========
    // 将字符串形式的交易对(如 "BTC/USDT")转换为整数索引
    // 优势: 用整数索引代替字符串哈希查找,性能提升显著
    let instruments = IndexedInstruments::new(instruments);

    // ========== 步骤 4: 加载历史市场数据 ==========
    // 从文件读取历史交易数据(每行一个 JSON 格式的市场事件)
    let market_events = market_data_from_file(FILE_PATH_MARKET_DATA_INDEXED);

    // 将市场数据包装为 Arc (原子引用计数智能指针)
    // 关键优化: 10,000 个回测共享同一份数据,而不是各自复制一份
    // 这样可以节省大量内存(否则内存占用会是 10,000 倍)
    let market_data = MarketDataInMemory::new(Arc::new(market_events));

    // 获取第一个市场事件的时间戳,作为回测的起始时间
    let time_engine_start = market_data.time_first_event().await.unwrap();

    // ========== 步骤 5: 构建交易引擎初始状态 ==========
    // 创建引擎状态,包含账户余额、持仓、订单等信息
    let engine_state = EngineStateBuilder::new(
        &instruments,                               // 索引化的交易品种
        DefaultGlobalData::default(),               // 全局数据(如总资金、全局配置)
        |_| DefaultInstrumentMarketData::default(), // 为每个品种初始化市场数据
    )
    .time_engine_start(time_engine_start) // 设置引擎启动时间(回测起点)
    .trading_state(TradingState::Enabled) // 启用交易功能
    .build();

    // ========== 步骤 6: 构建常量回测参数 ==========
    // BacktestArgsConstant 包含所有回测共享的不变参数
    // 使用 Arc 包装,让 10,000 个回测共享同一份数据
    let args_constant = Arc::new(BacktestArgsConstant {
        instruments,             // 交易品种索引
        executions,              // 执行配置(如何模拟订单执行)
        market_data,             // 历史市场数据(已用 Arc 包装)
        summary_interval: Daily, // 统计摘要的时间间隔(每日)
        engine_state,            // 引擎初始状态
    });

    // ========== 步骤 7: 定义动态回测参数模板 ==========
    // BacktestArgsDynamic 包含每个回测独有的参数
    // 注意: 本示例中所有回测使用相同的默认策略,仅作并发性能演示
    // 实际应用中应该测试不同的策略参数,例如:
    //   - 不同的移动平均线周期
    //   - 不同的止损/止盈比例
    //   - 不同的仓位管理策略
    let dynamic_arg = BacktestArgsDynamic {
        id: SmolStr::default(),  // 回测唯一标识符
        risk_free_return,        // 无风险收益率
        strategy: DefaultStrategy::<EngineState<DefaultGlobalData, DefaultInstrumentMarketData>>::default(),  // 交易策略
        risk: DefaultRiskManager::<EngineState<DefaultGlobalData, DefaultInstrumentMarketData>>::default(),   // 风险管理器
    };

    // ========== 步骤 8: 生成 10,000 个回测参数 ==========
    // 为每个回测克隆动态参数,并分配唯一 ID
    // 注意: 实际应用中每个回测应该有不同的策略参数!
    let args_dynamic_iter = (0..NUM_BACKTESTS).map(|index| {
        let mut dynamic_args = dynamic_arg.clone();
        dynamic_args.id = index.to_smolstr(); // 设置回测 ID (0, 1, 2, ..., 9999)
        dynamic_args
    });

    // ========== 步骤 9: 并发执行所有回测 ==========
    // run_backtests 会自动:
    //   1. 将回测任务分配到多个 CPU 核心
    //   2. 并发执行所有回测
    //   3. 收集每个回测的统计摘要
    // 性能优势: 总耗时 ≈ 单次回测时间 × (回测数量 / CPU核心数)
    let mut summary = run_backtests(args_constant, args_dynamic_iter)
        .await
        .unwrap();

    // ========== 步骤 10: 分析回测结果 ==========

    // 打印基本统计信息
    println!("\n回测总数: {}", summary.num_backtests);
    println!("总耗时: {:?}", summary.duration);

    // 按累计盈亏(PnL)对所有回测结果排序,找出最佳策略
    summary.summaries.sort_by(|a, b| {
        // 计算回测 A 的总盈亏
        // 遍历所有交易品种,累加每个品种的 PnL
        let backtest_a_total_pnl = a
            .trading_summary
            .instruments
            .values()
            .map(|tear| tear.pnl) // 提取盈亏值
            .sum::<Decimal>(); // 求和得到总盈亏

        // 计算回测 B 的总盈亏
        let backtest_b_total_pnl = b
            .trading_summary
            .instruments
            .values()
            .map(|tear| tear.pnl)
            .sum::<Decimal>();

        // 按降序排列(盈亏最高的排在前面)
        backtest_a_total_pnl.cmp(&backtest_b_total_pnl).reverse()
    });

    // 获取表现最好的回测结果(总盈亏最高)
    // 注意: 变量名写错了,实际上是 best_cumulative_pnl,不是 sharpe
    let best_cumulative_sharpe = summary.summaries.first().unwrap();

    // 打印最佳回测的详细信息
    println!("\n最佳累计盈亏回测: ID = {}", best_cumulative_sharpe.id);

    // 打印详细的交易统计摘要,包括:
    //   - 总盈亏 (PnL)
    //   - 夏普比率 (Sharpe Ratio) - 风险调整后收益
    //   - 索提诺比率 (Sortino Ratio) - 只考虑下行风险
    //   - 最大回撤 (Max Drawdown) - 从峰值到谷底的最大跌幅
    //   - 胜率 (Win Rate) - 盈利交易占比
    //   - 盈利因子 (Profit Factor) - 总盈利/总亏损
    best_cumulative_sharpe.trading_summary.print_summary()
}

/// 从 JSON 文件加载回测配置
///
/// # 返回值
/// 包含无风险收益率和系统配置的 `Config` 结构体
///
/// # Panic
/// 如果文件不存在或 JSON 格式错误会 panic
pub fn load_config() -> Config {
    let file = File::open(CONFIG_PATH).expect("无法打开配置文件");
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).expect("配置文件 JSON 格式错误")
}

/// 从文件加载历史市场数据
///
/// # 参数
/// - `file_path`: 市场数据文件路径
///
/// # 文件格式
/// 文件中每行是一个 JSON 对象,代表一个市场事件(如交易成交、订单簿更新等)
///
/// # 泛型参数
/// - `InstrumentKey`: 交易品种标识符类型(如 `InstrumentIndex`)
/// - `Kind`: 市场数据类型(如 `PublicTrade`, `OrderBookL1` 等)
///
/// # 返回值
/// 所有市场事件的向量,按时间顺序排列
///
/// # Panic
/// 如果文件不存在或 JSON 格式错误会 panic
pub fn market_data_from_file<InstrumentKey, Kind>(
    file_path: &str,
) -> Vec<MarketStreamEvent<InstrumentKey, Kind>>
where
    InstrumentKey: for<'de> Deserialize<'de>, // 品种键必须可反序列化
    Kind: for<'de> Deserialize<'de>,          // 数据类型必须可反序列化
{
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);

    // 逐行读取文件,每行解析为一个市场事件
    reader
        .lines()
        .map(|line_result| {
            let line = line_result.unwrap();
            // 将 JSON 字符串反序列化为 MarketStreamEvent
            serde_json::from_str::<MarketStreamEvent<InstrumentKey, Kind>>(&line).unwrap()
        })
        .collect() // 收集所有事件到 Vec
}
