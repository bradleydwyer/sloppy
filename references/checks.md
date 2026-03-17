# sloppy — Checks Reference

Fifteen regex-based checks, each pure functions: text in, flags out.

## Checks

### lexical_blacklist
**Detects:** Words and phrases that appear in AI output at 10-50x the rate of human writing.
**Penalty:** 8 per flag, max 60.

**Banned words (57):** delve, tapestry, testament, vibrant, robust, crucial, pivotal, foster, cultivate, nestled, boasts, breathtaking, groundbreaking, showcasing, renowned, leverage, utilize, facilitate, harness, illuminate, endeavor, spearhead, streamline, catalyze, encompass, exemplify, embark, empower, bolster, galvanize, solidify, garner, beacon, cornerstone, linchpin, paradigm, synergy, seamless, holistic, noteworthy, commendable, meticulous, intricate, remarkable, exceptional, profound, invaluable, indispensable, paramount, stunning, multifaceted, nuanced.

**Banned phrases (39):** "underscore (verb)", "highlight (verb)", "landscape (metaphorical)", "a rich [noun] of", "stands as a", "serves as a", "holds the distinction", "reflects broader", "shaping the evolving", "marking a pivotal", "leaving an indelible mark", "it's worth noting", "it's important to note", "in today's [X]", "at the end of the day", "when it comes to", "plays a [adjective] role", "let's dive in", "at the forefront of", "rich cultural heritage", "enduring/lasting legacy", "indelible mark", "harness the power of", "unlocking the potential", "empowering [X] to", "setting the stage for", "key turning point", "watershed moment", "paradigm shift", "steadfast dedication", "deeply rooted", "valuable insights", "no discussion would be complete", "navigate (metaphorical)", "game-changing", "cutting-edge", "revolutionary", "transformative".

**False positives to watch for:**
- "landscape" used literally (geography, not metaphor)
- "testament" in religious or legal context
- "foster" as a proper name
- "robust" in technical/engineering contexts (robust error handling)
- "revolutionary" in historical context (Revolutionary War)
- "intricate" in craft/design context (intricate woodwork)

### trailing_participle
**Detects:** Sentences ending with `, [present participle]...` — e.g., ", reflecting the community's deep commitment."
**Penalty:** 10 per flag, max 30.
**Why:** The single most reliable structural AI tell. Humans rarely end sentences this way.

### rule_of_three
**Detects:** Comma-separated triplets — e.g., "safe, efficient, and reliable".
**Penalty:** 5 per flag, max 20.
**Why:** AI defaults to three-item lists. Humans use two or four more naturally.

### em_dash_count
**Detects:** Any em-dash in a piece.
**Penalty:** 10 per flag, max 10.
**Why:** AI scatters em-dashes everywhere. Humans use them sparingly.

### transition_openers
**Detects:** Paragraphs starting with: Moreover, Furthermore, Additionally, Consequently, As a result, In addition, On the other hand, Notably, Importantly, Crucially, Significantly, Interestingly, That said, That being said.
**Penalty:** 8 per flag, max 24.
**Why:** AI reaches for explicit logical connectors that human writers avoid.

### burstiness
**Detects:** Sentences all roughly the same length (low standard deviation < 5.0). Requires minimum 4 sentences.
**Penalty:** 20 per flag, max 20.
**Why:** Human writing has high length variance. AI flattens it.

### copulative_inflation
**Detects:** "serves as", "stands as", "functions as", "holds the distinction of being", "acts as".
**Penalty:** 5 per flag, max 20.
**Why:** AI inflates "is" into fancier constructions.

### formulaic_conclusion
**Detects:** "In summary", "In conclusion", "To summarize", "To conclude", "Overall", "Ultimately", "In closing", "Looking ahead", "Moving forward", "In the end", "Key takeaways", "Key takeaway", "In essence", "All in all", "The bottom line".
**Penalty:** 10 per flag, max 20.
**Why:** Boilerplate wrap-ups from training corpora. Strong writers end when they're done.

### patterned_negation
**Detects:** "It's not X. It's Y." or "Not merely X, but Y."
**Penalty:** 5 per flag, max 15.
**Why:** A rhetorical device AI overuses far beyond what humans do.

### throat_clearing
**Detects:** Meta-commentary openers where the writer announces what they're about to say instead of saying it. "Here's the thing:", "Let me be clear", "The truth is", "The reality is", "But here's the thing", "Let's be honest", "Think about it:", "And that's okay."
**Penalty:** 8 per flag, max 24.
**Why:** AI uses these as filler to sound conversational. They delay the point.

### chatbot_artifacts
**Detects:** Unedited AI output tells: "Great question!", "I'd be happy to", "Hope this helps", "Feel free to", "Certainly!", "Absolutely!", "As an AI", "As a language model", "Let me know if you".
**Penalty:** 10 per flag, max 20.
**Why:** Dead giveaways of copy-pasted chatbot output. Zero false-positive risk.

### paragraph_uniformity
**Detects:** All paragraphs roughly the same length (low standard deviation of sentence counts). Requires minimum 4 paragraphs.
**Penalty:** 15 per flag, max 15.
**Why:** AI produces uniform 3-sentence paragraphs. Human writing has asymmetric paragraphs.

### emphasis_crutches
**Detects:** "Full stop.", "Period.", "Let that sink in.", "Make no mistake", "This matters because", "X. That's it. That's the Y."
**Penalty:** 5 per flag, max 15.
**Why:** AI tells the reader something is important rather than demonstrating it through content.

### vague_attribution
**Detects:** "many experts agree", "studies show", "some critics argue", "industry reports suggest", "it is widely believed", "observers have noted".
**Penalty:** 5 per flag, max 15.
**Why:** AI uses weasel-phrase sourcing to give an appearance of authority without actually citing anything.

### wordiness
**Detects:** Verbose constructions with shorter equivalents: "in order to" (use "to"), "due to the fact that" (use "because"), "at this point in time" (use "now"), "the fact that" (cut), "it should be noted that" (cut).
**Penalty:** 3 per flag, max 12.
**Why:** AI uses wordy constructions at far higher rates than edited human prose.

## Scoring

Each check contributes penalties capped at its max. Raw penalties are normalized to 0-100 with a minimum denominator floor (sum of the 3 largest check maxes), so a single check on short text can't inflate the score to 100. Default pass threshold is 30.

| Range | Meaning |
|-------|---------|
| 0-10 | Clean human prose |
| 10-30 | Minor tells, probably fine |
| 30-60 | Noticeable AI patterns |
| 60-100 | Unmistakably AI-generated |
