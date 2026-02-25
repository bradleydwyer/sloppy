---
name: review-prose
description: "Detect and fix AI prose tells (slop) in text. Two-layer system: fast regex-based detection via the slop-detector CLI, plus LLM contextual review and rewriting. Use when reviewing, editing, or generating prose that needs to read as human-written."
source: personal
risk: safe
domain: writing
category: workflow
version: 0.1.0
---

# Review Prose for AI Tells

Two-layer anti-slop system for catching and fixing AI writing patterns.

**Layer 1** — Deterministic regex detection via the `slop-detector` CLI. Fast (<100ms), consistent, zero false negatives for known patterns. Handles word blacklists, structural patterns, and statistical analysis that LLMs can't do reliably (counting, standard deviations).

**Layer 2** — LLM contextual review. Interprets detector flags in context, catches what regex misses (hedging, equivocation, tonal flatness), and produces specific rewrites.

## When to Use This Skill

- Reviewing AI-generated prose before publishing
- Editing drafts that sound "too AI"
- Generating content that needs to read as human-written
- Checking your own writing for patterns you've absorbed from AI
- Building content pipelines that need a quality gate

## Prerequisites

The `slop-detector` CLI must be installed:

```bash
pip install git+https://github.com/bradleydwyer/slop-detector.git
```

Verify: `slop-detector analyze -q <<< "test"` should output a score.

## Workflow

### Step 1: Run the Detector

Save the text to a temporary file and run the deterministic detector:

```bash
slop-detector analyze -f json /tmp/slop_review_input.md
```

Parse the JSON output. It returns:
- `score`: 0-100 (0 = pristine, 100 = maximum slop)
- `passed`: true/false against the threshold (default 30)
- `flags`: array of detected patterns, each with `check_name`, `description`, `location`, `severity`
- `summary.checks_triggered`: which of the 9 checks fired

Report the score and pass/fail status to the user.

### Step 2: Interpret Flags in Context

For each flag from the detector, explain:
- **Why** this pattern reads as AI-generated (not just that it was detected)
- **Where** it appears (quote the surrounding context from the `location` field)
- Whether it's a true positive or a false positive in this specific context

False positive examples the detector cannot distinguish:
- "landscape" used literally (geography) vs. metaphorically (flagged)
- "testament" in religious/legal context vs. as AI filler
- "foster" as a proper name vs. as a verb
- A rule-of-three that's genuinely the best rhetorical choice

### Step 3: Contextual Review (Beyond Regex)

Look for AI tells the detector doesn't cover:

- **Hedging language**: "it could be argued", "in many ways", "to some extent"
- **Balanced-perspective equivocation**: presenting "both sides" where a committed stance is needed
- **Generic abstractions**: "innovation", "collaboration", "community" without concrete specifics
- **Sycophantic softeners**: "Great question!", "That's a really interesting point", "I'd be happy to"
- **Uniform paragraph rhythm**: even if individual sentences vary, paragraphs all the same shape
- **Excessive qualifiers and throat-clearing**: "It's worth noting that", "It's important to remember"
- **False gravitas**: inflating mundane observations into profound insights
- **Dramatic isolated fragments**: single-sentence paragraphs for emphasis (AI overuses this)

### Step 4: Produce Revised Text

Rewrite the full text addressing all flags and contextual issues. Follow these principles:

- **Do not merely remove flagged words.** Restructure sentences so they don't need those words.
- **Replace copulative inflation** ("serves as") with direct verbs ("is"), but also consider whether the whole sentence needs rethinking.
- **When eliminating rule-of-three triplets**, don't just delete one item. Ask whether the list is necessary at all, or whether a single vivid specific would be stronger.
- **For trailing participles**, rewrite as a new sentence or restructure the clause entirely.
- **Vary sentence length aggressively.** Mix fragments under 6 words with compound sentences over 25 words.
- **Anchor in specific, concrete, unusual details** rather than generic abstractions.
- **Take committed stances.** No balanced-perspective hedging.
- **No formulaic conclusions.** End when you're done. Don't summarize.

### Step 5: Re-check

Run the detector again on the revised text:

```bash
slop-detector analyze -f json /tmp/slop_review_revised.md
```

Report the new score. If it still fails the threshold, iterate on the remaining flags.

## Voice Directive (Prevention)

To prevent slop at generation time rather than catching it after, inject the voice directive into your system prompt:

```bash
slop-detector voice
```

This generates a constraint block derived from the same rules the detector uses, keeping prevention and detection in sync. Use it in any LLM system prompt where the output needs to read as human-written.

## What the Detector Checks

| Check | Detects | Why It's an AI Tell |
|-------|---------|-------------------|
| **lexical_blacklist** | "delve", "tapestry", "vibrant", "robust", 20+ more | These words appear in AI output at 10-50x the rate of human writing |
| **trailing_participle** | ", reflecting the community's deep commitment." | The single most reliable structural AI tell |
| **rule_of_three** | "safe, efficient, and reliable" | AI defaults to comma-separated triplets |
| **em_dash_count** | More than 1 em-dash per piece | AI scatters em-dashes; humans use them sparingly |
| **transition_openers** | "Moreover", "Furthermore", "Additionally" | AI reaches for explicit logical connectors |
| **burstiness** | Sentences all roughly the same length | Human writing has high variance; AI flattens it |
| **copulative_inflation** | "serves as", "stands as", "functions as" | AI inflates "is" into fancier verbs |
| **formulaic_conclusion** | "In summary", "Overall", "Moving forward" | Boilerplate wrap-ups from training corpora |
| **patterned_negation** | "It's not X. It's Y." | A rhetorical device AI overuses |

## Configuration

The detector is configurable via `.slop-detector.toml` in the project root. Run `slop-detector config --init` to create a template. Users can add/remove words, adjust penalty weights, change thresholds, or disable checks entirely.
