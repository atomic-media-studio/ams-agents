#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

cargo build --workspace --release

mkdir -p binaries
cp target/release/ams-agents binaries/ 2>/dev/null || true
cp target/release/gen_master_hash binaries/ 2>/dev/null || true
cp target/release/timings_report binaries/ 2>/dev/null || true

echo "Build complete. Binaries copied to ./binaries"
