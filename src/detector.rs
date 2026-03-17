//! Slop detection orchestrator.
//!
//! Wires check functions to configuration and produces scored results.
//! No LLM calls — pure regex and string analysis.

use std::collections::BTreeMap;

use crate::checks::*;
use crate::config::Config;
use crate::models::{CheckScore, SlopFlag, SlopResult};

type CheckFn = fn(&str, Option<&toml::Table>) -> Vec<SlopFlag>;

struct CheckDef {
    func: CheckFn,
    name: &'static str,
    penalty_per_flag: u32,
    max_penalty: u32,
}

const DEFAULT_CHECKS: &[CheckDef] = &[
    CheckDef {
        func: check_lexical_blacklist,
        name: "lexical_blacklist",
        penalty_per_flag: 8,
        max_penalty: 60,
    },
    CheckDef {
        func: check_em_dash_count,
        name: "em_dash_count",
        penalty_per_flag: 10,
        max_penalty: 10,
    },
    CheckDef {
        func: check_trailing_participle,
        name: "trailing_participle",
        penalty_per_flag: 10,
        max_penalty: 30,
    },
    CheckDef {
        func: check_rule_of_three,
        name: "rule_of_three",
        penalty_per_flag: 5,
        max_penalty: 20,
    },
    CheckDef {
        func: check_transition_openers,
        name: "transition_openers",
        penalty_per_flag: 8,
        max_penalty: 24,
    },
    CheckDef {
        func: check_burstiness,
        name: "burstiness",
        penalty_per_flag: 20,
        max_penalty: 20,
    },
    CheckDef {
        func: check_copulative_inflation,
        name: "copulative_inflation",
        penalty_per_flag: 5,
        max_penalty: 20,
    },
    CheckDef {
        func: check_formulaic_conclusion,
        name: "formulaic_conclusion",
        penalty_per_flag: 10,
        max_penalty: 20,
    },
    CheckDef {
        func: check_patterned_negation,
        name: "patterned_negation",
        penalty_per_flag: 5,
        max_penalty: 15,
    },
    CheckDef {
        func: check_throat_clearing,
        name: "throat_clearing",
        penalty_per_flag: 8,
        max_penalty: 24,
    },
    CheckDef {
        func: check_chatbot_artifacts,
        name: "chatbot_artifacts",
        penalty_per_flag: 10,
        max_penalty: 20,
    },
    CheckDef {
        func: check_paragraph_uniformity,
        name: "paragraph_uniformity",
        penalty_per_flag: 15,
        max_penalty: 15,
    },
    CheckDef {
        func: check_emphasis_crutches,
        name: "emphasis_crutches",
        penalty_per_flag: 5,
        max_penalty: 15,
    },
    CheckDef {
        func: check_vague_attribution,
        name: "vague_attribution",
        penalty_per_flag: 5,
        max_penalty: 15,
    },
    CheckDef {
        func: check_wordiness,
        name: "wordiness",
        penalty_per_flag: 3,
        max_penalty: 12,
    },
];

/// Resolved check with penalties (possibly overridden by config).
struct ResolvedCheck {
    func: CheckFn,
    name: &'static str,
    penalty_per_flag: u32,
    max_penalty: u32,
}

/// Run all slop checks on `text` and return a SlopResult.
///
/// - `score` is in [0, 100] where 0 is pristine and 100 is maximum slop.
/// - `flags` lists every individual match from every check.
/// - `passed` is true when `score < slop_threshold`.
pub fn analyze(text: &str, slop_threshold: u32, config: Option<&Config>) -> SlopResult {
    if text.trim().is_empty() {
        return SlopResult {
            score: 0,
            flags: Vec::new(),
            passed: true,
            check_scores: BTreeMap::new(),
        };
    }

    let (threshold, checks) = if let Some(cfg) = config {
        let t = if slop_threshold == 30 {
            cfg.threshold
        } else {
            slop_threshold
        };
        (t, resolve_checks(cfg))
    } else {
        (
            slop_threshold,
            DEFAULT_CHECKS
                .iter()
                .map(|c| ResolvedCheck {
                    func: c.func,
                    name: c.name,
                    penalty_per_flag: c.penalty_per_flag,
                    max_penalty: c.max_penalty,
                })
                .collect(),
        )
    };

    let mut all_flags: Vec<SlopFlag> = Vec::new();
    let mut raw_penalty: u32 = 0;
    let mut applicable_max: u32 = 0;
    let mut check_scores: BTreeMap<String, CheckScore> = BTreeMap::new();

    for check in &checks {
        // Get check-specific params from config
        let params = config
            .and_then(|cfg| cfg.checks.get(check.name))
            .map(|cc| &cc.params);

        let flags = (check.func)(text, params);
        let contribution = (flags.len() as u32 * check.penalty_per_flag).min(check.max_penalty);
        raw_penalty += contribution;

        if !flags.is_empty() {
            // Check fired — count its max toward the denominator
            applicable_max += check.max_penalty;
            check_scores.insert(
                check.name.to_string(),
                CheckScore {
                    penalty: contribution,
                    max: check.max_penalty,
                    flags: flags.len() as u32,
                },
            );
        }

        all_flags.extend(flags);
    }

    // Floor: denominator must be at least the sum of the 3 largest check
    // max_penalties. Prevents a single check on short text from scoring 100.
    let mut all_maxes: Vec<u32> = checks.iter().map(|c| c.max_penalty).collect();
    all_maxes.sort_unstable_by(|a, b| b.cmp(a));
    let floor: u32 = all_maxes.iter().take(3).sum();
    let effective_max = applicable_max.max(floor);

    let max_raw = if effective_max == 0 { 1 } else { effective_max };
    let score = ((raw_penalty as f64 / max_raw as f64) * 100.0).floor() as u32;
    let score = score.min(100);

    SlopResult {
        score,
        flags: all_flags,
        passed: score < threshold,
        check_scores,
    }
}

fn resolve_checks(config: &Config) -> Vec<ResolvedCheck> {
    let fn_map: std::collections::HashMap<&str, CheckFn> =
        DEFAULT_CHECKS.iter().map(|c| (c.name, c.func)).collect();

    let mut checks = Vec::new();
    for default in DEFAULT_CHECKS {
        if let Some(cc) = config.checks.get(default.name) {
            if !cc.enabled {
                continue;
            }
            checks.push(ResolvedCheck {
                func: *fn_map.get(default.name).unwrap(),
                name: default.name,
                penalty_per_flag: cc.penalty_per_flag,
                max_penalty: cc.max_penalty,
            });
        } else {
            checks.push(ResolvedCheck {
                func: default.func,
                name: default.name,
                penalty_per_flag: default.penalty_per_flag,
                max_penalty: default.max_penalty,
            });
        }
    }
    checks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string_returns_zero() {
        let result = analyze("", 30, None);
        assert_eq!(result.score, 0);
        assert!(result.flags.is_empty());
        assert!(result.passed);
    }

    #[test]
    fn test_whitespace_only_returns_zero() {
        let result = analyze("   \n\n\t  ", 30, None);
        assert_eq!(result.score, 0);
        assert!(result.passed);
    }

    #[test]
    fn test_clean_text_scores_low() {
        let clean = "She found fourteen dollars in the pocket of a coat she hadn't worn since 2019. \
                      The coat smelled like a restaurant that no longer exists. \
                      She put the money back.";
        let result = analyze(clean, 30, None);
        assert!(result.score < 30);
        assert!(result.passed);
    }

    #[test]
    fn test_sloppy_text_scores_high() {
        let slop = "This groundbreaking initiative serves as a testament to the vibrant, robust, and crucial \
                     work being done by renowned experts. It's not just research. It's a movement. \
                     Furthermore, the tapestry of collaboration here is breathtaking, highlighting its \
                     potential to shape the evolving landscape of modern science\u{2014}and\u{2014}indeed\u{2014}\
                     leaving an indelible mark on the field. In conclusion, we must delve deeper and foster \
                     greater understanding, cultivating new perspectives that reflect broader trends. \
                     Overall, this stands as a pivotal moment.";
        let result = analyze(slop, 30, None);
        assert!(result.score >= 30);
        assert!(!result.passed);
    }

    #[test]
    fn test_score_bounded_0_to_100() {
        let result = analyze(&"x".repeat(10_000), 30, None);
        assert!(result.score <= 100);
    }

    #[test]
    fn test_custom_threshold_changes_passed() {
        let text = "This serves as a crucial and robust system. Furthermore, it delves into vibrant \
                     new territory, highlighting its groundbreaking potential.";
        let strict = analyze(text, 5, None);
        let lenient = analyze(text, 100, None);
        assert!(!strict.passed);
        assert!(lenient.passed);
    }

    #[test]
    fn test_flags_have_required_fields() {
        let result = analyze("The vibrant tapestry of innovation.", 30, None);
        for flag in &result.flags {
            assert!(!flag.check_name.is_empty());
            assert!(!flag.description.is_empty());
            assert!(!flag.severity.is_empty());
        }
    }

    #[test]
    fn test_all_check_names_represented() {
        let mega_slop = "\
            This groundbreaking work serves as a testament. \
            It delves into the vibrant, robust, and crucial landscape of innovation. \
            She holds the distinction of being renowned. \
            They cultivate and foster the tapestry, showcasing breathtaking results. \
            This stands as a pivotal moment, reflecting broader trends. \
            A pause\u{2014}another pause\u{2014}one more pause. \
            The summit concluded, reflecting the community's deep connection. \
            Make no mistake about it. In order to succeed, many experts agree this is key. \
            \n\nFurthermore, we must note the following. \
            It's not a setback. It's an opportunity. \
            Here's the thing: this matters. \
            Great question! Feel free to ask more. \
            \n\nIn conclusion, this was groundbreaking. \
            The system is safe, efficient, and reliable. ";

        let uniform_block = vec!["The team worked on the project every single day."; 6].join(" ");

        let full_text = format!("{mega_slop}\n\n{uniform_block}");

        let result = analyze(&full_text, 30, None);
        let triggered: std::collections::HashSet<&str> =
            result.flags.iter().map(|f| f.check_name.as_str()).collect();

        let expected = [
            "lexical_blacklist",
            "em_dash_count",
            "trailing_participle",
            "transition_opener",
            "patterned_negation",
            "formulaic_conclusion",
            "rule_of_three",
            "copulative_inflation",
            "burstiness",
            "throat_clearing",
            "chatbot_artifacts",
            // paragraph_uniformity tested separately (requires all paragraphs uniform)
            "emphasis_crutches",
            "vague_attribution",
            "wordiness",
        ];

        let missing: Vec<&&str> = expected
            .iter()
            .filter(|name| !triggered.contains(**name))
            .collect();
        assert!(
            missing.is_empty(),
            "These checks did not fire: {:?}",
            missing
        );
    }

    #[test]
    fn test_single_check_on_short_text_does_not_score_100() {
        // Short uniform-sentence text triggers only burstiness.
        // Before the floor fix this scored 100 (20/20). With the floor
        // the denominator is the sum of the 3 largest check maxes, so
        // a single check alone can't blow up the score.
        let short = "Queue p75 under target across all platforms. \
                      Kochiku p75 has held at seven seconds since October. \
                      Stress test scaled to thirty-eight thousand workers. \
                      New subnet deployed for additional worker capacity. \
                      Artifactory investigation confirmed pull storm was benign. \
                      NFS mount slowdown identified as queue time impact.";
        let result = analyze(short, 30, None);
        assert!(
            result.score < 30,
            "Single-check short text scored {} (expected < 30)",
            result.score
        );
        assert!(result.passed);
    }

    #[test]
    fn test_score_increases_with_more_flags() {
        let light = analyze("The vibrant community gathered.", 30, None);
        let heavy = analyze(
            "The vibrant tapestry of groundbreaking and crucial work serves as a \
             testament to robust innovation. It's not just research. It's a movement. \
             Furthermore, delve deeper into the breathtaking landscape of science. \
             In conclusion, this stands as pivotal. Safe, efficient, and reliable.",
            30,
            None,
        );
        assert!(heavy.score >= light.score);
    }
}
