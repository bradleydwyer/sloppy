---
name: sloppy
description: "Detect and fix AI prose tells (slop) in text. Two-layer system: fast regex-based detection via the sloppy CLI, plus LLM contextual review and rewriting. Use when reviewing, editing, checking, or cleaning up prose that needs to read as human-written. Also use when generating content that should avoid AI patterns, or building a quality gate for AI-generated text."
allowed-tools:
  - Bash(sloppy:*)
  - Bash(cat:*)
  - Bash(mktemp:*)
  - Read
  - Write
  - Grep
  - Glob
user-invocable: true
argument-hint: "[file to review, or 'prompt' to generate a chat/system prompt]"
metadata:
  author: bradleydwyer
  version: "1.0.1"
  status: experimental
---

# Sloppy — AI Prose Detection & Repair

Two-layer anti-slop system. Layer 1 is deterministic regex detection via the `sloppy` CLI — fast (<30ms), consistent, handles counting and statistical analysis that LLMs can't do reliably. Layer 2 is LLM contextual review — interprets flags in context, catches what regex misses, produces rewrites.

## When to Use This Skill

- Reviewing AI-generated prose before publishing
- Editing drafts that sound "too AI"
- Generating content that needs to read as human-written
- Checking your own writing for absorbed AI patterns
- Building content pipelines that need a quality gate
- Generating a voice directive to prevent slop at generation time

## Installation

The `sloppy` CLI must be available on PATH.

**Homebrew (recommended):**
```bash
brew install bradleydwyer/tap/sloppy
```

**From source (requires Rust toolchain):**
```bash
cargo install --git https://github.com/bradleydwyer/sloppy --tag v1.0.1
```

**Verify installation:**
```bash
sloppy analyze -q <<< "test"
```

If the command is not found, install it before proceeding.

## Mode Detection

Determine the mode from the user's request:

| User Says | Mode | Entry Point |
|---|---|---|
| "check this", "review this text", "is this sloppy?" | **Analyze** | Step 1 → full workflow |
| "fix this", "clean this up", "rewrite this" | **Fix** | Step 1 → Step 4 (produce rewrite) |
| "voice directive", "prevention prompt", "anti-slop prompt", "give me a prompt" | **Prompt** | Prompt Generation section |
| file path or pasted text with no other instruction | **Analyze** | Step 1 → full workflow |

---

## Workflow

### Step 1: Run the Detector

If the user provides a file path, analyze it directly. If they paste text, write it to a temp file first.

```bash
# File path (|| true prevents non-zero exit code on FAIL — read pass/fail from JSON)
sloppy analyze -f json path/to/file.md || true

# From pasted text
TMPFILE=$(mktemp /tmp/sloppy_XXXXXXXX)
cat > "$TMPFILE" << 'SLOP_EOF'
[pasted text here]
SLOP_EOF
sloppy analyze -f json "$TMPFILE" || true
```

Parse the JSON output:
- `score`: 0–100 (0 = clean, 100 = maximum slop)
- `passed`: true/false against threshold (default 30)
- `flags`: array with `check_name`, `description`, `location`, `severity`
- `check_scores`: per-check penalty breakdown (`penalty`, `max`, `flags` count)
- `summary.checks_triggered`: which of the 15 checks fired
- `summary.warnings` / `summary.info`: counts by severity

**Report the score and pass/fail to the user immediately.** Don't bury it in analysis.

### Step 2: Interpret Flags in Context

Read `references/checks.md` for the full check reference.

For each flag from the detector, explain:
1. **Why** this pattern reads as AI-generated — not just that it was detected
2. **Where** it appears — quote the surrounding context from the `location` field
3. **Whether it's a true positive or false positive** in this specific context

False positive judgment is critical. The detector can't distinguish:
- "landscape" used literally vs. metaphorically
- "testament" in religious/legal context vs. as filler
- "foster" as a proper name vs. as a verb
- "robust" in a genuine engineering context vs. as a vague superlative
- "revolutionary" in historical context vs. as a promotional adjective
- A rule-of-three that's genuinely the right rhetorical choice

**Mark clear false positives explicitly.** Don't count them toward the effective score.

### Step 3: Contextual Review (Beyond Regex)

Read `references/contextual-review.md` for the full list.

Look for AI tells the detector doesn't cover:
- **Hedging language**: "it could be argued", "in many ways", "to some extent"
- **Balanced-perspective equivocation**: both-sidesing where a stance is needed
- **Generic abstractions**: "innovation", "collaboration", "community" without specifics
- **Subtle sycophancy**: "That's a really interesting point", "Excellent observation!" (the most obvious chatbot artifacts are now caught by Layer 1, but subtler forms still need judgment)
- **Structural paragraph rhythm**: paragraphs that all follow the same topic-evidence-transition structure even when sentence counts differ (basic length uniformity is now caught by Layer 1)
- **Subtle hedging**: "One might argue", "It is generally the case that" (the most formulaic openers are now caught by Layer 1)
- **False gravitas**: inflating mundane observations into profundity
- **Dramatic isolated fragments**: single-sentence paragraphs for emphasis (AI overuses this)

Present contextual findings separately from detector flags. Be specific — quote the text, explain the problem.

### Step 4: Produce Revised Text

**Only if the user requested a rewrite, or if the score fails the threshold and the user is in Fix mode.**

Rewrite the full text addressing all true-positive flags and contextual issues:

- **Don't merely swap flagged words.** Restructure sentences so they don't need those words.
- **Replace copulative inflation** ("serves as") with direct verbs ("is"), but also consider if the whole sentence needs rethinking.
- **When eliminating rule-of-three triplets**, ask whether the list is necessary at all. A single vivid specific is often stronger.
- **For trailing participles**, rewrite as a new sentence or restructure the clause.
- **Vary sentence length aggressively.** Mix fragments under 6 words with compound sentences over 25 words.
- **Anchor in specific, concrete, unusual details** over generic abstractions.
- **Take committed stances.** No balanced-perspective hedging.
- **No formulaic conclusions.** End when done. Don't summarize.

### Step 5: Re-check

Run the detector again on the revised text:

```bash
sloppy analyze -f json /tmp/slop_review_revised.md || true
```

Report the new score. If it still fails the threshold, iterate on remaining flags. Maximum 3 rewrite iterations before presenting the best version and noting remaining issues.

---

## Prompt Generation (Prevention)

When the user wants to prevent slop at generation time rather than catch it after:

```bash
# Prompt for writing clean prose (paste into any chat window)
sloppy prompt generate

# Prompt for cleaning up existing sloppy text
sloppy prompt cleanup

# Raw system-level constraint block (for API system prompts)
sloppy prompt system

# Any of the above with --copy to copy to clipboard
sloppy prompt generate --copy
```

The `generate` and `cleanup` modes produce chat-ready prompts that work when pasted into ChatGPT, Claude, Gemini, or any other chat interface. The `system` mode produces a raw constraint block for API system prompts. All modes reflect custom `.sloppy.toml` settings if present.

---

## Configuration

The detector is configurable via `.sloppy.toml` in the project root.

```bash
# Create a template config
sloppy config --init

# View the fully resolved config (defaults + overrides)
sloppy config --dump
```

Users can: add/remove banned words, adjust penalty weights per check, change the pass/fail threshold, or disable checks entirely. All config is optional — everything works with zero configuration.

---

## CLI Quick Reference

```bash
# Analyze a file (human-readable output)
sloppy analyze file.md

# Analyze with JSON output (for programmatic use)
sloppy analyze -f json file.md

# Analyze from stdin
echo "text" | sloppy analyze

# Quiet mode — score and pass/fail only
sloppy analyze -q file.md

# Custom threshold
sloppy analyze -t 20 file.md

# Disable specific checks
sloppy analyze --disable burstiness --disable rule_of_three file.md

# Run only one check
sloppy analyze --only lexical_blacklist file.md

# Analyze multiple files
sloppy analyze *.md

# Generate a prompt for clean writing (paste into any chat)
sloppy prompt generate

# Generate a prompt for cleaning up sloppy text
sloppy prompt cleanup

# Generate raw system prompt constraints
sloppy prompt system

# Any prompt mode with clipboard copy
sloppy prompt cleanup --copy

# Initialize config
sloppy config --init

# Dump resolved config
sloppy config --dump
```

---

## Tips

- **Score ≤ 10 is the goal** for polished prose. 10–30 is acceptable for internal docs.
- **Don't chase score 0.** Some flagged patterns are legitimate in context. Judge false positives.
- **Use `--disable` for domain-specific exceptions.** Technical docs might legitimately use "robust" — disable `lexical_blacklist` or customize the word list.
- **The prompt command is the highest-leverage output.** One prompt pasted into a chat window or system prompt prevents hundreds of downstream fixes.
- **JSON output + jq** makes sloppy composable in pipelines: `sloppy analyze -f json file.md | jq '.flags[] | .check_name'`
- **Run on your own prompts and system messages too.** AI slop in prompts begets AI slop in outputs.
