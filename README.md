# sloppy

<p align="center">
  <img src="logos/sloppy-logo-2.png" width="256" alt="sloppy logo" />
</p>

Detect AI prose tells ("slop") in text. Scores 0-100 using regex pattern matching. No LLM calls, no heavy NLP. Runs in under 30ms.

Works standalone as a CLI, or paired with an agent skill for LLM-powered contextual review on top.

## Install

```bash
brew install bradleydwyer/tap/sloppy
```

Or from source:

```bash
cargo install --path .
```

## Agent Skill

sloppy includes a skill installer for AI coding agents. The skill adds an LLM review layer on top of the CLI:

- **CLI layer:** Regex-based detection. Fast, deterministic, handles counting and statistics.
- **LLM layer:** Contextual review guided by the skill. Catches hedging, tonal flatness, and false positives. Produces rewrites.

### Install the skill

```bash
# With equip (recommended)
equip install bradleydwyer/sloppy

# Or with the built-in installer
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

Claude Code gets the full skill (SKILL.md + reference files). Other agents get a rules file in their native format.

Run `sloppy skill` with no flags to see all supported agents and their install paths.

### What you can ask

- **"check this for slop"** - runs the CLI, reports the score, explains flags in context, identifies false positives, does a contextual review
- **"clean this up, it reads too AI"** - analyzes, rewrites, and re-checks until the score passes
- **"generate a prompt"** - produces a chat or system prompt to prevent slop at generation time

## Usage

```bash
# Analyze a file
sloppy analyze README.md

# Pipe from stdin
echo "The vibrant tapestry of innovation delves deeper." | sloppy analyze

# JSON output
sloppy analyze -f json document.md

# Quiet mode (just score and pass/fail)
sloppy analyze -q document.md

# Custom threshold (default: 30)
sloppy analyze -t 20 document.md

# Disable specific checks
sloppy analyze --disable burstiness --disable rule_of_three document.md

# Run only one check
sloppy analyze --only lexical_blacklist document.md

# Multiple files
sloppy analyze *.md

# Generate a prompt for clean writing
sloppy prompt

# Generate a cleanup prompt
sloppy prompt cleanup

# Raw system prompt constraints (for API use)
sloppy prompt system

# Copy any prompt to clipboard
sloppy prompt cleanup --copy
```

## What It Detects

15 checks across lexical, structural, and statistical patterns:

| Check | Example | Why |
|-------|---------|-----|
| **lexical_blacklist** | "delve", "tapestry", "vibrant", 90+ more | Appear 10-50x more in AI output than human writing |
| **trailing_participle** | ", reflecting the community's deep commitment." | Most reliable structural AI tell |
| **rule_of_three** | "safe, efficient, and reliable" | AI defaults to comma-separated triplets |
| **em_dash_count** | Any em-dash | AI scatters them; humans use them sparingly |
| **transition_openers** | "Moreover", "Furthermore", "Additionally" | AI leans on explicit connectors |
| **burstiness** | Sentences all roughly the same length | Humans vary sentence length more |
| **copulative_inflation** | "serves as", "stands as", "functions as" | AI inflates "is" into fancier constructions |
| **formulaic_conclusion** | "In summary", "Moving forward", "Key takeaways" | Boilerplate wrap-ups |
| **patterned_negation** | "It's not X. It's Y." | A rhetorical device AI overuses |
| **throat_clearing** | "Here's the thing:", "Let me be clear" | Meta-commentary that delays the point |
| **chatbot_artifacts** | "Great question!", "I'd be happy to" | Unedited chatbot output |
| **paragraph_uniformity** | All paragraphs roughly the same length | AI produces uniform blocks |
| **emphasis_crutches** | "Full stop.", "Let that sink in." | Tells rather than shows importance |
| **vague_attribution** | "many experts agree", "studies show" | Weasel phrases without actual citations |
| **wordiness** | "in order to", "due to the fact that" | Verbose constructions AI favors |

## Scoring

Each check adds a penalty (per flag, capped per check). Raw penalties normalize to 0-100. Default pass threshold is 30.

- **0-10**: Clean prose
- **10-30**: Minor tells, probably fine
- **30-60**: Noticeable patterns
- **60-100**: Unmistakable

Output includes per-check breakdowns showing which checks contributed most.

## Prompt Generation

`sloppy prompt` generates prompts you can paste into any LLM to prevent slop:

- **`sloppy prompt`** - chat-ready prompt for writing clean
- **`sloppy prompt cleanup`** - chat-ready prompt for rewriting sloppy text
- **`sloppy prompt system`** - raw constraint block for API system prompts

Add `--copy` to put any of these on your clipboard.

## Configuration

Create a `.sloppy.toml` in your project root:

```bash
sloppy config --init
```

Add/remove words, adjust penalty weights, change thresholds, or disable checks entirely.

```bash
# View resolved config
sloppy config --dump
```

## JSON Output

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

## More Tools

**Naming & Availability**
- [available](https://github.com/bradleydwyer/available) — AI-powered project name finder (uses parked, staked & published)
- [parked](https://github.com/bradleydwyer/parked) — Domain availability checker (DNS → WHOIS → RDAP)
- [staked](https://github.com/bradleydwyer/staked) — Package registry name checker (npm, PyPI, crates.io + 19 more)
- [published](https://github.com/bradleydwyer/published) — App store name checker (App Store & Google Play)

**AI Tooling**
- [caucus](https://github.com/bradleydwyer/caucus) — Multi-LLM consensus engine
- [nanaban](https://github.com/bradleydwyer/nanaban) — Gemini image generation CLI
- [equip](https://github.com/bradleydwyer/equip) — Cross-agent skill manager
