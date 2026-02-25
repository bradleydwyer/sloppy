# slop-detector

Fast regex-based detection of AI prose tells ("slop"). Scores text 0–100.

No LLM calls. No heavy NLP. Single static binary. Runs in <30ms.

## Install

```bash
cargo install --path .
```

Or build manually:

```bash
cargo build --release
cp target/release/slop-detector ~/.local/bin/
```

## Usage

```bash
# Analyze a file
slop-detector analyze README.md

# Pipe from stdin
echo "The vibrant tapestry of innovation delves deeper." | slop-detector analyze

# JSON output for programmatic use
slop-detector analyze -f json document.md

# Quiet mode — just score and pass/fail
slop-detector analyze -q document.md

# Custom threshold (default: 30)
slop-detector analyze -t 20 document.md

# Disable specific checks
slop-detector analyze --disable burstiness --disable rule_of_three document.md
```

## What It Detects

| Check | Detects | Why It's an AI Tell |
|-------|---------|-------------------|
| **lexical_blacklist** | "delve", "tapestry", "vibrant", "robust", 20+ more | These words appear in AI output at 10–50x the rate of human writing |
| **trailing_participle** | ", reflecting the community's deep commitment." | The single most reliable structural AI tell |
| **rule_of_three** | "safe, efficient, and reliable" | AI defaults to comma-separated triplets |
| **em_dash_count** | More than 1 em-dash per piece | AI scatters em-dashes; humans use them sparingly |
| **transition_openers** | "Moreover", "Furthermore", "Additionally" | AI reaches for explicit logical connectors |
| **burstiness** | Sentences all roughly the same length | Human writing has high variance; AI flattens it |
| **copulative_inflation** | "serves as", "stands as", "functions as" | AI inflates "is" into fancier verbs |
| **formulaic_conclusion** | "In summary", "Overall", "Moving forward" | Boilerplate wrap-ups from training corpora |
| **patterned_negation** | "It's not X. It's Y." | A rhetorical device AI overuses |

## Scoring

Each check contributes a penalty (per flag, capped per check). Raw penalties are normalized to 0–100. Default pass threshold is 30.

- **0–10**: Clean human prose
- **10–30**: Minor tells, probably fine
- **30–60**: Noticeable AI patterns
- **60–100**: Unmistakably AI-generated

## Configuration

Create a `.slop-detector.toml` in your project root:

```bash
slop-detector config --init
```

You can add/remove words, adjust penalty weights, change thresholds, or disable checks entirely.

```bash
# View resolved config
slop-detector config --dump
```

## Voice Directive

Generate a system prompt directive that prevents slop at generation time:

```bash
slop-detector voice
```

This outputs constraints derived from the same rules the detector uses, keeping prevention and detection in sync.

## JSON Output Schema

```json
{
  "score": 42,
  "threshold": 30,
  "passed": false,
  "flags": [
    {
      "check_name": "lexical_blacklist",
      "description": "Banned phrase \"delve\" found",
      "location": "...\"we must delve deeper into\"...",
      "severity": "warning"
    }
  ],
  "summary": {
    "total_flags": 7,
    "checks_triggered": ["lexical_blacklist", "rule_of_three"]
  }
}
```

## License

MIT
