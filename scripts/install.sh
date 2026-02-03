#!/bin/bash
set -euo pipefail

# Install script for Discord bot systemd service
# Usage: ./scripts/install.sh <bot-name> <ssh-host> <user>

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
TEMPLATE_FILE="$ROOT_DIR/bot.service.template"

usage() {
    echo "Usage: $0 <bot-name> <ssh-host> <user>"
    echo ""
    echo "Arguments:"
    echo "  bot-name   Name of the bot (must exist as a directory in the workspace)"
    echo "  ssh-host   SSH host (e.g., pi@raspberrypi.local)"
    echo "  user       User to run the service as on the remote host"
    echo ""
    echo "Example:"
    echo "  $0 cleanup-bot pi@raspberrypi.local pi"
    exit 1
}

# Check arguments
if [ $# -ne 3 ]; then
    usage
fi

BOT_NAME="$1"
SSH_HOST="$2"
USER="$3"

# Validate bot exists in workspace
if [ ! -d "$ROOT_DIR/$BOT_NAME" ]; then
    echo "Error: Bot '$BOT_NAME' not found in workspace"
    echo "Available bots:"
    for dir in "$ROOT_DIR"/*/; do
        if [ -f "$dir/Cargo.toml" ]; then
            echo "  - $(basename "$dir")"
        fi
    done
    exit 1
fi

# Check template exists
if [ ! -f "$TEMPLATE_FILE" ]; then
    echo "Error: Service template not found at $TEMPLATE_FILE"
    exit 1
fi

echo "Installing $BOT_NAME service on $SSH_HOST..."

# Generate service file from template
SERVICE_FILE="/tmp/${BOT_NAME}.service"
sed -e "s/{{BOT_NAME}}/$BOT_NAME/g" -e "s/{{USER}}/$USER/g" "$TEMPLATE_FILE" > "$SERVICE_FILE"

echo "Generated service file:"
cat "$SERVICE_FILE"
echo ""

# Copy service file to remote
echo "Copying service file to remote host..."
scp "$SERVICE_FILE" "$SSH_HOST:/tmp/"

# Create data directory and install service
echo "Installing service on remote host..."
ssh "$SSH_HOST" "sudo mkdir -p /var/lib/$BOT_NAME && \
    sudo chown $USER:$USER /var/lib/$BOT_NAME && \
    sudo mv /tmp/${BOT_NAME}.service /etc/systemd/system/ && \
    sudo systemctl daemon-reload && \
    sudo systemctl enable $BOT_NAME"

# Clean up local temp file
rm "$SERVICE_FILE"

echo ""
echo "Installation complete!"
echo ""
echo "The service is enabled but not started. To start it:"
echo "  ssh $SSH_HOST sudo systemctl start $BOT_NAME"
echo ""
echo "To check status:"
echo "  ssh $SSH_HOST systemctl status $BOT_NAME"
