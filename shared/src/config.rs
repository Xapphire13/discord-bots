use std::env;

use anyhow::{Context, Result};

pub struct BotConfig {
    /// Token allowing bot to connect bot to Discord
    pub discord_token: String,
}

impl BotConfig {
    pub fn load() -> Result<Self> {
        #[cfg(debug_assertions)]
        dotenvy::dotenv()?;

        Ok(Self {
            discord_token: env::var("DISCORD_TOKEN")
                .context("Expected DISCORD_TOKEN in environment")?,
        })
    }
}
