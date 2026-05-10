cd ./arp/apps/arpsci

# Fast compile check (all bins and tests)
cargo check --all-targets

# user input, if doesnt exist
cargo run --bin gen_master_hash

# Run app with embedded Rocket API enabled
ARPSCI_WEB_ENABLED=true cargo run

