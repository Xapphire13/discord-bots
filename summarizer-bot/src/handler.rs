use std::time::Instant;

use metrics_client::MetricsClient;
use serenity::{
    all::{CreateEmbed, CreateMessage, EditMessage, EventHandler, Mentionable, Message, Ready},
    async_trait,
};
use tracing::{error, info};

use crate::{
    config::Config,
    llm::{SummaryError, SummaryGenerator},
    metrics::Event,
};

pub struct Handler {
    summary_generator: SummaryGenerator,
    // Messages at least this long are summarized
    message_length_min: usize,
    // Messages longer than this are not summarized
    message_length_max: usize,
    // Reports metrics to a service-panel instance. `None` when metrics are
    // disabled, in which case every emit is a no-op.
    metrics: Option<MetricsClient<Event>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: serenity::client::Context, msg: Message) {
        // Ignore bot messages to prevent loops
        if msg.author.bot {
            return;
        }

        let is_dm = msg.guild_id.is_none();
        let source = if is_dm { "dm" } else { "guild" };

        // DMs are always summarized; guild messages must fall within the
        // configured length window.
        if !is_dm {
            if msg.content.len() < self.message_length_min {
                self.record_skip("too_short");
                return;
            }
            if msg.content.len() > self.message_length_max {
                self.record_skip("too_long");
                return;
            }
        }

        if is_dm {
            info!(
                "Summarizing direct message from {}",
                msg.author.display_name()
            )
        } else {
            info!(
                "Summarizing message in {} from {}",
                msg.channel_id
                    .name(&ctx.http)
                    .await
                    .unwrap_or("unknown channel".to_string()),
                msg.author.display_name()
            )
        }

        // The summary leads with a preamble linking back to the original
        // message and referencing its author. Masked links only render
        // inside embeds, so the preamble is sent as an embed.
        let message_link = msg.link();
        let author_ref = msg.author.mention().to_string();

        let mut response = match msg
            .channel_id
            .send_message(
                &ctx.http,
                CreateMessage::new().embed(CreateEmbed::new().description(format!(
                    "### :hourglass: Summarizing [message]({message_link}) from {author_ref}"
                ))),
            )
            .await
        {
            Ok(msg) => msg,
            Err(why) => {
                error!("Error sending initial message: {why:?}");
                self.record_api_error("send");
                return;
            }
        };

        let input_len = msg.content.len();
        let author_id = msg.author.id.to_string();
        let started = Instant::now();
        let summary = self
            .summary_generator
            .generate_summary(msg.author.display_name(), &msg.content)
            .await;
        let latency_ms = started.elapsed().as_millis() as f64;

        let summary = match summary {
            Ok(summary) => {
                self.record_summary(
                    source,
                    &author_id,
                    "success",
                    latency_ms,
                    input_len,
                    Some(summary.len()),
                );
                summary
            }
            Err(why) => {
                error!("Error summarizing message: {why:?}");
                let outcome = match why {
                    SummaryError::Timeout => "timeout",
                    SummaryError::Generation(_) => "llm_error",
                };
                self.record_summary(source, &author_id, outcome, latency_ms, input_len, None);

                if let Err(why) = response.delete(&ctx.http).await {
                    error!("Error deleting initial message: {:?}", why);
                }

                return;
            }
        };

        let body =
            format!("### Summarized [message]({message_link}) from {author_ref}\n\n{summary}");

        if let Err(why) = response
            .edit(
                &ctx.http,
                EditMessage::new().embed(CreateEmbed::new().description(body)),
            )
            .await
        {
            error!("Error sending message: {:?}", why);
            self.record_api_error("edit");
        }
    }

    async fn ready(&self, _: serenity::client::Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

impl Handler {
    pub fn new(
        summary_generator: SummaryGenerator,
        config: &Config,
        metrics: Option<MetricsClient<Event>>,
    ) -> Self {
        Handler {
            summary_generator,
            message_length_min: config.message_length_min,
            message_length_max: config.message_length_max,
            metrics,
        }
    }

    /// Records a message that was dropped without being summarized.
    fn record_skip(&self, reason: &'static str) {
        if let Some(metrics) = &self.metrics {
            metrics
                .event(Event::MessageSkipped)
                .label("reason", reason)
                .record();
        }
    }

    /// Records the outcome of a summarization attempt. `output_len` is only
    /// present on success.
    fn record_summary(
        &self,
        source: &'static str,
        author_id: &str,
        outcome: &'static str,
        latency_ms: f64,
        input_len: usize,
        output_len: Option<usize>,
    ) {
        if let Some(metrics) = &self.metrics {
            let mut event = metrics
                .event(Event::SummaryGenerated)
                .label("source", source)
                .label("author_id", author_id)
                .label("outcome", outcome)
                .value("latency_ms", latency_ms)
                .value("input_len", input_len as f64);
            if let Some(output_len) = output_len {
                event = event.value("output_len", output_len as f64);
            }
            event.record();
        }
    }

    /// Records a failed Discord API call (`op` is `send` or `edit`).
    fn record_api_error(&self, op: &'static str) {
        if let Some(metrics) = &self.metrics {
            metrics
                .event(Event::DiscordApiError)
                .label("op", op)
                .record();
        }
    }
}
