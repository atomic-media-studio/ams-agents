# ARP Ecosystem

## Development startup flow

1. Start the Rust app with embedded Rocket API:

```sh
AMS_WEB_ENABLED=true cargo run -p ams-agents
```

2. Start the FastAPI platform:

```sh
cd platform
uv sync --dev
uv run uvicorn src.main:app --reload --host 127.0.0.1 --port 8080
```

3. Validate bridge calls:

- `GET http://127.0.0.1:8080/api/rust/health`
- `GET http://127.0.0.1:8080/api/rust/capabilities`
- `POST http://127.0.0.1:8080/api/rust/bridge/ping`

## Docker

From repo root:

```sh
docker compose up --build platform
```

The platform container calls Rocket at `host.docker.internal:8000` by default.
