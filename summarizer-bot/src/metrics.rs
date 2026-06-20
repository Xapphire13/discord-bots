//! Metric event ids, label keys, and label values reported by the bot.
//!
//! Every string that goes on the wire lives here exactly once. Event ids and
//! the closed sets of label values are modelled as enums so the compiler
//! rejects any id/value the bot hasn't declared; the open-ended keys are
//! constants so call sites can't drift apart by a typo.

/// The complete set of metric event ids the summarizer emits.
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

/// String label keys attached to events.
pub mod label {
    pub const SOURCE: &str = "source";
    pub const OUTCOME: &str = "outcome";
    pub const REASON: &str = "reason";
    pub const OP: &str = "op";
    pub const AUTHOR_ID: &str = "author_id";
}

/// Numeric value names attached to events.
pub mod value {
    pub const LATENCY_MS: &str = "latency_ms";
    pub const INPUT_LEN: &str = "input_len";
    pub const OUTPUT_LEN: &str = "output_len";
}

/// Where a summarized message originated.
#[derive(Debug, Clone, Copy)]
pub enum Source {
    Dm,
    Guild,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Dm => "dm",
            Source::Guild => "guild",
        }
    }
}

/// The outcome of a summarization attempt.
#[derive(Debug, Clone, Copy)]
pub enum Outcome {
    Success,
    Timeout,
    LlmError,
}

impl Outcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Success => "success",
            Outcome::Timeout => "timeout",
            Outcome::LlmError => "llm_error",
        }
    }
}

/// Why a message was dropped without being summarized.
#[derive(Debug, Clone, Copy)]
pub enum SkipReason {
    TooShort,
    TooLong,
}

impl SkipReason {
    pub fn as_str(self) -> &'static str {
        match self {
            SkipReason::TooShort => "too_short",
            SkipReason::TooLong => "too_long",
        }
    }
}

/// The Discord API operation that failed.
#[derive(Debug, Clone, Copy)]
pub enum ApiOp {
    Send,
    Edit,
}

impl ApiOp {
    pub fn as_str(self) -> &'static str {
        match self {
            ApiOp::Send => "send",
            ApiOp::Edit => "edit",
        }
    }
}
