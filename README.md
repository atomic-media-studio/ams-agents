# arpsci-dashboard

Agents Research Platform for HCI and Cognitive Sciences (Dashboard).



### Overview

- Multi-agent conversations
- Prompt design and injection
- Reproducibility of inference
- Local and field-first architecture


### Main Dependencies


- rust-adk 
    - (adk-agent, adk-model, adk-runner, adk-session, adk-core)
- eframe
- egui-phosphor


### Building

```sh
# One-time vault: interactive prompt writes `runs/.master_hash` (PHC Argon2id hash for the password gate)
cargo run --bin gen_master_hash
# For Windows 11 on a separate target/ dir
# $env:CARGO_TARGET_DIR="target-hash-win11"; cargo run --bin gen_master_hash


# Development: run the application (`target/debug/`)
cargo run

# Development: run with embedded web server (`target/debug/`)
AMS_WEB_ENABLED=true cargo run
# http://127.0.0.1:8000/api/health
# http://127.0.0.1:8000/api/outgoing-http-log

# Distribution: build the application ('target/release/')
cargo build --release
```


### Communication and Security

- **Vault:** master password verification uses **Argon2id** with non-trivial defaults (`m=65536 KiB`, `t=3`, `p<=4`), persisted as PHC hash in `runs/.master_hash` (or `AMS_MASTER_HASH`). KDF tuning: `AMS_ARGON2_M_KIB`, `AMS_ARGON2_T`, `AMS_ARGON2_P`.
- **Vault key derivation + encryption:** vault payload keys are derived separately from the master password (`Argon2id + HKDF-SHA256`) and encrypted with AEAD (`ChaCha20-Poly1305`) using random salt/nonce and versioned metadata.
- **Outbound HTTP:** JSON bodies are `POST`ed to `CONVERSATION_HTTP_ENDPOINT` (default `http://localhost:3000/`) unless air-gap mode is enabled.
- **Air-gap mode:** set `AMS_AIR_GAP=1` to block non-loopback outbound HTTP; optional `AMS_ALLOW_LOCAL_OLLAMA=0` also blocks local Ollama requests. Blocked attempts are mirrored to the run ledger as `transport.http_blocked` events.

### Reproducibility

- The `./runs/` folder is the application persisted state and run history:
    1) a workspace snapshot you can load/save outside a run,
    2) per-experiment/per-run execution artifacts


### Timing and Tracing

- Tracing is opt-in and captures Ollama inference timings to JSONL for offline research.
- Captured fields include `t_start`, `t_first_token` (when streaming yields text), `t_end`, `duration_ms`, and `ttft_ms`.
- Inter-turn pacing is also recorded (`turn_timing`) with `gap_ms` between turns.
- Default output file: `metrics/timings.jsonl`.
- The `metrics/` folder is intended for local artifacts and is excluded from git.

Configuration options:

- UI: Settings > Reproducibility > Timing and Tracing
    - Enable/disable tracing
    - Set output JSONL file path
- Env vars:
    - `AMS_TRACING_ENABLED=1`
    - `AMS_TRACING_FILE=metrics/timings.jsonl`

Sample JSONL records:

```json
{"event_type":"inference_timing","source":"dialogue.turn","duration_ms":1289,"ttft_ms":214}
{"event_type":"turn_timing","turn_index":4,"speaker_name":"Agent A","receiver_name":"Agent B","gap_ms":411}
```


### Python Runtimes

You can create and manage isolated Python virtual environments per experiment via the **Python** tab.

- Create a venv from any system interpreter, install packages into it, and destroy it when done.
- Each runtime is stored under `runtimes/python/{id}/` and tracked in `python_runtimes.json`.
- `pip freeze` is captured on creation for reproducibility; execution is fully traceable via the run ledger.

Install NumPy on cursom venv:  

```python
import numpy as np
print(np.arange(6).reshape(2, 3))
```
