#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$ROOT_DIR/apps/arpsci"
HASH_FILE="$APP_DIR/runs/.master_hash"

cd "$APP_DIR"

echo "[run_dev_app] Working directory: $APP_DIR"
echo

# Fast compile check (all bins and tests)
echo "[run_dev_app] Running cargo check --all-targets ..."
cargo check --all-targets
echo "[run_dev_app] Check passed."
echo

# Generate master hash if it does not exist yet
if [[ ! -f "$HASH_FILE" ]]; then
	echo "[run_dev_app] $HASH_FILE not found."
	read -r -p "  Enter a password to hash and write to runs/.master_hash: " -s password
	echo
	if [[ -z "$password" ]]; then
		echo "[run_dev_app] No password provided — skipping hash generation."
	else
		mkdir -p "$APP_DIR/runs"
		ARPSCI_HASH_PASSWORD="$password" cargo run --bin gen_master_hash
		echo "[run_dev_app] Hash written to $HASH_FILE"
	fi
else
	echo "[run_dev_app] Master hash already exists at $HASH_FILE — skipping generation."
fi
echo

# Run app with embedded Rocket API enabled
echo "[run_dev_app] Starting arpsci (ARPSCI_WEB_ENABLED=true) ..."
ARPSCI_WEB_ENABLED=true cargo run

