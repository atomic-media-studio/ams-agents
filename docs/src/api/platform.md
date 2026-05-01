# Platform API Routes

The platform exposes a minimal API surface used by the web dashboard.

Current routes:

- `GET /api/health`
- `GET /api/rust/app/status`
- `POST /api/rust/app/compile`
- `POST /api/rust/app/start`
- `POST /api/rust/app/stop`

These routes live in `platform/src/api/routes.py` and proxy lifecycle operations
to host runner when `ARP_RUST_APP_RUNNER_BASE_URL` is configured.

::: api.routes
