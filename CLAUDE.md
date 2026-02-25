# slopcheck

Fast regex-based AI prose detection. Scores text 0-100 for "slop" — AI writing tells.
No LLM calls, no heavy NLP dependencies. Pure Rust + regex. Single binary, zero runtime deps.

## Quick reference

- `slopcheck analyze file.md` — CLI
- `slopcheck analyze -f json file.md` — JSON output for programmatic use
- `slopcheck voice` — generate voice directive prompt from config

## Architecture

- `src/checks.rs` — 9 check functions, pure regex
- `src/detector.rs` — orchestrator, calls checks, computes score
- `src/config.rs` — TOML config loading and merging
- `src/models.rs` — SlopFlag, SlopResult structs
- `src/voice.rs` — voice directive generation from config
- `src/defaults.toml` — default word lists and penalties (embedded at compile time)
- `src/main.rs` — Clap CLI entry point
- `src/lib.rs` — public API

## Build & install

```
cargo build --release
cp target/release/slopcheck ~/.local/bin/  # or anywhere on PATH
```

## Running tests

```
cargo test
```

## Key conventions

- No LLM calls, no network, no heavy NLP
- All checks are pure functions: text in, Vec<SlopFlag> out
- Config is optional — everything works with zero configuration
- defaults.toml is embedded in the binary via include_str!
