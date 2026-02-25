"""Tests for voice directive generation."""

from __future__ import annotations

from slop_detector.config import load_config
from slop_detector.voice import generate_voice_directive


class TestVoiceDirective:
    def test_generates_non_empty_directive(self) -> None:
        directive = generate_voice_directive()
        assert len(directive) > 100

    def test_contains_lexical_section(self) -> None:
        directive = generate_voice_directive()
        assert "LEXICAL RESTRICTIONS" in directive
        assert "delve" in directive

    def test_contains_punctuation_section(self) -> None:
        directive = generate_voice_directive()
        assert "PUNCTUATION AND SYNTAX" in directive
        assert "em-dash" in directive

    def test_contains_structure_section(self) -> None:
        directive = generate_voice_directive()
        assert "RHYTHM AND STRUCTURE" in directive
        assert "sentence length" in directive.lower()

    def test_contains_tone_section(self) -> None:
        directive = generate_voice_directive()
        assert "TONE" in directive
        assert "hedging" in directive

    def test_reflects_config_changes(self, tmp_path) -> None:
        # Create a config that disables em_dash_count
        custom = tmp_path / ".slop-detector.toml"
        custom.write_text("[checks.em_dash_count]\nenabled = false\n")
        config = load_config(project_dir=tmp_path)
        directive = generate_voice_directive(config)
        # em-dash rule should not appear
        assert "em-dash" not in directive.lower()

    def test_custom_words_appear_in_directive(self, tmp_path) -> None:
        custom = tmp_path / ".slop-detector.toml"
        custom.write_text(
            '[checks.lexical_blacklist.words]\n'
            'simple = ["synergy", "leverage"]\n'
        )
        config = load_config(project_dir=tmp_path)
        directive = generate_voice_directive(config)
        assert "synergy" in directive
        assert "leverage" in directive
