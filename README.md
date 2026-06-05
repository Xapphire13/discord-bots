# Discord Bots

A Rust workspace monorepo containing Discord bots.

## Bots

- **[cleanup-bot](./cleanup-bot/)** - A Discord bot that automatically deletes old messages based on configurable retention policies
- **[summarizer-bot](./summarizer-bot/)** - A Discord bot that summarizes conversations using Ollama

## Building

```bash
# Build all bots
cargo build --release

# Build a specific bot
cargo build --release -p summarizer-bot
```

## Deployment

### Initial Setup

Before deploying a bot for the first time, run the install script to configure the systemd service:

```bash
./scripts/install.sh <bot-name> <ssh-host> <user>

# Example
./scripts/install.sh cleanup-bot pi@raspberrypi.local pi
```

This script:

- Creates the `/var/lib/<bot-name>/` directory on the remote host (owned by service user)
- Installs a systemd service file for the bot
- Enables the service to start on boot

You only need to run this once per bot. After that, use `deploy.sh` for updates.

### Deploying Updates

Deploy a bot to a Raspberry Pi (or other aarch64 Linux host) using the deploy script:

```bash
./scripts/deploy.sh <bot-name> <ssh-host>

# Examples
./scripts/deploy.sh cleanup-bot pi@raspberrypi.local
./scripts/deploy.sh summarizer-bot pi@192.168.1.100
```

### Prerequisites

- [Podman](https://podman.io/) installed and its machine running:
  `brew install podman && podman machine init && podman machine start`
- SSH access to the target host (key-based authentication recommended)
- A systemd service configured on the target host for each bot (run `install.sh` first)

The deploy build runs inside a Debian bookworm container (matching the target's
OS and glibc) rather than on the local host, so no Rust toolchain or
cross-compilation target is required on the build machine. On an Apple Silicon
Mac this is a native aarch64 Linux build; the podman machine is itself a
lightweight Linux VM managed by podman.

### What the script does

1. Builds the bot for `aarch64-unknown-linux-gnu` inside a Debian bookworm container
2. Copies the binary and config files to the remote host
3. Stops the systemd service
4. Installs the binary to `/usr/local/bin/<bot-name>`
5. Installs config files to `/var/lib/<bot-name>/`
6. Starts the systemd service

## Adding a New Bot

1. Create a new directory for your bot (e.g., `my-bot/`)
2. Add a `Cargo.toml` with your bot's dependencies
3. Add the bot to the workspace members in the root `Cargo.toml`:

```toml
[workspace]
members = ["...", "my-bot"]
```

## License

MIT License - see [LICENSE](LICENSE) for details.
