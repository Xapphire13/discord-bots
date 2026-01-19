# Discord Bots

A Rust workspace monorepo containing Discord bots.

## Bots

- **[summarizer-bot](./summarizer-bot/)** - A Discord bot that summarizes conversations using Ollama

## Building

```bash
# Build all bots
cargo build --release

# Build a specific bot
cargo build --release -p summarizer-bot
```

## Adding a New Bot

1. Create a new directory for your bot (e.g., `my-bot/`)
2. Add a `Cargo.toml` with your bot's dependencies
3. Add the bot to the workspace members in the root `Cargo.toml`:

## License

MIT License - see [LICENSE](LICENSE) for details.
