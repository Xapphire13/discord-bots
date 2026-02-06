use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use poise::samples::register_in_guild;
use serenity::{Client, all::GatewayIntents};
use tokio::sync::Mutex as TokioMutex;
use tracing::{error, info};

use crate::{
    backup::BackupQueue,
    cancellation::CancellationRegistry,
    command::{CommandData, cleanup},
    config::{Config, ConfigStore},
    onedrive::{OneDriveClient, TokenStore},
    scheduler::spawn_scheduler,
};

mod backup;
mod cancellation;
mod cleanup;
mod command;
mod config;
mod media;
mod onedrive;
mod scheduler;

#[tokio::main]
async fn main() -> Result<()> {
    shared::init_tracing!()?;
    let bot_config = shared::load_bot_config!()?;
    let config = Config::load()?;
    let backup_worker_config = config.media_backup.worker.clone();
    let onedrive_config = config.onedrive.clone();
    let config_store = ConfigStore::new(config);
    let backup_queue = Arc::new(Mutex::new(BackupQueue::load()?));
    let cancellation = Arc::new(Mutex::new(CancellationRegistry::new()));
    let intents = GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MESSAGES;

    // Initialize OneDrive client if configured
    let onedrive_client = if let Some(od_config) = onedrive_config {
        let token_store = Arc::new(TokioMutex::new(TokenStore::new(
            od_config.client_id.clone(),
        )));

        // Check if we need to authenticate
        if !token_store.lock().await.has_tokens() {
            info!("OneDrive tokens not found, starting device code flow...");
            token_store.lock().await.device_code_flow().await?;
        }

        Some(Arc::new(OneDriveClient::new(
            token_store,
            od_config.upload_folder,
        )))
    } else {
        info!("OneDrive not configured, backups will be stored locally only");
        None
    };

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![cleanup()],
            ..Default::default()
        })
        .setup({
            let config_store = config_store.clone();
            let cancellation = Arc::clone(&cancellation);

            move |ctx, ready, framework| {
                let http = Arc::clone(&ctx.http);

                Box::pin(async move {
                    info!("Connected!");

                    for guild_id in &ready.guilds {
                        register_in_guild(ctx, &framework.options().commands, guild_id.id).await?;
                    }

                    // Spawn the backup worker (only if we have somewhere to back up to)
                    if let Some(onedrive_client) = onedrive_client {
                        backup::spawn_worker(
                            Arc::clone(&backup_queue),
                            backup_worker_config,
                            onedrive_client,
                        );
                    }

                    // Spawn the cleanup scheduler
                    spawn_scheduler(
                        Arc::clone(&http),
                        config_store.clone(),
                        backup_queue,
                        Arc::clone(&cancellation),
                    );

                    Ok(CommandData {
                        config: config_store,
                        cancellation,
                    })
                })
            }
        })
        .build();

    let mut client = Client::builder(&bot_config.discord_token, intents)
        .framework(framework)
        .await
        .context("Error creating client")?;

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}
