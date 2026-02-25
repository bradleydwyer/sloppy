# slop-detector

Fast regex-based AI prose detection. Scores text 0-100 for "slop" — AI writing tells.
No LLM calls, no heavy NLP dependencies. Pure Python + regex. Runs in <100ms per piece.

## Quick reference

- `from slop_detector import analyze` — main API, returns SlopResult
- `slop-detector analyze file.md` — CLI
- `slop-detector analyze -f json file.md` — JSON output for programmatic use
- `slop-detector voice` — generate voice directive prompt from config

## Architecture

- `src/slop_detector/checks.py` — 9 check functions, pure regex
- `src/slop_detector/detector.py` — orchestrator, calls checks, computes score
- `src/slop_detector/config.py` — TOML config loading and merging
- `src/slop_detector/models.py` — SlopFlag, SlopResult dataclasses
- `src/slop_detector/voice.py` — voice directive generation from config
- `src/slop_detector/defaults.toml` — default word lists and penalties
- `src/slop_detector/cli.py` — Click CLI entry point

## Running tests

```
pytest
```

## Key conventions

- No LLM calls, no network, no heavy NLP
- All checks are pure functions: text in, list[SlopFlag] out
- Config is optional — everything works with zero configuration
- Python 3.11+ required (uses stdlib tomllib)
