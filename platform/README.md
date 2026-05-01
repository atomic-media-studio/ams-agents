# ARP Platform

FastAPI platform service for the ARP ecosystem.

## Runtime model

- The Rust app (`apps/ams-agents`) exposes local Rocket APIs.
- This platform calls Rocket via HTTP using `httpx`.
- No subprocess bridge is used in this first implementation.

## Local development with uv

```sh
cd platform
uv sync --dev
uv run uvicorn src.main:app --reload --host 127.0.0.1 --port 8080
```

Set the Rocket base URL if needed:

```sh
export ARP_ROCKET_BASE_URL=http://127.0.0.1:8000
```

## API

- `GET /api/health`
- `GET /api/rust/health`
- `GET /api/rust/capabilities`
- `POST /api/rust/bridge/ping`
