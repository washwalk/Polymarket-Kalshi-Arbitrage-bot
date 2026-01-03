//! Status cache for lock-free access to position summary data.
//!
//! This module provides a thread-safe, lock-free cache for high-level bot status
//! that can be read by the Telegram interface without blocking the trading loop.

use arc_swap::ArcSwap;
use std::sync::Arc;

/// High-level position summary for status checks.
/// Updated immediately after executions for real-time accuracy.
#[derive(Clone, Debug)]
pub struct PositionSummary {
    /// Total realized profit/loss in dollars
    pub realized_pnl: f64,
    /// Number of currently open positions
    pub open_positions: usize,
    /// Profit/loss from the most recent trade
    pub last_trade_profit: f64,
    /// Unix timestamp (nanoseconds) of the last arbitrage execution
    pub last_execution_time: u64,
    /// Whether the bot is in dry-run mode
    pub dry_run: bool,
}

impl Default for PositionSummary {
    fn default() -> Self {
        Self {
            realized_pnl: 0.0,
            open_positions: 0,
            last_trade_profit: 0.0,
            last_execution_time: 0,
            dry_run: true,
        }
    }
}

/// Lock-free status cache using ArcSwap for atomic updates.
pub type StatusCache = Arc<ArcSwap<PositionSummary>>;
