use anyhow::{Context, Result};
use regex::Regex;
use serde_json::Value;
use std::path::PathBuf;

pub mod inspection;
pub mod template;

/// # Errors
///
/// This function will return an error if the string cannot be parsed as JSON.
pub fn parse_json(s: &str) -> Result<Value> {
    serde_json::from_str(s).with_context(|| format!("Failed to parse JSON: {s}"))
}

/// # Errors
///
/// This function will return an error if the path does not exist.
pub fn validate_path_exists(s: &str) -> Result<PathBuf> {
    let path = PathBuf::from(s);
    if !path.exists() {
        anyhow::bail!("Path does not exist: {s}");
    }
    Ok(path)
}

/// # Errors
///
/// This function will return an error if the string is not a valid name.
pub fn validate_name(s: &str) -> Result<String> {
    let re = Regex::new(r"^[a-zA-Z0-9_-]+$")?;
    if !re.is_match(s) {
        anyhow::bail!("Name can only contain alphanumeric characters, underscores, and dashes.")
    }
    Ok(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json() {
        let json_str = r#"{"key": "value"}"#;
        let value = parse_json(json_str).unwrap();
        assert_eq!(value["key"], "value");
    }

    #[test]
    fn test_parse_json_invalid() {
        let invalid_json_str = r#"{"key": "value""#; // Missing closing brace
        let result = parse_json(invalid_json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to parse JSON")
        );
    }

    #[test]
    fn test_validate_path_exists() {
        let path = validate_path_exists(".").unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_validate_path_exists_invalid() {
        let result = validate_path_exists("non_existent_path");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Path does not exist: non_existent_path"
        );
    }

    #[test]
    fn test_validate_name() {
        let name = validate_name("valid_name-123").unwrap();
        assert_eq!(name, "valid_name-123");
    }

    #[test]
    fn test_validate_name_invalid() {
        let result = validate_name("name with spaces");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Name can only contain alphanumeric characters, underscores, and dashes."
        );
    }
}
