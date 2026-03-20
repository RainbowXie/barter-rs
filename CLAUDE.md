# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Barter** is a high-performance, event-driven algorithmic trading framework written in Rust. It provides a modular ecosystem for building live trading, paper trading, and backtesting systems with support for multiple exchanges simultaneously.

### Workspace Structure

This is a Rust workspace with six crates:

- **`barter`**: Core trading engine with state management, Strategy/RiskManager interfaces, and performance statistics
- **`barter-data`**: WebSocket streaming of public market data (trades, order books, candles) from exchanges
- **`barter-execution`**: Order execution, account data streaming, and mock execution for testing
- **`barter-instrument`**: Data structures for exchanges, instruments, and assets
- **`barter-integration`**: Low-level REST/WebSocket integration primitives
- **`barter-macro`**: Procedural macros used across the workspace

## Commands

### Build
```bash
cargo build
```

### Run Tests
```bash
# All tests
cargo test

# Specific crate
cargo test -p barter
cargo test -p barter-data
cargo test -p barter-execution
```

### Run Examples
```bash
# List available examples
ls barter/examples

# Run specific example
cargo run --example engine_sync_with_live_market_data_and_mock_execution_and_audit

# Other examples:
cargo run --example engine_sync_with_audit_replica_engine_state
cargo run --example backtests_concurrent
cargo run --example statistical_trading_summary
```

### Run Benchmarks
```bash
cargo bench --bench backtest
```

### Code Formatting
```bash
cargo fmt
```

## Architecture

### Event-Driven Engine

The core `Engine` processes an event stream in a loop:
- **Market Events**: Public market data (trades, order books) from `barter-data`
- **Account Events**: Private account data (balances, orders, fills) from `barter-execution`
- **Commands**: External directives (ClosePositions, CancelOrders, etc.) for runtime control

### State Management

`EngineState` is a centralized, cache-friendly state system using indexed data structures for O(1) lookups:
- **Connectivity states**: Track exchange connection status
- **Asset states**: Balance and position information per asset
- **Instrument states**: Market data, orders, and positions per instrument
- **Order states**: In-flight requests and order lifecycle tracking

Indexing: `ExchangeIndex`, `AssetIndex`, `InstrumentIndex` provide type-safe integer indices instead of string lookups.

### Strategy & Risk Management

- **Strategy traits**: `AlgoStrategy` (generate orders), `ClosePositionsStrategy`, `OnDisconnectStrategy`, `OnTradingDisabled`
- **RiskManager trait**: Review and filter order requests before execution
- Plug-and-play design allows custom implementations for different trading strategies (market making, stat arb, HFT, etc.)

### Execution System

- **ExecutionClient trait**: Unified interface for live or mock order execution
- **ExecutionManager**: Routes execution requests to the appropriate exchange
- **MockExecutionClient**: Simulates order execution for backtesting/paper trading

### Multi-Exchange Support

The Engine can trade on multiple exchanges simultaneously:
- Each exchange has its own `ExecutionClient` and market data stream
- `IndexedInstruments` maps instruments across exchanges
- State is indexed by exchange for fast lookups

### Audit System

When audit mode is enabled, the Engine emits `AuditTick` events containing:
- Input event processed
- Actions taken (orders generated, positions closed, etc.)
- State changes

Use case: Replicate `EngineState` in external process for monitoring/UI.

### Backtesting

- Use `MockExecutionClient` and historical market data
- Run concurrent backtests efficiently with `backtests_concurrent` utilities
- Near-identical code path for backtesting vs. live trading

### Trading Summaries

`TradingSummaryGenerator` produces comprehensive performance metrics:
- PnL (realized/unrealized)
- Sharpe ratio, Sortino ratio
- Maximum drawdown
- Win rate, profit factor
- Aggregated by period (Daily, Weekly, Monthly, etc.)

## Key Patterns

### Indexed Data Structures

Throughout the codebase, string-based lookups are replaced with integer indices:
- `IndexedInstruments`: Maps `InstrumentIndex` to instrument metadata
- `ExecutionTxMap`: Maps `ExchangeIndex` to execution transmitters
- State maps use indices as keys for performance

### Feed Modes

Engine supports two feed modes:
- **Iterator mode**: Synchronous event processing (useful for backtesting)
- **Stream mode**: Asynchronous event processing (useful for live trading)

### Typed Order States

Orders use compile-time state transitions:
- `Order<_, _, RequestOpen>` → `Order<_, _, Open>` → `Order<_, _, Closed>`
- Prevents invalid state transitions at compile time

### Tokio Async Runtime

Most I/O operations use Tokio:
- WebSocket connections for market/account data
- Async HTTP requests for REST APIs
- Multi-threaded runtime for concurrent exchange connections

## Development Notes

### Adding New Exchange Integration

1. Implement `Connector` trait in `barter-integration`
2. Add exchange-specific types in `barter-data/src/exchange/{exchange_name}`
3. Implement `ExchangeTransformer` to normalize exchange data
4. Add `ExecutionClient` implementation in `barter-execution/src/exchange/{exchange_name}`

### Testing Strategy

- Unit tests for individual components
- Integration tests in `/tests` directory (see `test_engine_process_engine_event_with_audit.rs`)
- Use `MockExecutionClient` for testing without live exchange connections
- Examples serve as integration test reference

### Rust Edition & Toolchain

- Uses Rust edition 2024
- See `rust-toolchain.toml` for specific toolchain version
- Code style enforced by `rustfmt.toml`
