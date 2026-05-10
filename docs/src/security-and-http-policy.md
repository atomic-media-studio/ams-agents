# Security and HTTP Policy

## Vault gate

The desktop app starts behind `MasterVault` unless vault skipping is explicitly enabled for development.

The unlock gate accepts a PHC-format Argon2id hash from either:

- `ARPSCI_MASTER_HASH`, or
- the first line of `runs/.master_hash`.

Important current behavior:

- `ARPSCI_SKIP_VAULT` is treated as enabled only when it equals `1`.
- if no stored hash is configured, the unlock UI stays on screen and tells the operator how to provide one.
- the UI adds a top lock bar after unlock so the workspace can be re-locked without restarting the app.

## Encrypted in-memory vault blob

The internal `Vault` type is a small encrypted blob container, not a general persistence layer. It derives encryption material from the master password using:

- Argon2id,
- HKDF-SHA256 expansion,
- ChaCha20-Poly1305 for authenticated encryption.

Argon2 parameters are configurable through `ARPSCI_ARGON2_M_KIB`, `ARPSCI_ARGON2_T`, and `ARPSCI_ARGON2_P`.

## Outbound HTTP policy

Outbound HTTP is guarded by `HttpPolicy` in `src/web/mod.rs`. The policy has two flags:

- `air_gap_enabled`
- `allow_local_ollama`

When air-gap mode is disabled, outbound webhook traffic is allowed normally.

When air-gap mode is enabled:

- generic outbound HTTP is allowed only to loopback hosts,
- Ollama requests are allowed only when `allow_local_ollama` is also enabled and the Ollama host resolves to loopback,
- blocked attempts are written to the run ledger as `transport.http_blocked`.

The loopback check currently treats `localhost` and loopback IP literals as local.

## Embedded server and webhook separation

The current code keeps inbound and outbound web features separate.

- `ARPSCI_WEB_ENABLED` controls the embedded Rocket server.
- `ARPSCI_WEBHOOKS_ENABLED` controls outbound webhook POSTs.

Enabling Rocket does not automatically enable outbound posts, and disabling webhooks does not disable the local API.

## Embedded API surface

When enabled, Rocket mounts three routes under `/api`:

- `/health`
- `/outgoing-http-log`
- `/outgoing-http-log/live`

The log endpoints expose the in-memory outgoing HTTP log used for operator visibility. They do not replace the run ledger; they complement it with a live view.
