use ::tracing::{error, info};
use anyhow::{Context, Result};
use serenity::{Client, all::GatewayIntents};
use shared::{config::BotConfig, tracing};

#[tokio::main]
async fn main() -> Result<()> {
    tracing::init(env!("CARGO_PKG_NAME"))?;

    let config = BotConfig::load()?;

    let intents = GatewayIntents::empty();

    let mut client = Client::builder(&config.discord_token, intents)
        // .event_handler(handler)
        .await
        .context("Error creating client")?;

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}
