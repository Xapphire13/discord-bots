use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use shared::config::BotConfig;

/// Default interval between automatic heartbeats when `METRICS_HEARTBEAT_INTERVAL`
/// is unset.
const DEFAULT_HEARTBEAT_INTERVAL_SECS: u64 = 30;

pub struct Config {
    pub bot: BotConfig,
    pub llm_model: String,
    pub llm_host: String,
    pub llm_port: u16,
    pub message_length_min: usize,
    pub message_length_max: usize,
    /// System prompt for the summarizer, loaded from `system_prompt.txt` in the
    /// app's data directory at startup. Restart the service to pick up edits.
    pub system_prompt: String,
    /// Metrics reporting config. `None` when the `METRICS_*` env vars are unset,
    /// in which case the bot runs without reporting metrics.
    pub metrics: Option<MetricsConfig>,
}

/// Config for reporting metrics to a service-panel instance.
pub struct MetricsConfig {
    pub ingest_endpoint: String,
    pub heartbeat_endpoint: String,
    pub heartbeat_interval: Duration,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let config = Self {
            bot: shared::load_bot_config!()?,
            llm_model: env::var("LLM_MODEL").context("Expected LLM_MODEL in environment")?,
            llm_host: env::var("LLM_HOST").context("Expected LLM_HOST in environment")?,
            llm_port: env::var("LLM_PORT")
                .context("Expected LLM_PORT in environment")?
                .parse()
                .context("LLM_PORT must be a valid port number")?,
            message_length_min: env::var("MESSAGE_LENGTH_MIN")
                .context("Expected MESSAGE_LENGTH_MIN in environment")?
                .parse()
                .context("MESSAGE_LENGTH_MIN must be a valid number")?,
            message_length_max: env::var("MESSAGE_LENGTH_MAX")
                .context("Expected MESSAGE_LENGTH_MAX in environment")?
                .parse()
                .context("MESSAGE_LENGTH_MAX must be a valid number")?,
            system_prompt: load_system_prompt()?,
            metrics: load_metrics_config()?,
        };

        if config.message_length_min > config.message_length_max {
            return Err(anyhow!("MESSAGE_LENGTH_MIN must be <= MESSAGE_LENGTH_MAX"));
        }

        Ok(config)
    }
}

/// Reads the optional metrics config.
///
/// Metrics are enabled only when both `METRICS_INGEST_ENDPOINT` and
/// `METRICS_HEARTBEAT_ENDPOINT` are set; if neither is set the bot runs without
/// metrics. A blank value counts as unset, so an empty endpoint can't slip
/// through as a silently-failing URL. Setting only one is treated as a
/// misconfiguration so a typo doesn't silently disable reporting.
fn load_metrics_config() -> Result<Option<MetricsConfig>> {
    // Treat a blank value the same as unset.
    let read = |key| env::var(key).ok().filter(|value| !value.is_empty());
    let ingest_endpoint = read("METRICS_INGEST_ENDPOINT");
    let heartbeat_endpoint = read("METRICS_HEARTBEAT_ENDPOINT");

    match (ingest_endpoint, heartbeat_endpoint) {
        (None, None) => Ok(None),
        (Some(ingest_endpoint), Some(heartbeat_endpoint)) => {
            let heartbeat_interval = match read("METRICS_HEARTBEAT_INTERVAL") {
                Some(secs) => {
                    let secs: u64 = secs
                        .parse()
                        .context("METRICS_HEARTBEAT_INTERVAL must be a number of seconds")?;
                    // A zero interval would panic `tokio::time::interval`.
                    if secs == 0 {
                        return Err(anyhow!(
                            "METRICS_HEARTBEAT_INTERVAL must be greater than zero"
                        ));
                    }
                    Duration::from_secs(secs)
                }
                None => Duration::from_secs(DEFAULT_HEARTBEAT_INTERVAL_SECS),
            };

            Ok(Some(MetricsConfig {
                ingest_endpoint,
                heartbeat_endpoint,
                heartbeat_interval,
            }))
        }
        _ => Err(anyhow!(
            "METRICS_INGEST_ENDPOINT and METRICS_HEARTBEAT_ENDPOINT must both be set or both unset"
        )),
    }
}

/// Reads the system prompt from `system_prompt.txt`.
///
/// In release builds the file is resolved relative to the working directory
/// (the systemd `WorkingDirectory`, i.e. the app's data directory), so the
/// prompt can be edited and picked up with a service restart — no rebuild
/// required. In debug builds it is resolved relative to the crate's manifest
/// directory for convenient local development, mirroring how `.env` is loaded.
fn load_system_prompt() -> Result<String> {
    let path = system_prompt_path();
    fs::read_to_string(&path)
        .with_context(|| format!("Failed to read system prompt from {}", path.display()))
}

#[cfg(debug_assertions)]
fn system_prompt_path() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("system_prompt.txt")
}

#[cfg(not(debug_assertions))]
fn system_prompt_path() -> PathBuf {
    PathBuf::from("./system_prompt.txt")
}
