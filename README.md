# slop-detector

Catch AI prose tells before your readers do.

Fast, deterministic, regex-based detection of the patterns that make AI-generated text sound like AI-generated text. Scores prose 0-100. No LLM calls, no NLP dependencies, runs in under 100ms.

## What it catches

| Check | What it flags | Why it matters |
|-------|--------------|----------------|
| **Lexical blacklist** | "delve", "tapestry", "vibrant", "robust", "groundbreaking", and 20+ more | Words and phrases that appear in AI output at 10-50x the rate of human writing |
| **Trailing participles** | ", reflecting the community's deep commitment." | The single most reliable structural AI tell |
| **Rule of three** | "safe, efficient, and reliable" | AI defaults to comma-separated triplets; humans rarely do |
| **Em-dash overuse** | More than one em-dash per piece | AI scatters em-dashes; most human writers use them sparingly |
| **Transition openers** | "Moreover", "Furthermore", "Additionally" | AI reaches for explicit logical connectors that human essayists avoid |
| **Burstiness** | Sentences all roughly the same length | Human writing has high variance in sentence length; AI flattens it |
| **Copulative inflation** | "serves as", "stands as", "functions as" | AI inflates "is" into fancier verbs for no reason |
| **Formulaic conclusions** | "In summary", "Overall", "Moving forward" | Boilerplate wrap-ups learned from academic/journalistic corpora |
| **Patterned negation** | "It's not X. It's Y." | A rhetorical device AI overuses to the point of self-parody |

## Installation

```bash
pip install git+https://github.com/bradleydwyer/slop-detector.git
```

Requires Python 3.11+. Only runtime dependency is `click`.

## Usage

### CLI

```bash
# Analyze a file
slop-detector analyze draft.md

# Pipe from stdin
echo "The vibrant tapestry of innovation delves deeper." | slop-detector analyze

# JSON output (for programmatic use)
slop-detector analyze -f json draft.md

# Custom threshold (default: 30)
slop-detector analyze -t 20 draft.md

# Quiet mode (score only)
slop-detector analyze -q draft.md
```

Example output:

```
Score: 30/100  FAIL

  [warning]  lexical_blacklist: Banned phrase "tapestry" found
             ..."Furthermore, the tapestry of collaboration is"...
  [warning]  lexical_blacklist: Banned phrase "vibrant" found
             ..."a testament to the vibrant, robust, and crucia"...
  [warning]  trailing_participle: Trailing participial phrase detected
             ...", highlighting its potential to reshape the landscape."...
  [info]     rule_of_three: Rule-of-three triplet detected
             "vibrant, robust, and crucial"

14 flag(s) from 4 check(s)
```

Exit code is 0 on pass, 1 on fail — works in CI pipelines.

### Python library

```python
from slop_detector import analyze

result = analyze("The vibrant tapestry of innovation delves deeper.")
print(result.score)   # 0-100
print(result.passed)  # True/False
for flag in result.flags:
    print(f"{flag.check_name}: {flag.description}")
```

### Voice directive generation

Generate a system prompt directive derived from the same rules the detector uses — keeps your prevention layer and detection layer in sync:

```bash
slop-detector voice
```

```python
from slop_detector import generate_voice_directive

directive = generate_voice_directive()
# Inject into your LLM system prompt
```

## Configuration

Everything works with zero configuration. To customize, create a `.slop-detector.toml` in your project root:

```bash
slop-detector config --init  # copies defaults as a starting point
```

Override anything:

```toml
[general]
threshold = 20  # stricter than default 30

[checks.em_dash_count]
enabled = false  # we like em-dashes

[checks.lexical_blacklist.words]
simple = [
    "delve", "tapestry", "vibrant",
    "synergy", "leverage", "paradigm",  # add your own
]

[checks.transition_openers]
banned = [
    "Moreover", "Furthermore", "Additionally",
    "Notably", "Importantly",  # add your own
]
```

Lists in your config **replace** the defaults (not append). Copy the full list from `defaults.toml` if you want to extend.

## Claude Code skill

This repo includes a `/review-prose` slash command for Claude Code. It runs the deterministic detector, then adds LLM-based contextual review on top:

1. Runs `slop-detector analyze -f json` for reliable pattern matching
2. Interprets flags in context (is "landscape" literal geography or a metaphor?)
3. Catches what regex can't — hedging, equivocation, generic abstractions
4. Suggests specific rewrites for every flagged passage
5. Re-runs the detector to verify the score dropped

To use it, clone this repo and the skill is available in any Claude Code session within the project.

## Development

```bash
git clone https://github.com/bradleydwyer/slop-detector.git
cd slop-detector
pip install -e ".[dev]"
pytest
```

133 tests. They run in under a second.

## How scoring works

Each check contributes a penalty per flag hit, capped at a per-check maximum. The raw penalty is normalized to 0-100. A score of 0 is pristine; 100 is maximally sloppy. The default pass/fail threshold is 30.

| Check | Penalty per flag | Max penalty |
|-------|-----------------|-------------|
| lexical_blacklist | 8 | 40 |
| trailing_participle | 10 | 30 |
| transition_openers | 8 | 24 |
| burstiness | 20 | 20 |
| formulaic_conclusion | 10 | 20 |
| rule_of_three | 5 | 20 |
| copulative_inflation | 5 | 20 |
| patterned_negation | 5 | 15 |
| em_dash_count | 10 | 10 |

All weights are configurable via TOML.

## License

MIT. See [LICENSE](LICENSE).
