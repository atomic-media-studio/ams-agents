# ARPSCI Ecosystem

Under heavy development. 

Monorepo for the ARPSCI-Ecosystem.

## Repository layout

- `apps/ams-agents`: Existing Rust dashboard app (moved as-is).
- `platform`: Dockerized FastAPI platform managed with `uv`.
- `tests`: Root integration/e2e tests.
- `binaries`: Staging directory for compiled Rust binaries.
- `docs`: Architecture and ecosystem runbooks.
- `metrics`, `runs`, `runtimes`: Existing runtime data directories kept in place.

# Reproduce

```sh
# Terminal 1:
cd ./arp/platform
uv sync --dev
uv run uvicorn src.host_runner_main:app --host 127.0.0.1 --port 8090

# Terminal 2:
cd ./arp
docker compose up --build

# Visit:
# http://localhost:8080/ (platform dashboard)
# http://localhost:8081/ (docs)
```


## Rust application

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

The dashboard lets you inspect platform health and compile / start / stop the Rust app from the browser.

## Python platform (FastAPI + uv)

```sh
cd platform
uv sync --dev
uv run uvicorn src.main:app --reload --host 127.0.0.1 --port 8080
```

Platform endpoints:

- `http://127.0.0.1:8080/`
- `http://127.0.0.1:8080/api/health`
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

For Rust app compile/start/stop from the dashboard while platform runs in Docker,
run the host runner on your machine (not in a container):

```sh
cd platform
uv sync --dev
uv run uvicorn src.host_runner_main:app --host 0.0.0.0 --port 8090
```

Host runner endpoint:

- `http://127.0.0.1:8090/health`

Why `0.0.0.0`? Docker reaches host services via `host.docker.internal`, not host
loopback (`127.0.0.1`). If host runner is bound only to `127.0.0.1`, the platform
container cannot connect.

Bring up the platform and docs site together:

```sh
docker compose up --build
```

Or individually:

```sh
docker compose up --build platform   # FastAPI platform
docker compose up --build docs       # MkDocs documentation
```

Visit in browser:

- Platform dashboard: `http://localhost:8080/`
- API docs (OpenAPI): `http://localhost:8080/docs`
- Documentation site: `http://localhost:8081/`

With Docker, platform app-control routes are proxied to the host runner
(`ARP_RUST_APP_RUNNER_BASE_URL=http://host.docker.internal:8090`), so Rust apps
compile and run on the host.

When started from the host runner, the Rust app is launched with
`ROCKET_ADDRESS=0.0.0.0` so the platform container can reach
`http://host.docker.internal:8000`.

To build and serve docs locally without Docker:

```sh
cd platform
uv sync --dev
uv run mkdocs serve --config-file ../docs/mkdocs.yml
```

Docs will be available at `http://127.0.0.1:8000/`.
