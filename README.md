# sloppy

Fast regex-based detection of AI prose tells ("slop"). Scores text 0-100.

No LLM calls. No heavy NLP. Single static binary. Runs in <30ms.

Works standalone as a CLI, or as Layer 1 of a two-layer system with the included agent skill (SKILL.md) providing LLM-powered contextual review on top.

## Agent Skill

Sloppy includes a built-in skill installer for AI coding agents. The skill turns sloppy into a two-layer system:

- **Layer 1 (CLI):** Deterministic regex detection. Fast, consistent, handles counting and statistical analysis that LLMs can't do reliably.
- **Layer 2 (LLM):** Contextual review guided by the skill. Interprets flags in context, catches what regex misses (hedging, equivocation, tonal flatness), judges false positives, and produces rewrites.

### Install the skill

After installing the CLI, run:

```bash
# Claude Code (default — full two-layer skill with references)
sloppy skill --install

# Other agents
sloppy skill --install --agent cursor
sloppy skill --install --agent windsurf
sloppy skill --install --agent copilot
sloppy skill --install --agent cline
sloppy skill --install --agent roo
sloppy skill --install --agent continue
sloppy skill --install --agent amp
sloppy skill --install --agent goose
sloppy skill --install --agent aider
```

Claude Code gets the full skill (SKILL.md + reference files in `~/.claude/skills/sloppy/`). Other agents get a rules file in their native format that teaches them how to use the sloppy CLI.

Run `sloppy skill` with no flags to see all supported agents and their install paths.

### What you can ask

- **"check this for slop"** — runs the CLI, reports the score, explains every flag in context, identifies false positives, and does a contextual review beyond what regex catches.
- **"clean this up, it reads too AI"** — analyzes, rewrites, and re-checks until the score passes.
- **"generate a prompt"** — produces a chat or system prompt to prevent slop at generation time.

## Install (CLI)

**Homebrew (macOS/Linux):**
```bash
brew tap bradleydwyer/sloppy
brew install sloppy
```

**From source (any platform, requires Rust toolchain):**
```bash
cargo install --path .
```

**Or build manually:**
```bash
cargo build --release
# macOS/Linux:
cp target/release/sloppy ~/.local/bin/
# Windows:
copy target\release\sloppy.exe %USERPROFILE%\.local\bin\
```

## CLI Usage

```bash
# Analyze a file
sloppy analyze README.md

# Pipe from stdin
echo "The vibrant tapestry of innovation delves deeper." | sloppy analyze

# JSON output for programmatic use
sloppy analyze -f json document.md

# Quiet mode — just score and pass/fail
sloppy analyze -q document.md

# Custom threshold (default: 30)
sloppy analyze -t 20 document.md

# Disable specific checks
sloppy analyze --disable burstiness --disable rule_of_three document.md

# Run only one check
sloppy analyze --only lexical_blacklist document.md

# Analyze multiple files
sloppy analyze *.md

# Generate a prompt for clean writing (paste into any chat window)
sloppy prompt

# Generate a prompt for cleaning up sloppy text
sloppy prompt cleanup

# Raw system prompt constraints (for API system prompts)
sloppy prompt system

# Copy any prompt to clipboard
sloppy prompt cleanup --copy
```

## What It Detects

15 checks across lexical, structural, and statistical dimensions:

| Check | Detects | Why It's an AI Tell |
|-------|---------|-------------------|
| **lexical_blacklist** | "delve", "tapestry", "vibrant", "synergy", "paradigm", 90+ more words and phrases | These appear in AI output at 10-50x the rate of human writing |
| **trailing_participle** | ", reflecting the community's deep commitment." | The single most reliable structural AI tell |
| **rule_of_three** | "safe, efficient, and reliable" | AI defaults to comma-separated triplets |
| **em_dash_count** | Any em-dash in a piece | AI scatters em-dashes; humans use them sparingly |
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

## Prompt Generation

The `sloppy prompt` command generates prompts you can paste into any LLM chat window or system prompt to prevent slop at generation time:

- **`sloppy prompt`** (or `sloppy prompt generate`) — a chat-ready prompt for writing clean prose
- **`sloppy prompt cleanup`** — a chat-ready prompt for rewriting sloppy text (includes a `[PASTE YOUR TEXT HERE]` placeholder)
- **`sloppy prompt system`** — raw constraint block for API system prompts

Add `--copy` to any of these to copy the output to your clipboard.

## Configuration

Create a `.sloppy.toml` in your project root:

```bash
sloppy config --init
```

You can add/remove words, adjust penalty weights, change thresholds, or disable checks entirely. Users who want to allow em-dashes can set `max_allowed = 1` (or higher) in their config.

```bash
# View resolved config
sloppy config --dump
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
