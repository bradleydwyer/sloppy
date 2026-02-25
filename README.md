# slopcheck

Fast regex-based detection of AI prose tells ("slop"). Scores text 0-100.

No LLM calls. No heavy NLP. Single static binary. Runs in <30ms.

Works standalone as a CLI, or as Layer 1 of a two-layer system with the included agent skill (SKILL.md) providing LLM-powered contextual review on top.

## Install

**Homebrew (macOS):**
```bash
brew install bradleydwyer/slopcheck/slopcheck
```

**From source (requires Rust toolchain):**
```bash
cargo install --path .
```

**Or build manually:**
```bash
cargo build --release
cp target/release/slopcheck ~/.local/bin/
```

## Usage

```bash
# Analyze a file
slopcheck analyze README.md

# Pipe from stdin
echo "The vibrant tapestry of innovation delves deeper." | slopcheck analyze

# JSON output for programmatic use
slopcheck analyze -f json document.md

# Quiet mode — just score and pass/fail
slopcheck analyze -q document.md

# Custom threshold (default: 30)
slopcheck analyze -t 20 document.md

# Disable specific checks
slopcheck analyze --disable burstiness --disable rule_of_three document.md

# Run only one check
slopcheck analyze --only lexical_blacklist document.md

# Analyze multiple files
slopcheck analyze *.md

# Generate a voice directive (system prompt to prevent slop at generation time)
slopcheck voice
```

## What It Detects

15 checks across lexical, structural, and statistical dimensions:

| Check | Detects | Why It's an AI Tell |
|-------|---------|-------------------|
| **lexical_blacklist** | "delve", "tapestry", "vibrant", "synergy", "paradigm", 90+ more words and phrases | These appear in AI output at 10-50x the rate of human writing |
| **trailing_participle** | ", reflecting the community's deep commitment." | The single most reliable structural AI tell |
| **rule_of_three** | "safe, efficient, and reliable" | AI defaults to comma-separated triplets |
| **em_dash_count** | More than 1 em-dash per piece | AI scatters em-dashes; humans use them sparingly |
| **transition_openers** | "Moreover", "Furthermore", "Additionally", "Notably" | AI reaches for explicit logical connectors |
| **burstiness** | Sentences all roughly the same length | Human writing has high variance; AI flattens it |
| **copulative_inflation** | "serves as", "stands as", "functions as" | AI inflates "is" into fancier verbs |
| **formulaic_conclusion** | "In summary", "Overall", "Moving forward", "Key takeaways" | Boilerplate wrap-ups from training corpora |
| **patterned_negation** | "It's not X. It's Y." | A rhetorical device AI overuses |
| **throat_clearing** | "Here's the thing:", "Let me be clear", "The truth is" | Meta-commentary that delays the point |
| **chatbot_artifacts** | "Great question!", "I'd be happy to", "Feel free to" | Dead giveaways of unedited AI output |
| **paragraph_uniformity** | All paragraphs roughly the same length | AI produces uniform 3-sentence paragraphs |
| **emphasis_crutches** | "Full stop.", "Let that sink in.", "Make no mistake" | Telling instead of showing importance |
| **vague_attribution** | "many experts agree", "studies show" | Weasel-phrase sourcing without actual citations |
| **wordiness** | "in order to", "due to the fact that" | Verbose constructions AI uses at high rates |

## Scoring

Each check contributes a penalty (per flag, capped per check). Raw penalties are normalized to 0-100. Default pass threshold is 30.

- **0-10**: Clean human prose
- **10-30**: Minor tells, probably fine
- **30-60**: Noticeable AI patterns
- **60-100**: Unmistakably AI-generated

Output includes per-check breakdowns showing which checks contributed most to the score.

## Agent Skill

The included `SKILL.md` turns slopcheck into a two-layer system when used with any AI coding agent that supports skills (Claude Code, Amp, Goose, etc.):

- **Layer 1 (CLI):** Deterministic regex detection. Fast, consistent, handles counting and statistical analysis that LLMs can't do reliably.
- **Layer 2 (LLM):** Contextual review guided by the skill. Interprets flags in context, catches what regex misses (hedging, equivocation, tonal flatness), judges false positives, and produces rewrites.

Install the CLI first, then add the skill to your agent:

```bash
# Install the CLI
brew install bradleydwyer/slopcheck/slopcheck

# Add skill to your agent (example for Claude Code)
ln -s /path/to/slopcheck ~/.claude/skills/slopcheck
```

Then ask your agent to review prose, and it will run the CLI, interpret the results, and offer contextual fixes. The `references/` directory contains the full check reference and contextual review guide that the skill loads.

## Voice Directive

The `slopcheck voice` command generates a system prompt directive from the same rules the detector uses, so you can prevent slop at generation time rather than catching it after.

## Configuration

Create a `.slopcheck.toml` in your project root:

```bash
slopcheck config --init
```

You can add/remove words, adjust penalty weights, change thresholds, or disable checks entirely.

```bash
# View resolved config
slopcheck config --dump
```

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
  "check_scores": {
    "lexical_blacklist": { "penalty": 40, "max": 60, "flags": 5 },
    "throat_clearing": { "penalty": 8, "max": 24, "flags": 1 }
  },
  "summary": {
    "total_flags": 7,
    "warnings": 5,
    "info": 2,
    "checks_triggered": ["lexical_blacklist", "throat_clearing"]
  }
}
```

Exit codes: 0 = pass, 1 = fail, 2 = error.

## License

MIT
