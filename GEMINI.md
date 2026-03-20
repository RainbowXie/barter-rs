# Gemini Code Assistant Context

This document provides context for the Gemini Code Assistant to understand the `barter-rs` project.

## Project Overview

`barter-rs` is a high-performance, event-driven algorithmic trading framework written in Rust. It provides a modular and extensible ecosystem for building live trading, paper trading, and backtesting systems. The project is structured as a Rust workspace with several crates, each responsible for a specific aspect of the trading system.

### Key Features

- **Event-Driven Architecture:** The core of the framework is an `Engine` that processes a stream of events, including market data, account updates, and user commands.
- **Modularity:** The framework is divided into several crates, including `barter-data` for market data streams, `barter-execution` for order execution, `barter-instrument` for financial instrument definitions, and `barter-integration` for low-level exchange integrations.
- **Customizable Strategies and Risk Management:** Users can implement their own trading strategies and risk management logic by implementing the `Strategy` and `RiskManager` traits.
- **Backtesting and Paper Trading:** The framework supports backtesting and paper trading by using mock market data and execution clients.
- **High Performance:** The framework is designed for high performance, with a focus on low-latency and efficient memory usage.

### Project Structure

The `barter-rs` project is a Rust workspace with the following key crates:

- **`barter`:** The core crate that contains the trading engine, state management, and interfaces for strategies and risk management.
- **`barter-data`:** Provides tools for subscribing to and processing real-time market data from various exchanges.
- **`barter-execution`:** Handles order management, execution, and interaction with exchange APIs.
- **`barter-instrument`:** Defines data structures for financial instruments, such as stocks, futures, and options.
- **`barter-integration`:** Provides low-level building blocks for integrating with exchange APIs.
- **`barter-macro`:** Contains procedural macros used throughout the workspace.

## Building and Running

### Building the Project

The project can be built using the standard Rust toolchain.

```bash
cargo build
```

### Running Examples

The `barter` crate contains several examples that demonstrate how to use the framework. To run an example, use the following command:

```bash
cargo run --example <example_name>
```

For example, to run the paper trading example with live market data, use the following command:

```bash
cargo run --example engine_sync_with_live_market_data_and_mock_execution_and_audit
```

### Running Tests

The project has a comprehensive test suite. To run the tests, use the following command:

```bash
cargo test
```

## Development Conventions

### Coding Style

The project follows the standard Rust coding style, as enforced by `rustfmt`.

### Testing

The project has a strong emphasis on testing. New features should be accompanied by unit and integration tests.

### Contributions

Contributions are welcome. Please follow the contribution guidelines in the `README.md` file.
