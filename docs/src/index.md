# ARPSCI Platform and App Control Docs

This documentation is intentionally scoped to platform API and app lifecycle control.

Current focus:

- FastAPI platform routes exposed on `:8080`
- host runner routes exposed on `:8090`
- compile/start/stop/status flow for `apps/ams-agents`
- Docker-to-host networking requirements for app control

Out of scope for this doc set:

- conversation internals,
- runtime implementation deep dives,
- legacy Rust Bridge route usage.

Use these pages in order:

1. App Control Architecture
2. App Control Flow
3. Platform Routes
4. Host Runner API
5. OpenAPI Explorer
