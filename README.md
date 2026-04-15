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
