"""Configuration loading and merging for slop-detector.

Three layers, merged top-down:
1. Built-in defaults (defaults.toml shipped with the package)
2. Project config (.slop-detector.toml in the working directory)
3. Runtime overrides (threshold, disabled checks)
"""

from __future__ import annotations

import tomllib
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


@dataclass
class CheckConfig:
    """Configuration for a single check."""

    enabled: bool = True
    penalty_per_flag: int = 10
    max_penalty: int = 20
    severity: str = "warning"
    params: dict[str, Any] = field(default_factory=dict)


@dataclass
class Config:
    """Fully resolved configuration."""

    threshold: int = 30
    checks: dict[str, CheckConfig] = field(default_factory=dict)


_DEFAULTS_PATH = Path(__file__).parent / "defaults.toml"

# Keys that are check metadata, not check-specific params
_META_KEYS = {"enabled", "penalty_per_flag", "max_penalty", "severity"}


def load_config(
    path: str | Path | None = None,
    *,
    project_dir: str | Path | None = None,
) -> Config:
    """Load configuration from defaults and optional project file.

    Parameters
    ----------
    path:
        Explicit path to a config file. If provided, project_dir is ignored.
    project_dir:
        Directory to search for .slop-detector.toml. Defaults to cwd.
    """
    # 1. Load built-in defaults
    with open(_DEFAULTS_PATH, "rb") as f:
        base = tomllib.load(f)

    # 2. Merge project config if present
    if path is not None:
        config_path = Path(path)
    else:
        search_dir = Path(project_dir) if project_dir else Path.cwd()
        config_path = search_dir / ".slop-detector.toml"

    if config_path.exists():
        with open(config_path, "rb") as f:
            override = tomllib.load(f)
        base = _deep_merge(base, override)

    return _parse_config(base)


def _deep_merge(base: dict, override: dict) -> dict:
    """Deep-merge two dicts. Override wins on conflicts. Lists are replaced, not appended."""
    result = base.copy()
    for key, val in override.items():
        if key in result and isinstance(result[key], dict) and isinstance(val, dict):
            result[key] = _deep_merge(result[key], val)
        else:
            result[key] = val
    return result


def _parse_config(raw: dict) -> Config:
    """Parse a raw TOML dict into a Config object."""
    general = raw.get("general", {})
    threshold = general.get("threshold", 30)

    checks: dict[str, CheckConfig] = {}
    for name, check_raw in raw.get("checks", {}).items():
        if not isinstance(check_raw, dict):
            continue
        # Separate meta keys from check-specific params
        params = {k: v for k, v in check_raw.items() if k not in _META_KEYS}
        checks[name] = CheckConfig(
            enabled=check_raw.get("enabled", True),
            penalty_per_flag=check_raw.get("penalty_per_flag", 10),
            max_penalty=check_raw.get("max_penalty", 20),
            severity=check_raw.get("severity", "warning"),
            params=params,
        )

    return Config(threshold=threshold, checks=checks)


def dump_config(config: Config) -> str:
    """Dump a Config as human-readable TOML-ish text for inspection."""
    lines = [f"[general]", f"threshold = {config.threshold}", ""]
    for name, cc in sorted(config.checks.items()):
        lines.append(f"[checks.{name}]")
        lines.append(f"enabled = {str(cc.enabled).lower()}")
        lines.append(f"penalty_per_flag = {cc.penalty_per_flag}")
        lines.append(f"max_penalty = {cc.max_penalty}")
        lines.append(f'severity = "{cc.severity}"')
        if cc.params:
            for k, v in cc.params.items():
                lines.append(f"{k} = {v!r}")
        lines.append("")
    return "\n".join(lines)
