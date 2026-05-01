# ARPSCI-Ecosystem

Monorepo for the ARPSCI-Ecosystem.

## Repository layout

- `apps/ams-agents`: Existing Rust dashboard app (moved as-is).
- `platform`: Dockerized FastAPI platform managed with `uv`.
- `tests`: Root integration/e2e tests.
- `binaries`: Staging directory for compiled Rust binaries.
- `docs`: Architecture and ecosystem runbooks.
- `metrics`, `runs`, `runtimes`: Existing runtime data directories kept in place.

## Rust app (ams-agents)

```sh
# Build workspace
cargo build --workspace

# One-time setup: create runs/.master_hash
cargo run -p ams-agents --bin gen_master_hash

# Run Rust app with embedded Rocket API
AMS_WEB_ENABLED=true cargo run -p ams-agents
```

Rocket endpoints:

- `http://127.0.0.1:8000/api/health`
- `http://127.0.0.1:8000/api/capabilities`
- `http://127.0.0.1:8000/api/bridge/ping`

## Dashboard

Once the platform is running, open the dashboard at:

```
http://127.0.0.1:8080/
```

The dashboard lets you inspect service health, view and trigger all API endpoints, and compile / start / stop the Rust app from the browser.

## Python platform (FastAPI + uv)

```sh
cd platform
uv sync --dev
uv run uvicorn src.main:app --reload --host 127.0.0.1 --port 8080
```

Platform endpoints:

- `http://127.0.0.1:8080/`
- `http://127.0.0.1:8080/api/health`
- `http://127.0.0.1:8080/api/rust/health`
- `http://127.0.0.1:8080/api/rust/capabilities`
- `http://127.0.0.1:8080/api/rust/bridge/ping`
- `http://127.0.0.1:8080/api/rust/app/status`
- `POST http://127.0.0.1:8080/api/rust/app/compile`
- `POST http://127.0.0.1:8080/api/rust/app/start`
- `POST http://127.0.0.1:8080/api/rust/app/stop`

Rust app control notes:

- Compile uses `apps/ams-agents` and writes to `apps/ams-agents/target`.
- Start launches `apps/ams-agents/target/debug/ams-agents` with `AMS_WEB_ENABLED=true`.

Dashboard static files:

- `platform/src/ui/index.html`
- `platform/src/ui/styles.css`
- `platform/src/ui/app.js`

## Docker (ongoing)

```sh
docker compose up --build platform
```

See `docs/ecosystem.md` and `docs/architecture.md` for details.
