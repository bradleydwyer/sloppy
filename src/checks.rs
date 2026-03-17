//! All slop detection checks — pure regex, no LLM calls.
//!
//! Each check function takes text (and optional params) and returns a Vec of SlopFlags.
//! The checks are stateless and deterministic.

use regex::Regex;
use std::sync::LazyLock;

use crate::models::SlopFlag;

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn sentences(text: &str) -> Vec<String> {
    static RE_WHITESPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());
    // Match sentence-ending punctuation followed by whitespace and an uppercase letter/quote/paren.
    // Rust regex doesn't support look-behind, so we capture the boundary and reconstruct.
    static RE_BOUNDARY: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"([.!?])\s+(["'(A-Z])"#).unwrap());

    let flat = RE_WHITESPACE.replace_all(text.trim(), " ");
    // Insert a sentinel at each sentence boundary so we can split on it.
    let marked = RE_BOUNDARY.replace_all(&flat, "$1\x00$2");
    let parts: Vec<String> = marked
        .split('\x00')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    parts
}

fn word_count(sentence: &str) -> usize {
    sentence.split_whitespace().count()
}

// ---------------------------------------------------------------------------
// 1. Lexical blacklist
// ---------------------------------------------------------------------------

struct WordPattern {
    regex: Regex,
    label: &'static str,
}

static DEFAULT_WORD_PATTERNS: LazyLock<Vec<WordPattern>> = LazyLock::new(|| {
    vec![
        // Single words — word-boundary anchored
        WordPattern {
            regex: Regex::new(r"(?i)\bdelve\b").unwrap(),
            label: "delve",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\btapestry\b").unwrap(),
            label: "tapestry",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\btestament\b").unwrap(),
            label: "testament",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\bvibrant\b").unwrap(),
            label: "vibrant",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\brobust\b").unwrap(),
            label: "robust",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\bcrucial\b").unwrap(),
            label: "crucial",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\bpivotal\b").unwrap(),
            label: "pivotal",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\bfoster\b").unwrap(),
            label: "foster",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\bcultivate\b").unwrap(),
            label: "cultivate",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\bnestled\b").unwrap(),
            label: "nestled",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\bboasts\b").unwrap(),
            label: "boasts",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\bbreathtaking\b").unwrap(),
            label: "breathtaking",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\bgroundbreaking\b").unwrap(),
            label: "groundbreaking",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\bshowcasing\b").unwrap(),
            label: "showcasing",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\brenowned\b").unwrap(),
            label: "renowned",
        },
        // Verb uses
        WordPattern {
            regex: Regex::new(r"(?i)\bunderscore[sd]?\s+the\b").unwrap(),
            label: "underscore (verb)",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\bhighlight[sd]?\s+(?:the|its|their|our|a|an)\b").unwrap(),
            label: "highlight (verb)",
        },
        // Metaphorical landscape
        WordPattern {
            regex: Regex::new(r"(?i)\b(?:the|a|an)\s+landscape\s+of\b").unwrap(),
            label: "landscape (metaphorical)",
        },
        // Multi-word phrases
        WordPattern {
            regex: Regex::new(r"(?i)\ba rich \w+ of\b").unwrap(),
            label: "a rich [noun] of",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\bstands as a\b").unwrap(),
            label: "stands as a",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\bserves as a\b").unwrap(),
            label: "serves as a",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\bholds the distinction\b").unwrap(),
            label: "holds the distinction",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\breflects broader\b").unwrap(),
            label: "reflects broader",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\bshaping the evolving\b").unwrap(),
            label: "shaping the evolving",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\bmarking a pivotal\b").unwrap(),
            label: "marking a pivotal",
        },
        WordPattern {
            regex: Regex::new(r"(?i)\bleaving an indelible mark\b").unwrap(),
            label: "leaving an indelible mark",
        },
    ]
});

pub fn check_lexical_blacklist(text: &str, params: Option<&toml::Table>) -> Vec<SlopFlag> {
    let mut flags = Vec::new();

    if let Some(params) = params {
        // Config-driven patterns
        let patterns = build_word_patterns(params);
        for (re, label) in &patterns {
            for m in re.find_iter(text) {
                let snippet = snippet_around(text, m.start(), m.end(), 20);
                flags.push(SlopFlag::warning(
                    "lexical_blacklist",
                    &format!("Banned phrase \"{label}\" found"),
                    &format!("...\"{}\"...", snippet),
                ));
            }
        }
    } else {
        // Default patterns
        for wp in DEFAULT_WORD_PATTERNS.iter() {
            for m in wp.regex.find_iter(text) {
                let snippet = snippet_around(text, m.start(), m.end(), 20);
                flags.push(SlopFlag::warning(
                    "lexical_blacklist",
                    &format!("Banned phrase \"{}\" found", wp.label),
                    &format!("...\"{}\"...", snippet),
                ));
            }
        }
    }

    flags
}

fn build_word_patterns(params: &toml::Table) -> Vec<(Regex, String)> {
    let mut patterns = Vec::new();

    // Simple words get auto-wrapped in word boundaries
    if let Some(words) = params
        .get("words")
        .and_then(|w| w.as_table())
        .and_then(|w| w.get("simple"))
        .and_then(|s| s.as_array())
    {
        for word in words {
            if let Some(w) = word.as_str()
                && let Ok(re) = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(w)))
            {
                patterns.push((re, w.to_string()));
            }
        }
    }

    // Explicit regex patterns
    if let Some(entries) = params
        .get("patterns")
        .and_then(|p| p.as_table())
        .and_then(|p| p.get("entries"))
        .and_then(|e| e.as_array())
    {
        for entry in entries {
            if let Some(arr) = entry.as_array()
                && arr.len() >= 2
                && let (Some(pat), Some(label)) = (arr[0].as_str(), arr[1].as_str())
                && let Ok(re) = Regex::new(&format!("(?i){pat}"))
            {
                patterns.push((re, label.to_string()));
            }
        }
    }

    patterns
}

fn snippet_around(text: &str, start: usize, end: usize, context: usize) -> String {
    // Work with char boundaries
    let s = text
        .char_indices()
        .rev()
        .find(|&(i, _)| i <= start.saturating_sub(context))
        .map(|(i, _)| i)
        .unwrap_or(0);
    let e = text
        .char_indices()
        .find(|&(i, _)| i >= (end + context).min(text.len()))
        .map(|(i, _)| i)
        .unwrap_or(text.len());
    text[s..e].replace('\n', " ").trim().to_string()
}

// ---------------------------------------------------------------------------
// 2. Em-dash count
// ---------------------------------------------------------------------------

pub fn check_em_dash_count(text: &str, params: Option<&toml::Table>) -> Vec<SlopFlag> {
    let max_allowed = params
        .and_then(|p| p.get("max_allowed"))
        .and_then(|v| v.as_integer())
        .unwrap_or(0) as usize;

    let em_dashes = text.chars().filter(|&c| c == '\u{2014}').count();
    if em_dashes <= max_allowed {
        return Vec::new();
    }

    vec![SlopFlag::warning(
        "em_dash_count",
        &format!("Text contains {em_dashes} em-dashes (max {max_allowed} allowed)"),
        "",
    )]
}

// ---------------------------------------------------------------------------
// 3. Trailing participle
// ---------------------------------------------------------------------------

static TRAILING_PARTICIPLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i),\s+[A-Za-z]+ing\s+(?:the|its|their|our|an?|his|her|this|that|each|all)\b[^.!?]*[.!?]",
    )
    .unwrap()
});

pub fn check_trailing_participle(text: &str, _params: Option<&toml::Table>) -> Vec<SlopFlag> {
    let mut flags = Vec::new();
    for m in TRAILING_PARTICIPLE_RE.find_iter(text) {
        let snippet: String = m.as_str().chars().take(80).collect();
        let snippet = snippet.replace('\n', " ");
        flags.push(SlopFlag::warning(
            "trailing_participle",
            "Trailing participial phrase detected",
            &format!("...\"{}\"...", snippet),
        ));
    }
    flags
}

// ---------------------------------------------------------------------------
// 4. Rule of three
// ---------------------------------------------------------------------------

static RULE_OF_THREE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(?:(?:very|quite|rather|truly|deeply|highly|incredibly|extremely)\s+)?[A-Za-z]{2,},\s+(?:(?:very|quite|rather|truly|deeply|highly|incredibly|extremely)\s+)?[A-Za-z]{2,},\s+(?:and|or)\s+(?:(?:very|quite|rather|truly|deeply|highly|incredibly|extremely)\s+)?[A-Za-z]{2,}\b",
    )
    .unwrap()
});

pub fn check_rule_of_three(text: &str, _params: Option<&toml::Table>) -> Vec<SlopFlag> {
    let mut flags = Vec::new();
    for m in RULE_OF_THREE_RE.find_iter(text) {
        let snippet: String = m.as_str().chars().take(80).collect();
        flags.push(SlopFlag::info(
            "rule_of_three",
            "Rule-of-three triplet detected",
            &format!("\"{}\"", snippet),
        ));
    }
    flags
}

// ---------------------------------------------------------------------------
// 5. Transition openers
// ---------------------------------------------------------------------------

const DEFAULT_TRANSITION_OPENERS: &[&str] = &[
    "Moreover",
    "Furthermore",
    "Additionally",
    "Consequently",
    "As a result",
    "In addition",
    "On the other hand",
];

pub fn check_transition_openers(text: &str, params: Option<&toml::Table>) -> Vec<SlopFlag> {
    let banned: Vec<String> = if let Some(p) = params {
        p.get("banned")
            .and_then(|b| b.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(|| {
                DEFAULT_TRANSITION_OPENERS
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            })
    } else {
        DEFAULT_TRANSITION_OPENERS
            .iter()
            .map(|s| s.to_string())
            .collect()
    };

    let escaped: Vec<String> = banned.iter().map(|b| regex::escape(b)).collect();
    let pattern = format!(r"(?i)(?:^|\n\n)[ \t]*({})\b", escaped.join("|"));
    let re = Regex::new(&pattern).unwrap();

    let mut flags = Vec::new();
    for caps in re.captures_iter(text) {
        let matched = caps.get(1).unwrap().as_str();
        let full: String = caps
            .get(0)
            .unwrap()
            .as_str()
            .trim()
            .chars()
            .take(60)
            .collect();
        flags.push(SlopFlag::warning(
            "transition_opener",
            &format!("Paragraph opens with banned transition \"{matched}\""),
            &format!("\"{}\"", full),
        ));
    }
    flags
}

// ---------------------------------------------------------------------------
// 6. Burstiness
// ---------------------------------------------------------------------------

pub fn check_burstiness(text: &str, params: Option<&toml::Table>) -> Vec<SlopFlag> {
    let threshold = params
        .and_then(|p| p.get("std_dev_threshold"))
        .and_then(|v| v.as_float())
        .unwrap_or(5.0);
    let min_sentences = params
        .and_then(|p| p.get("min_sentences"))
        .and_then(|v| v.as_integer())
        .unwrap_or(4) as usize;

    let sents = sentences(text);
    if sents.len() < min_sentences {
        return Vec::new();
    }

    let lengths: Vec<f64> = sents.iter().map(|s| word_count(s) as f64).collect();
    let n = lengths.len() as f64;
    let mean = lengths.iter().sum::<f64>() / n;
    let variance = lengths.iter().map(|l| (l - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std_dev = variance.sqrt();

    if std_dev >= threshold {
        return Vec::new();
    }

    vec![SlopFlag::warning(
        "burstiness",
        &format!(
            "Sentence lengths too uniform (std dev {:.1} < {threshold}). Mean sentence length: {mean:.1} words across {} sentences.",
            std_dev,
            sents.len()
        ),
        "",
    )]
}

// ---------------------------------------------------------------------------
// 7. Copulative inflation
// ---------------------------------------------------------------------------

struct CopulativePattern {
    regex: Regex,
    label: &'static str,
}

static DEFAULT_COPULATIVE_PATTERNS: LazyLock<Vec<CopulativePattern>> = LazyLock::new(|| {
    vec![
        CopulativePattern {
            regex: Regex::new(r"(?i)\bserves as\b").unwrap(),
            label: "serves as",
        },
        CopulativePattern {
            regex: Regex::new(r"(?i)\bstand(?:s|ing)?\s+as\b").unwrap(),
            label: "stands as",
        },
        CopulativePattern {
            regex: Regex::new(r"(?i)\bfunction(?:s|ing)?\s+as\b").unwrap(),
            label: "functions as",
        },
        CopulativePattern {
            regex: Regex::new(r"(?i)\bholds? the distinction of being\b").unwrap(),
            label: "holds the distinction of being",
        },
        CopulativePattern {
            regex: Regex::new(r"(?i)\bacts? as\b").unwrap(),
            label: "acts as",
        },
    ]
});

pub fn check_copulative_inflation(text: &str, params: Option<&toml::Table>) -> Vec<SlopFlag> {
    let mut flags = Vec::new();

    if let Some(raw_patterns) = params
        .and_then(|p| p.get("patterns"))
        .and_then(|v| v.as_array())
    {
        for entry in raw_patterns {
            if let Some(arr) = entry.as_array()
                && arr.len() >= 2
                && let (Some(pat), Some(label)) = (arr[0].as_str(), arr[1].as_str())
                && let Ok(re) = Regex::new(&format!("(?i){pat}"))
            {
                for m in re.find_iter(text) {
                    let snippet = snippet_around(text, m.start(), m.end(), 15);
                    flags.push(SlopFlag::info(
                        "copulative_inflation",
                        &format!("Copulative inflation \"{label}\" — prefer \"is/are\""),
                        &format!("...\"{}\"...", snippet),
                    ));
                }
            }
        }
    } else {
        static RE_DETERRENT: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"(?i)\bacts? as\s+a\s+deterrent\b").unwrap());

        for cp in DEFAULT_COPULATIVE_PATTERNS.iter() {
            for m in cp.regex.find_iter(text) {
                // Skip "acts as a deterrent" — idiomatic usage
                if cp.label == "acts as" && RE_DETERRENT.is_match(&text[m.start()..]) {
                    continue;
                }
                let snippet = snippet_around(text, m.start(), m.end(), 15);
                flags.push(SlopFlag::info(
                    "copulative_inflation",
                    &format!("Copulative inflation \"{}\" — prefer \"is/are\"", cp.label),
                    &format!("...\"{}\"...", snippet),
                ));
            }
        }
    }

    flags
}

// ---------------------------------------------------------------------------
// 8. Formulaic conclusion
// ---------------------------------------------------------------------------

const DEFAULT_CONCLUSION_OPENERS: &[&str] = &[
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
];

pub fn check_formulaic_conclusion(text: &str, params: Option<&toml::Table>) -> Vec<SlopFlag> {
    let openers: Vec<String> = if let Some(p) = params {
        p.get("openers")
            .and_then(|o| o.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(|| {
                DEFAULT_CONCLUSION_OPENERS
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            })
    } else {
        DEFAULT_CONCLUSION_OPENERS
            .iter()
            .map(|s| s.to_string())
            .collect()
    };

    let escaped: Vec<String> = openers.iter().map(|o| regex::escape(o)).collect();
    let pattern = format!(r"(?i)(?:^|\n+)\s*({})\b", escaped.join("|"));
    let re = Regex::new(&pattern).unwrap();

    let mut flags = Vec::new();
    for caps in re.captures_iter(text) {
        let matched = caps.get(1).unwrap().as_str();
        let full: String = caps
            .get(0)
            .unwrap()
            .as_str()
            .trim()
            .chars()
            .take(60)
            .collect();
        flags.push(SlopFlag::warning(
            "formulaic_conclusion",
            &format!("Formulaic conclusion opener \"{matched}\""),
            &format!("\"{}\"", full),
        ));
    }
    flags
}

// ---------------------------------------------------------------------------
// 9. Patterned negation
// ---------------------------------------------------------------------------

struct NegationPattern {
    regex: Regex,
    label: &'static str,
}

static DEFAULT_NEGATION_PATTERNS: LazyLock<Vec<NegationPattern>> = LazyLock::new(|| {
    vec![
        NegationPattern {
            regex: Regex::new(r"(?is)It'?s?\s+not\b[^.!?]{1,80}[.!?]\s+It'?s?\b").unwrap(),
            label: "It's not X. It's Y.",
        },
        NegationPattern {
            regex: Regex::new(r"(?i)\bNot\s+\w[\w\s]{1,40},\s+but\b").unwrap(),
            label: "Not X, but Y",
        },
        NegationPattern {
            regex: Regex::new(r"(?is)\b(?:This|That|These|Those)\s+isn'?t\s+about\b[^.!?]{1,80}[.!?]\s+It'?s?\s+about\b").unwrap(),
            label: "This isn't about X. It's about Y.",
        },
    ]
});

pub fn check_patterned_negation(text: &str, params: Option<&toml::Table>) -> Vec<SlopFlag> {
    let mut flags = Vec::new();

    if let Some(raw_patterns) = params
        .and_then(|p| p.get("patterns"))
        .and_then(|v| v.as_array())
    {
        for entry in raw_patterns {
            if let Some(arr) = entry.as_array()
                && arr.len() >= 2
                && let (Some(pat), Some(label)) = (arr[0].as_str(), arr[1].as_str())
                && let Ok(re) = Regex::new(&format!("(?i){pat}"))
            {
                for m in re.find_iter(text) {
                    let snippet: String = m
                        .as_str()
                        .chars()
                        .take(80)
                        .collect::<String>()
                        .replace('\n', " ");
                    flags.push(SlopFlag::info(
                        "patterned_negation",
                        &format!("Patterned negation \"{label}\" detected"),
                        &format!("\"{}\"", snippet),
                    ));
                }
            }
        }
    } else {
        for np in DEFAULT_NEGATION_PATTERNS.iter() {
            for m in np.regex.find_iter(text) {
                let snippet: String = m
                    .as_str()
                    .chars()
                    .take(80)
                    .collect::<String>()
                    .replace('\n', " ");
                flags.push(SlopFlag::info(
                    "patterned_negation",
                    &format!("Patterned negation \"{}\" detected", np.label),
                    &format!("\"{}\"", snippet),
                ));
            }
        }
    }

    flags
}

// ---------------------------------------------------------------------------
// 10. Throat-clearing openers
// ---------------------------------------------------------------------------

struct ThroatClearingPattern {
    regex: Regex,
    label: &'static str,
}

static DEFAULT_THROAT_CLEARING_PATTERNS: LazyLock<Vec<ThroatClearingPattern>> =
    LazyLock::new(|| {
        vec![
            ThroatClearingPattern {
                regex: Regex::new(r"(?i)\bhere.s the (?:thing|catch|bind|problem)\b").unwrap(),
                label: "Here's the thing",
            },
            ThroatClearingPattern {
                regex: Regex::new(r"(?i)\bhere.s what I (?:mean|find interesting)\b").unwrap(),
                label: "Here's what I mean",
            },
            ThroatClearingPattern {
                regex: Regex::new(r"(?i)\blet me (?:be clear|explain)\b").unwrap(),
                label: "Let me be clear",
            },
            ThroatClearingPattern {
                regex: Regex::new(r"(?i)\bthe (?:uncomfortable )?truth is\b").unwrap(),
                label: "The truth is",
            },
            ThroatClearingPattern {
                regex: Regex::new(r"(?i)\bthe reality is\b").unwrap(),
                label: "The reality is",
            },
            ThroatClearingPattern {
                regex: Regex::new(r"(?i)(?:^|\.\s+)It turns out\b").unwrap(),
                label: "It turns out",
            },
            ThroatClearingPattern {
                regex: Regex::new(r"(?i)\bcan we talk about\b").unwrap(),
                label: "Can we talk about",
            },
            ThroatClearingPattern {
                regex: Regex::new(r"(?i)\bthink about it[.:]\b?").unwrap(),
                label: "Think about it",
            },
            ThroatClearingPattern {
                regex: Regex::new(r"(?i)\bconsider this[.:]\b?").unwrap(),
                label: "Consider this",
            },
            ThroatClearingPattern {
                regex: Regex::new(r"(?i)\band that.s okay\.\s*$").unwrap(),
                label: "And that's okay.",
            },
            ThroatClearingPattern {
                regex: Regex::new(r"(?i)\bbut here.s the (?:thing|catch|bind|kicker)\b").unwrap(),
                label: "But here's the thing",
            },
            ThroatClearingPattern {
                regex: Regex::new(r"(?i)\blet.s (?:be honest|be real|face it)\b").unwrap(),
                label: "Let's be honest",
            },
        ]
    });

pub fn check_throat_clearing(text: &str, params: Option<&toml::Table>) -> Vec<SlopFlag> {
    let mut flags = Vec::new();

    if let Some(raw_patterns) = params
        .and_then(|p| p.get("patterns"))
        .and_then(|v| v.as_array())
    {
        for entry in raw_patterns {
            if let Some(arr) = entry.as_array()
                && arr.len() >= 2
                && let (Some(pat), Some(label)) = (arr[0].as_str(), arr[1].as_str())
                && let Ok(re) = Regex::new(&format!("(?i){pat}"))
            {
                for m in re.find_iter(text) {
                    let snippet = snippet_around(text, m.start(), m.end(), 20);
                    flags.push(SlopFlag::warning(
                        "throat_clearing",
                        &format!("Throat-clearing opener \"{label}\" detected"),
                        &format!("...\"{}\"...", snippet),
                    ));
                }
            }
        }
    } else {
        for tc in DEFAULT_THROAT_CLEARING_PATTERNS.iter() {
            for m in tc.regex.find_iter(text) {
                let snippet = snippet_around(text, m.start(), m.end(), 20);
                flags.push(SlopFlag::warning(
                    "throat_clearing",
                    &format!("Throat-clearing opener \"{}\" detected", tc.label),
                    &format!("...\"{}\"...", snippet),
                ));
            }
        }
    }

    flags
}

// ---------------------------------------------------------------------------
// 11. Chatbot artifacts
// ---------------------------------------------------------------------------

struct ChatbotPattern {
    regex: Regex,
    label: &'static str,
}

static DEFAULT_CHATBOT_PATTERNS: LazyLock<Vec<ChatbotPattern>> = LazyLock::new(|| {
    vec![
        ChatbotPattern {
            regex: Regex::new(r"(?i)\b(?:great|excellent|wonderful|fantastic) question\b").unwrap(),
            label: "Great question!",
        },
        ChatbotPattern {
            regex: Regex::new(
                r"(?i)\bthat.s a (?:great|excellent|good|interesting) (?:question|point)\b",
            )
            .unwrap(),
            label: "That's a great question",
        },
        ChatbotPattern {
            regex: Regex::new(r"(?i)\bI.(?:d|would) be (?:happy|glad|delighted) to\b").unwrap(),
            label: "I'd be happy to",
        },
        ChatbotPattern {
            regex: Regex::new(r"(?i)\b(?:I )?hope (?:this|that) helps\b").unwrap(),
            label: "Hope this helps",
        },
        ChatbotPattern {
            regex: Regex::new(r"(?i)\blet me know if you\b").unwrap(),
            label: "Let me know if you",
        },
        ChatbotPattern {
            regex: Regex::new(r"(?i)\bfeel free to\b").unwrap(),
            label: "Feel free to",
        },
        ChatbotPattern {
            regex: Regex::new(r"(?i)(?:^|\.\s+)(?:Certainly|Absolutely)!\s").unwrap(),
            label: "Certainly!/Absolutely!",
        },
        ChatbotPattern {
            regex: Regex::new(r"(?i)\bas an AI\b").unwrap(),
            label: "As an AI",
        },
        ChatbotPattern {
            regex: Regex::new(r"(?i)\bas a language model\b").unwrap(),
            label: "As a language model",
        },
        ChatbotPattern {
            regex: Regex::new(r"(?i)\bhappy to help\b").unwrap(),
            label: "Happy to help",
        },
    ]
});

pub fn check_chatbot_artifacts(text: &str, params: Option<&toml::Table>) -> Vec<SlopFlag> {
    let mut flags = Vec::new();

    if let Some(raw_patterns) = params
        .and_then(|p| p.get("patterns"))
        .and_then(|v| v.as_array())
    {
        for entry in raw_patterns {
            if let Some(arr) = entry.as_array()
                && arr.len() >= 2
                && let (Some(pat), Some(label)) = (arr[0].as_str(), arr[1].as_str())
                && let Ok(re) = Regex::new(&format!("(?i){pat}"))
            {
                for m in re.find_iter(text) {
                    let snippet = snippet_around(text, m.start(), m.end(), 20);
                    flags.push(SlopFlag::warning(
                        "chatbot_artifacts",
                        &format!("Chatbot artifact \"{label}\" detected"),
                        &format!("...\"{}\"...", snippet),
                    ));
                }
            }
        }
    } else {
        for cp in DEFAULT_CHATBOT_PATTERNS.iter() {
            for m in cp.regex.find_iter(text) {
                let snippet = snippet_around(text, m.start(), m.end(), 20);
                flags.push(SlopFlag::warning(
                    "chatbot_artifacts",
                    &format!("Chatbot artifact \"{}\" detected", cp.label),
                    &format!("...\"{}\"...", snippet),
                ));
            }
        }
    }

    flags
}

// ---------------------------------------------------------------------------
// 12. Paragraph uniformity
// ---------------------------------------------------------------------------

pub fn check_paragraph_uniformity(text: &str, params: Option<&toml::Table>) -> Vec<SlopFlag> {
    let threshold = params
        .and_then(|p| p.get("std_dev_threshold"))
        .and_then(|v| v.as_float())
        .unwrap_or(1.0);
    let min_paragraphs = params
        .and_then(|p| p.get("min_paragraphs"))
        .and_then(|v| v.as_integer())
        .unwrap_or(4) as usize;

    let paragraphs: Vec<&str> = text
        .split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();

    if paragraphs.len() < min_paragraphs {
        return Vec::new();
    }

    let counts: Vec<f64> = paragraphs
        .iter()
        .map(|p| sentences(p).len() as f64)
        .collect();
    let n = counts.len() as f64;
    let mean = counts.iter().sum::<f64>() / n;
    let variance = counts.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std_dev = variance.sqrt();

    if std_dev >= threshold {
        return Vec::new();
    }

    vec![SlopFlag::warning(
        "paragraph_uniformity",
        &format!(
            "Paragraph lengths too uniform (std dev {std_dev:.1} < {threshold}). \
             Mean {mean:.1} sentences across {} paragraphs.",
            paragraphs.len()
        ),
        "",
    )]
}

// ---------------------------------------------------------------------------
// 13. Emphasis crutches
// ---------------------------------------------------------------------------

struct EmphasisPattern {
    regex: Regex,
    label: &'static str,
}

static DEFAULT_EMPHASIS_PATTERNS: LazyLock<Vec<EmphasisPattern>> = LazyLock::new(|| {
    vec![
        EmphasisPattern {
            regex: Regex::new(r"(?i)\bFull stop\.\s").unwrap(),
            label: "Full stop.",
        },
        EmphasisPattern {
            regex: Regex::new(r"(?i)(?:^|\.\s+)Period\.\s").unwrap(),
            label: "Period.",
        },
        EmphasisPattern {
            regex: Regex::new(r"(?i)\bLet that sink in\.").unwrap(),
            label: "Let that sink in.",
        },
        EmphasisPattern {
            regex: Regex::new(r"(?i)\bMake no mistake\b").unwrap(),
            label: "Make no mistake",
        },
        EmphasisPattern {
            regex: Regex::new(r"(?i)\bThis matters because\b").unwrap(),
            label: "This matters because",
        },
        EmphasisPattern {
            regex: Regex::new(r"(?i)\w+\.\s+That.s it\.\s+That.s the\b").unwrap(),
            label: "X. That's it. That's the Y.",
        },
    ]
});

pub fn check_emphasis_crutches(text: &str, params: Option<&toml::Table>) -> Vec<SlopFlag> {
    let mut flags = Vec::new();

    if let Some(raw_patterns) = params
        .and_then(|p| p.get("patterns"))
        .and_then(|v| v.as_array())
    {
        for entry in raw_patterns {
            if let Some(arr) = entry.as_array()
                && arr.len() >= 2
                && let (Some(pat), Some(label)) = (arr[0].as_str(), arr[1].as_str())
                && let Ok(re) = Regex::new(&format!("(?i){pat}"))
            {
                for m in re.find_iter(text) {
                    let snippet = snippet_around(text, m.start(), m.end(), 20);
                    flags.push(SlopFlag::info(
                        "emphasis_crutches",
                        &format!("Emphasis crutch \"{label}\" detected"),
                        &format!("...\"{}\"...", snippet),
                    ));
                }
            }
        }
    } else {
        for ep in DEFAULT_EMPHASIS_PATTERNS.iter() {
            for m in ep.regex.find_iter(text) {
                let snippet = snippet_around(text, m.start(), m.end(), 20);
                flags.push(SlopFlag::info(
                    "emphasis_crutches",
                    &format!("Emphasis crutch \"{}\" detected", ep.label),
                    &format!("...\"{}\"...", snippet),
                ));
            }
        }
    }

    flags
}

// ---------------------------------------------------------------------------
// 14. Vague attribution
// ---------------------------------------------------------------------------

struct AttributionPattern {
    regex: Regex,
    label: &'static str,
}

static DEFAULT_ATTRIBUTION_PATTERNS: LazyLock<Vec<AttributionPattern>> = LazyLock::new(|| {
    vec![
        AttributionPattern {
            regex: Regex::new(r"(?i)\bmany experts (?:agree|say|believe)\b").unwrap(),
            label: "many experts agree",
        },
        AttributionPattern {
            regex: Regex::new(r"(?i)\bexperts (?:say|suggest|note|agree|believe)\b").unwrap(),
            label: "experts say",
        },
        AttributionPattern {
            regex: Regex::new(r"(?i)\bsome (?:experts|critics|observers|analysts)\b").unwrap(),
            label: "some experts",
        },
        AttributionPattern {
            regex: Regex::new(
                r"(?i)\b(?:studies|research) (?:show|shows|suggest|suggests|indicate|indicates)\b",
            )
            .unwrap(),
            label: "studies show",
        },
        AttributionPattern {
            regex: Regex::new(r"(?i)\bindustry (?:reports?|experts?|analysts?) suggest\b").unwrap(),
            label: "industry reports suggest",
        },
        AttributionPattern {
            regex: Regex::new(
                r"(?i)\bit is (?:widely|generally) (?:believed|accepted|acknowledged)\b",
            )
            .unwrap(),
            label: "it is widely believed",
        },
        AttributionPattern {
            regex: Regex::new(r"(?i)\bobservers have (?:noted|cited)\b").unwrap(),
            label: "observers have noted",
        },
    ]
});

pub fn check_vague_attribution(text: &str, params: Option<&toml::Table>) -> Vec<SlopFlag> {
    let mut flags = Vec::new();

    if let Some(raw_patterns) = params
        .and_then(|p| p.get("patterns"))
        .and_then(|v| v.as_array())
    {
        for entry in raw_patterns {
            if let Some(arr) = entry.as_array()
                && arr.len() >= 2
                && let (Some(pat), Some(label)) = (arr[0].as_str(), arr[1].as_str())
                && let Ok(re) = Regex::new(&format!("(?i){pat}"))
            {
                for m in re.find_iter(text) {
                    let snippet = snippet_around(text, m.start(), m.end(), 20);
                    flags.push(SlopFlag::info(
                        "vague_attribution",
                        &format!("Vague attribution \"{label}\" — cite a specific source"),
                        &format!("...\"{}\"...", snippet),
                    ));
                }
            }
        }
    } else {
        for ap in DEFAULT_ATTRIBUTION_PATTERNS.iter() {
            for m in ap.regex.find_iter(text) {
                let snippet = snippet_around(text, m.start(), m.end(), 20);
                flags.push(SlopFlag::info(
                    "vague_attribution",
                    &format!(
                        "Vague attribution \"{}\" — cite a specific source",
                        ap.label
                    ),
                    &format!("...\"{}\"...", snippet),
                ));
            }
        }
    }

    flags
}

// ---------------------------------------------------------------------------
// 15. Wordiness
// ---------------------------------------------------------------------------

struct WordinessPattern {
    regex: Regex,
    label: &'static str,
    suggestion: &'static str,
}

static DEFAULT_WORDINESS_PATTERNS: LazyLock<Vec<WordinessPattern>> = LazyLock::new(|| {
    vec![
        WordinessPattern {
            regex: Regex::new(r"(?i)\bin order to\b").unwrap(),
            label: "in order to",
            suggestion: "to",
        },
        WordinessPattern {
            regex: Regex::new(r"(?i)\bdue to the fact that\b").unwrap(),
            label: "due to the fact that",
            suggestion: "because",
        },
        WordinessPattern {
            regex: Regex::new(r"(?i)\bat this point in time\b").unwrap(),
            label: "at this point in time",
            suggestion: "now",
        },
        WordinessPattern {
            regex: Regex::new(r"(?i)\bfor the purpose of\b").unwrap(),
            label: "for the purpose of",
            suggestion: "to/for",
        },
        WordinessPattern {
            regex: Regex::new(r"(?i)\ba large number of\b").unwrap(),
            label: "a large number of",
            suggestion: "many",
        },
        WordinessPattern {
            regex: Regex::new(r"(?i)\bit should be noted that\b").unwrap(),
            label: "it should be noted that",
            suggestion: "(cut)",
        },
        WordinessPattern {
            regex: Regex::new(r"(?i)\bthe fact that\b").unwrap(),
            label: "the fact that",
            suggestion: "(cut or rephrase)",
        },
    ]
});

pub fn check_wordiness(text: &str, params: Option<&toml::Table>) -> Vec<SlopFlag> {
    let mut flags = Vec::new();

    if let Some(raw_patterns) = params
        .and_then(|p| p.get("patterns"))
        .and_then(|v| v.as_array())
    {
        for entry in raw_patterns {
            if let Some(arr) = entry.as_array()
                && arr.len() >= 3
                && let (Some(pat), Some(label), Some(sug)) =
                    (arr[0].as_str(), arr[1].as_str(), arr[2].as_str())
                && let Ok(re) = Regex::new(&format!("(?i){pat}"))
            {
                for m in re.find_iter(text) {
                    let snippet = snippet_around(text, m.start(), m.end(), 15);
                    flags.push(SlopFlag::info(
                        "wordiness",
                        &format!("\"{label}\" — try \"{sug}\""),
                        &format!("...\"{}\"...", snippet),
                    ));
                }
            }
        }
    } else {
        for wp in DEFAULT_WORDINESS_PATTERNS.iter() {
            for m in wp.regex.find_iter(text) {
                let snippet = snippet_around(text, m.start(), m.end(), 15);
                flags.push(SlopFlag::info(
                    "wordiness",
                    &format!("\"{}\" — try \"{}\"", wp.label, wp.suggestion),
                    &format!("...\"{}\"...", snippet),
                ));
            }
        }
    }

    flags
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Lexical blacklist ---

    #[test]
    fn test_detects_delve() {
        let flags = check_lexical_blacklist("We must delve deeper into the subject.", None);
        assert!(flags.iter().any(|f| f.description.contains("delve")));
    }

    #[test]
    fn test_detects_tapestry() {
        let flags = check_lexical_blacklist("The city's tapestry of cultures is remarkable.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_testament() {
        let flags =
            check_lexical_blacklist("This building is a testament to human ambition.", None);
        assert!(flags.iter().any(|f| f.description.contains("testament")));
    }

    #[test]
    fn test_detects_vibrant() {
        let flags = check_lexical_blacklist("The vibrant community gathered downtown.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_robust() {
        let flags = check_lexical_blacklist("A robust system of governance is needed.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_crucial() {
        let flags = check_lexical_blacklist("Communication is crucial to success.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_pivotal() {
        let flags = check_lexical_blacklist("This is a pivotal moment in history.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_foster() {
        let flags = check_lexical_blacklist("We aim to foster a culture of collaboration.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_cultivate() {
        let flags = check_lexical_blacklist("Leaders must cultivate trust.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_nestled() {
        let flags = check_lexical_blacklist("The café is nestled in the heart of the city.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_boasts() {
        let flags = check_lexical_blacklist("The university boasts a world-class faculty.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_breathtaking() {
        let flags = check_lexical_blacklist("The view is breathtaking.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_groundbreaking() {
        let flags = check_lexical_blacklist("This is groundbreaking research.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_showcasing() {
        let flags =
            check_lexical_blacklist("The exhibition showcasing local talent opens Friday.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_renowned() {
        let flags = check_lexical_blacklist("She is a renowned expert in her field.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_underscore_verb() {
        let flags = check_lexical_blacklist("These findings underscore the need for reform.", None);
        assert!(flags.iter().any(|f| f.description.contains("underscore")));
    }

    #[test]
    fn test_underscore_not_flagged_as_noun() {
        let flags = check_lexical_blacklist("The variable name uses an underscore.", None);
        assert!(
            !flags
                .iter()
                .any(|f| f.description.contains("underscore (verb)"))
        );
    }

    #[test]
    fn test_detects_highlight_verb() {
        let flags = check_lexical_blacklist("The report highlights the importance of sleep.", None);
        assert!(flags.iter().any(|f| f.description.contains("highlight")));
    }

    #[test]
    fn test_highlight_noun_not_flagged() {
        let flags = check_lexical_blacklist("The highlight of the evening was the speech.", None);
        assert!(
            !flags
                .iter()
                .any(|f| f.description.contains("highlight (verb)"))
        );
    }

    #[test]
    fn test_detects_metaphorical_landscape() {
        let flags = check_lexical_blacklist("The landscape of modern finance has shifted.", None);
        assert!(flags.iter().any(|f| f.description.contains("landscape")));
    }

    #[test]
    fn test_literal_landscape_not_flagged() {
        let flags = check_lexical_blacklist("The landscape was covered in snow.", None);
        assert!(
            !flags
                .iter()
                .any(|f| f.description.contains("landscape (metaphorical)"))
        );
    }

    #[test]
    fn test_detects_a_rich_noun_of() {
        let flags = check_lexical_blacklist("The city offers a rich array of options.", None);
        assert!(flags.iter().any(|f| f.description.contains("a rich")));
    }

    #[test]
    fn test_detects_stands_as_a() {
        let flags = check_lexical_blacklist("The treaty stands as a landmark achievement.", None);
        assert!(flags.iter().any(|f| f.description.contains("stands as a")));
    }

    #[test]
    fn test_detects_serves_as_a() {
        let flags = check_lexical_blacklist("The document serves as a guide.", None);
        assert!(flags.iter().any(|f| f.description.contains("serves as a")));
    }

    #[test]
    fn test_detects_holds_the_distinction() {
        let flags = check_lexical_blacklist("She holds the distinction of being the first.", None);
        assert!(
            flags
                .iter()
                .any(|f| f.description.contains("holds the distinction"))
        );
    }

    #[test]
    fn test_detects_reflects_broader() {
        let flags = check_lexical_blacklist("This reflects broader trends in society.", None);
        assert!(
            flags
                .iter()
                .any(|f| f.description.contains("reflects broader"))
        );
    }

    #[test]
    fn test_detects_shaping_the_evolving() {
        let flags =
            check_lexical_blacklist("These forces are shaping the evolving landscape.", None);
        assert!(
            flags
                .iter()
                .any(|f| f.description.contains("shaping the evolving"))
        );
    }

    #[test]
    fn test_detects_marking_a_pivotal() {
        let flags = check_lexical_blacklist(
            "This decision, marking a pivotal shift, changed everything.",
            None,
        );
        assert!(
            flags
                .iter()
                .any(|f| f.description.contains("marking a pivotal"))
        );
    }

    #[test]
    fn test_detects_leaving_indelible_mark() {
        let flags = check_lexical_blacklist(
            "He retired, leaving an indelible mark on the institution.",
            None,
        );
        assert!(
            flags
                .iter()
                .any(|f| f.description.contains("leaving an indelible mark"))
        );
    }

    #[test]
    fn test_clean_text_no_flags() {
        let clean = "She handed him the invoice. He looked at it for a long time. \
                      Then he looked at her. Then at the invoice again. \
                      'Fourteen dollars,' he said. 'For what?' \
                      She pointed at the jar on the counter.";
        let flags = check_lexical_blacklist(clean, None);
        assert!(flags.is_empty());
    }

    #[test]
    fn test_multiple_hits() {
        let text =
            "Vibrant and robust, the initiative serves as a testament to groundbreaking work.";
        let flags = check_lexical_blacklist(text, None);
        assert!(flags.len() >= 4);
    }

    #[test]
    fn test_case_insensitive() {
        let flags = check_lexical_blacklist("DELVE into the data. VIBRANT colours.", None);
        assert!(
            flags
                .iter()
                .any(|f| f.description.to_lowercase().contains("delve"))
        );
        assert!(
            flags
                .iter()
                .any(|f| f.description.to_lowercase().contains("vibrant"))
        );
    }

    // --- Em-dash count ---

    #[test]
    fn test_zero_em_dashes_passes() {
        assert!(check_em_dash_count("No em dashes here.", None).is_empty());
    }

    #[test]
    fn test_one_em_dash_flagged() {
        let flags = check_em_dash_count("A pause\u{2014}and then silence.", None);
        assert_eq!(flags.len(), 1);
    }

    #[test]
    fn test_two_em_dashes_flagged() {
        let flags = check_em_dash_count("First\u{2014}second\u{2014}third.", None);
        assert_eq!(flags.len(), 1);
        assert!(flags[0].description.contains("2"));
    }

    #[test]
    fn test_three_em_dashes_flagged() {
        let flags = check_em_dash_count("A\u{2014}B\u{2014}C\u{2014}D.", None);
        assert_eq!(flags.len(), 1);
        assert!(flags[0].description.contains("3"));
    }

    #[test]
    fn test_em_dash_check_name() {
        let flags = check_em_dash_count("X\u{2014}Y\u{2014}Z", None);
        assert_eq!(flags[0].check_name, "em_dash_count");
    }

    // --- Trailing participle ---

    #[test]
    fn test_detects_reflecting_the() {
        let flags = check_trailing_participle(
            "The event was a success, reflecting the community's dedication.",
            None,
        );
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_underscoring_its() {
        let flags = check_trailing_participle(
            "The data showed a decline, underscoring its fragility.",
            None,
        );
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_highlighting_a() {
        let flags = check_trailing_participle(
            "The study found no correlation, highlighting a significant gap.",
            None,
        );
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_clean_sentence_passes_trailing() {
        let flags = check_trailing_participle(
            "She walked to the window. Outside it was raining. She closed the blind.",
            None,
        );
        assert!(flags.is_empty());
    }

    #[test]
    fn test_mid_sentence_participle_not_flagged() {
        let flags = check_trailing_participle(
            "Reflecting the sun, the lake shimmered. It was very still.",
            None,
        );
        assert!(flags.is_empty());
    }

    #[test]
    fn test_trailing_participle_check_name() {
        let flags = check_trailing_participle("She smiled, revealing the secret.", None);
        assert!(flags.iter().all(|f| f.check_name == "trailing_participle"));
    }

    // --- Rule of three ---

    #[test]
    fn test_detects_adjective_triplet() {
        let flags = check_rule_of_three("The system is safe, efficient, and reliable.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_item_triplet() {
        let flags = check_rule_of_three("We need bread, butter, and jam.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_or_variant() {
        let flags = check_rule_of_three("Choose red, blue, or green.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_pair_not_flagged() {
        let flags = check_rule_of_three("The system is fast and reliable.", None);
        assert!(flags.is_empty());
    }

    #[test]
    fn test_rule_of_three_check_name() {
        let flags = check_rule_of_three("bold, vibrant, and timeless", None);
        assert!(flags.iter().all(|f| f.check_name == "rule_of_three"));
    }

    #[test]
    fn test_clean_prose_passes_rot() {
        assert!(check_rule_of_three("She took two aspirin and lay down.", None).is_empty());
    }

    // --- Transition openers ---

    #[test]
    fn test_detects_opener_at_paragraph_start() {
        for opener in DEFAULT_TRANSITION_OPENERS {
            let text = format!("First paragraph.\n\n{opener}, this is important.");
            let flags = check_transition_openers(&text, None);
            assert!(!flags.is_empty(), "Expected flag for opener '{opener}'");
        }
    }

    #[test]
    fn test_opener_mid_sentence_not_flagged() {
        let flags =
            check_transition_openers("He said that, moreover, the cost was prohibitive.", None);
        assert!(flags.is_empty());
    }

    #[test]
    fn test_opener_at_text_start_flagged() {
        let flags = check_transition_openers("Furthermore, the results were inconclusive.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_clean_paragraph_transitions_pass() {
        let text = "She found the receipt behind the couch.\n\n\
                     The amount surprised her. Seven dollars and forty cents.";
        assert!(check_transition_openers(text, None).is_empty());
    }

    #[test]
    fn test_transition_opener_check_name() {
        let flags = check_transition_openers("Additionally, we must consider the cost.", None);
        assert!(flags.iter().all(|f| f.check_name == "transition_opener"));
    }

    // --- Burstiness ---

    #[test]
    fn test_uniform_sentences_flagged() {
        let text = "The cat sat on the mat today. \
                     The dog ran after the ball fast. \
                     The bird flew over the house roof. \
                     The fish swam under the old bridge. \
                     The fox hid behind the old tree.";
        let flags = check_burstiness(text, None);
        assert!(
            !flags.is_empty(),
            "Expected burstiness flag for uniform sentences"
        );
    }

    #[test]
    fn test_varied_sentences_pass() {
        let text = "Wait. \
                     The entire infrastructure, built over four decades by people who genuinely believed they \
                     were making something lasting, collapsed in an afternoon because someone forgot to renew \
                     a domain name. \
                     Nobody noticed for three weeks. \
                     By then the domain was owned by a company selling ergonomic chair cushions.";
        let flags = check_burstiness(text, None);
        assert!(flags.is_empty());
    }

    #[test]
    fn test_fewer_than_four_sentences_skipped() {
        assert!(check_burstiness("Short. Also short. Still short.", None).is_empty());
    }

    #[test]
    fn test_burstiness_check_name() {
        let uniform = vec!["This is a sentence of eight words here."; 5].join(" ");
        let flags = check_burstiness(&uniform, None);
        assert!(flags.iter().all(|f| f.check_name == "burstiness"));
    }

    #[test]
    fn test_burstiness_description_contains_std_dev() {
        let uniform = vec!["This is a sentence of eight words here."; 5].join(" ");
        let flags = check_burstiness(&uniform, None);
        if !flags.is_empty() {
            assert!(flags[0].description.contains("std dev"));
        }
    }

    // --- Copulative inflation ---

    #[test]
    fn test_detects_serves_as() {
        let flags = check_copulative_inflation("The building serves as a museum.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_stands_as_cop() {
        let flags = check_copulative_inflation("The treaty stands as a landmark agreement.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_functions_as() {
        let flags = check_copulative_inflation("The hub functions as a community centre.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_holds_distinction_of_being() {
        let flags = check_copulative_inflation(
            "She holds the distinction of being the youngest recipient.",
            None,
        );
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_acts_as() {
        let flags = check_copulative_inflation("The park acts as a refuge for residents.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_clean_text_passes_cop() {
        let flags = check_copulative_inflation(
            "The building is a museum. She is the youngest recipient.",
            None,
        );
        assert!(flags.is_empty());
    }

    #[test]
    fn test_copulative_check_name() {
        let flags = check_copulative_inflation("It serves as a reminder.", None);
        assert!(flags.iter().all(|f| f.check_name == "copulative_inflation"));
    }

    // --- Formulaic conclusion ---

    #[test]
    fn test_detects_conclusion_openers() {
        for opener in DEFAULT_CONCLUSION_OPENERS {
            let text = format!("The project went well.\n\n{opener}, this was a success.");
            let flags = check_formulaic_conclusion(&text, None);
            assert!(!flags.is_empty(), "Expected flag for '{opener}'");
        }
    }

    #[test]
    fn test_opener_at_text_start_flagged_conclusion() {
        let flags = check_formulaic_conclusion("Overall, the results were positive.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_clean_ending_passes() {
        let text = "The project wrapped up on Thursday.\n\n\
                     He handed in the keys and drove home.";
        assert!(check_formulaic_conclusion(text, None).is_empty());
    }

    #[test]
    fn test_formulaic_conclusion_check_name() {
        let flags = check_formulaic_conclusion("In conclusion, everything worked out.", None);
        assert!(flags.iter().all(|f| f.check_name == "formulaic_conclusion"));
    }

    // --- Patterned negation ---

    #[test]
    fn test_detects_its_not_its() {
        let flags = check_patterned_negation("It's not a bug. It's a feature.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_not_x_but_y() {
        let flags = check_patterned_negation("Not a setback, but an opportunity.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_this_isnt_about() {
        let flags = check_patterned_negation("This isn't about money. It's about principle.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_clean_text_passes_negation() {
        let flags = check_patterned_negation(
            "She preferred coffee. He liked tea. They compromised on water.",
            None,
        );
        assert!(flags.is_empty());
    }

    #[test]
    fn test_patterned_negation_check_name() {
        let flags = check_patterned_negation("It's not broken. It's character.", None);
        assert!(flags.iter().all(|f| f.check_name == "patterned_negation"));
    }

    // --- Throat-clearing openers ---

    #[test]
    fn test_detects_heres_the_thing() {
        let flags = check_throat_clearing("Here's the thing: nobody cares.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_let_me_be_clear() {
        let flags = check_throat_clearing("Let me be clear about this.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_the_truth_is() {
        let flags = check_throat_clearing("The truth is, nobody expected this.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_the_reality_is() {
        let flags = check_throat_clearing("The reality is that budgets are tight.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_but_heres_the_thing() {
        let flags = check_throat_clearing("But here's the thing about that.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_lets_be_honest() {
        let flags = check_throat_clearing("Let's be honest about this.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_clean_text_passes_throat() {
        let flags = check_throat_clearing(
            "She found the receipt. The amount was seven dollars. She put it back.",
            None,
        );
        assert!(flags.is_empty());
    }

    #[test]
    fn test_throat_clearing_check_name() {
        let flags = check_throat_clearing("Here's the thing: it matters.", None);
        assert!(flags.iter().all(|f| f.check_name == "throat_clearing"));
    }

    // --- Chatbot artifacts ---

    #[test]
    fn test_detects_great_question() {
        let flags = check_chatbot_artifacts("Great question! Let me explain.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_thats_a_great_question() {
        let flags = check_chatbot_artifacts("That's a great question to ask.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_id_be_happy_to() {
        let flags = check_chatbot_artifacts("I'd be happy to help with that.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_hope_this_helps() {
        let flags = check_chatbot_artifacts("I hope this helps with your project.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_feel_free_to() {
        let flags = check_chatbot_artifacts("Feel free to reach out.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_as_an_ai() {
        let flags = check_chatbot_artifacts("As an AI, I cannot do that.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_certainly_excl() {
        let flags = check_chatbot_artifacts("Certainly! Here is the answer.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_let_me_know() {
        let flags = check_chatbot_artifacts("Let me know if you have questions.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_clean_text_passes_chatbot() {
        let flags = check_chatbot_artifacts(
            "She found the receipt. The amount was seven dollars. She put it back.",
            None,
        );
        assert!(flags.is_empty());
    }

    #[test]
    fn test_chatbot_artifacts_check_name() {
        let flags = check_chatbot_artifacts("Great question! Thanks for asking.", None);
        assert!(flags.iter().all(|f| f.check_name == "chatbot_artifacts"));
    }

    // --- Paragraph uniformity ---

    #[test]
    fn test_uniform_paragraphs_flagged() {
        let text = "First sentence here. Second one too. Third as well.\n\n\
                     Another first here. Another second. Another third.\n\n\
                     Yet another first. Yet another second. Yet another third.\n\n\
                     Final first sentence. Final second here. Final third one.";
        let flags = check_paragraph_uniformity(text, None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_varied_paragraphs_pass() {
        let text = "Short.\n\n\
                     This paragraph is much longer and has several sentences in it. \
                     It covers more ground. It explains things in detail. And then some.\n\n\
                     Medium length here. Two sentences.\n\n\
                     Another single-sentence paragraph but this one is pretty long and has lots of words in it.";
        let flags = check_paragraph_uniformity(text, None);
        assert!(flags.is_empty());
    }

    #[test]
    fn test_fewer_than_four_paragraphs_skipped() {
        let text = "One paragraph.\n\nTwo paragraphs.\n\nThree paragraphs.";
        assert!(check_paragraph_uniformity(text, None).is_empty());
    }

    #[test]
    fn test_paragraph_uniformity_check_name() {
        let text = "A. B. C.\n\nD. E. F.\n\nG. H. I.\n\nJ. K. L.";
        let flags = check_paragraph_uniformity(text, None);
        assert!(flags.iter().all(|f| f.check_name == "paragraph_uniformity"));
    }

    // --- Emphasis crutches ---

    #[test]
    fn test_detects_full_stop() {
        let flags =
            check_emphasis_crutches("This is non-negotiable. Full stop. We must act.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_let_that_sink_in() {
        let flags = check_emphasis_crutches("We lost everything. Let that sink in.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_make_no_mistake() {
        let flags = check_emphasis_crutches("Make no mistake, this changes everything.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_clean_text_passes_emphasis() {
        let flags = check_emphasis_crutches(
            "She found the receipt. The amount was seven dollars. She put it back.",
            None,
        );
        assert!(flags.is_empty());
    }

    #[test]
    fn test_emphasis_crutches_check_name() {
        let flags = check_emphasis_crutches("Make no mistake about this.", None);
        assert!(flags.iter().all(|f| f.check_name == "emphasis_crutches"));
    }

    // --- Vague attribution ---

    #[test]
    fn test_detects_many_experts_agree() {
        let flags = check_vague_attribution("Many experts agree this is a problem.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_studies_show() {
        let flags = check_vague_attribution("Studies show that sleep is important.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_some_critics() {
        let flags = check_vague_attribution("Some critics have questioned this approach.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_detects_widely_believed() {
        let flags = check_vague_attribution("It is widely believed that this is true.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_clean_text_passes_attribution() {
        let flags = check_vague_attribution(
            "A 2023 Stanford study by Chen et al. found a 40% improvement.",
            None,
        );
        assert!(flags.is_empty());
    }

    #[test]
    fn test_vague_attribution_check_name() {
        let flags = check_vague_attribution("Many experts agree on this.", None);
        assert!(flags.iter().all(|f| f.check_name == "vague_attribution"));
    }

    // --- Wordiness ---

    #[test]
    fn test_detects_in_order_to() {
        let flags = check_wordiness("In order to succeed, you must try.", None);
        assert!(!flags.is_empty());
        assert!(flags[0].description.contains("to"));
    }

    #[test]
    fn test_detects_due_to_the_fact_that() {
        let flags = check_wordiness("Due to the fact that it rained, we stayed in.", None);
        assert!(!flags.is_empty());
        assert!(flags[0].description.contains("because"));
    }

    #[test]
    fn test_detects_at_this_point_in_time() {
        let flags = check_wordiness("At this point in time, we are ready.", None);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_clean_text_passes_wordiness() {
        let flags = check_wordiness(
            "To succeed, try harder. Because it rained, we stayed.",
            None,
        );
        assert!(flags.is_empty());
    }

    #[test]
    fn test_wordiness_check_name() {
        let flags = check_wordiness("In order to finish, we worked late.", None);
        assert!(flags.iter().all(|f| f.check_name == "wordiness"));
    }
}
