//! CLI for slopcheck.

use std::collections::BTreeSet;
use std::io::{IsTerminal, Read};
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand, ValueEnum};
use serde_json::json;

use slopcheck::config::{dump_config, load_config};
use slopcheck::detector::analyze;
use slopcheck::voice::generate_voice_directive;

#[derive(Parser)]
#[command(
    name = "slopcheck",
    version,
    about = "Fast regex-based detection of AI prose tells."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze text for AI prose tells. Reads from FILE(s) or stdin.
    Analyze {
        /// File(s) to analyze (reads stdin if omitted)
        file: Vec<PathBuf>,

        /// Override pass/fail threshold
        #[arg(short, long)]
        threshold: Option<u32>,

        /// Config file path
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_parser = ["text", "json"], default_value = "text")]
        format: String,

        /// Disable a check by name (repeatable)
        #[arg(long)]
        disable: Vec<String>,

        /// Run only this check (disables all others)
        #[arg(long)]
        only: Option<String>,

        /// Only print score and pass/fail
        #[arg(short, long)]
        quiet: bool,
    },

    /// Generate a voice directive prompt from the current configuration.
    Voice {
        /// Config file path
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Inspect or initialize configuration.
    Config {
        /// Print the fully resolved config.
        #[arg(long)]
        dump: bool,

        /// Create a .slopcheck.toml template.
        #[arg(long)]
        init: bool,
    },

    /// Install or uninstall the agent skill for AI coding agents (Claude Code, etc.).
    Skill {
        /// Action to perform.
        action: SkillAction,

        /// Target agent. Currently only "claude-code" is supported.
        #[arg(short, long, value_parser = ["claude-code"], default_value = "claude-code")]
        agent: String,
    },
}

#[derive(Clone, ValueEnum)]
enum SkillAction {
    /// Install the skill files to the agent's skills directory.
    Install,
    /// Remove the skill files from the agent's skills directory.
    Uninstall,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze {
            file,
            threshold,
            config: config_path,
            format,
            disable,
            only,
            quiet,
        } => cmd_analyze(file, threshold, config_path, format, disable, only, quiet),

        Commands::Voice {
            config: config_path,
        } => cmd_voice(config_path),

        Commands::Config { dump, init } => cmd_config(dump, init),

        Commands::Skill { action, agent } => cmd_skill(action, &agent),
    }
}

fn cmd_analyze(
    files: Vec<PathBuf>,
    threshold: Option<u32>,
    config_path: Option<PathBuf>,
    format: String,
    disable: Vec<String>,
    only: Option<String>,
    quiet: bool,
) {
    // Load config
    let mut config = if let Some(ref cp) = config_path {
        load_config(Some(cp.as_path()), None)
    } else {
        load_config(None, None)
    };

    // Apply --only: disable everything except the named check
    if let Some(ref only_name) = only {
        for (name, cc) in config.checks.iter_mut() {
            cc.enabled = name == only_name;
        }
    }

    // Apply --disable
    for check_name in &disable {
        if let Some(cc) = config.checks.get_mut(check_name) {
            cc.enabled = false;
        }
    }

    let effective_threshold = threshold.unwrap_or(config.threshold);

    if files.is_empty() {
        // Read from stdin
        if std::io::stdin().is_terminal() {
            eprintln!("Reading from stdin (Ctrl+D to end)...");
        }
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .unwrap_or_else(|e| {
                eprintln!("Error reading stdin: {e}");
                process::exit(2);
            });

        if buf.trim().is_empty() {
            eprintln!("No text provided.");
            process::exit(2);
        }

        let result = analyze(&buf, effective_threshold, Some(&config));
        print_result(&result, effective_threshold, &format, quiet, None);
        process::exit(if result.passed { 0 } else { 1 });
    } else if files.len() == 1 {
        // Single file
        let path = &files[0];
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {e}", path.display());
            process::exit(2);
        });

        if text.trim().is_empty() {
            eprintln!("No text provided.");
            process::exit(2);
        }

        let result = analyze(&text, effective_threshold, Some(&config));
        print_result(&result, effective_threshold, &format, quiet, None);
        process::exit(if result.passed { 0 } else { 1 });
    } else {
        // Multiple files
        let mut any_failed = false;
        for path in &files {
            let text = match std::fs::read_to_string(path) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Error reading {}: {e}", path.display());
                    any_failed = true;
                    continue;
                }
            };

            if text.trim().is_empty() {
                continue;
            }

            let result = analyze(&text, effective_threshold, Some(&config));
            if !result.passed {
                any_failed = true;
            }
            print_result(&result, effective_threshold, &format, quiet, Some(path));
        }
        process::exit(if any_failed { 1 } else { 0 });
    }
}

fn print_result(
    result: &slopcheck::SlopResult,
    effective_threshold: u32,
    format: &str,
    quiet: bool,
    file: Option<&PathBuf>,
) {
    let warnings = result
        .flags
        .iter()
        .filter(|f| f.severity == "warning")
        .count();
    let infos = result.flags.iter().filter(|f| f.severity == "info").count();
    let file_prefix = file
        .map(|p| format!("{}: ", p.display()))
        .unwrap_or_default();

    if format == "json" {
        let mut output = json!({
            "score": result.score,
            "threshold": effective_threshold,
            "passed": result.passed,
            "flags": result.flags.iter().map(|f| {
                json!({
                    "check_name": f.check_name,
                    "description": f.description,
                    "location": f.location,
                    "severity": f.severity,
                })
            }).collect::<Vec<_>>(),
            "check_scores": result.check_scores,
            "summary": {
                "total_flags": result.flags.len(),
                "warnings": warnings,
                "info": infos,
                "checks_triggered": result.flags.iter()
                    .map(|f| f.check_name.as_str())
                    .collect::<BTreeSet<_>>(),
            },
        });
        if let Some(path) = file {
            output["file"] = json!(path.display().to_string());
        }
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else if quiet {
        let status = if result.passed { "PASS" } else { "FAIL" };
        println!("{file_prefix}Score: {}/100  {status}", result.score);
    } else {
        let status = if result.passed { "PASS" } else { "FAIL" };
        println!("{file_prefix}Score: {}/100  {status}", result.score);
        println!();
        if result.flags.is_empty() {
            println!("  No flags detected. Text looks clean.");
        } else {
            // Per-check score breakdown
            for (name, cs) in &result.check_scores {
                println!(
                    "  {name:24} {penalty:>3}/{max:<3}  ({flags} flag{s})",
                    penalty = cs.penalty,
                    max = cs.max,
                    flags = cs.flags,
                    s = if cs.flags == 1 { "" } else { "s" },
                );
            }
            println!();
            // Individual flags
            for flag in &result.flags {
                let sev = format!("[{}]", flag.severity);
                println!("  {sev:10} {}: {}", flag.check_name, flag.description);
                if !flag.location.is_empty() {
                    println!("             {}", flag.location);
                }
            }
            println!();
            println!(
                "{} flag(s) from {} check(s): {} warning{}, {} info",
                result.flags.len(),
                result.check_scores.len(),
                warnings,
                if warnings == 1 { "" } else { "s" },
                infos,
            );
        }
    }
}

fn cmd_voice(config_path: Option<PathBuf>) {
    let config = if let Some(ref cp) = config_path {
        load_config(Some(cp.as_path()), None)
    } else {
        load_config(None, None)
    };
    println!("{}", generate_voice_directive(&config));
}

fn cmd_config(dump: bool, init: bool) {
    if init {
        let target = ".slopcheck.toml";
        let defaults = include_str!("defaults.toml");
        std::fs::write(target, defaults).unwrap_or_else(|e| {
            eprintln!("Error writing {target}: {e}");
            process::exit(2);
        });
        println!("Created {target} from defaults. Edit to customize.");
        return;
    }

    if dump {
        let config = load_config(None, None);
        println!("{}", dump_config(&config));
        return;
    }

    println!("Use --dump to show config or --init to create a template.");
}

const SKILL_MD: &str = include_str!("../SKILL.md");
const CHECKS_MD: &str = include_str!("../references/checks.md");
const CONTEXTUAL_REVIEW_MD: &str = include_str!("../references/contextual-review.md");

fn skill_dir(agent: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| {
        eprintln!("Could not determine home directory.");
        process::exit(2);
    });
    match agent {
        "claude-code" => PathBuf::from(home).join(".claude").join("skills").join("slopcheck"),
        _ => {
            eprintln!("Unsupported agent: {agent}");
            process::exit(2);
        }
    }
}

fn cmd_skill(action: SkillAction, agent: &str) {
    let dir = skill_dir(agent);

    match action {
        SkillAction::Install => {
            let refs_dir = dir.join("references");

            // Create directories
            std::fs::create_dir_all(&refs_dir).unwrap_or_else(|e| {
                eprintln!("Error creating {}: {e}", refs_dir.display());
                process::exit(2);
            });

            // Write files
            let skill_md = dir.join("SKILL.md");
            let checks_md = refs_dir.join("checks.md");
            let contextual_md = refs_dir.join("contextual-review.md");

            for (path, content) in [
                (skill_md.as_path(), SKILL_MD),
                (checks_md.as_path(), CHECKS_MD),
                (contextual_md.as_path(), CONTEXTUAL_REVIEW_MD),
            ] {
                std::fs::write(path, content).unwrap_or_else(|e| {
                    eprintln!("Error writing {}: {e}", path.display());
                    process::exit(2);
                });
            }

            println!("Installed slopcheck skill to {}", dir.display());
            println!();
            println!("The skill is now available. Restart your agent or start a new conversation to use it.");
        }
        SkillAction::Uninstall => {
            if !dir.exists() {
                println!("Nothing to uninstall — {} does not exist.", dir.display());
                return;
            }

            std::fs::remove_dir_all(&dir).unwrap_or_else(|e| {
                eprintln!("Error removing {}: {e}", dir.display());
                process::exit(2);
            });

            println!("Uninstalled slopcheck skill from {}", dir.display());
        }
    }
}
