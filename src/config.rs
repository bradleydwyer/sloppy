use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Embedded defaults — compiled into the binary.
const DEFAULTS_TOML: &str = include_str!("defaults.toml");

/// Meta keys that are check config, not check-specific params.
const META_KEYS: &[&str] = &["enabled", "penalty_per_flag", "max_penalty", "severity"];

/// Configuration for a single check.
#[derive(Debug, Clone)]
pub struct CheckConfig {
    pub enabled: bool,
    pub penalty_per_flag: u32,
    pub max_penalty: u32,
    pub severity: String,
    pub params: toml::Table,
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            penalty_per_flag: 10,
            max_penalty: 20,
            severity: "warning".to_string(),
            params: toml::Table::new(),
        }
    }
}

/// Fully resolved configuration.
#[derive(Debug, Clone)]
pub struct Config {
    pub threshold: u32,
    pub checks: HashMap<String, CheckConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            threshold: 30,
            checks: HashMap::new(),
        }
    }
}

/// Load configuration from defaults and optional project file.
///
/// Three layers, merged top-down:
/// 1. Built-in defaults (defaults.toml embedded in binary)
/// 2. Project config (.sloppy.toml in working directory)
/// 3. Runtime overrides (threshold, disabled checks)
pub fn load_config(path: Option<&Path>, project_dir: Option<&Path>) -> Config {
    // 1. Load built-in defaults
    let base: toml::Table = toml::from_str(DEFAULTS_TOML).expect("embedded defaults.toml is valid");

    // 2. Find override file
    let config_path: Option<PathBuf> = if let Some(p) = path {
        Some(p.to_path_buf())
    } else {
        let search_dir = project_dir
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let candidate = search_dir.join(".sloppy.toml");
        if candidate.exists() {
            Some(candidate)
        } else {
            None
        }
    };

    // 3. Merge if override exists
    let merged = if let Some(cp) = config_path {
        if cp.exists() {
            let content = std::fs::read_to_string(&cp).unwrap_or_default();
            if let Ok(override_table) = toml::from_str::<toml::Table>(&content) {
                deep_merge(base, &override_table)
            } else {
                base
            }
        } else {
            base
        }
    } else {
        base
    };

    parse_config(&merged)
}

/// Deep-merge two TOML tables. Override wins on conflicts. Lists are replaced.
fn deep_merge(mut base: toml::Table, over: &toml::Table) -> toml::Table {
    for (key, val) in over {
        if let Some(base_val) = base.get(key)
            && let (toml::Value::Table(base_t), toml::Value::Table(over_t)) = (base_val, val)
        {
            let merged = deep_merge(base_t.clone(), over_t);
            base.insert(key.clone(), toml::Value::Table(merged));
            continue;
        }
        base.insert(key.clone(), val.clone());
    }
    base
}

/// Parse a raw TOML table into a Config.
fn parse_config(raw: &toml::Table) -> Config {
    let threshold = raw
        .get("general")
        .and_then(|g| g.as_table())
        .and_then(|g| g.get("threshold"))
        .and_then(|t| t.as_integer())
        .unwrap_or(30) as u32;

    let mut checks = HashMap::new();

    if let Some(checks_table) = raw.get("checks").and_then(|c| c.as_table()) {
        for (name, check_raw) in checks_table {
            if let Some(ct) = check_raw.as_table() {
                let enabled = ct.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
                let penalty_per_flag = ct
                    .get("penalty_per_flag")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(10) as u32;
                let max_penalty = ct
                    .get("max_penalty")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(20) as u32;
                let severity = ct
                    .get("severity")
                    .and_then(|v| v.as_str())
                    .unwrap_or("warning")
                    .to_string();

                // Everything that isn't a meta key goes into params
                let mut params = toml::Table::new();
                for (k, v) in ct {
                    if !META_KEYS.contains(&k.as_str()) {
                        params.insert(k.clone(), v.clone());
                    }
                }

                checks.insert(
                    name.clone(),
                    CheckConfig {
                        enabled,
                        penalty_per_flag,
                        max_penalty,
                        severity,
                        params,
                    },
                );
            }
        }
    }

    Config { threshold, checks }
}

/// Dump a Config as human-readable TOML-ish text.
pub fn dump_config(config: &Config) -> String {
    let mut lines = vec![
        "[general]".to_string(),
        format!("threshold = {}", config.threshold),
        String::new(),
    ];

    let mut check_names: Vec<&String> = config.checks.keys().collect();
    check_names.sort();

    for name in check_names {
        let cc = &config.checks[name];
        lines.push(format!("[checks.{name}]"));
        lines.push(format!("enabled = {}", cc.enabled));
        lines.push(format!("penalty_per_flag = {}", cc.penalty_per_flag));
        lines.push(format!("max_penalty = {}", cc.max_penalty));
        lines.push(format!("severity = \"{}\"", cc.severity));
        if !cc.params.is_empty() {
            for (k, v) in &cc.params {
                lines.push(format!("{k} = {v}"));
            }
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loads_defaults() {
        let config = load_config(None, Some(Path::new("/nonexistent")));
        assert_eq!(config.threshold, 30);
        assert!(config.checks.contains_key("lexical_blacklist"));
        assert!(config.checks.contains_key("burstiness"));
        assert_eq!(config.checks.len(), 15);
    }

    #[test]
    fn test_all_checks_enabled_by_default() {
        let config = load_config(None, Some(Path::new("/nonexistent")));
        for (name, cc) in &config.checks {
            assert!(cc.enabled, "Check {name} should be enabled by default");
        }
    }

    #[test]
    fn test_deep_merge_override_scalar() {
        let base: toml::Table = toml::from_str("[general]\nthreshold = 30").unwrap();
        let over: toml::Table = toml::from_str("[general]\nthreshold = 20").unwrap();
        let result = deep_merge(base, &over);
        assert_eq!(
            result["general"].as_table().unwrap()["threshold"]
                .as_integer()
                .unwrap(),
            20
        );
    }

    #[test]
    fn test_deep_merge_preserves_unmentioned_keys() {
        let base: toml::Table =
            toml::from_str("[general]\nthreshold = 30\n\n[other]\nval = \"x\"").unwrap();
        let over: toml::Table = toml::from_str("[general]\nthreshold = 20").unwrap();
        let result = deep_merge(base, &over);
        assert!(result.contains_key("other"));
    }

    #[test]
    fn test_deep_merge_list_replaced() {
        let base: toml::Table = toml::from_str("words = [\"a\", \"b\", \"c\"]").unwrap();
        let over: toml::Table = toml::from_str("words = [\"x\", \"y\"]").unwrap();
        let result = deep_merge(base, &over);
        let words = result["words"].as_array().unwrap();
        assert_eq!(words.len(), 2);
    }

    #[test]
    fn test_parse_config_threshold() {
        let raw: toml::Table = toml::from_str("[general]\nthreshold = 20\n\n[checks]").unwrap();
        let config = parse_config(&raw);
        assert_eq!(config.threshold, 20);
    }

    #[test]
    fn test_parse_check_config() {
        let raw: toml::Table = toml::from_str(
            r#"
[general]
threshold = 30

[checks.lexical_blacklist]
enabled = true
penalty_per_flag = 8
max_penalty = 40
severity = "warning"

[checks.lexical_blacklist.words]
simple = ["delve"]
"#,
        )
        .unwrap();
        let config = parse_config(&raw);
        let cc = &config.checks["lexical_blacklist"];
        assert!(cc.enabled);
        assert_eq!(cc.penalty_per_flag, 8);
        assert_eq!(cc.max_penalty, 40);
        assert!(cc.params.contains_key("words"));
    }

    #[test]
    fn test_defaults_for_missing_fields() {
        let raw: toml::Table = toml::from_str("[checks.custom_check]\nenabled = true").unwrap();
        let config = parse_config(&raw);
        let cc = &config.checks["custom_check"];
        assert_eq!(cc.penalty_per_flag, 10);
        assert_eq!(cc.max_penalty, 20);
    }

    #[test]
    fn test_load_with_override_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".sloppy.toml"),
            "[general]\nthreshold = 15\n",
        )
        .unwrap();
        let config = load_config(None, Some(dir.path()));
        assert_eq!(config.threshold, 15);
        assert!(config.checks.contains_key("lexical_blacklist"));
    }

    #[test]
    fn test_explicit_path() {
        let dir = tempfile::tempdir().unwrap();
        let custom = dir.path().join("custom.toml");
        std::fs::write(&custom, "[general]\nthreshold = 50\n").unwrap();
        let config = load_config(Some(&custom), None);
        assert_eq!(config.threshold, 50);
    }
}
