use serde::Serialize;

/// A single detected AI prose tell.
#[derive(Debug, Clone, Serialize)]
pub struct SlopFlag {
    pub check_name: String,
    pub description: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub location: String,
    pub severity: String,
}

impl SlopFlag {
    pub fn new(check_name: &str, description: &str, location: &str, severity: &str) -> Self {
        Self {
            check_name: check_name.to_string(),
            description: description.to_string(),
            location: location.to_string(),
            severity: severity.to_string(),
        }
    }

    pub fn warning(check_name: &str, description: &str, location: &str) -> Self {
        Self::new(check_name, description, location, "warning")
    }

    pub fn info(check_name: &str, description: &str, location: &str) -> Self {
        Self::new(check_name, description, location, "info")
    }
}

/// Aggregated result from running all slop checks on a text.
#[derive(Debug, Serialize)]
pub struct SlopResult {
    /// 0–100 where 0 is pristine and 100 is maximum slop.
    pub score: u32,
    /// Every individual match from every check.
    pub flags: Vec<SlopFlag>,
    /// True when score < the configured threshold.
    pub passed: bool,
}
