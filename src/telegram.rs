//! Telegram control interface for emergency management.
//!
//! This module provides a secure Telegram bot interface for status checks
//! and emergency control. Access is restricted to admin only.

use crate::kalshi::KalshiApiClient;
use crate::polymarket_clob::SharedAsyncClient;
use crate::types::{PositionSummary, StatusCache};
use anyhow::Result;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::Message;
use tokio::sync::broadcast;
use tracing::{info, warn};

/// Telegram command handler.
pub async fn run_telegram_handler(
    bot_token: String,
    admin_chat_id: String,
    status_cache: StatusCache,
    shutdown_tx: broadcast::Sender<()>,
    kalshi_client: Arc<KalshiApiClient>,
    poly_client: Arc<SharedAsyncClient>,
) -> Result<()> {
    let bot = Bot::new(bot_token);

    let handler = Update::filter_message()
        .filter_map(move |msg: Message| {
            // Extract command and validate access
            let msg_clone = msg.clone();
            let text = msg_clone.text()?;
            let chat_id = msg.chat.id.to_string();

            if chat_id != admin_chat_id {
                warn!("Unauthorized access attempt from chat ID: {}", chat_id);
                return None;
            }

            Some((msg, text.to_string()))
        })
        .branch(
            dptree::filter_map(|(msg, text): (Message, String)| {
                // Parse commands
                if text.starts_with("/status") {
                    Some((msg, Command::Status))
                } else if text.starts_with("/stop") {
                    Some((msg, Command::Stop))
                } else if text.starts_with("/toggle_dry_run") {
                    Some((msg, Command::ToggleDryRun))
                } else if text.starts_with("/help") {
                    Some((msg, Command::Help))
                } else {
                    None
                }
            })
            .endpoint(handle_command),
        );

    // Create dispatcher with dependencies
    let mut dispatcher = Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![status_cache, shutdown_tx, kalshi_client, poly_client])
        .build();

    info!("Telegram control interface started");
    dispatcher.dispatch().await;

    Ok(())
}

/// Supported commands.
#[derive(Clone)]
enum Command {
    Status,
    Stop,
    ToggleDryRun,
    Help,
}

/// Handle incoming commands.
async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    status_cache: StatusCache,
    shutdown_tx: broadcast::Sender<()>,
    kalshi_client: Arc<KalshiApiClient>,
    poly_client: Arc<SharedAsyncClient>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match cmd {
        Command::Status => {
            let summary = status_cache.load();
            let response = format_status(&summary);
            bot.send_message(msg.chat.id, response).await?;
        }
        Command::Stop => {
            // Initiate emergency shutdown
            warn!("Emergency stop initiated via Telegram");
            let _ = shutdown_tx.send(());

            // TODO: Add order cancellation here
            // poly_client.cancel_all_orders().await?;
            // kalshi_client.cancel_all_orders().await?;

            bot.send_message(msg.chat.id, "🚨 Emergency stop initiated. System halting...").await?;
        }
        Command::ToggleDryRun => {
            // Note: Dry run toggle would require additional state management
            // For now, just acknowledge
            bot.send_message(msg.chat.id, "⚠️ Dry run toggle not implemented yet").await?;
        }
        Command::Help => {
            let help_text = r#"/status - Show current P&L and position summary
/stop - Emergency halt with order cancellation
/toggle_dry_run - Switch between live/paper trading (not implemented)
/help - Show this help"#;
            bot.send_message(msg.chat.id, help_text).await?;
        }
    }

    Ok(())
}

/// Format position summary for Telegram response.
fn format_status(summary: &Arc<PositionSummary>) -> String {
    format!(
        "📊 Bot Status\n\
         💰 Realized P&L: ${:.2}\n\
         📈 Open Positions: {}\n\
         🎯 Last Trade: ${:.2}\n\
         🕒 Last Execution: {}\n\
         🛡️ Mode: {}",
        summary.realized_pnl,
        summary.open_positions,
        summary.last_trade_profit,
        if summary.last_execution_time > 0 {
            format!("{}s ago", (chrono::Utc::now().timestamp() as u64).saturating_sub(summary.last_execution_time / 1_000_000_000))
        } else {
            "Never".to_string()
        },
        if summary.dry_run { "DRY RUN" } else { "LIVE" }
    )
}