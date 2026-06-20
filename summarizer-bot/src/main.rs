use ::tracing::{error, info};
use anyhow::{Context, Result};
use metrics_client::{ClientConfig, MetricsClient};
use serenity::prelude::*;

use crate::config::Config;
use crate::handler::Handler;
use crate::llm::SummaryGenerator;

mod config;
mod handler;
mod llm;
mod metrics;

/// Service identifier reported with every metric and heartbeat.
const METRICS_SOURCE: &str = "summarizer-bot";

#[tokio::main]
async fn main() -> Result<()> {
    shared::init_tracing!()?;
    let config = Config::from_env()?;

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;

    let metrics = config.metrics.as_ref().map(|metrics| {
        info!("Metrics enabled, reporting to {}", metrics.ingest_endpoint);
        MetricsClient::<metrics::Event>::new(
            ClientConfig::new(
                &metrics.ingest_endpoint,
                &metrics.heartbeat_endpoint,
                METRICS_SOURCE,
            )
            .with_heartbeat_interval(metrics.heartbeat_interval),
        )
    });

    let summary_generator = SummaryGenerator::new(&config);
    let handler = Handler::new(summary_generator, &config, metrics.clone());

    let mut client = Client::builder(&config.bot.discord_token, intents)
        .event_handler(handler)
        .await
        .context("Error creating client")?;

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }

    // Flush any buffered metrics before exiting.
    if let Some(metrics) = metrics {
        metrics.shutdown().await;
    }

    Ok(())
}
