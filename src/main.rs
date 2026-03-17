//! CLI for sloppy.

use std::collections::BTreeSet;
use std::io::{IsTerminal, Read};
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand, ValueEnum};
use serde_json::json;

use sloppy::config::{dump_config, load_config};
use sloppy::detector::analyze;
use sloppy::voice::{generate_chat_prompt, generate_voice_directive};

#[derive(Parser)]
#[command(
    name = "sloppy",
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

    /// Generate a prompt to paste into any LLM chat window or system prompt.
    Prompt {
        /// What kind of prompt to generate
        #[arg(value_enum, default_value_t = PromptMode::Generate)]
        mode: PromptMode,

        /// Copy output to clipboard
        #[arg(long)]
        copy: bool,

        /// Config file path
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Inspect or initialize configuration.
    Config {
        /// Print the fully resolved config.
        #[arg(long)]
        dump: bool,

        /// Create a .sloppy.toml template.
        #[arg(long)]
        init: bool,
    },
}

#[derive(Clone, ValueEnum)]
enum PromptMode {
    /// Prompt for writing clean, human-sounding prose
    Generate,
    /// Prompt for rewriting sloppy text
    Cleanup,
    /// Raw system-level constraint block (for API system prompts)
    System,
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

        Commands::Prompt {
            mode,
            copy,
            config: config_path,
        } => cmd_prompt(mode, copy, config_path),

        Commands::Config { dump, init } => cmd_config(dump, init),
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
    result: &sloppy::SlopResult,
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

fn cmd_prompt(mode: PromptMode, copy: bool, config_path: Option<PathBuf>) {
    let config = if let Some(ref cp) = config_path {
        load_config(Some(cp.as_path()), None)
    } else {
        load_config(None, None)
    };

    let output = match mode {
        PromptMode::System => generate_voice_directive(&config),
        PromptMode::Generate => generate_chat_prompt(&config, "generate"),
        PromptMode::Cleanup => generate_chat_prompt(&config, "cleanup"),
    };

    println!("{output}");

    if copy {
        copy_to_clipboard(&output);
    }
}

fn copy_to_clipboard(text: &str) {
    use std::io::Write;
    use std::process::Command;

    // Try pbcopy (macOS), then xclip, then xsel
    let commands = [
        ("pbcopy", &[] as &[&str]),
        ("xclip", &["-selection", "clipboard"] as &[&str]),
        ("xsel", &["--clipboard", "--input"] as &[&str]),
    ];

    for (cmd, args) in &commands {
        if let Ok(mut child) = Command::new(cmd)
            .args(*args)
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(ref mut stdin) = child.stdin {
                if stdin.write_all(text.as_bytes()).is_ok() && child.wait().is_ok() {
                    eprintln!("Copied to clipboard.");
                    return;
                }
            }
        }
    }

    eprintln!("Could not copy to clipboard (no pbcopy, xclip, or xsel found).");
}

fn cmd_config(dump: bool, init: bool) {
    if init {
        let target = ".sloppy.toml";
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
