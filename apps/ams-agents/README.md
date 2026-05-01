# arpsci-dashboard

Under heavy development. 

Agents Research Platform for HCI and Cognitive Sciences (Dashboard).

- Multi-agent conversations
- Interactive prompt design
- Reproducibility of inference
- Local and field-first architecture


### Building

```sh
# One-time vault: Writes `runs/.master_hash` (PHC Argon2id hash)
# Ubuntu 22
cargo run --bin gen_master_hash

# Windows 11
$env:CARGO_TARGET_DIR="target-hash-win11"; cargo run --bin gen_master_hash

# Development: run the application (`target/debug/`)
cargo run

# Development: run with embedded web server (`target/debug/`)
AMS_WEB_ENABLED=true cargo run
# http://127.0.0.1:8000/api/health
# http://127.0.0.1:8000/api/outgoing-http-log

# Distribution: build the application ('target/release/')
cargo build --release

```

### Metrics

[README.md](../../metrics/README.md)

### Docs

[README.md](../../docs/README.md)

### Tests

[README.md](./tests/README.md)
