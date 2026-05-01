# Root Tests

This folder contains cross-application integration tests for the ARP ecosystem.

## Current coverage

- Python platform bridge contract tests (FastAPI and bridge behavior).
- End-to-end HTTP flow tests should be added here as the Rocket API surface grows.

## Run

```sh
cd platform
uv sync --dev
uv run pytest ../tests/python
```
