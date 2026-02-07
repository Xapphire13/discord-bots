pub struct BotConfig {
    /// Token allowing bot to connect bot to Discord
    pub discord_token: String,
}

/// Load bot config using the calling crate's manifest directory.
#[macro_export]
macro_rules! load_bot_config {
    () => {{
        use $crate::__private::anyhow::Context as _;

        #[cfg(debug_assertions)]
        $crate::__private::dotenvy::from_path(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env"),
        )
        .context("Can't find .env file")?;

        Ok::<$crate::config::BotConfig, $crate::__private::anyhow::Error>(
            $crate::config::BotConfig {
                discord_token: std::env::var("DISCORD_TOKEN")
                    .context("Expected DISCORD_TOKEN in environment")?,
            },
        )
    }};
}
