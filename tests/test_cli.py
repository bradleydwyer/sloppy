"""Tests for the CLI."""

from __future__ import annotations

import json

from click.testing import CliRunner

from slop_detector.cli import main


class TestAnalyzeCommand:
    def test_stdin_clean_text(self) -> None:
        runner = CliRunner()
        result = runner.invoke(main, ["analyze"], input="She put the money back.")
        assert result.exit_code == 0
        assert "PASS" in result.output

    def test_stdin_sloppy_text(self) -> None:
        runner = CliRunner()
        text = (
            "This groundbreaking initiative serves as a testament to the vibrant, robust, and crucial "
            "work being done by renowned experts. Furthermore, the tapestry of collaboration here is "
            "breathtaking, highlighting its potential. In conclusion, we must delve deeper."
        )
        result = runner.invoke(main, ["analyze"], input=text)
        assert result.exit_code == 1
        assert "FAIL" in result.output

    def test_json_output(self) -> None:
        runner = CliRunner()
        text = "The vibrant tapestry delves into groundbreaking territory."
        result = runner.invoke(main, ["analyze", "-f", "json"], input=text)
        data = json.loads(result.output)
        assert "score" in data
        assert "flags" in data
        assert "passed" in data
        assert isinstance(data["flags"], list)

    def test_quiet_mode(self) -> None:
        runner = CliRunner()
        result = runner.invoke(main, ["analyze", "-q"], input="Clean text here.")
        assert "Score:" in result.output
        # Quiet mode should not show individual flags
        lines = result.output.strip().split("\n")
        assert len(lines) == 1

    def test_custom_threshold(self) -> None:
        runner = CliRunner()
        text = "The vibrant community gathered."
        # With very lenient threshold, should pass
        result = runner.invoke(main, ["analyze", "-t", "100"], input=text)
        assert result.exit_code == 0

    def test_file_input(self, tmp_path) -> None:
        f = tmp_path / "test.md"
        f.write_text("She put the money back.")
        runner = CliRunner()
        result = runner.invoke(main, ["analyze", str(f)])
        assert result.exit_code == 0

    def test_empty_input(self) -> None:
        runner = CliRunner()
        result = runner.invoke(main, ["analyze"], input="")
        assert result.exit_code == 1


class TestVoiceCommand:
    def test_generates_output(self) -> None:
        runner = CliRunner()
        result = runner.invoke(main, ["voice"])
        assert result.exit_code == 0
        assert "LEXICAL RESTRICTIONS" in result.output


class TestConfigCommand:
    def test_dump(self) -> None:
        runner = CliRunner()
        result = runner.invoke(main, ["config", "--dump"])
        assert result.exit_code == 0
        assert "threshold" in result.output

    def test_init(self, tmp_path) -> None:
        runner = CliRunner()
        with runner.isolated_filesystem(temp_dir=tmp_path):
            result = runner.invoke(main, ["config", "--init"])
            assert result.exit_code == 0
            assert "Created" in result.output
