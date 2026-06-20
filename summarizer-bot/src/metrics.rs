//! Metric event ids reported by the bot.

/// The complete set of metric event ids the summarizer emits.
///
/// Pinning [`MetricsClient`](metrics_client::MetricsClient) to this enum keeps
/// every event id declared in one place and lets the compiler reject any id the
/// bot hasn't defined, preventing typos and drift between call sites.
#[derive(Debug, Clone, Copy)]
pub enum Event {
    /// A summarization attempt completed (successfully or not).
    SummaryGenerated,
    /// A message was dropped without being summarized.
    MessageSkipped,
    /// A Discord API call failed.
    DiscordApiError,
}

impl From<Event> for String {
    fn from(event: Event) -> String {
        match event {
            Event::SummaryGenerated => "summary_generated",
            Event::MessageSkipped => "message_skipped",
            Event::DiscordApiError => "discord_api_error",
        }
        .to_owned()
    }
}
