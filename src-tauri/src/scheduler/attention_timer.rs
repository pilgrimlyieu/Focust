use chrono::offset::LocalResult;
use chrono::{DateTime, Duration, Local, Utc};
use chrono::{Datelike, NaiveDate, NaiveTime};
use tauri::{AppHandle, Manager};
use tokio::sync::{mpsc, watch};
use tokio::time::sleep;

use super::models::{Command, SchedulerEvent};
use crate::core::schedule::AttentionSettings;
use crate::platform::create_break_windows;
use crate::{config::SharedConfig, core::schedule::AttentionId};

/// A simple timer for attention reminders
/// Unlike breaks, attentions are not affected by scheduler pause state
pub struct AttentionTimer {
    app_handle: AppHandle,
    shutdown_rx: watch::Receiver<()>,
}

impl AttentionTimer {
    pub fn new(app_handle: AppHandle, shutdown_rx: watch::Receiver<()>) -> Self {
        Self {
            app_handle,
            shutdown_rx,
        }
    }

    pub async fn run(&mut self, mut cmd_rx: mpsc::Receiver<Command>) {
        tracing::info!("AttentionTimer started");

        loop {
            // Check for incoming commands first
            if let Ok(cmd) = cmd_rx.try_recv() {
                self.handle_command(cmd).await;
                continue;
            }

            let next_attention = {
                let config = self.app_handle.state::<SharedConfig>();
                let config_guard = config.read().await;
                Self::calculate_next_attention(&config_guard.attentions)
            };

            if let Some((attention_id, attention_time)) = next_attention {
                let now = Utc::now();
                let duration_to_wait = attention_time - now;

                if duration_to_wait <= Duration::zero() {
                    // Attention time already passed, execute immediately and recalculate
                    tracing::warn!(
                        "Attention time already passed, triggering immediately and recalculating"
                    );
                    self.trigger_attention(attention_id);
                    continue;
                }

                tracing::info!(
                    "Next attention in {} seconds",
                    duration_to_wait.num_seconds()
                );

                tokio::select! {
                    biased;
                    _ = self.shutdown_rx.changed() => {
                        tracing::info!("AttentionTimer received shutdown signal");
                        break;
                    }
                    () = sleep(duration_to_wait.to_std().unwrap_or(std::time::Duration::ZERO)) => {
                        self.trigger_attention(attention_id);
                    }
                    Some(cmd) = cmd_rx.recv() => {
                        self.handle_command(cmd).await;
                    }
                }
            } else {
                tracing::debug!("No enabled attentions, waiting for config change or shutdown");

                // Wait for shutdown or config update command
                tokio::select! {
                    biased;
                    _ = self.shutdown_rx.changed() => {
                        tracing::info!("AttentionTimer received shutdown signal");
                        break;
                    }
                    Some(cmd) = cmd_rx.recv() => {
                        self.handle_command(cmd).await;
                    }
                }
            }
        }

        tracing::info!("AttentionTimer shutting down");
    }

    /// Calculate the next attention time across all enabled attentions
    fn calculate_next_attention(
        attentions: &[AttentionSettings],
    ) -> Option<(AttentionId, DateTime<Utc>)> {
        let now = Utc::now();
        let now_local = now.with_timezone(&Local);

        attentions
            .iter()
            .filter_map(|attention| {
                Self::get_next_attention_time(attention, now_local).map(|time| (attention.id, time))
            })
            .min_by_key(|(_, time)| *time)
    }

    /// Get the next occurrence time for a specific attention
    fn get_next_attention_time(
        attention: &AttentionSettings,
        now: DateTime<Local>,
    ) -> Option<DateTime<Utc>> {
        if (!attention.enabled) || attention.times.is_empty() || attention.days_of_week.is_empty() {
            tracing::debug!(
                "Attention '{}' is disabled or has no times/days configured.",
                attention.name
            );
            return None;
        }

        let now_date = now.date_naive();
        let now_time = now.time();

        let to_utc = |dt_local: DateTime<Local>| -> Option<DateTime<Utc>> {
            tracing::debug!(
                "Found potential attention '{}' time: {} (local)",
                attention.name,
                dt_local.to_rfc2822()
            );
            Some(dt_local.with_timezone(&Utc))
        };

        let build_datetime = |date: NaiveDate, time: NaiveTime| -> Option<DateTime<Local>> {
            match date.and_time(time).and_local_timezone(Local) {
                LocalResult::Single(dt) => Some(dt),
                LocalResult::Ambiguous(dt1, _) => {
                    tracing::warn!(
                        "Ambiguous local time encountered for {time} on {date}. Using the first one."
                    );
                    Some(dt1)
                }
                LocalResult::None => {
                    tracing::error!("No valid local time found for {time} on {date}.");
                    None
                }
            }
        };

        // Check if there's a time today
        if attention.days_of_week.contains(&now.weekday())
            && let Some(next_time_today) = attention.times.earliest_after(&now_time)
            && let Some(dt_local) = build_datetime(now_date, next_time_today)
        {
            return to_utc(dt_local);
        }

        // Check next 7 days
        for i in 1..=7 {
            let next_date = now_date + chrono::Duration::days(i);
            if attention.days_of_week.contains(&next_date.weekday())
                && let Some(first_time) = attention.times.first()
                && let Some(dt_local) = build_datetime(next_date, first_time)
            {
                return to_utc(dt_local);
            }
        }
        unreachable!("Impossible if attention has times and days configured");
    }

    /// Trigger an attention reminder
    fn trigger_attention(&self, attention_id: AttentionId) {
        tracing::info!("Triggering attention: {attention_id}");

        let event = SchedulerEvent::Attention(attention_id);

        let app_handle = self.app_handle.clone();
        tokio::spawn(async move {
            create_break_windows(&app_handle, event)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to create attention windows: {e}");
                });
        });
    }

    /// Handle incoming commands
    async fn handle_command(&mut self, cmd: Command) {
        tracing::debug!("AttentionTimer handling command: {cmd}");

        match cmd {
            Command::UpdateConfig(new_config) => {
                tracing::debug!("Updating config in AttentionTimer");
                {
                    let config = self.app_handle.state::<SharedConfig>();
                    let mut config_guard = config.write().await;
                    *config_guard = new_config;
                }
                // Config updated, will recalculate next attention in next loop iteration
            }
            Command::TriggerEvent(SchedulerEvent::Attention(attention_id)) => {
                tracing::info!("Manually triggering attention: {attention_id}");
                self.trigger_attention(attention_id);
            }
            // AttentionTimer ignores other commands (they're for BreakScheduler)
            // Maybe handle more commands in the future
            _ => {}
        }
    }
}
