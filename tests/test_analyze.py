"""Tests for the analyze() public API."""

from __future__ import annotations

from slop_detector import analyze
from slop_detector.models import SlopResult


class TestAnalyze:
    def test_empty_string_returns_zero_score(self) -> None:
        result = analyze("")
        assert result.score == 0
        assert result.flags == []
        assert result.passed is True

    def test_whitespace_only_returns_zero_score(self) -> None:
        result = analyze("   \n\n\t  ")
        assert result.score == 0
        assert result.passed is True

    def test_clean_text_scores_low(self) -> None:
        clean = (
            "She found fourteen dollars in the pocket of a coat she hadn't worn since 2019. "
            "The coat smelled like a restaurant that no longer exists. "
            "She put the money back."
        )
        result = analyze(clean)
        assert result.score < 30
        assert result.passed is True

    def test_sloppy_text_scores_high(self) -> None:
        slop = (
            "This groundbreaking initiative serves as a testament to the vibrant, robust, and crucial "
            "work being done by renowned experts. It's not just research. It's a movement. "
            "Furthermore, the tapestry of collaboration here is breathtaking, highlighting its "
            "potential to shape the evolving landscape of modern science\u2014and\u2014indeed\u2014"
            "leaving an indelible mark on the field. In conclusion, we must delve deeper and foster "
            "greater understanding, cultivating new perspectives that reflect broader trends. "
            "Overall, this stands as a pivotal moment."
        )
        result = analyze(slop)
        assert result.score >= 30
        assert result.passed is False

    def test_returns_slop_result_type(self) -> None:
        result = analyze("Some text.")
        assert isinstance(result, SlopResult)

    def test_flags_list_populated_on_sloppy_text(self) -> None:
        result = analyze("The vibrant tapestry of innovation delves into the groundbreaking.")
        assert len(result.flags) > 0

    def test_score_bounded_0_to_100(self) -> None:
        result = analyze("x" * 10_000)
        assert 0 <= result.score <= 100

    def test_custom_threshold_changes_passed(self) -> None:
        text = (
            "This serves as a crucial and robust system. Furthermore, it delves into vibrant "
            "new territory, highlighting its groundbreaking potential."
        )
        result_strict = analyze(text, slop_threshold=5)
        result_lenient = analyze(text, slop_threshold=100)
        assert result_strict.passed is False
        assert result_lenient.passed is True

    def test_flags_have_required_fields(self) -> None:
        result = analyze("The vibrant tapestry of innovation.")
        for flag in result.flags:
            assert isinstance(flag.check_name, str) and flag.check_name
            assert isinstance(flag.description, str) and flag.description
            assert isinstance(flag.severity, str)
            assert isinstance(flag.location, str)

    def test_all_check_names_represented_in_sloppy_text(self) -> None:
        mega_slop = (
            # lexical blacklist
            "This groundbreaking work serves as a testament. "
            "It delves into the vibrant, robust, and crucial landscape of innovation. "
            "She holds the distinction of being renowned. "
            "They cultivate and foster the tapestry, showcasing breathtaking results. "
            "This stands as a pivotal moment, reflecting broader trends. "
            # em-dash
            "A pause\u2014another pause\u2014one more pause. "
            # trailing participle
            "The summit concluded, reflecting the community's deep connection. "
            # transition opener
            "\n\nFurthermore, we must note the following. "
            # patterned negation
            "It's not a setback. It's an opportunity. "
            # formulaic conclusion
            "\n\nIn conclusion, this was groundbreaking. "
            # rule of three
            "The system is safe, efficient, and reliable. "
        )
        # Add repeated identical sentences for burstiness failure.
        uniform_block = " ".join(
            ["The team worked on the project every single day."] * 6
        )
        full_text = mega_slop + "\n\n" + uniform_block

        result = analyze(full_text)
        triggered = {f.check_name for f in result.flags}

        expected_checks = {
            "lexical_blacklist",
            "em_dash_count",
            "trailing_participle",
            "transition_opener",
            "patterned_negation",
            "formulaic_conclusion",
            "rule_of_three",
            "copulative_inflation",
            "burstiness",
        }
        missing = expected_checks - triggered
        assert not missing, f"These checks did not fire: {missing}"

    def test_score_increases_with_more_flags(self) -> None:
        light = analyze("The vibrant community gathered.")
        heavy = analyze(
            "The vibrant tapestry of groundbreaking and crucial work serves as a "
            "testament to robust innovation. It's not just research. It's a movement. "
            "Furthermore, delve deeper into the breathtaking landscape of science. "
            "In conclusion, this stands as pivotal. Safe, efficient, and reliable."
        )
        assert heavy.score >= light.score
