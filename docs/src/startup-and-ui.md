# App Control Flow

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

- **Compile** runs `cargo build -p ams-agents --target-dir target` in `apps/ams-agents`.
- **Start** launches `apps/ams-agents/target/debug/ams-agents`.
- **Stop** terminates the app process group.
- **Status** returns running state, PID, target dir, and log tail.

## Common failure mode

If dashboard shows `host-runner-unreachable`, host runner is usually bound to
`127.0.0.1`. Bind it to `0.0.0.0` so Docker can reach it via
`host.docker.internal`.
