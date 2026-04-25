# ARPSCI Dashboard Docs

This folder contains project documentation for the ARPSCI Dashboard.

## What Is Here

- Narrative docs in Markdown for contributors and operators.
- mdBook scaffold for publishing these docs as a static site.
- Content reused by Rust API docs (`cargo doc`) via crate-level include.

## Build Code

```sh
cargo build
```

## Build Rust API Docs

```sh
cargo doc --no-deps
```

Generated docs are placed in `target/doc/`.

## Build Book Docs (mdBook)

Install mdBook once:

```sh
cargo install mdbook
```

Build the docs site:

```sh
mdbook build docs
```

Serve locally:

```sh
mdbook serve docs -n 127.0.0.1 -p 3001
```