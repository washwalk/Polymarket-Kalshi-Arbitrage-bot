# Agent Coding Guidelines for Polymarket-Kalshi-Arbitrage-Bot

This document provides coding guidelines and commands for agents working on the Polymarket-Kalshi-Arbitrage-Bot Rust codebase.

## Build, Lint, and Test Commands

### Building
- **Full release build**: `cargo build --release`
- **Debug build**: `cargo build`
- **Check for compilation errors**: `cargo check`

### Testing
- **Run all tests**: `cargo test`
- **Run specific test**: `cargo test test_name` (e.g., `cargo test test_pack_unpack_roundtrip`)
- **Run tests with output**: `cargo test -- --nocapture`
- **Run benchmarks**: `cargo bench`
- **Run tests in release mode**: `cargo test --release`

### Linting and Formatting
- **Lint with clippy**: `cargo clippy`
- **Fix clippy warnings**: `cargo clippy --fix`
- **Format code**: `cargo fmt`
- **Check formatting**: `cargo fmt --check`

### Running the Application
- **Development run (dry-run)**: `dotenvx run -- cargo run --release`
- **Production run**: `DRY_RUN=0 dotenvx run -- cargo run --release`
- **With specific log level**: `RUST_LOG=debug dotenvx run -- cargo run --release`

## Code Style Guidelines

### General Principles
- Write clear, readable, and maintainable code
- Prefer explicit over implicit behavior
- Use Rust idioms and best practices
- Prioritize performance for arbitrage detection logic
- Include comprehensive documentation for public APIs

### Imports and Dependencies
- Group imports in this order:
  1. Standard library (`std::*`)
  2. External crates (alphabetical)
  3. Local modules (`crate::*`)
- Use `use` statements to avoid repetition
- Prefer qualified imports for conflicting names
- Example:
```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::types::{PriceCents, SizeCents};
use crate::cache::TeamCache;
```

### Formatting and Structure
- Use `rustfmt` (default settings) for consistent formatting
- Maximum line length: 100 characters (default rustfmt)
- Use 4 spaces for indentation (default)
- Align struct fields and enum variants
- Use trailing commas in multi-line structures

### Naming Conventions
- **Functions and variables**: `snake_case`
- **Structs, enums, traits**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`
- **Type parameters**: Single uppercase letters (T, U, V)
- **Lifetimes**: Single lowercase letters ('a, 'b)

Examples:
```rust
pub struct AtomicMarketState { ... }
pub const MAX_MARKETS: usize = 1024;
pub fn check_arbs(&self, threshold_cents: PriceCents) -> u8 { ... }
```

### Types and Data Structures
- Use strongly typed interfaces with domain-specific types
- Define type aliases for clarity (e.g., `PriceCents = u16`)
- Use `#[derive]` for common traits when appropriate
- Prefer `Arc<T>` over `Rc<T>` for shared ownership across threads
- Use atomic types for lock-free concurrency
- Example:
```rust
pub type PriceCents = u16;
pub type SizeCents = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArbType {
    PolyYesKalshiNo,
    KalshiYesPolyNo,
    PolyOnly,
    KalshiOnly,
}
```

### Error Handling
- Use `anyhow::Result<T>` for all fallible operations
- Use `Context` for error messages: `context("descriptive message")`
- Prefer early returns with `?` operator
- Log errors appropriately (debug/warn/error)
- Example:
```rust
pub fn load_config() -> Result<Config> {
    let api_key = std::env::var("API_KEY")
        .context("API_KEY environment variable not set")?;
    Ok(Config { api_key })
}
```

### Performance Considerations
- Use `#[inline(always)]` for hot path functions (arbitrage detection, price updates)
- Prefer stack allocation over heap when possible
- Use SIMD operations for bulk computations (via `wide` crate)
- Minimize allocations in tight loops
- Use atomic operations for lock-free data structures
- Example:
```rust
#[inline(always)]
pub fn check_arbs(&self, threshold_cents: PriceCents) -> u8 {
    // SIMD-accelerated arbitrage detection
    use wide::{i16x8, CmpLt};
    // ... implementation
}
```

### Documentation
- Use `///` doc comments for all public APIs
- Include examples in doc comments when helpful
- Document struct fields and enum variants
- Explain complex algorithms or invariants
- Example:
```rust
/// Calculate Kalshi trading fee in cents for a single contract.
/// For typical prices (10-90 cents), fees are usually 1-2 cents per contract.
#[inline(always)]
pub fn kalshi_fee_cents(price_cents: PriceCents) -> PriceCents {
    // implementation
}
```

### Testing
- Write comprehensive unit tests for all public functions
- Use descriptive test names: `test_function_name_behavior`
- Test edge cases and error conditions
- Use `#[cfg(test)]` modules for test-specific code
- Example:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_pack_unpack_roundtrip() {
        // Test implementation
    }
}
```

### Concurrency and Safety
- Use `Arc<RwLock<T>>` for shared mutable state
- Prefer atomic operations over locks when possible
- Use `tokio::spawn` for async tasks
- Handle cancellation properly in async functions
- Example:
```rust
let state = Arc::new(RwLock::new(GlobalState::new()));
tokio::spawn(async move {
    // Task implementation
});
```

### Logging
- Use `tracing` crate for structured logging
- Log levels: error, warn, info, debug, trace
- Include context in log messages
- Use structured fields when appropriate
- Example:
```rust
info!("📊 Market discovery complete: matched {} pairs", pairs.len());
error!("[KALSHI] WebSocket error: {}", e);
```

### Code Organization
- Group related functionality into modules
- Use `mod.rs` files for module definitions
- Keep functions small and focused (single responsibility)
- Extract constants to module level
- Example module structure:
```
src/
├── main.rs          # Application entry point
├── types.rs         # Core types and data structures
├── execution.rs     # Order execution logic
├── kalshi.rs        # Kalshi platform integration
└── polymarket.rs    # Polymarket platform integration
```

### Security Best Practices
- Never log sensitive information (API keys, private keys)
- Validate inputs and handle edge cases
- Use secure defaults for configuration
- Avoid exposing internal state unnecessarily
- Example:
```rust
// Never log private keys
let config = KalshiConfig::from_env()?;
// info!("Loaded API key: {}", config.api_key); // DON'T DO THIS
info!("Kalshi API configuration loaded");
```

### Configuration and Environment
- Use environment variables for runtime configuration
- Provide sensible defaults where possible
- Document all configuration options
- Use `.env` files for local development (with `.env.example`)
- Example:
```rust
pub struct Config {
    pub dry_run: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let dry_run = std::env::var("DRY_RUN")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(true);
        Ok(Self { dry_run })
    }
}
```

### Dependencies
- Minimize external dependencies
- Prefer well-maintained crates from crates.io
- Pin versions in `Cargo.toml` for reproducibility
- Review security advisories regularly
- Current key dependencies:
  - `tokio`: Async runtime
  - `anyhow`: Error handling
  - `serde`: Serialization
  - `tracing`: Logging
  - `reqwest`: HTTP client
  - `tokio-tungstenite`: WebSocket client

Follow these guidelines to maintain code quality, performance, and consistency across the arbitrage bot codebase.