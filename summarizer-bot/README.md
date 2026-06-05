# Summarizer Bot

A Discord bot that automatically summarizes long messages using a local LLM
(Ollama). When users post messages exceeding a configurable length threshold,
the bot generates concise 1-3 sentence summaries that get straight to the point.

## Features

- Automatic detection of long messages based on configurable thresholds
- Local LLM inference via Ollama (no cloud API dependencies)
- Concise, to-the-point summaries

## Requirements

- Rust (Edition 2024)
- [Ollama](https://ollama.ai/) running on an accessible
  host with your preferred model
- Discord bot token with `GUILD_MESSAGES` and `MESSAGE_CONTENT` intents

## Configuration

Create a `.env` file with the following variables:

```env
DISCORD_TOKEN=<YOUR_DISCORD_BOT_TOKEN>
LLM_HOST=http://your-ollama-host
LLM_PORT=11434
LLM_MODEL=<YOUR_LLM_MODEL>
MESSAGE_LENGTH_MIN=500
MESSAGE_LENGTH_MAX=2000
```

| Variable                   | Description                                                     |
| -------------------------- | --------------------------------------------------------------- |
| `DISCORD_TOKEN`            | Your Discord bot authentication token                           |
| `LLM_HOST`                 | Ollama server hostname (e.g., `http://localhost`)               |
| `LLM_PORT`                 | Ollama server port (default: `11434`)                           |
| `LLM_MODEL`                | Model to use for summarization (e.g., `llama3.2:3b`)            |
| `MESSAGE_LENGTH_MIN`       | Minimum message length to trigger summarization                 |
| `MESSAGE_LENGTH_MAX`       | Maximum message length to process (longer messages are ignored) |

### System prompt

The LLM system prompt lives in `system_prompt.txt` rather than being baked into
the binary. It is read at startup from the working directory in release builds
(i.e. the systemd `WorkingDirectory`, `/var/lib/summarizer-bot/`), so you can
tweak the prompt and apply it with a service restart — no rebuild required:

```bash
sudo nano /var/lib/summarizer-bot/system_prompt.txt
sudo systemctl restart summarizer-bot
```

In debug builds the file is read from the crate directory
(`summarizer-bot/system_prompt.txt`) for convenient local development.

## Building

From the workspace root:

```bash
cargo build --release -p summarizer-bot
```

The compiled binary will be at `target/release/summarizer-bot`.

## Deployment

### 1. Copy files to the server

```bash
scp target/release/summarizer-bot user@server:/opt/summarizer-bot/
scp .env user@server:/opt/summarizer-bot/
```

### 2. Set up the systemd service

Copy and customize the service file:

```bash
scp summarizer-bot.service.example user@server:/tmp/
```

On the server, edit and install the service:

```bash
sudo cp /tmp/summarizer-bot.service.example /etc/systemd/system/summarizer-bot.service
sudo nano /etc/systemd/system/summarizer-bot.service
```

Update the `User` and `WorkingDirectory` fields to match your setup.

### 3. Enable and start the service

```bash
sudo systemctl daemon-reload
sudo systemctl enable summarizer-bot
sudo systemctl start summarizer-bot
```

### 4. Check status

```bash
sudo systemctl status summarizer-bot
sudo journalctl -u summarizer-bot -f
```
