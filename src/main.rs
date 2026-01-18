use anyhow::{Context, Result};
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use serenity::all::EditMessage;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::env;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{error, info, instrument};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug)]
struct Handler {
    ollama_client: Ollama,
    llm_model: String,
    // Messages at least this long are summarized
    message_length_min: usize,
    // Messages longer than this are not summarized
    message_length_max: usize,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: serenity::client::Context, msg: Message) {
        // Ignore bot messages to prevent loops
        if msg.author.bot {
            return;
        }

        if (msg.content.len() >= self.message_length_min
            && msg.content.len() <= self.message_length_max)
            || msg.guild_id.is_none()
        {
            let mut response = match msg
                .channel_id
                .say(
                    &ctx.http,
                    format!(
                        ":hourglass: Summarizing message from {}",
                        msg.author.mention()
                    ),
                )
                .await
            {
                Ok(msg) => msg,
                Err(why) => {
                    error!("Error sending initial message: {why:?}");
                    return;
                }
            };

            let summary = match self
                .generate_summary(msg.author.display_name(), &msg.content)
                .await
            {
                Ok(summary) => summary,
                Err(why) => {
                    error!("Error summarizing message: {why:?}");

                    if let Err(why) = response.delete(&ctx.http).await {
                        error!("Error deleting initial message: {:?}", why);
                    }

                    return;
                }
            };

            if let Err(why) = response
                .edit(&ctx.http, EditMessage::new().content(summary))
                .await
            {
                error!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: serenity::client::Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

impl Handler {
    #[instrument(level = "trace", skip_all)]
    async fn generate_summary(&self, author: &str, content: &str) -> Result<String> {
        let result = timeout(
            Duration::from_mins(10),
            self.ollama_client.generate(
                GenerationRequest::new(
                    self.llm_model.clone(),
                    format!("Author: {author}\nMessage: {content}"),
                )
                .system(include_str!("../system_prompt.txt")),
            ),
        )
        .await
        .context("LLM request timed out")?
        .context("LLM generation failed")?;

        Ok(result.response)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let tracing_registry = tracing_subscriber::registry();

    match tracing_journald::layer() {
        Ok(journald_layer) => tracing_registry.with(journald_layer).init(),
        Err(_) => tracing_registry
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::NEW | FmtSpan::CLOSE))
            .init(),
    };

    #[cfg(debug_assertions)]
    dotenvy::dotenv()?;

    let ollama_client = Ollama::new(
        env::var("LLM_HOST").context("Expected LLM_HOST in environment")?,
        env::var("LLM_PORT")
            .context("Expected LLM_PORT in environment")?
            .parse()?,
    );

    let token = env::var("DISCORD_TOKEN").context("Expected DISCORD_TOKEN in environment")?;
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;
    let handler = Handler {
        ollama_client,
        llm_model: env::var("LLM_MODEL").context("Expected LLM_MODEL in environment")?,
        message_length_min: env::var("MESSAGE_LENGTH_MIN")
            .context("Expected MESSAGE_LENGTH_MIN in environment")?
            .parse()?,
        message_length_max: env::var("MESSAGE_LENGTH_MAX")
            .context("Expected MESSAGE_LENGTH_MAX in environment")?
            .parse()?,
    };

    if handler.message_length_min > handler.message_length_max {
        panic!("MESSAGE_LENGTH_MIN must be <= MESSAGE_LENGTH_MAX");
    }

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .context("Error creating client")?;

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}
