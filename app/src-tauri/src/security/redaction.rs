use regex::Regex;
use std::sync::OnceLock;

static REDACT_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();

pub struct Redactor;

impl Redactor {
    fn patterns() -> &'static Vec<Regex> {
        REDACT_PATTERNS.get_or_init(|| {
            vec![
                // Bearer tokens and Authorization headers
                Regex::new(r#"(?i)(authorization:\s*bearer\s+)[a-zA-Z0-9_\-\.]{8,}"#).unwrap(),
                Regex::new(r#"(?i)(bearer\s+)[a-zA-Z0-9_\-\.]{12,}"#).unwrap(),
                // API keys with common prefixes
                Regex::new(r#"(?i)(sk-[a-zA-Z0-9_\-]{16,})"#).unwrap(),
                Regex::new(r#"(?i)(deepseek-[a-zA-Z0-9_\-]{16,})"#).unwrap(),
                Regex::new(r#"(?i)(api[_-]?key[\s:=]+['"]?)[a-zA-Z0-9_\-\.]{8,}"#).unwrap(),
                // Password and Secret fields
                Regex::new(r#"(?i)(password[\s:=]+['"]?)[^\s'";]{4,}"#).unwrap(),
                Regex::new(r#"(?i)(secret[\s:=]+['"]?)[a-zA-Z0-9_\-\.]{8,}"#).unwrap(),
                // Generic tokens
                Regex::new(r#"(?i)(token[\s:=]+['"]?)[a-zA-Z0-9_\-\.]{12,}"#).unwrap(),
            ]
        })
    }

    /// Redacts sensitive keys, tokens, and authorization headers from a string.
    pub fn sanitize(input: &str) -> String {
        let mut result = input.to_string();
        for pattern in Self::patterns() {
            result = pattern.replace_all(&result, "$1••••••••••••••••").to_string();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redaction() {
        let text = "Authorization: Bearer sk-ant-api03-abcdef1234567890xyz and api_key=secretKey12345";
        let sanitized = Redactor::sanitize(text);
        assert!(!sanitized.contains("sk-ant-api03-abcdef1234567890xyz"));
        assert!(!sanitized.contains("secretKey12345"));
        assert!(sanitized.contains("••••••••"));
    }
}
