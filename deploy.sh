#!/bin/bash
set -euo pipefail

if [ -f .env ]; then
  # shellcheck disable=SC1091
  source .env
else
  echo "Error: .env not found"
  exit 1
fi

: "${PI_USER:?set PI_USER in .env}"
: "${PI_HOST:?set PI_HOST in .env}"
: "${PI_PORT:?set PI_PORT in .env}"
: "${APP_DIR:?set APP_DIR in .env}"

SSH="ssh -p ${PI_PORT} ${PI_USER}@${PI_HOST}"
SSH_TTY="ssh -t -p ${PI_PORT} ${PI_USER}@${PI_HOST}"

echo "==> Ensuring app directory exists..."
$SSH "mkdir -p ${APP_DIR}/db"

echo "==> Syncing source to Pi..."
rsync -avz --delete -e "ssh -p ${PI_PORT}" \
  --exclude '.git' \
  --exclude '.venv' \
  --exclude 'target' \
  --exclude '__pycache__' \
  --exclude '*.db' --exclude '*.db-shm' --exclude '*.db-wal' \
  --exclude '.env' --exclude '.env.production' \
  src pyproject.toml README.md birdfeed.service \
  "${PI_USER}@${PI_HOST}:${APP_DIR}/"

echo "==> Installing production environment file..."
scp -P "${PI_PORT}" .env.production "${PI_USER}@${PI_HOST}:${APP_DIR}/.env"

echo "==> Creating/updating virtualenv and installing package (editable)..."
# Editable install so an rsync of src/ updates what the service serves and
# imports; a plain `pip install .` copies into site-packages and would be
# skipped on redeploys when the version is unchanged.
$SSH "cd ${APP_DIR} && \
  (test -d .venv || python3 -m venv .venv) && \
  .venv/bin/pip install --upgrade pip --quiet && \
  .venv/bin/pip install -e . --quiet && \
  .venv/bin/birdfeed init-db"

echo "==> Privileged cutover (sudo will prompt for your password)..."
$SSH_TTY "sudo cp ${APP_DIR}/birdfeed.service /etc/systemd/system/birdfeed.service && \
  sudo systemctl daemon-reload && \
  sudo systemctl enable birdfeed && \
  sudo systemctl restart birdfeed && \
  sleep 2 && \
  sudo systemctl --no-pager status birdfeed | head -n 12"

echo "Deployment complete!"
