"""Slop detection orchestrator.

Wires check functions to configuration and produces scored results.
No LLM calls — pure regex and string analysis. Runs in <100ms per piece.
"""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Any, Callable

from .checks import (
    check_burstiness,
    check_copulative_inflation,
    check_em_dash_count,
    check_formulaic_conclusion,
    check_lexical_blacklist,
    check_patterned_negation,
    check_rule_of_three,
    check_trailing_participle,
    check_transition_openers,
)
from .models import SlopFlag, SlopResult


@dataclass
class _CheckConfig:
    fn: Callable[[str, dict[str, Any] | None], list[SlopFlag]]
    name: str
    penalty_per_flag: int
    max_penalty: int


_DEFAULT_CHECKS: list[_CheckConfig] = [
    _CheckConfig(check_lexical_blacklist, "lexical_blacklist", penalty_per_flag=8, max_penalty=40),
    _CheckConfig(check_em_dash_count, "em_dash_count", penalty_per_flag=10, max_penalty=10),
    _CheckConfig(check_trailing_participle, "trailing_participle", penalty_per_flag=10, max_penalty=30),
    _CheckConfig(check_rule_of_three, "rule_of_three", penalty_per_flag=5, max_penalty=20),
    _CheckConfig(check_transition_openers, "transition_openers", penalty_per_flag=8, max_penalty=24),
    _CheckConfig(check_burstiness, "burstiness", penalty_per_flag=20, max_penalty=20),
    _CheckConfig(check_copulative_inflation, "copulative_inflation", penalty_per_flag=5, max_penalty=20),
    _CheckConfig(check_formulaic_conclusion, "formulaic_conclusion", penalty_per_flag=10, max_penalty=20),
    _CheckConfig(check_patterned_negation, "patterned_negation", penalty_per_flag=5, max_penalty=15),
]

_MAX_RAW_PENALTY: int = sum(c.max_penalty for c in _DEFAULT_CHECKS)


def analyze(
    text: str,
    slop_threshold: int = 30,
    config: Any | None = None,
) -> SlopResult:
    """Run all slop checks on *text* and return a SlopResult.

    Parameters
    ----------
    text:
        The prose to analyse. May contain markdown formatting.
    slop_threshold:
        Score at or above which the result is considered a failure.
        Defaults to 30.
    config:
        Optional Config object. When provided, check-specific parameters
        and penalties are read from it. When None, hardcoded defaults are used.

    Returns
    -------
    SlopResult
        ``score`` is in [0, 100] where 0 is pristine and 100 is maximum slop.
        ``flags`` lists every individual match from every check.
        ``passed`` is True when ``score < slop_threshold``.
    """
    if not text or not text.strip():
        return SlopResult(score=0, flags=[], passed=True)

    # Resolve checks and config
    if config is not None:
        threshold = config.threshold if slop_threshold == 30 else slop_threshold
        checks = _resolve_checks(config)
    else:
        threshold = slop_threshold
        checks = _DEFAULT_CHECKS

    max_raw = sum(c.max_penalty for c in checks) or 1

    all_flags: list[SlopFlag] = []
    raw_penalty = 0

    for check in checks:
        # Pass config params if available
        params = None
        if config is not None and check.name in config.checks:
            params = config.checks[check.name].params

        flags = check.fn(text, params)
        all_flags.extend(flags)
        contribution = min(len(flags) * check.penalty_per_flag, check.max_penalty)
        raw_penalty += contribution

    score = math.floor((raw_penalty / max_raw) * 100) if max_raw else 0
    score = min(score, 100)

    return SlopResult(
        score=score,
        flags=all_flags,
        passed=score < threshold,
    )


def _resolve_checks(config: Any) -> list[_CheckConfig]:
    """Build check list from config, respecting enabled/disabled and custom penalties."""
    # Map check names to their functions
    fn_map = {c.name: c.fn for c in _DEFAULT_CHECKS}

    checks: list[_CheckConfig] = []
    for default in _DEFAULT_CHECKS:
        if default.name in config.checks:
            cc = config.checks[default.name]
            if not cc.enabled:
                continue
            checks.append(
                _CheckConfig(
                    fn=fn_map[default.name],
                    name=default.name,
                    penalty_per_flag=cc.penalty_per_flag,
                    max_penalty=cc.max_penalty,
                )
            )
        else:
            checks.append(default)
    return checks
