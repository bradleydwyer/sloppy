# Contextual Review — Beyond Regex

AI tells the detector cannot catch. These require LLM judgment.

## Patterns to Look For

### Hedging Language
"it could be argued", "in many ways", "to some extent", "it's worth noting that", "it's important to remember"

Hedge phrases let the writer avoid committing to a claim. AI uses them to seem balanced. They weaken prose.

### Balanced-Perspective Equivocation
Presenting "both sides" where a committed stance is needed. The classic AI move: "On one hand X, on the other hand Y." If the user is writing an opinion piece, recommendation, or argument — equivocation is a defect.

### Generic Abstractions
"innovation", "collaboration", "community", "ecosystem" without concrete specifics. These words do no work. Replace with the specific thing.

### Sycophantic Softeners
Now partially covered by the `chatbot_artifacts` check (Layer 1). Layer 2 should still catch subtler forms: "That's a really interesting point", "Excellent observation!" — anything that flatters the reader before answering.

### Uniform Paragraph Rhythm
Now covered by the `paragraph_uniformity` check (Layer 1). Layer 2 should catch subtler rhythm issues that pure sentence counting misses: paragraphs that all follow the same topic-evidence-transition structure even when sentence counts differ.

### Excessive Qualifiers
Now partially covered by `throat_clearing` (Layer 1) for the most formulaic openers. Layer 2 should still catch subtler hedging: "One might argue", "It is generally the case that".

### False Gravitas
Inflating mundane observations into profound insights. "This simple change represents a fundamental shift in how we think about..." — no it doesn't.

### Dramatic Isolated Fragments
Single-sentence paragraphs used for emphasis. AI overuses this device. One per piece, max.

## Rewriting Principles

When fixing detected or contextual issues:

1. **Don't merely swap words.** Restructure sentences so they don't need the flagged word.
2. **Replace copulative inflation** with direct verbs, but also ask if the whole sentence needs rethinking.
3. **When eliminating rule-of-three triplets**, don't just delete one item. Ask whether the list is necessary at all.
4. **For trailing participles**, rewrite as a new sentence or restructure the clause entirely.
5. **Vary sentence length aggressively.** Mix fragments under 6 words with compound sentences over 25 words.
6. **Anchor in specific, concrete, unusual details** rather than generic abstractions.
7. **Take committed stances.** No balanced-perspective hedging.
8. **No formulaic conclusions.** End when you're done. Don't summarize.
