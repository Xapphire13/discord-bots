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
            echo ".env system_prompt.txt"
            ;;
        *)
            echo ".env"
            ;;
    esac
}

# Files that should always be (re)deployed, overwriting the remote copy, because
# they are loaded from disk at runtime and must reflect changes from the repo.
# Other config files are preserved on the remote if they already exist.
get_always_overwrite_files() {
    case "$1" in
        summarizer-bot)
            echo "system_prompt.txt"
            ;;
        *)
            echo ""
            ;;
    esac
}

CONFIG_FILES=$(get_config_files "$BOT_NAME")
ALWAYS_OVERWRITE_FILES=$(get_always_overwrite_files "$BOT_NAME")
TARGET="aarch64-unknown-linux-gnu"
BINARY_PATH="$REPO_ROOT/target/$TARGET/release/$BOT_NAME"
BUILDER_IMAGE="discord-bots-builder"
CARGO_CACHE_VOLUME="discord-bots-cargo-registry"

echo "Deploying $BOT_NAME to $SSH_HOST"

# Preflight: podman must be installed
if ! command -v podman >/dev/null 2>&1; then
    echo "Error: podman is not installed."
    echo "Install it with: brew install podman && podman machine init"
    exit 1
fi

# Ensure the podman machine is running. `podman info` succeeds only when the
# backing machine is up (on a native Linux host it always succeeds, so this is a
# no-op there). If we have to start the machine, stop it again on exit so we
# leave the host as we found it.
STARTED_PODMAN_MACHINE=false
stop_podman_machine() {
    if [[ "$STARTED_PODMAN_MACHINE" == true ]]; then
        echo "Stopping podman machine..."
        podman machine stop || true
    fi
}
trap stop_podman_machine EXIT

if ! podman info >/dev/null 2>&1; then
    echo "Starting podman machine..."
    podman machine start
    STARTED_PODMAN_MACHINE=true
fi

# Step 1: Build inside a Linux container matching the target (Debian bookworm /
# glibc 2.36). On an Apple Silicon Mac the podman machine is aarch64 Linux, so
# this is a native build for aarch64-unknown-linux-gnu — no cross toolchain.
# The repo is bind-mounted, so the binary lands at $BINARY_PATH on the host just
# as a local `cargo build` would; the cargo registry is cached across runs.
echo "Building $BOT_NAME for $TARGET in container..."
podman build -t "$BUILDER_IMAGE" -f "$REPO_ROOT/Containerfile.build" "$REPO_ROOT"
podman run --rm \
    -v "$REPO_ROOT":/src \
    -v "$CARGO_CACHE_VOLUME":/usr/local/cargo/registry \
    "$BUILDER_IMAGE" \
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
        # Files in ALWAYS_OVERWRITE_FILES are loaded at runtime and must always
        # reflect the repo, so copy them even if they already exist on the remote.
        if [[ " $ALWAYS_OVERWRITE_FILES " != *" $config_file "* ]] && ssh "$SSH_HOST" "test -f /var/lib/$BOT_NAME/$config_file"; then
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
echo "Installing binary to /usr/local/bin/..."
ssh "$SSH_HOST" "sudo mv /tmp/$BOT_NAME/$BOT_NAME /usr/local/bin/"

echo "Installing config files to /var/lib/$BOT_NAME/..."
ssh "$SSH_HOST" "if ls /tmp/$BOT_NAME/* >/dev/null 2>&1; then sudo mv /tmp/$BOT_NAME/* /var/lib/$BOT_NAME/; fi"

# Step 5: Start service
echo "Starting $BOT_NAME service..."
ssh "$SSH_HOST" "sudo systemctl start $BOT_NAME"

echo "Deployment complete!"
echo "Check status with: ssh $SSH_HOST systemctl status $BOT_NAME"
