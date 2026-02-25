//! CLI for slop-detector.

use std::collections::BTreeSet;
use std::io::{IsTerminal, Read};
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use serde_json::json;

use slop_detector::config::{dump_config, load_config};
use slop_detector::detector::analyze;
use slop_detector::voice::generate_voice_directive;

#[derive(Parser)]
#[command(name = "slop-detector", version, about = "Fast regex-based detection of AI prose tells.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze text for AI prose tells. Reads from FILE or stdin.
    Analyze {
        /// File to analyze (reads stdin if omitted)
        file: Option<PathBuf>,

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

        /// Create a .slop-detector.toml template.
        #[arg(long)]
        init: bool,
    },
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
            quiet,
        } => cmd_analyze(file, threshold, config_path, format, disable, quiet),

        Commands::Voice { config: config_path } => cmd_voice(config_path),

        Commands::Config { dump, init } => cmd_config(dump, init),
    }
}

fn cmd_analyze(
    file: Option<PathBuf>,
    threshold: Option<u32>,
    config_path: Option<PathBuf>,
    format: String,
    disable: Vec<String>,
    quiet: bool,
) {
    // Read input
    let text = if let Some(path) = file {
        std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {e}", path.display());
            process::exit(1);
        })
    } else {
        if std::io::stdin().is_terminal() {
            eprintln!("Reading from stdin (Ctrl+D to end)...");
        }
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
            eprintln!("Error reading stdin: {e}", );
            process::exit(1);
        });
        buf
    };

    if text.trim().is_empty() {
        eprintln!("No text provided.");
        process::exit(1);
    }

    // Load config
    let mut config = if let Some(ref cp) = config_path {
        load_config(Some(cp.as_path()), None)
    } else {
        load_config(None, None)
    };

    // Apply disabled checks
    for check_name in &disable {
        if let Some(cc) = config.checks.get_mut(check_name) {
            cc.enabled = false;
        }
    }

    let effective_threshold = threshold.unwrap_or(config.threshold);

    // Run analysis
    let result = analyze(&text, effective_threshold, Some(&config));

    // Output
    if format == "json" {
        let output = json!({
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
            "summary": {
                "total_flags": result.flags.len(),
                "checks_triggered": result.flags.iter()
                    .map(|f| f.check_name.as_str())
                    .collect::<BTreeSet<_>>(),
            },
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else if quiet {
        let status = if result.passed { "PASS" } else { "FAIL" };
        println!("Score: {}/100  {status}", result.score);
    } else {
        let status = if result.passed { "PASS" } else { "FAIL" };
        println!("Score: {}/100  {status}", result.score);
        println!();
        if result.flags.is_empty() {
            println!("  No flags detected. Text looks clean.");
        } else {
            for flag in &result.flags {
                let sev = format!("[{}]", flag.severity);
                println!("  {sev:10} {}: {}", flag.check_name, flag.description);
                if !flag.location.is_empty() {
                    println!("             {}", flag.location);
                }
            }
            println!();
            let checks_hit: BTreeSet<&str> =
                result.flags.iter().map(|f| f.check_name.as_str()).collect();
            println!(
                "{} flag(s) from {} check(s)",
                result.flags.len(),
                checks_hit.len()
            );
        }
    }

    process::exit(if result.passed { 0 } else { 1 });
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
        let target = ".slop-detector.toml";
        let defaults = include_str!("defaults.toml");
        std::fs::write(target, defaults).unwrap_or_else(|e| {
            eprintln!("Error writing {target}: {e}");
            process::exit(1);
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
