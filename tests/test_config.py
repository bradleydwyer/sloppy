"""Tests for configuration loading and merging."""

from __future__ import annotations

from pathlib import Path

from slop_detector.config import Config, CheckConfig, load_config, _deep_merge, _parse_config


class TestDeepMerge:
    def test_override_scalar(self) -> None:
        base = {"general": {"threshold": 30}}
        override = {"general": {"threshold": 20}}
        result = _deep_merge(base, override)
        assert result["general"]["threshold"] == 20

    def test_override_preserves_unmentioned_keys(self) -> None:
        base = {"general": {"threshold": 30}, "other": "value"}
        override = {"general": {"threshold": 20}}
        result = _deep_merge(base, override)
        assert result["other"] == "value"

    def test_list_replaced_not_appended(self) -> None:
        base = {"words": ["a", "b", "c"]}
        override = {"words": ["x", "y"]}
        result = _deep_merge(base, override)
        assert result["words"] == ["x", "y"]

    def test_nested_merge(self) -> None:
        base = {"checks": {"lexical": {"enabled": True, "penalty": 8}}}
        override = {"checks": {"lexical": {"penalty": 12}}}
        result = _deep_merge(base, override)
        assert result["checks"]["lexical"]["enabled"] is True
        assert result["checks"]["lexical"]["penalty"] == 12


class TestParseConfig:
    def test_parses_threshold(self) -> None:
        raw = {"general": {"threshold": 20}, "checks": {}}
        config = _parse_config(raw)
        assert config.threshold == 20

    def test_parses_check_config(self) -> None:
        raw = {
            "general": {"threshold": 30},
            "checks": {
                "lexical_blacklist": {
                    "enabled": True,
                    "penalty_per_flag": 8,
                    "max_penalty": 40,
                    "severity": "warning",
                    "words": {"simple": ["delve"]},
                }
            },
        }
        config = _parse_config(raw)
        cc = config.checks["lexical_blacklist"]
        assert cc.enabled is True
        assert cc.penalty_per_flag == 8
        assert cc.max_penalty == 40
        assert cc.params["words"]["simple"] == ["delve"]

    def test_defaults_for_missing_fields(self) -> None:
        raw = {"checks": {"custom_check": {"enabled": True}}}
        config = _parse_config(raw)
        cc = config.checks["custom_check"]
        assert cc.penalty_per_flag == 10  # default
        assert cc.max_penalty == 20  # default


class TestLoadConfig:
    def test_loads_defaults(self) -> None:
        config = load_config(project_dir="/nonexistent")
        assert config.threshold == 30
        assert "lexical_blacklist" in config.checks
        assert "burstiness" in config.checks
        assert len(config.checks) == 9

    def test_all_checks_enabled_by_default(self) -> None:
        config = load_config(project_dir="/nonexistent")
        for name, cc in config.checks.items():
            assert cc.enabled is True, f"Check {name} should be enabled by default"

    def test_loads_with_override_file(self, tmp_path: Path) -> None:
        override = tmp_path / ".slop-detector.toml"
        override.write_text('[general]\nthreshold = 15\n')
        config = load_config(project_dir=tmp_path)
        assert config.threshold == 15
        # Other checks still present from defaults
        assert "lexical_blacklist" in config.checks

    def test_explicit_path(self, tmp_path: Path) -> None:
        custom = tmp_path / "custom.toml"
        custom.write_text('[general]\nthreshold = 50\n')
        config = load_config(path=custom)
        assert config.threshold == 50
