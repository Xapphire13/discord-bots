use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use shared::config::BotConfig;

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
        };

        if config.message_length_min > config.message_length_max {
            return Err(anyhow!("MESSAGE_LENGTH_MIN must be <= MESSAGE_LENGTH_MAX"));
        }

        Ok(config)
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
