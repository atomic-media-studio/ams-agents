# Configuration Reference

## Core environment variables

| Variable | Default | Description |
| --- | --- | --- |
| `OLLAMA_HOST` | `http://127.0.0.1:11434` | Base host used for Ollama inference and, unless overridden, model-tag lookup |
| `OLLAMA_MODEL` | unset in UI state; inference fallback `glm-4.7-flash:latest` | Default model used when no per-call override is provided |
| `OLLAMA_TAGS_URL` | derived from `OLLAMA_HOST + /api/tags` | Optional override for model-list discovery |
| `ARPSCI_OLLAMA_CONTEXT_WINDOW` | unset | Optional Ollama `num_ctx` value passed with inference requests |
| `ARPSCI_WEB_ENABLED` | `false` | Enables the embedded Rocket server |
| `ARPSCI_WEBHOOKS_ENABLED` | `false` | Enables outbound webhook POSTs for conversation, evaluator, and researcher events |
| `ARPSCI_WEB_ADDRESS` | `127.0.0.1` | Bind address for the embedded server |
| `ARPSCI_WEB_PORT` | `8000` | Bind port for the embedded server |
| `ARPSCI_CONVERSATION_HTTP_STREAM_ENABLED` | `false` | Sends conversation start and turn messages to the configured HTTP endpoint |
| `ARPSCI_CHAT_STREAM_ENABLED` | `true` | Forwards run messages into the active Overview chat room |
| `ARPSCI_AIR_GAP` | `false` | Blocks non-loopback outbound HTTP through the app policy layer |
| `ARPSCI_ALLOW_LOCAL_OLLAMA` | `true` | Allows loopback Ollama access when air-gap mode is enabled |
| `CONVERSATION_HTTP_ENDPOINT` | `http://localhost:3000/` | Webhook endpoint used by conversation and sidecar streaming |
| `ARPSCI_LOG_PLAY_PLAN` | `false` | Logs the resolved conversation play plan before execution |
| `ARPSCI_CONVERSATION_GROUP_SIZE` | `2` | Conversation group size used when partitioning eligible workers |
| `ARPSCI_RESEARCH_POLICY` | `inline` | Researcher scheduling policy: `off`, `inline`, or `background` |
| `ARPSCI_EVALUATOR_POLICY` | `inline` | Evaluator scheduling policy: `off`, `inline`, or `batched:N` |
| `ARPSCI_METRICS_FILE` | `metrics/timings.jsonl` | Path used by the metrics JSONL sink |
| `ARPSCI_MASTER_HASH` | unset | Argon2id PHC hash accepted by the vault unlock gate |
| `ARPSCI_ARGON2_M_KIB` | `65536` | Vault Argon2 memory cost in KiB |
| `ARPSCI_ARGON2_T` | `3` | Vault Argon2 time cost |
| `ARPSCI_ARGON2_P` | auto-clamped to `1..4` | Vault Argon2 parallelism |
| `ARPSCI_SKIP_VAULT` | disabled unless exactly `1` | Bypasses the vault gate for development |

## Runtime defaults not driven by env

Some important behavior is configured in code or through the UI instead of environment variables:

- dialogue history size defaults to `5` and is changed from the Settings panel,
- metrics capture starts enabled by default and can be disabled from Settings,
- the app window starts at `900x840`,
- the Catppuccin Latte theme is applied automatically on first frame.

## Boolean parsing notes

Most app booleans use the shared parser that accepts `1`, `true`, `yes`, and `on` as true, plus `0`, `false`, `no`, and `off` as false.

`ARPSCI_SKIP_VAULT` is the exception: the current vault code enables skip mode only when the variable is exactly `1`.

## Operational notes

- `ARPSCI_WEB_ENABLED` and `ARPSCI_WEBHOOKS_ENABLED` are independent.
- air-gap mode affects outbound HTTP guards, not whether the embedded Rocket server can bind locally.
- `ARPSCI_CONVERSATION_HTTP_STREAM_ENABLED` only has an effect when outbound webhooks are also enabled.
- changing metrics settings in the UI rebuilds the active metrics sink at runtime.
