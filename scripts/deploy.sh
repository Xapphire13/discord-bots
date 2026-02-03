#!/bin/bash
set -euo pipefail

# Deploy a Discord bot to a remote host via SSH

usage() {
    echo "Usage: $0 <bot-name> <ssh-host>"
    echo "Example: $0 cleanup-bot pi@raspberrypi.local"
    exit 1
}

# Validate inputs
if [[ $# -ne 2 ]]; then
    usage
fi

BOT_NAME="$1"
SSH_HOST="$2"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

# Check bot exists in workspace
if [[ ! -d "$REPO_ROOT/$BOT_NAME" ]]; then
    echo "Error: Bot '$BOT_NAME' not found in workspace"
    echo "Available bots:"
    ls -d "$REPO_ROOT"/*-bot 2>/dev/null | xargs -n1 basename || echo "  (none)"
    exit 1
fi

# Determine config files based on bot
get_config_files() {
    case "$1" in
        cleanup-bot)
            echo ".env config.toml"
            ;;
        summarizer-bot)
            echo ".env"
            ;;
        *)
            echo ".env"
            ;;
    esac
}

CONFIG_FILES=$(get_config_files "$BOT_NAME")
TARGET="aarch64-unknown-linux-gnu"
BINARY_PATH="$REPO_ROOT/target/$TARGET/release/$BOT_NAME"

echo "Deploying $BOT_NAME to $SSH_HOST"

# Step 1: Cross-compile
echo "Building $BOT_NAME for $TARGET..."
cargo build --release -p "$BOT_NAME" --target "$TARGET"

if [[ ! -f "$BINARY_PATH" ]]; then
    echo "Error: Binary not found at $BINARY_PATH"
    exit 1
fi

# Step 2: Copy files to remote temp directory
echo "Copying files to remote..."
ssh "$SSH_HOST" "rm -rf /tmp/$BOT_NAME && mkdir -p /tmp/$BOT_NAME"
scp "$BINARY_PATH" "$SSH_HOST:/tmp/$BOT_NAME/"

for config_file in $CONFIG_FILES; do
    if [[ -f "$REPO_ROOT/$BOT_NAME/$config_file" ]]; then
        if ssh "$SSH_HOST" "test -f /opt/$BOT_NAME/$config_file"; then
            echo "  Skipping $config_file (already exists on remote)"
        else
            echo "  Copying $config_file"
            scp "$REPO_ROOT/$BOT_NAME/$config_file" "$SSH_HOST:/tmp/$BOT_NAME/"
        fi
    fi
done

# Step 3: Stop service
echo "Stopping $BOT_NAME service..."
ssh "$SSH_HOST" "sudo systemctl stop $BOT_NAME" || echo "  (service may not have been running)"

# Step 4: Install files
echo "Installing files to /opt/$BOT_NAME/..."
ssh "$SSH_HOST" "sudo mkdir -p /opt/$BOT_NAME && sudo mv /tmp/$BOT_NAME/* /opt/$BOT_NAME/"

# Step 5: Start service
echo "Starting $BOT_NAME service..."
ssh "$SSH_HOST" "sudo systemctl start $BOT_NAME"

echo "Deployment complete!"
echo "Check status with: ssh $SSH_HOST systemctl status $BOT_NAME"
