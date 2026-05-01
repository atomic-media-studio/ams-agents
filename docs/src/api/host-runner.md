# Host Runner API

Host runner is a host-side FastAPI service used by platform to compile and
control Rust applications outside containers.

Routes:

- `GET /health`
- `GET /rust/app/status`
- `POST /rust/app/compile`
- `POST /rust/app/start`
- `POST /rust/app/stop`

Reference:

::: host_runner_main
