use crate::engine::Processor;
use barter_data::event::MarketEvent;
use barter_execution::AccountEvent;
use serde::{Deserialize, Serialize};

/// 默认全局数据 - 空结构体,用于不需要维护全局状态的策略和风险管理器
///
/// # 用途
/// `DefaultGlobalData` 是一个零大小类型(Zero-Sized Type, ZST),用于简单的交易策略。
///
/// ## 什么是全局数据?
/// 全局数据是 `EngineState` 中跨所有交易品种共享的数据,例如:
/// - 账户总资金
/// - 全局风险敞口
/// - 跨品种的统计指标
/// - 策略的全局参数
///
/// ## 何时使用 `DefaultGlobalData`?
/// - 当你的策略只关注单个品种的数据时
/// - 当你不需要跨品种的状态共享时
/// - 当你想快速开始,不想定义自定义全局数据时
///
/// ## 何时使用自定义全局数据?
/// 如果你的策略需要:
/// - 跟踪总账户净值变化
/// - 实现跨品种的风险限制(如总杠杆率上限)
/// - 维护全局的市场状态(如市场波动率指数)
/// - 在多个品种间分配资金
///
/// 那么你应该定义自己的全局数据结构并实现 `Processor` trait。
///
/// # 示例
/// ```ignore
/// // 自定义全局数据示例
/// #[derive(Debug, Clone, Deserialize, Serialize)]
/// pub struct MyGlobalData {
///     total_equity: Decimal,      // 总权益
///     max_leverage: Decimal,      // 最大杠杆率
///     market_regime: MarketRegime, // 市场状态(牛市/熊市/震荡)
/// }
///
/// impl Processor<&AccountEvent<...>> for MyGlobalData {
///     type Audit = ();
///     fn process(&mut self, event: &AccountEvent<...>) -> Self::Audit {
///         // 更新总权益
///         if let AccountEvent::BalanceUpdate(balance) = event {
///             self.total_equity = balance.total;
///         }
///     }
/// }
/// ```
#[derive(
    Debug,       // 可打印调试信息
    Copy,        // 可按位复制(因为是空结构体,复制成本为零)
    Clone,       // 可克隆
    Eq,          // 可判断相等性
    PartialEq,   // 可判断部分相等性
    Ord,         // 可排序
    PartialOrd,  // 可部分排序
    Hash,        // 可哈希(用于 HashMap 等)
    Default,     // 有默认值(空结构体的默认值就是它自己)
    Deserialize, // 可从 JSON 等格式反序列化
    Serialize,   // 可序列化为 JSON 等格式
)]
pub struct DefaultGlobalData;

// ============================================================================
// Processor 实现 - 处理账户事件
// ============================================================================

/// 为 `DefaultGlobalData` 实现账户事件处理
///
/// # Processor Trait 解释
/// `Processor` 是事件处理器 trait,定义如下:
/// ```ignore
/// pub trait Processor<Event> {
///     type Audit;  // 审计输出类型(用于记录处理过程)
///     fn process(&mut self, event: Event) -> Self::Audit;
/// }
/// ```
///
/// # 泛型参数
/// - `ExchangeKey`: 交易所标识符类型(如 `ExchangeIndex`)
/// - `AssetKey`: 资产标识符类型(如 `AssetIndex`,代表 BTC、ETH 等)
/// - `InstrumentKey`: 交易品种标识符类型(如 `InstrumentIndex`,代表 BTC/USDT 等)
///
/// # AccountEvent 是什么?
/// `AccountEvent` 是账户相关的事件,包括:
/// - 余额更新 (BalanceUpdate): 账户资金变化
/// - 订单更新 (OrderUpdate): 订单状态变化(已提交、已成交、已取消等)
/// - 成交通知 (Fill): 订单成交信息
/// - 持仓更新 (PositionUpdate): 持仓数量和均价变化
///
/// # 为什么是空实现?
/// `DefaultGlobalData` 是空结构体,不维护任何状态,所以:
/// - 接收账户事件但不做任何处理(函数体为空)
/// - 返回 `()` (unit type,表示无审计输出)
/// - 参数名用 `_` 表示不使用(避免编译器警告)
///
/// # 实际应用示例
/// 如果你想跟踪总权益,可以这样实现:
/// ```ignore
/// impl Processor<&AccountEvent<...>> for MyGlobalData {
///     type Audit = ();
///     fn process(&mut self, event: &AccountEvent<...>) -> Self::Audit {
///         match event {
///             AccountEvent::BalanceUpdate { exchange, asset, balance } => {
///                 self.total_equity += balance.available;
///             }
///             AccountEvent::Fill { fill, .. } => {
///                 // 记录成交,更新统计
///             }
///             _ => {}
///         }
///     }
/// }
/// ```
impl<ExchangeKey, AssetKey, InstrumentKey>
    Processor<&AccountEvent<ExchangeKey, AssetKey, InstrumentKey>> for DefaultGlobalData
{
    type Audit = (); // 无审计输出

    // 空实现:接收事件但不做任何处理
    // 参数名使用 `_` 表示忽略该参数
    fn process(&mut self, _: &AccountEvent<ExchangeKey, AssetKey, InstrumentKey>) -> Self::Audit {}
}

// ============================================================================
// Processor 实现 - 处理市场事件
// ============================================================================

/// 为 `DefaultGlobalData` 实现市场事件处理
///
/// # 泛型参数
/// - `InstrumentKey`: 交易品种标识符类型(如 `InstrumentIndex`)
/// - `Kind`: 市场数据类型,可以是多种形式:
///   - `PublicTrade`: 公开成交数据(价格、数量、时间)
///   - `OrderBookL1`: 一档订单簿(最优买价/卖价)
///   - `OrderBookL2`: 多档订单簿(完整深度数据)
///   - `Candle`: K线/蜡烛图数据(OHLCV)
///   - `Liquidation`: 爆仓数据
///
/// # MarketEvent 是什么?
/// `MarketEvent` 是公开市场数据事件,包含:
/// - 交易所信息
/// - 交易品种信息
/// - 市场数据内容(由泛型参数 `Kind` 决定)
/// - 时间戳
///
/// 例如一个交易成交事件:
/// ```ignore
/// MarketEvent {
///     exchange: "binance",
///     instrument: InstrumentIndex(0),  // 代表 BTC/USDT
///     kind: PublicTrade {
///         price: Decimal::from(50000),
///         quantity: Decimal::from_str("0.5").unwrap(),
///         side: Side::Buy,
///     },
///     time: SystemTime::now(),
/// }
/// ```
///
/// # 为什么是空实现?
/// `DefaultGlobalData` 不需要处理市场事件,原因:
/// - 市场数据通常由 `InstrumentMarketData` 处理(每个品种独立维护)
/// - 全局数据主要关注账户级别的状态
/// - 如果需要基于市场数据的全局状态(如市场波动率指数),应该自定义全局数据类型
///
/// # 实际应用示例
/// 如果你想在全局数据中跟踪市场状态:
/// ```ignore
/// impl Processor<&MarketEvent<InstrumentIndex, PublicTrade>> for MyGlobalData {
///     type Audit = ();
///     fn process(&mut self, event: &MarketEvent<...>) -> Self::Audit {
///         // 根据市场成交量判断市场活跃度
///         if event.kind.quantity > Decimal::from(100) {
///             self.market_regime = MarketRegime::HighVolatility;
///         }
///     }
/// }
/// ```
impl<InstrumentKey, Kind> Processor<&MarketEvent<InstrumentKey, Kind>> for DefaultGlobalData {
    type Audit = (); // 无审计输出

    // 空实现:接收市场事件但不做任何处理
    fn process(&mut self, _: &MarketEvent<InstrumentKey, Kind>) -> Self::Audit {}
}
