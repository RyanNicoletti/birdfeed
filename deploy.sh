#!/bin/bash
set -e

if [ -f .env ]; then
    source .env
else
    echo "Error: .env not found"
    exit 1
fi

ssh -p $PI_PORT $PI_USER@$PI_HOST "sudo systemctl stop birdfeed"

echo "Building release binary..."
cargo build --release --target aarch64-unknown-linux-gnu

echo "Copying binary to Pi..."
scp -P $PI_PORT target/aarch64-unknown-linux-gnu/release/birdfeed $PI_USER@$PI_HOST:$APP_DIR/

rsync -avz -e "ssh -p $PI_PORT" --exclude 'target' --exclude '.git' migrations/ $PI_USER@$PI_HOST:$APP_DIR/migrations/

scp -P $PI_PORT .env.production $PI_USER@$PI_HOST:$APP_DIR/.env

echo "Restarting service..."
ssh -p $PI_PORT $PI_USER@$PI_HOST "sudo systemctl start birdfeed"

echo "Deployment complete!"
