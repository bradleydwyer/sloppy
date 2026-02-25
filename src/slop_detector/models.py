"""Data models for slop detection results.

SlopFlag represents a single detected AI-tell.
SlopResult aggregates all flags into a scored result.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass, field


@dataclass(frozen=True)
class SlopFlag:
    """A single detected AI prose tell."""

    check_name: str
    description: str
    location: str = ""
    severity: str = "warning"


@dataclass
class SlopResult:
    """Aggregated result from running all slop checks on a text.

    Attributes:
        score: 0-100 where 0 is pristine and 100 is maximum slop.
        flags: Every individual match from every check.
        passed: True when score < the configured threshold.
    """

    score: int = 0
    flags: list[SlopFlag] = field(default_factory=list)
    passed: bool = True

    def to_dict(self) -> dict:
        return asdict(self)
