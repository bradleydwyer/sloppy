"""Voice directive generation from slop-detector configuration.

Generates a system-level prompt directive that tells an LLM what to avoid,
derived from the same word lists and patterns that the detector uses for scoring.
This ensures the directive and detector always agree.
"""

from __future__ import annotations

from .config import Config, load_config


def generate_voice_directive(config: Config | None = None) -> str:
    """Generate a voice authenticity directive from config.

    The output is suitable for injection into an LLM system prompt.
    When config is None, loads the default configuration.
    """
    if config is None:
        config = load_config()

    sections: list[str] = [
        "[System-level writing constraints — apply to ALL generated content]",
    ]

    # Lexical restrictions
    lexical = config.checks.get("lexical_blacklist")
    if lexical and lexical.enabled:
        sections.append(_build_lexical_section(lexical.params))

    # Punctuation and syntax
    sections.append(_build_punctuation_section(config))

    # Rhythm and structure
    sections.append(_build_structure_section(config))

    # Tone
    sections.append(_build_tone_section(config))

    return "\n\n".join(sections)


def _build_lexical_section(params: dict) -> str:
    words = params.get("words", {}).get("simple", [])
    patterns = params.get("patterns", {}).get("entries", [])

    lines = ["LEXICAL RESTRICTIONS:"]
    if words:
        word_list = ", ".join(words)
        lines.append(f"Never use these words: {word_list}.")
    if patterns:
        phrase_list = ", ".join(f'"{p[1]}"' for p in patterns)
        lines.append(f"Never use these phrases: {phrase_list}.")
    lines.append(
        "Do not use promotional superlatives or inflate the significance of mundane things."
    )
    return "\n".join(lines)


def _build_punctuation_section(config: Config) -> str:
    lines = ["PUNCTUATION AND SYNTAX:"]

    em_dash = config.checks.get("em_dash_count")
    if em_dash and em_dash.enabled:
        max_allowed = em_dash.params.get("max_allowed", 1)
        lines.append(
            f"- Maximum {max_allowed} em-dash(\u2014) per piece. "
            "Prefer parentheses or semicolons for asides."
        )

    trailing = config.checks.get("trailing_participle")
    if trailing and trailing.enabled:
        lines.append(
            '- Never end a sentence with a comma followed by a present participle\n'
            '  (e.g. ", reflecting the..." or ", underscoring the importance of...").'
        )

    rot = config.checks.get("rule_of_three")
    if rot and rot.enabled:
        lines.append(
            "- Do not group adjectives, examples, or clauses in threes. Use two or four."
        )

    trans = config.checks.get("transition_openers")
    if trans and trans.enabled:
        banned = trans.params.get("banned", [])
        if banned:
            banned_str = ", ".join(banned)
            lines.append(f"- Do not start paragraphs with: {banned_str}.")

    cop = config.checks.get("copulative_inflation")
    if cop and cop.enabled:
        lines.append(
            '- Use "is" and "are" instead of "serves as", "stands as", "functions as".'
        )

    return "\n".join(lines)


def _build_structure_section(config: Config) -> str:
    lines = ["RHYTHM AND STRUCTURE:"]

    burst = config.checks.get("burstiness")
    if burst and burst.enabled:
        lines.append(
            "- Vary sentence length sharply. Mix fragments under 6 words with compound sentences\n"
            "  over 30 words. Never write three consecutive sentences of similar length."
        )

    lines.append("- Paragraphs must be asymmetrical \u2014 varying numbers of sentences, varying lengths.")

    neg = config.checks.get("patterned_negation")
    if neg and neg.enabled:
        lines.append(
            '- No patterned negations ("It\'s not X. It\'s Y." or "Not merely X, but Y.").'
        )

    lines.append("- No dramatic isolated fragments for false emphasis.")

    return "\n".join(lines)


def _build_tone_section(config: Config) -> str:
    lines = ["TONE:"]
    lines.append("- Take definitive, committed stances. No balanced-perspective hedging.")
    lines.append("- State facts directly without inflating their importance.")

    conc = config.checks.get("formulaic_conclusion")
    if conc and conc.enabled:
        openers = conc.params.get("openers", [])
        if openers:
            examples = ", ".join(f'"{o}"' for o in openers[:4])
            lines.append(
                f"- No formulaic conclusions. Never use {examples}, etc."
            )

    lines.append('- No sycophantic softeners ("Great question!", "I\'d be happy to...").')
    lines.append(
        "- Anchor writing in specific, unusual, concrete details rather than\n"
        "  generic abstractions."
    )

    return "\n".join(lines)
