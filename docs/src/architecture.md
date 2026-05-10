# ARPSCI Architecture

The platform now follows a two-service control model:

1. **Platform API** (`platform/src/main.py`, port `8080`)
2. **Host Runner API** (`platform/src/host_runner_main.py`, port `8090`)

The platform serves the web UI and proxies app lifecycle actions to the host runner.
The host runner executes `cargo` and the Rust app process on the host machine.

## Components

- `platform/src/api/routes.py`
	- Public API for dashboard/UI.
	- Exposes `/api/health` and `/api/rust/app/*`.
	- Proxies lifecycle calls to host runner when `ARP_RUST_APP_RUNNER_BASE_URL` is set.
- `platform/src/api/rust_app_runner.py`
	- Process manager for `arpsci`.
	- Compiles in `apps/arpsci` using local target dir.
	- Starts/stops the binary and returns status/log tail.
- `platform/src/host_runner_main.py`
	- Host-only API for `/rust/app/status|compile|start|stop`.
	- Used when platform runs in Docker but app must run on host.
- `docker-compose.yml`
	- Runs `platform` and `docs` containers.
	- Maps `host.docker.internal` to host gateway.

## Network model

- Browser -> `localhost:8080` -> platform container
- Platform container -> `host.docker.internal:8090` -> host runner
- Host runner -> local filesystem/processes -> `apps/arpsci`

When host runner starts the Rust app, it sets:

- `ARPSCI_WEB_ENABLED=true`
- `ROCKET_ADDRESS=0.0.0.0`

This allows the platform container to reach Rust app HTTP endpoints on host port `8000` if needed.




## App Control Flow

## Local development flow

1. Start host runner on host:

   ```sh
   cd platform
   uv sync --dev
   uv run uvicorn src.host_runner_main:app --host 0.0.0.0 --port 8090
   ```

2. Start platform and docs in Docker:

   ```sh
   docker compose up --build
   ```

3. Open dashboard at `http://localhost:8080/`.

## Dashboard lifecycle actions

The dashboard uses these platform routes:

- `GET /api/rust/app/status`
- `POST /api/rust/app/compile`
- `POST /api/rust/app/start`
- `POST /api/rust/app/stop`

Platform forwards those calls to host runner.

## Runtime behavior

- **Compile** runs `cargo build -p arpsci --target-dir target` in `apps/arpsci`.
- **Start** launches `apps/arpsci/target/debug/arpsci`.
- **Stop** terminates the app process group.
- **Status** returns running state, PID, target dir, and log tail.

## Common failure mode

If dashboard shows `host-runner-unreachable`, host runner is usually bound to
`127.0.0.1`. Bind it to `0.0.0.0` so Docker can reach it via
`host.docker.internal`.
