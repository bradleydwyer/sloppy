# Review Prose for AI Tells

Analyze text for AI writing patterns (slop) using the deterministic `slopcheck` tool, then apply contextual LLM judgment to suggest specific rewrites.

## Input

$ARGUMENTS

## Workflow

1. **Run the detector.** Save the input text to a temporary file and run:
   ```
   slopcheck analyze -f json /tmp/slop_review.md
   ```
   Parse the JSON output to get the score, flags, and pass/fail status.

2. **Report the score.** Show the user the numeric score (0-100) and whether it passed or failed the threshold.

3. **For each flag**, explain:
   - **Why** this pattern reads as AI-generated (not just that it was detected)
   - **Where** it appears (quote the surrounding context)
   - A **specific rewrite** that eliminates the tell without losing meaning

4. **Apply contextual judgment.** Beyond what the regex caught, look for:
   - Hedging language ("it could be argued", "in many ways")
   - Balanced-perspective equivocation where a stance is needed
   - Generic abstractions where concrete details would be stronger
   - Sycophantic softeners ("Great question!", "That's a really interesting point")
   - Uniform paragraph rhythm (even if individual sentences vary)
   - Excessive qualifiers and throat-clearing

5. **Produce a revised version** of the full text with all flags addressed and contextual issues fixed.

6. **Re-run the detector** on the revised text:
   ```
   slopcheck analyze -f json /tmp/slop_review_revised.md
   ```
   Report the new score. If it still fails, iterate.

## Rewriting principles

- Do not merely remove flagged words. Restructure sentences so they don't need those words.
- Replace copulative inflation ("serves as") with direct verbs ("is"), but also consider whether the whole sentence structure needs rethinking.
- When eliminating rule-of-three triplets, don't just delete one item. Consider whether the list is necessary at all, or whether a single vivid specific would be stronger.
- For trailing participles, rewrite as a new sentence or restructure the clause entirely.
- Vary sentence length aggressively. Mix fragments under 6 words with compound sentences over 25 words.
- Anchor in specific, concrete, unusual details rather than generic abstractions.
- Take committed stances. No balanced-perspective hedging.
