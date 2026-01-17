use anyhow::Result;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::env;

const MESSAGE_LENGTH_THRESHOLD: usize = 50; // TODO, set higher once done testing

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore bot messages to prevent loops
        if msg.author.bot {
            return;
        }

        if msg.content.len() > MESSAGE_LENGTH_THRESHOLD {
            println!(
                "Long message detected from {}: {} characters",
                msg.author.name,
                msg.content.len()
            );

            let summary = generate_summary(&msg.content).await;

            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, format!("📝 **Summary:**\n{}", summary))
                .await
            {
                eprintln!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

// Placeholder function for LLM summarization
async fn generate_summary(content: &str) -> String {
    format!(
        "This is a placeholder summary. Original message was {} characters long.",
        content.len()
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }

    Ok(())
}
