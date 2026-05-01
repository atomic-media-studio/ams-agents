# Ecosystem Architecture

This file documents the monorepo architecture introduced in issue #29.

## High-level layout

- `apps/ams-agents`: Rust application (current app, moved as-is).
- `platform`: Python FastAPI platform using `uv` for dependency/runtime management.
- `tests`: Root integration/e2e test area.
- `binaries`: Build output staging for compiled Rust artifacts.

## Rust to Python interaction model

- The Rust app exposes local Rocket endpoints under `/api/*`.
- The Python platform calls Rocket over HTTP via `httpx`.
- The first bridge contract includes:
  - `GET /api/health`
  - `GET /api/capabilities`
  - `POST /api/bridge/ping`

This keeps both runtimes decoupled and testable independently.
