"""All slop detection checks — pure regex, no LLM calls.

Each check function takes text (and optional params dict) and returns a list
of SlopFlag instances. The checks are stateless and deterministic.
"""

from __future__ import annotations

import re
import statistics
from typing import Any

from .models import SlopFlag


# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------


def _sentences(text: str) -> list[str]:
    """Split text into sentences on terminal punctuation."""
    flat = re.sub(r"\s+", " ", text.strip())
    raw = re.split(r"(?<=[.!?])\s+(?=[A-Z\"'\(])|(?<=[.!?])$", flat)
    return [s.strip() for s in raw if s.strip()]


def _word_count(sentence: str) -> int:
    return len(sentence.split())


def _paragraphs(text: str) -> list[str]:
    return [p.strip() for p in re.split(r"\n{2,}", text) if p.strip()]


# ---------------------------------------------------------------------------
# 1. Lexical blacklist
# ---------------------------------------------------------------------------

_DEFAULT_WORD_PATTERNS: list[tuple[str, str]] = [
    # Single words — word-boundary anchored
    (r"\bdelve\b", "delve"),
    (r"\btapestry\b", "tapestry"),
    (r"\btestament\b", "testament"),
    (r"\bvibrant\b", "vibrant"),
    (r"\brobust\b", "robust"),
    (r"\bcrucial\b", "crucial"),
    (r"\bpivotal\b", "pivotal"),
    (r"\bfoster\b", "foster"),
    (r"\bcultivate\b", "cultivate"),
    (r"\bnestled\b", "nestled"),
    (r"\bboasts\b", "boasts"),
    (r"\bbreathtaking\b", "breathtaking"),
    (r"\bgroundbreaking\b", "groundbreaking"),
    (r"\bshowcasing\b", "showcasing"),
    (r"\brenowned\b", "renowned"),
    # Verb uses of words that are fine as nouns/other POS.
    (r"\bunderscore[sd]?\s+the\b", "underscore (verb)"),
    (r"\bhighlight[sd]?\s+(?:the|its|their|our|a|an)\b", "highlight (verb)"),
    # Metaphorical landscape
    (r"\b(?:the|a|an)\s+landscape\s+of\b", "landscape (metaphorical)"),
    # Multi-word phrases
    (r"\ba rich \w+ of\b", "a rich [noun] of"),
    (r"\bstands as a\b", "stands as a"),
    (r"\bserves as a\b", "serves as a"),
    (r"\bholds the distinction\b", "holds the distinction"),
    (r"\breflects broader\b", "reflects broader"),
    (r"\bshaping the evolving\b", "shaping the evolving"),
    (r"\bmarking a pivotal\b", "marking a pivotal"),
    (r"\bleaving an indelible mark\b", "leaving an indelible mark"),
]


def check_lexical_blacklist(
    text: str, params: dict[str, Any] | None = None
) -> list[SlopFlag]:
    """Scan for banned AI-tell words and phrases."""
    if params is not None:
        word_patterns = _build_word_patterns(params)
    else:
        word_patterns = _DEFAULT_WORD_PATTERNS

    flags: list[SlopFlag] = []
    for pattern, label in word_patterns:
        for m in re.finditer(pattern, text, re.IGNORECASE):
            start = max(0, m.start() - 20)
            end = min(len(text), m.end() + 20)
            snippet = text[start:end].replace("\n", " ").strip()
            flags.append(
                SlopFlag(
                    check_name="lexical_blacklist",
                    description=f'Banned phrase "{label}" found',
                    location=f'..."{ snippet}"...',
                    severity="warning",
                )
            )
    return flags


def _build_word_patterns(params: dict[str, Any]) -> list[tuple[str, str]]:
    """Build word patterns from config params."""
    patterns: list[tuple[str, str]] = []
    # Simple words get auto-wrapped in word boundaries
    for word in params.get("words", {}).get("simple", []):
        patterns.append((rf"\b{re.escape(word)}\b", word))
    # Explicit regex patterns
    for entry in params.get("patterns", {}).get("entries", []):
        patterns.append((entry[0], entry[1]))
    return patterns


# ---------------------------------------------------------------------------
# 2. Em-dash count
# ---------------------------------------------------------------------------


def check_em_dash_count(
    text: str, params: dict[str, Any] | None = None
) -> list[SlopFlag]:
    """Flag more than one em-dash in the text."""
    max_allowed = 1
    if params is not None:
        max_allowed = params.get("max_allowed", 1)

    em_dashes = text.count("\u2014")  # —
    if em_dashes <= max_allowed:
        return []
    return [
        SlopFlag(
            check_name="em_dash_count",
            description=f"Text contains {em_dashes} em-dashes (max {max_allowed} allowed)",
            location="",
            severity="warning",
        )
    ]


# ---------------------------------------------------------------------------
# 3. Trailing participle
# ---------------------------------------------------------------------------


def check_trailing_participle(
    text: str, params: dict[str, Any] | None = None
) -> list[SlopFlag]:
    """Detect sentences ending with a trailing participial phrase.

    This is one of the most reliable single-feature AI tells.
    """
    pattern = re.compile(
        r",\s+[A-Za-z]+ing\s+(?:the|its|their|our|an?|his|her|this|that|each|all)\b"
        r"[^.!?]*[.!?]",
        re.IGNORECASE,
    )
    flags: list[SlopFlag] = []
    for m in re.finditer(pattern, text):
        snippet = m.group(0)[:80].replace("\n", " ")
        flags.append(
            SlopFlag(
                check_name="trailing_participle",
                description="Trailing participial phrase detected",
                location=f'..."{ snippet}"...',
                severity="warning",
            )
        )
    return flags


# ---------------------------------------------------------------------------
# 4. Rule of three
# ---------------------------------------------------------------------------


def check_rule_of_three(
    text: str, params: dict[str, Any] | None = None
) -> list[SlopFlag]:
    """Detect comma-separated adjective/item triplets."""
    pattern = re.compile(
        r"\b(?:(?:very|quite|rather|truly|deeply|highly|incredibly|extremely)\s+)?"
        r"[A-Za-z]{2,}"
        r",\s+"
        r"(?:(?:very|quite|rather|truly|deeply|highly|incredibly|extremely)\s+)?"
        r"[A-Za-z]{2,}"
        r",\s+(?:and|or)\s+"
        r"(?:(?:very|quite|rather|truly|deeply|highly|incredibly|extremely)\s+)?"
        r"[A-Za-z]{2,}\b",
        re.IGNORECASE,
    )
    flags: list[SlopFlag] = []
    for m in re.finditer(pattern, text):
        snippet = m.group(0)[:80]
        flags.append(
            SlopFlag(
                check_name="rule_of_three",
                description="Rule-of-three triplet detected",
                location=f'"{snippet}"',
                severity="info",
            )
        )
    return flags


# ---------------------------------------------------------------------------
# 5. Transition openers
# ---------------------------------------------------------------------------

_DEFAULT_TRANSITION_OPENERS = [
    "Moreover",
    "Furthermore",
    "Additionally",
    "Consequently",
    "As a result",
    "In addition",
    "On the other hand",
]


def check_transition_openers(
    text: str, params: dict[str, Any] | None = None
) -> list[SlopFlag]:
    """Flag paragraphs that open with banned transitional adverbs."""
    banned = _DEFAULT_TRANSITION_OPENERS
    if params is not None:
        banned = params.get("banned", _DEFAULT_TRANSITION_OPENERS)

    opener_re = re.compile(
        r"(?:^|\n\n)[ \t]*(" + "|".join(re.escape(b) for b in banned) + r")\b",
        re.IGNORECASE,
    )
    flags: list[SlopFlag] = []
    for m in re.finditer(opener_re, text):
        flags.append(
            SlopFlag(
                check_name="transition_opener",
                description=f'Paragraph opens with banned transition "{m.group(1)}"',
                location=f'"{m.group(0).strip()[:60]}"',
                severity="warning",
            )
        )
    return flags


# ---------------------------------------------------------------------------
# 6. Burstiness
# ---------------------------------------------------------------------------


def check_burstiness(
    text: str, params: dict[str, Any] | None = None
) -> list[SlopFlag]:
    """Flag texts with suspiciously uniform sentence lengths.

    LLMs tend to produce sentences of similar length, yielding a low std dev.
    """
    threshold = 5.0
    min_sentences = 4
    if params is not None:
        threshold = params.get("std_dev_threshold", 5.0)
        min_sentences = params.get("min_sentences", 4)

    sents = _sentences(text)
    if len(sents) < min_sentences:
        return []

    lengths = [_word_count(s) for s in sents]
    try:
        std = statistics.stdev(lengths)
    except statistics.StatisticsError:
        return []

    if std >= threshold:
        return []

    mean = statistics.mean(lengths)
    return [
        SlopFlag(
            check_name="burstiness",
            description=(
                f"Sentence lengths too uniform (std dev {std:.1f} < {threshold}). "
                f"Mean sentence length: {mean:.1f} words across {len(sents)} sentences."
            ),
            location="",
            severity="warning",
        )
    ]


# ---------------------------------------------------------------------------
# 7. Copulative inflation
# ---------------------------------------------------------------------------

_DEFAULT_COPULATIVE_PATTERNS: list[tuple[str, str]] = [
    (r"\bserves as\b", "serves as"),
    (r"\bstand(?:s|ing)?\s+as\b", "stands as"),
    (r"\bfunction(?:s|ing)?\s+as\b", "functions as"),
    (r"\bholds? the distinction of being\b", "holds the distinction of being"),
    (r"\bacts? as\b(?!\s+a\s+deterrent)", "acts as"),
]


def check_copulative_inflation(
    text: str, params: dict[str, Any] | None = None
) -> list[SlopFlag]:
    """Flag verbs that inflate 'is/are' into fancier copulatives."""
    patterns = _DEFAULT_COPULATIVE_PATTERNS
    if params is not None:
        raw = params.get("patterns", None)
        if raw is not None:
            patterns = [(p[0], p[1]) for p in raw]

    flags: list[SlopFlag] = []
    for pat, label in patterns:
        for m in re.finditer(pat, text, re.IGNORECASE):
            start = max(0, m.start() - 15)
            end = min(len(text), m.end() + 30)
            snippet = text[start:end].replace("\n", " ").strip()
            flags.append(
                SlopFlag(
                    check_name="copulative_inflation",
                    description=f'Copulative inflation "{label}" — prefer "is/are"',
                    location=f'..."{ snippet}"...',
                    severity="info",
                )
            )
    return flags


# ---------------------------------------------------------------------------
# 8. Formulaic conclusion
# ---------------------------------------------------------------------------

_DEFAULT_CONCLUSION_OPENERS = [
    "In summary",
    "In conclusion",
    "To summarize",
    "To conclude",
    "Overall",
    "Ultimately",
    "In closing",
    "Challenges and Future",
    "Looking ahead",
    "Moving forward",
    "In the end",
]


def check_formulaic_conclusion(
    text: str, params: dict[str, Any] | None = None
) -> list[SlopFlag]:
    """Detect boilerplate conclusion openers."""
    openers = _DEFAULT_CONCLUSION_OPENERS
    if params is not None:
        openers = params.get("openers", _DEFAULT_CONCLUSION_OPENERS)

    pattern = re.compile(
        r"(?:^|\n+)\s*(" + "|".join(re.escape(o) for o in openers) + r")\b",
        re.IGNORECASE,
    )
    flags: list[SlopFlag] = []
    for m in re.finditer(pattern, text):
        flags.append(
            SlopFlag(
                check_name="formulaic_conclusion",
                description=f'Formulaic conclusion opener "{m.group(1)}"',
                location=f'"{m.group(0).strip()[:60]}"',
                severity="warning",
            )
        )
    return flags


# ---------------------------------------------------------------------------
# 9. Patterned negation
# ---------------------------------------------------------------------------

_DEFAULT_NEGATION_PATTERNS: list[tuple[str, str]] = [
    (
        r"It'?s?\s+not\b[^.!?]{1,80}[.!?]\s+It'?s?\b",
        "It's not X. It's Y.",
    ),
    (
        r"\bNot\s+\w[\w\s]{1,40},\s+but\b",
        "Not X, but Y",
    ),
    (
        r"\b(?:This|That|These|Those)\s+isn'?t\s+about\b[^.!?]{1,80}[.!?]\s+It'?s?\s+about\b",
        "This isn't about X. It's about Y.",
    ),
]


def check_patterned_negation(
    text: str, params: dict[str, Any] | None = None
) -> list[SlopFlag]:
    """Detect the dramatic 'It's not X. It's Y.' construction."""
    patterns = _DEFAULT_NEGATION_PATTERNS
    if params is not None:
        raw = params.get("patterns", None)
        if raw is not None:
            patterns = [(p[0], p[1]) for p in raw]

    flags: list[SlopFlag] = []
    for pat, label in patterns:
        for m in re.finditer(pat, text, re.IGNORECASE | re.DOTALL):
            snippet = m.group(0)[:80].replace("\n", " ")
            flags.append(
                SlopFlag(
                    check_name="patterned_negation",
                    description=f'Patterned negation "{label}" detected',
                    location=f'"{snippet}"',
                    severity="info",
                )
            )
    return flags
