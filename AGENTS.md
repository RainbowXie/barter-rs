# Repository Guidelines

## Project Structure & Module Organization

This Cargo workspace hosts multiple trading crates: barter/ (engine and strategies), barter-data/
(market-data streams), barter-execution/ (order routing), barter-instrument/ (instrument metadata),
barter-integration/ (protocol tooling), and barter-macro/ (proc macros). Common integration tests live
in barter/tests/. Examples and reference configs sit under each crate’s examples/ directory.

## Build, Test, and Development Commands

Use cargo build --workspace for a full compile of every crate. Run cargo test --workspace to execute
unit and integration tests across packages; scope to one crate via cargo test -p barter. Lint with
cargo clippy --workspace --all-targets --all-features. Format using cargo fmt --all. When iterating on
examples, run them directly, e.g. cargo run -p barter --example statistical_trading_summary.

## Coding Style & Naming Conventions

Follow Rust 2024 edition defaults with rustfmt (configured via rustfmt.toml; crate-level import
grouping is enforced). Prefer four-space indentation, CamelCase types, snake_case modules/functions,
SCREAMING_SNAKE_CASE constants. Keep public APIs documented with /// comments and favor descriptive
alias/enum names that mirror exchange terminology.

## Testing Guidelines

Primary tests rely on the standard Rust test harness. Mirror module names with <module>::tests
submodules and use descriptive it_should_* function names for behavior specs. Integration flows belong
in barter/tests/ with filenames matching the feature under test. When adding new exchange clients or
strategies, pair unit coverage with at least one example or regression test. Run targeted suites before
pushing, e.g. cargo test -p barter-data exchange::binance.

## Commit & Pull Request Guidelines

Follow the existing Conventional Commit leaning history; prefix scopes such as feat:, fix:, or chore:
and reference issues with (#123) when applicable. Each PR should summarize intent, enumerate key
changes, and list verification commands (cargo fmt, cargo clippy, cargo test). Include configuration or
log snippets if the change impacts runtime behavior. Coordinate large refactors across crates to avoid
breaking the workspace build.
