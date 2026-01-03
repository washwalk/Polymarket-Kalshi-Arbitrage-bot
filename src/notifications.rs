//! Notification system for reliable alerts.
//!
//! This module handles asynchronous notifications via Pushover API.
//! Uses a dedicated worker to prevent network I/O from blocking the trading loop.

use crate::types::AlertMessage;
use anyhow::Result;
use reqwest::Client;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};

/// Pushover notification payload.
#[derive(Serialize)]
struct PushoverPayload {
    token: String,
    user: String,
    message: String,
    title: String,
    priority: i32,
}

/// Notification manager with dedicated worker.
pub struct NotificationManager {
    pub sender: mpsc::UnboundedSender<AlertMessage>,
    _worker_handle: tokio::task::JoinHandle<()>,
}

impl NotificationManager {
    /// Create a new notification manager with worker.
    pub fn new(
        pushover_token: String,
        pushover_user_key: String,
        http_client: Arc<Client>,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let worker_handle = tokio::spawn(notification_worker(
            rx,
            pushover_token,
            pushover_user_key,
            http_client,
        ));

        Self {
            sender: tx,
            _worker_handle: worker_handle,
        }
    }

    /// Send an alert (non-blocking).
    /// Returns false if the channel is full (shouldn't happen with unbounded).
    pub fn send_alert(&self, alert: AlertMessage) -> bool {
        self.sender.send(alert).is_ok()
    }
}

/// Dedicated notification worker task.
/// Processes alerts sequentially to avoid overwhelming the HTTP client.
async fn notification_worker(
    mut rx: mpsc::UnboundedReceiver<AlertMessage>,
    pushover_token: String,
    pushover_user_key: String,
    http_client: Arc<Client>,
) {
    while let Some(alert) = rx.recv().await {
        if let Err(e) = send_pushover_alert(&alert, &pushover_token, &pushover_user_key, &http_client).await {
            error!("Failed to send Pushover alert: {}", e);
        }
    }
}

/// Send a single alert via Pushover API.
async fn send_pushover_alert(
    alert: &AlertMessage,
    token: &str,
    user_key: &str,
    client: &Client,
) -> Result<()> {
    let (title, message, priority) = match alert {
        AlertMessage::ArbExecuted { profit, market, .. } => (
            "🎯 Arbitrage Executed".to_string(),
            format!("Profit: ${:.2} | Market: {}", profit, market),
            0, // Normal priority
        ),
        AlertMessage::SystemHalted { reason } => (
            "🚨 System Halted".to_string(),
            format!("Reason: {}", reason),
            1, // High priority
        ),
    };

    let payload = PushoverPayload {
        token: token.to_string(),
        user: user_key.to_string(),
        message,
        title: title.clone(),
        priority,
    };

    let response = client
        .post("https://api.pushover.net/1/messages.json")
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        info!("Pushover alert sent: {}", title);
    } else {
        error!("Pushover API error: {}", response.status());
    }

    Ok(())
}