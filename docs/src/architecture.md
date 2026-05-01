# App Control Architecture

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
	- Process manager for `ams-agents`.
	- Compiles in `apps/ams-agents` using local target dir.
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
- Host runner -> local filesystem/processes -> `apps/ams-agents`

When host runner starts the Rust app, it sets:

- `AMS_WEB_ENABLED=true`
- `ROCKET_ADDRESS=0.0.0.0`

This allows the platform container to reach Rust app HTTP endpoints on host port `8000` if needed.
