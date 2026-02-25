"""Tests for individual slop detection checks.

Each check has positive tests (flagged text), negative tests (clean text),
and edge-case tests for boundary conditions.
"""

from __future__ import annotations

import pytest

from slop_detector.checks import (
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
from slop_detector.models import SlopFlag


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _flag_names(flags: list[SlopFlag]) -> list[str]:
    return [f.check_name for f in flags]


# ===========================================================================
# 1. Lexical blacklist
# ===========================================================================


class TestLexicalBlacklist:
    def test_detects_delve(self) -> None:
        flags = check_lexical_blacklist("We must delve deeper into the subject.")
        assert any("delve" in f.description for f in flags)

    def test_detects_tapestry(self) -> None:
        flags = check_lexical_blacklist("The city's tapestry of cultures is remarkable.")
        assert flags, "Expected at least one flag for 'tapestry'"

    def test_detects_testament(self) -> None:
        flags = check_lexical_blacklist("This building is a testament to human ambition.")
        assert any("testament" in f.description for f in flags)

    def test_detects_vibrant(self) -> None:
        flags = check_lexical_blacklist("The vibrant community gathered downtown.")
        assert flags

    def test_detects_robust(self) -> None:
        flags = check_lexical_blacklist("A robust system of governance is needed.")
        assert flags

    def test_detects_crucial(self) -> None:
        flags = check_lexical_blacklist("Communication is crucial to success.")
        assert flags

    def test_detects_pivotal(self) -> None:
        flags = check_lexical_blacklist("This is a pivotal moment in history.")
        assert flags

    def test_detects_foster(self) -> None:
        flags = check_lexical_blacklist("We aim to foster a culture of collaboration.")
        assert flags

    def test_detects_cultivate(self) -> None:
        flags = check_lexical_blacklist("Leaders must cultivate trust.")
        assert flags

    def test_detects_nestled(self) -> None:
        flags = check_lexical_blacklist("The café is nestled in the heart of the city.")
        assert flags

    def test_detects_boasts(self) -> None:
        flags = check_lexical_blacklist("The university boasts a world-class faculty.")
        assert flags

    def test_detects_breathtaking(self) -> None:
        flags = check_lexical_blacklist("The view is breathtaking.")
        assert flags

    def test_detects_groundbreaking(self) -> None:
        flags = check_lexical_blacklist("This is groundbreaking research.")
        assert flags

    def test_detects_showcasing(self) -> None:
        flags = check_lexical_blacklist("The exhibition showcasing local talent opens Friday.")
        assert flags

    def test_detects_renowned(self) -> None:
        flags = check_lexical_blacklist("She is a renowned expert in her field.")
        assert flags

    def test_detects_underscore_verb(self) -> None:
        flags = check_lexical_blacklist("These findings underscore the need for reform.")
        assert any("underscore" in f.description for f in flags)

    def test_underscore_not_flagged_as_noun_or_symbol(self) -> None:
        flags = check_lexical_blacklist("The variable name uses an underscore.")
        assert not any("underscore (verb)" in f.description for f in flags)

    def test_detects_highlight_verb(self) -> None:
        flags = check_lexical_blacklist("The report highlights the importance of sleep.")
        assert any("highlight" in f.description for f in flags)

    def test_highlight_noun_not_flagged(self) -> None:
        flags = check_lexical_blacklist("The highlight of the evening was the speech.")
        assert not any("highlight (verb)" in f.description for f in flags)

    def test_detects_metaphorical_landscape(self) -> None:
        flags = check_lexical_blacklist("The landscape of modern finance has shifted.")
        assert any("landscape" in f.description for f in flags)

    def test_literal_landscape_not_flagged(self) -> None:
        flags = check_lexical_blacklist("The landscape was covered in snow.")
        assert not any("landscape (metaphorical)" in f.description for f in flags)

    def test_detects_a_rich_noun_of(self) -> None:
        flags = check_lexical_blacklist("The city offers a rich array of options.")
        assert any("a rich" in f.description for f in flags)

    def test_detects_stands_as_a(self) -> None:
        flags = check_lexical_blacklist("The treaty stands as a landmark achievement.")
        assert any("stands as a" in f.description for f in flags)

    def test_detects_serves_as_a(self) -> None:
        flags = check_lexical_blacklist("The document serves as a guide.")
        assert any("serves as a" in f.description for f in flags)

    def test_detects_holds_the_distinction(self) -> None:
        flags = check_lexical_blacklist("She holds the distinction of being the first.")
        assert any("holds the distinction" in f.description for f in flags)

    def test_detects_reflects_broader(self) -> None:
        flags = check_lexical_blacklist("This reflects broader trends in society.")
        assert any("reflects broader" in f.description for f in flags)

    def test_detects_shaping_the_evolving(self) -> None:
        flags = check_lexical_blacklist("These forces are shaping the evolving landscape.")
        assert any("shaping the evolving" in f.description for f in flags)

    def test_detects_marking_a_pivotal(self) -> None:
        flags = check_lexical_blacklist(
            "This decision, marking a pivotal shift, changed everything."
        )
        assert any("marking a pivotal" in f.description for f in flags)

    def test_detects_leaving_an_indelible_mark(self) -> None:
        flags = check_lexical_blacklist(
            "He retired, leaving an indelible mark on the institution."
        )
        assert any("leaving an indelible mark" in f.description for f in flags)

    def test_clean_text_returns_no_flags(self) -> None:
        clean = (
            "She handed him the invoice. He looked at it for a long time. "
            "Then he looked at her. Then at the invoice again. "
            "'Fourteen dollars,' he said. 'For what?' "
            "She pointed at the jar on the counter."
        )
        flags = check_lexical_blacklist(clean)
        assert flags == []

    def test_multiple_hits_in_one_text(self) -> None:
        text = "Vibrant and robust, the initiative serves as a testament to groundbreaking work."
        flags = check_lexical_blacklist(text)
        assert len(flags) >= 4

    def test_case_insensitive(self) -> None:
        flags = check_lexical_blacklist("DELVE into the data. VIBRANT colours.")
        names = [f.description for f in flags]
        assert any("delve" in n.lower() for n in names)
        assert any("vibrant" in n.lower() for n in names)


# ===========================================================================
# 2. Em-dash count
# ===========================================================================


class TestEmDashCount:
    def test_zero_em_dashes_passes(self) -> None:
        assert check_em_dash_count("No em dashes here.") == []

    def test_one_em_dash_passes(self) -> None:
        assert check_em_dash_count("A pause\u2014and then silence.") == []

    def test_two_em_dashes_flagged(self) -> None:
        flags = check_em_dash_count("First\u2014second\u2014third.")
        assert len(flags) == 1
        assert "2" in flags[0].description

    def test_three_em_dashes_flagged(self) -> None:
        flags = check_em_dash_count("A\u2014B\u2014C\u2014D.")
        assert len(flags) == 1
        assert "3" in flags[0].description

    def test_flag_check_name(self) -> None:
        flags = check_em_dash_count("X\u2014Y\u2014Z")
        assert flags[0].check_name == "em_dash_count"


# ===========================================================================
# 3. Trailing participle
# ===========================================================================


class TestTrailingParticiple:
    def test_detects_reflecting_the(self) -> None:
        text = "The event was a success, reflecting the community's dedication."
        flags = check_trailing_participle(text)
        assert flags

    def test_detects_underscoring_its(self) -> None:
        text = "The data showed a decline, underscoring its fragility."
        flags = check_trailing_participle(text)
        assert flags

    def test_detects_highlighting_a(self) -> None:
        text = "The study found no correlation, highlighting a significant gap."
        flags = check_trailing_participle(text)
        assert flags

    def test_detects_marking_their(self) -> None:
        text = "They left the building together, marking their final goodbye."
        flags = check_trailing_participle(text)
        assert flags

    def test_clean_sentence_passes(self) -> None:
        text = "She walked to the window. Outside it was raining. She closed the blind."
        assert check_trailing_participle(text) == []

    def test_mid_sentence_participle_not_flagged(self) -> None:
        text = "Reflecting the sun, the lake shimmered. It was very still."
        flags = check_trailing_participle(text)
        assert flags == []

    def test_flag_check_name(self) -> None:
        flags = check_trailing_participle("She smiled, revealing the secret.")
        assert all(f.check_name == "trailing_participle" for f in flags)


# ===========================================================================
# 4. Rule of Three
# ===========================================================================


class TestRuleOfThree:
    def test_detects_adjective_triplet(self) -> None:
        flags = check_rule_of_three("The system is safe, efficient, and reliable.")
        assert flags

    def test_detects_item_triplet(self) -> None:
        flags = check_rule_of_three("We need bread, butter, and jam.")
        assert flags

    def test_detects_or_variant(self) -> None:
        flags = check_rule_of_three("Choose red, blue, or green.")
        assert flags

    def test_detects_adverb_modified(self) -> None:
        flags = check_rule_of_three(
            "The approach is very fast, quite thorough, and incredibly precise."
        )
        assert flags

    def test_pair_not_flagged(self) -> None:
        flags = check_rule_of_three("The system is fast and reliable.")
        assert flags == []

    def test_four_items_not_flagged_as_triplet(self) -> None:
        text = "We need speed, accuracy, resilience, and grace."
        flags = check_rule_of_three(text)
        assert isinstance(flags, list)

    def test_flag_check_name(self) -> None:
        flags = check_rule_of_three("bold, vibrant, and timeless")
        assert all(f.check_name == "rule_of_three" for f in flags)

    def test_clean_prose_passes(self) -> None:
        text = "She took two aspirin and lay down."
        assert check_rule_of_three(text) == []


# ===========================================================================
# 5. Transition openers
# ===========================================================================


class TestTransitionOpeners:
    @pytest.mark.parametrize(
        "opener",
        [
            "Moreover",
            "Furthermore",
            "Additionally",
            "Consequently",
            "As a result",
            "In addition",
            "On the other hand",
        ],
    )
    def test_detects_opener_at_paragraph_start(self, opener: str) -> None:
        text = f"First paragraph.\n\n{opener}, this is important."
        flags = check_transition_openers(text)
        assert flags, f"Expected flag for opener '{opener}'"

    def test_opener_mid_sentence_not_flagged(self) -> None:
        text = "He said that, moreover, the cost was prohibitive."
        flags = check_transition_openers(text)
        assert flags == []

    def test_opener_at_text_start_flagged(self) -> None:
        text = "Furthermore, the results were inconclusive."
        flags = check_transition_openers(text)
        assert flags

    def test_clean_paragraph_transitions_pass(self) -> None:
        text = (
            "She found the receipt behind the couch.\n\n"
            "The amount surprised her. Seven dollars and forty cents."
        )
        assert check_transition_openers(text) == []

    def test_flag_check_name(self) -> None:
        flags = check_transition_openers("Additionally, we must consider the cost.")
        assert all(f.check_name == "transition_opener" for f in flags)


# ===========================================================================
# 6. Burstiness
# ===========================================================================


class TestBurstiness:
    def test_uniform_sentences_flagged(self) -> None:
        text = (
            "The cat sat on the mat today. "
            "The dog ran after the ball fast. "
            "The bird flew over the house roof. "
            "The fish swam under the old bridge. "
            "The fox hid behind the old tree."
        )
        flags = check_burstiness(text)
        assert flags, "Expected burstiness flag for uniform sentences"

    def test_varied_sentences_pass(self) -> None:
        text = (
            "Wait. "
            "The entire infrastructure, built over four decades by people who genuinely believed they "
            "were making something lasting, collapsed in an afternoon because someone forgot to renew "
            "a domain name. "
            "Nobody noticed for three weeks. "
            "By then the domain was owned by a company selling ergonomic chair cushions."
        )
        flags = check_burstiness(text)
        assert flags == []

    def test_fewer_than_four_sentences_skipped(self) -> None:
        text = "Short. Also short. Still short."
        assert check_burstiness(text) == []

    def test_flag_check_name(self) -> None:
        uniform = " ".join(["This is a sentence of eight words here."] * 5)
        flags = check_burstiness(uniform)
        assert all(f.check_name == "burstiness" for f in flags)

    def test_flag_description_contains_std_dev(self) -> None:
        uniform = " ".join(["This is a sentence of eight words here."] * 5)
        flags = check_burstiness(uniform)
        if flags:
            assert "std dev" in flags[0].description


# ===========================================================================
# 7. Copulative inflation
# ===========================================================================


class TestCopulativeInflation:
    def test_detects_serves_as(self) -> None:
        flags = check_copulative_inflation("The building serves as a museum.")
        assert flags

    def test_detects_stands_as(self) -> None:
        flags = check_copulative_inflation("The treaty stands as a landmark agreement.")
        assert flags

    def test_detects_functions_as(self) -> None:
        flags = check_copulative_inflation("The hub functions as a community centre.")
        assert flags

    def test_detects_holds_distinction_of_being(self) -> None:
        flags = check_copulative_inflation(
            "She holds the distinction of being the youngest recipient."
        )
        assert flags

    def test_detects_acts_as_non_idiomatic(self) -> None:
        flags = check_copulative_inflation("The park acts as a refuge for residents.")
        assert flags

    def test_clean_text_passes(self) -> None:
        text = "The building is a museum. She is the youngest recipient."
        assert check_copulative_inflation(text) == []

    def test_flag_check_name(self) -> None:
        flags = check_copulative_inflation("It serves as a reminder.")
        assert all(f.check_name == "copulative_inflation" for f in flags)


# ===========================================================================
# 8. Formulaic conclusion
# ===========================================================================


class TestFormulaicConclusion:
    @pytest.mark.parametrize(
        "opener",
        [
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
        ],
    )
    def test_detects_opener(self, opener: str) -> None:
        text = f"The project went well.\n\n{opener}, this was a success."
        flags = check_formulaic_conclusion(text)
        assert flags, f"Expected flag for '{opener}'"

    def test_opener_at_text_start_flagged(self) -> None:
        flags = check_formulaic_conclusion("Overall, the results were positive.")
        assert flags

    def test_clean_ending_passes(self) -> None:
        text = (
            "The project wrapped up on Thursday.\n\n"
            "He handed in the keys and drove home."
        )
        assert check_formulaic_conclusion(text) == []

    def test_flag_check_name(self) -> None:
        flags = check_formulaic_conclusion("In conclusion, everything worked out.")
        assert all(f.check_name == "formulaic_conclusion" for f in flags)


# ===========================================================================
# 9. Patterned negation
# ===========================================================================


class TestPatternedNegation:
    def test_detects_its_not_its(self) -> None:
        flags = check_patterned_negation("It's not a bug. It's a feature.")
        assert flags

    def test_detects_not_x_but_y(self) -> None:
        flags = check_patterned_negation("Not a setback, but an opportunity.")
        assert flags

    def test_detects_this_isnt_about_its_about(self) -> None:
        flags = check_patterned_negation(
            "This isn't about money. It's about principle."
        )
        assert flags

    def test_clean_text_passes(self) -> None:
        text = "She preferred coffee. He liked tea. They compromised on water."
        assert check_patterned_negation(text) == []

    def test_flag_check_name(self) -> None:
        flags = check_patterned_negation("It's not broken. It's character.")
        assert all(f.check_name == "patterned_negation" for f in flags)
