"""CLI for slop-detector."""

from __future__ import annotations

import json
import sys

import click

from .config import Config, dump_config, load_config
from .detector import analyze
from .voice import generate_voice_directive


@click.group()
@click.version_option(package_name="slop-detector")
def main() -> None:
    """Fast regex-based detection of AI prose tells."""


@main.command()
@click.argument("file", type=click.Path(exists=True), required=False)
@click.option("-t", "--threshold", type=int, default=None, help="Override pass/fail threshold.")
@click.option("-c", "--config", "config_path", type=click.Path(), default=None, help="Config file path.")
@click.option("-f", "--format", "fmt", type=click.Choice(["text", "json"]), default="text", help="Output format.")
@click.option("--disable", multiple=True, help="Disable a check by name (repeatable).")
@click.option("-q", "--quiet", is_flag=True, help="Only print score and pass/fail.")
def analyze_cmd(
    file: str | None,
    threshold: int | None,
    config_path: str | None,
    fmt: str,
    disable: tuple[str, ...],
    quiet: bool,
) -> None:
    """Analyze text for AI prose tells.

    Reads from FILE or stdin if no file is given.
    """
    # Read input
    if file:
        with open(file) as f:
            text = f.read()
    else:
        if sys.stdin.isatty():
            click.echo("Reading from stdin (Ctrl+D to end)...", err=True)
        text = sys.stdin.read()

    if not text.strip():
        click.echo("No text provided.", err=True)
        sys.exit(1)

    # Load config
    config: Config | None = None
    if config_path:
        config = load_config(path=config_path)
    else:
        try:
            config = load_config()
        except FileNotFoundError:
            config = None

    # Apply disabled checks
    if disable and config:
        for check_name in disable:
            if check_name in config.checks:
                config.checks[check_name].enabled = False

    # Run analysis
    result = analyze(
        text,
        slop_threshold=threshold or (config.threshold if config else 30),
        config=config,
    )

    # Output
    if fmt == "json":
        output = {
            "score": result.score,
            "threshold": threshold or (config.threshold if config else 30),
            "passed": result.passed,
            "flags": [
                {
                    "check_name": f.check_name,
                    "description": f.description,
                    "location": f.location,
                    "severity": f.severity,
                }
                for f in result.flags
            ],
            "summary": {
                "total_flags": len(result.flags),
                "checks_triggered": sorted({f.check_name for f in result.flags}),
            },
        }
        click.echo(json.dumps(output, indent=2))
    elif quiet:
        status = "PASS" if result.passed else "FAIL"
        click.echo(f"Score: {result.score}/100  {status}")
    else:
        status = "PASS" if result.passed else "FAIL"
        click.echo(f"Score: {result.score}/100  {status}")
        click.echo()
        if result.flags:
            for flag in result.flags:
                sev = f"[{flag.severity}]"
                click.echo(f"  {sev:10s} {flag.check_name}: {flag.description}")
                if flag.location:
                    click.echo(f"             {flag.location}")
            click.echo()
            checks_hit = len({f.check_name for f in result.flags})
            click.echo(f"{len(result.flags)} flag(s) from {checks_hit} check(s)")
        else:
            click.echo("  No flags detected. Text looks clean.")

    sys.exit(0 if result.passed else 1)


@main.command()
@click.option("-c", "--config", "config_path", type=click.Path(), default=None, help="Config file path.")
def voice(config_path: str | None) -> None:
    """Generate a voice directive prompt from the current configuration."""
    config = load_config(path=config_path) if config_path else load_config()
    click.echo(generate_voice_directive(config))


@main.command("config")
@click.option("--dump", "do_dump", is_flag=True, help="Print the fully resolved config.")
@click.option("--init", "do_init", is_flag=True, help="Create a .slop-detector.toml template.")
def config_cmd(do_dump: bool, do_init: bool) -> None:
    """Inspect or initialize configuration."""
    if do_init:
        target = ".slop-detector.toml"
        import shutil
        from pathlib import Path

        defaults = Path(__file__).parent / "defaults.toml"
        shutil.copy(defaults, target)
        click.echo(f"Created {target} from defaults. Edit to customize.")
        return

    if do_dump:
        config = load_config()
        click.echo(dump_config(config))
        return

    click.echo("Use --dump to show config or --init to create a template.")
