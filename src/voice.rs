//! Voice directive generation from sloppy configuration.
//!
//! Generates a system-level prompt directive that tells an LLM what to avoid,
//! derived from the same word lists and patterns that the detector uses for scoring.

use crate::config::Config;

/// Generate a voice authenticity directive from config.
/// The output is suitable for injection into an LLM system prompt.
pub fn generate_voice_directive(config: &Config) -> String {
    let mut sections =
        vec!["[System-level writing constraints — apply to ALL generated content]".to_string()];

    // Lexical restrictions
    if let Some(lexical) = config.checks.get("lexical_blacklist")
        && lexical.enabled
    {
        sections.push(build_lexical_section(&lexical.params));
    }

    // Punctuation and syntax
    sections.push(build_punctuation_section(config));

    // Rhythm and structure
    sections.push(build_structure_section(config));

    // Tone
    sections.push(build_tone_section(config));

    sections.join("\n\n")
}

fn build_lexical_section(params: &toml::Table) -> String {
    let mut lines = vec!["LEXICAL RESTRICTIONS:".to_string()];

    if let Some(words) = params
        .get("words")
        .and_then(|w| w.as_table())
        .and_then(|w| w.get("simple"))
        .and_then(|s| s.as_array())
    {
        let word_list: Vec<&str> = words.iter().filter_map(|v| v.as_str()).collect();
        if !word_list.is_empty() {
            lines.push(format!("Never use these words: {}.", word_list.join(", ")));
        }
    }

    if let Some(entries) = params
        .get("patterns")
        .and_then(|p| p.as_table())
        .and_then(|p| p.get("entries"))
        .and_then(|e| e.as_array())
    {
        let phrase_list: Vec<String> = entries
            .iter()
            .filter_map(|e| {
                e.as_array()
                    .and_then(|a| a.get(1))
                    .and_then(|v| v.as_str())
                    .map(|s| format!("\"{s}\""))
            })
            .collect();
        if !phrase_list.is_empty() {
            lines.push(format!(
                "Never use these phrases: {}.",
                phrase_list.join(", ")
            ));
        }
    }

    lines.push(
        "Do not use promotional superlatives or inflate the significance of mundane things."
            .to_string(),
    );

    lines.join("\n")
}

fn build_punctuation_section(config: &Config) -> String {
    let mut lines = vec!["PUNCTUATION AND SYNTAX:".to_string()];

    if let Some(em_dash) = config.checks.get("em_dash_count")
        && em_dash.enabled
    {
        let max_allowed = em_dash
            .params
            .get("max_allowed")
            .and_then(|v| v.as_integer())
            .unwrap_or(0);
        if max_allowed == 0 {
            lines.push(
                "- Do not use em-dashes (\u{2014}). Use parentheses, semicolons, or commas instead."
                    .to_string(),
            );
        } else {
            lines.push(format!(
                "- Maximum {max_allowed} em-dash(\u{2014}) per piece. \
                     Prefer parentheses or semicolons for asides."
            ));
        }
    }

    if let Some(trailing) = config.checks.get("trailing_participle")
        && trailing.enabled
    {
        lines.push(
            "- Never end a sentence with a comma followed by a present participle\n  \
                 (e.g. \", reflecting the...\" or \", underscoring the importance of...\")."
                .to_string(),
        );
    }

    if let Some(rot) = config.checks.get("rule_of_three")
        && rot.enabled
    {
        lines.push(
            "- Do not group adjectives, examples, or clauses in threes. Use two or four."
                .to_string(),
        );
    }

    if let Some(trans) = config.checks.get("transition_openers")
        && trans.enabled
        && let Some(banned) = trans.params.get("banned").and_then(|b| b.as_array())
    {
        let banned_str: Vec<&str> = banned.iter().filter_map(|v| v.as_str()).collect();
        if !banned_str.is_empty() {
            lines.push(format!(
                "- Do not start paragraphs with: {}.",
                banned_str.join(", ")
            ));
        }
    }

    if let Some(cop) = config.checks.get("copulative_inflation")
        && cop.enabled
    {
        lines.push(
            "- Use \"is\" and \"are\" instead of \"serves as\", \"stands as\", \"functions as\"."
                .to_string(),
        );
    }

    if let Some(wrd) = config.checks.get("wordiness")
        && wrd.enabled
    {
        lines.push(
                "- Cut wordy constructions: \"in order to\" -> \"to\", \"due to the fact that\" -> \"because\",\n  \
                 \"at this point in time\" -> \"now\", \"the fact that\" -> (cut)."
                    .to_string(),
            );
    }

    lines.join("\n")
}

fn build_structure_section(config: &Config) -> String {
    let mut lines = vec!["RHYTHM AND STRUCTURE:".to_string()];

    if let Some(burst) = config.checks.get("burstiness")
        && burst.enabled
    {
        lines.push(
                "- Vary sentence length sharply. Mix fragments under 6 words with compound sentences\n  \
                 over 30 words. Never write three consecutive sentences of similar length."
                    .to_string(),
            );
    }

    if let Some(pu) = config.checks.get("paragraph_uniformity")
        && pu.enabled
    {
        lines.push(
            "- Paragraphs must be asymmetrical: varying numbers of sentences, varying lengths.\n  \
                 Never write four consecutive paragraphs of equal length."
                .to_string(),
        );
    }

    if let Some(neg) = config.checks.get("patterned_negation")
        && neg.enabled
    {
        lines.push(
            "- No patterned negations (\"It's not X. It's Y.\" or \"Not merely X, but Y.\")."
                .to_string(),
        );
    }

    if let Some(ec) = config.checks.get("emphasis_crutches")
        && ec.enabled
    {
        lines.push(
                "- No emphasis crutches: \"Full stop.\", \"Let that sink in.\", \"Make no mistake\".\n  \
                 Show importance through content, not by announcing it."
                    .to_string(),
            );
    }

    lines.join("\n")
}

fn build_tone_section(config: &Config) -> String {
    let mut lines = vec!["TONE:".to_string()];
    lines
        .push("- Take definitive, committed stances. No balanced-perspective hedging.".to_string());
    lines.push("- State facts directly without inflating their importance.".to_string());

    if let Some(conc) = config.checks.get("formulaic_conclusion")
        && conc.enabled
        && let Some(openers) = conc.params.get("openers").and_then(|o| o.as_array())
    {
        let examples: Vec<String> = openers
            .iter()
            .take(4)
            .filter_map(|v| v.as_str().map(|s| format!("\"{s}\"")))
            .collect();
        if !examples.is_empty() {
            lines.push(format!(
                "- No formulaic conclusions. Never use {}, etc.",
                examples.join(", ")
            ));
        }
    }

    if let Some(tc) = config.checks.get("throat_clearing")
        && tc.enabled
    {
        lines.push(
            "- No throat-clearing openers: \"Here's the thing:\", \"Let me be clear\",\n  \
                 \"The truth is\", \"The reality is\". Start with the point."
                .to_string(),
        );
    }

    if let Some(cb) = config.checks.get("chatbot_artifacts")
        && cb.enabled
    {
        lines.push(
            "- No chatbot artifacts: \"Great question!\", \"I'd be happy to\",\n  \
                 \"Hope this helps\", \"Feel free to\", \"Certainly!\", \"Absolutely!\"."
                .to_string(),
        );
    }

    if let Some(va) = config.checks.get("vague_attribution")
        && va.enabled
    {
        lines.push(
                "- No vague attribution: \"many experts agree\", \"studies show\", \"some critics argue\".\n  \
                 Cite specific sources or make the claim directly."
                    .to_string(),
            );
    }

    lines.push(
        "- Anchor writing in specific, unusual, concrete details rather than\n  \
         generic abstractions."
            .to_string(),
    );

    lines.join("\n")
}

/// Generate a chat-ready prompt for pasting into any LLM chat window.
///
/// Two modes:
/// - `generate`: produces a prompt that instructs the LLM to write clean prose
/// - `cleanup`: produces a prompt that instructs the LLM to rewrite sloppy text
pub fn generate_chat_prompt(config: &Config, mode: &str) -> String {
    let directive = generate_voice_directive(config);

    match mode {
        "cleanup" => format!(
            "You are a prose editor specializing in removing AI-generated writing \
             patterns. I'm going to give you text that sounds like it was written by \
             an AI. Rewrite it so it reads like a human wrote it.\n\
             \n\
             Rules to follow:\n\
             \n\
             {directive}\n\
             \n\
             Additional guidance:\n\
             - Don't just swap flagged words for synonyms. Restructure sentences so \
             they don't need those words.\n\
             - Replace vague abstractions with specific, concrete details.\n\
             - Take committed stances instead of hedging.\n\
             - Vary sentence length aggressively. Mix short fragments with longer \
             compound sentences.\n\
             - End when you're done. No summary paragraph.\n\
             \n\
             Here is the text to rewrite:\n\
             \n\
             [PASTE YOUR TEXT HERE]"
        ),
        _ => format!(
            "You are a writer who produces clean, human-sounding prose. Follow \
             these constraints for everything you write in this conversation:\n\
             \n\
             {directive}\n\
             \n\
             Additional guidance:\n\
             - Anchor your writing in specific, concrete, unusual details rather \
             than generic abstractions.\n\
             - Take definitive stances. Don't hedge or both-sides things.\n\
             - Vary sentence length aggressively. Mix short fragments with longer \
             compound sentences.\n\
             - End when you're done. No summary paragraph, no \"in conclusion\".\n\
             - Write like a person who has opinions and knows things, not like a \
             helpful assistant."
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::load_config;
    use std::path::Path;

    fn default_config() -> Config {
        load_config(None, Some(Path::new("/nonexistent")))
    }

    #[test]
    fn test_generates_non_empty_directive() {
        let directive = generate_voice_directive(&default_config());
        assert!(directive.len() > 100);
    }

    #[test]
    fn test_contains_lexical_section() {
        let directive = generate_voice_directive(&default_config());
        assert!(directive.contains("LEXICAL RESTRICTIONS"));
        assert!(directive.contains("delve"));
    }

    #[test]
    fn test_contains_punctuation_section() {
        let directive = generate_voice_directive(&default_config());
        assert!(directive.contains("PUNCTUATION AND SYNTAX"));
        assert!(directive.contains("em-dash"));
    }

    #[test]
    fn test_contains_structure_section() {
        let directive = generate_voice_directive(&default_config());
        assert!(directive.contains("RHYTHM AND STRUCTURE"));
        assert!(directive.to_lowercase().contains("sentence length"));
    }

    #[test]
    fn test_contains_tone_section() {
        let directive = generate_voice_directive(&default_config());
        assert!(directive.contains("TONE"));
        assert!(directive.contains("hedging"));
    }

    #[test]
    fn test_reflects_config_changes() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".sloppy.toml"),
            "[checks.em_dash_count]\nenabled = false\n",
        )
        .unwrap();
        let config = load_config(None, Some(dir.path()));
        let directive = generate_voice_directive(&config);
        assert!(!directive.to_lowercase().contains("em-dash"));
    }

    #[test]
    fn test_custom_words_appear() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".sloppy.toml"),
            "[checks.lexical_blacklist.words]\nsimple = [\"synergy\", \"leverage\"]\n",
        )
        .unwrap();
        let config = load_config(None, Some(dir.path()));
        let directive = generate_voice_directive(&config);
        assert!(directive.contains("synergy"));
        assert!(directive.contains("leverage"));
    }
}
