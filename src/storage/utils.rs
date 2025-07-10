use super::error::{StorageError, StorageResult};

/// Helper functions for key validation
///
/// # Errors
/// Returns `StorageError::InvalidKey` if the key is invalid (empty, too long, contains forbidden characters, etc.)
pub fn validate_key(key: &str) -> StorageResult<()> {
    // Basic checks
    if key.is_empty() {
        return Err(StorageError::InvalidKey("Key cannot be empty".to_string()));
    }

    if key.len() > 1024 {
        return Err(StorageError::InvalidKey(
            "Key too long (max 1024 bytes)".to_string(),
        ));
    }

    // Whitespace checks - only spaces (tabs are caught by control character check)
    if key.starts_with(' ') || key.ends_with(' ') {
        return Err(StorageError::InvalidKey(
            "Key cannot start or end with spaces".to_string(),
        ));
    }

    // Dangerous characters
    if key.contains('\0') {
        return Err(StorageError::InvalidKey(
            "Key cannot contain null bytes".to_string(),
        ));
    }

    if key.contains('\n') || key.contains('\r') {
        return Err(StorageError::InvalidKey(
            "Key cannot contain line breaks".to_string(),
        ));
    }

    // Control characters (ASCII 0-31, 127)
    if key.chars().any(|c| c.is_ascii_control()) {
        return Err(StorageError::InvalidKey(
            "Key cannot contain control characters".to_string(),
        ));
    }

    // Reserved patterns for internal use
    if key.starts_with("__zephyrite_") {
        return Err(StorageError::InvalidKey(
            "Keys cannot start with '__zephyrite_' (reserved prefix)".to_string(),
        ));
    }

    // Path traversal prevention
    if key.contains("..") {
        return Err(StorageError::InvalidKey(
            "Key cannot contain '..' (security risk)".to_string(),
        ));
    }

    Ok(())
}

/// Validate key with configurable strictness
///
/// # Errors
/// Returns `StorageError::InvalidKey` if the key fails standard validation or additional strict checks
pub fn validate_key_strict(key: &str, allow_slashes: bool, allow_dots: bool) -> StorageResult<()> {
    // First run standard validation
    validate_key(key)?;

    // Additional strict checks
    if !allow_slashes && (key.contains('/') || key.contains('\\')) {
        return Err(StorageError::InvalidKey(
            "Key cannot contain path separators".to_string(),
        ));
    }

    if !allow_dots && key.contains('.') {
        return Err(StorageError::InvalidKey(
            "Key cannot contain dots".to_string(),
        ));
    }

    // Check for excessive consecutive special characters
    if key.contains("::") || key.contains("--") || key.contains("__") {
        return Err(StorageError::InvalidKey(
            "Key cannot contain consecutive special characters".to_string(),
        ));
    }

    Ok(())
}

/// Helper functions for value validation
///
/// # Errors
/// Returns `StorageError::InvalidValue` if the value is too large (exceeds 1MB limit)
pub fn validate_value(value: &str) -> StorageResult<()> {
    // For now, we'll allow any UTF-8 string as a value
    // In the future, we might add size limits or other constraints

    if value.len() > 1_048_576 {
        return Err(StorageError::InvalidValue(
            "Value too large (max 1MB)".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_keys() {
        // Basic valid keys
        assert!(validate_key("user:123").is_ok());
        assert!(validate_key("session_abc123").is_ok());
        assert!(validate_key("config.database.host").is_ok());
        assert!(validate_key("cache/user/profile/123").is_ok());
        assert!(validate_key("metrics:cpu.usage.percent").is_ok());
        assert!(validate_key("api_key_v2_production").is_ok());

        // Unicode support
        assert!(validate_key("🚀emoji_keys").is_ok());
        assert!(validate_key("中文key").is_ok());
        assert!(validate_key("العربية").is_ok());
        assert!(validate_key("Ελληνικά").is_ok());

        // Edge cases
        assert!(validate_key("a").is_ok()); // Single character
        assert!(validate_key(&"a".repeat(1024)).is_ok()); // Max length
    }

    #[test]
    fn test_empty_key() {
        let result = validate_key("");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid key: Key cannot be empty"
        );
    }

    #[test]
    fn test_key_too_long() {
        let long_key = "a".repeat(1025);
        let result = validate_key(&long_key);
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid key: Key too long (max 1024 bytes)"
        );
    }

    #[test]
    fn test_leading_trailing_spaces() {
        let result = validate_key(" leading_space");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid key: Key cannot start or end with spaces"
        );

        let result = validate_key("trailing_space ");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid key: Key cannot start or end with spaces"
        );

        // Spaces in the middle are OK
        assert!(validate_key("middle space ok").is_ok());
    }

    #[test]
    fn test_tabs_as_control_characters() {
        // Tabs are rejected as control characters regardless of position
        let result = validate_key("\tleading_tab");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid key: Key cannot contain control characters"
        );

        let result = validate_key("trailing_tab\t");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid key: Key cannot contain control characters"
        );

        let result = validate_key("middle\ttab\tok");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid key: Key cannot contain control characters"
        );
    }

    #[test]
    fn test_null_bytes() {
        let result = validate_key("key\0with\0null");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid key: Key cannot contain null bytes"
        );
    }

    #[test]
    fn test_line_breaks() {
        let result = validate_key("key\nwith\nnewline");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid key: Key cannot contain line breaks"
        );

        let result = validate_key("key\rwith\rcarriage");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid key: Key cannot contain line breaks"
        );
    }

    #[test]
    fn test_control_characters() {
        // Test various control characters
        let result = validate_key("key\x01control");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid key: Key cannot contain control characters"
        );

        // Test DEL character (127)
        let result = validate_key("key\x7Fdel");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid key: Key cannot contain control characters"
        );

        // Test other control characters
        for i in 1..32u8 {
            if i != b'\t' && i != b'\n' && i != b'\r' && i != 0 {
                let key = format!("key{}control", char::from(i));
                let result = validate_key(&key);
                assert!(matches!(result, Err(StorageError::InvalidKey(_))));
            }
        }
    }

    #[test]
    fn test_reserved_prefix() {
        let result = validate_key("__zephyrite_internal");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid key: Keys cannot start with '__zephyrite_' (reserved prefix)"
        );

        let result = validate_key("__zephyrite_config");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));

        // Similar but not exact prefix should be OK
        assert!(validate_key("_zephyrite_internal").is_ok());
        assert!(validate_key("__zephyrit_internal").is_ok());
    }

    #[test]
    fn test_path_traversal() {
        let result = validate_key("path/../../etc/passwd");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid key: Key cannot contain '..' (security risk)"
        );

        let result = validate_key("../config");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));

        let result = validate_key("data/../secrets");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));

        let result = validate_key("any..where");
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));

        // Single dots should be OK
        assert!(validate_key("config.file.name").is_ok());
        assert!(validate_key("version.1.2.3").is_ok());
    }

    #[test]
    fn test_strict_validation_path_separators() {
        // Should pass standard validation
        assert!(validate_key("path/to/file").is_ok());
        assert!(validate_key("path\\to\\file").is_ok());

        // Should fail strict validation when slashes not allowed
        let result = validate_key_strict("path/to/file", false, true);
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid key: Key cannot contain path separators"
        );

        let result = validate_key_strict("path\\to\\file", false, true);
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));

        // Should pass when slashes allowed
        assert!(validate_key_strict("path/to/file", true, true).is_ok());
        assert!(validate_key_strict("path\\to\\file", true, true).is_ok());
    }

    #[test]
    fn test_strict_validation_dots() {
        // Should pass standard validation
        assert!(validate_key("config.file.name").is_ok());

        // Should fail strict validation when dots not allowed
        let result = validate_key_strict("config.file.name", true, false);
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid key: Key cannot contain dots"
        );

        // Should pass when dots allowed
        assert!(validate_key_strict("config.file.name", true, true).is_ok());
    }

    #[test]
    fn test_strict_validation_consecutive_characters() {
        // These should fail strict validation
        let result = validate_key_strict("key::double", true, true);
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid key: Key cannot contain consecutive special characters"
        );

        let result = validate_key_strict("key--double", true, true);
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));

        let result = validate_key_strict("key__double", true, true);
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));

        // Single characters should be OK
        assert!(validate_key_strict("key:single", true, true).is_ok());
        assert!(validate_key_strict("key-single", true, true).is_ok());
        assert!(validate_key_strict("key_single", true, true).is_ok());
    }

    #[test]
    fn test_strict_validation_inherits_standard() {
        // Strict validation should fail on standard validation errors too
        let result = validate_key_strict("", true, true);
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));

        let result = validate_key_strict("key\0null", true, true);
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));

        let result = validate_key_strict("__zephyrite_internal", true, true);
        assert!(matches!(result, Err(StorageError::InvalidKey(_))));
    }

    #[test]
    fn test_value_validation_size_limit() {
        // Valid value
        assert!(validate_value("normal value").is_ok());

        // Large but within limit
        let large_value = "a".repeat(1_048_576); // Exactly 1MB
        assert!(validate_value(&large_value).is_ok());

        // Too large
        let too_large_value = "a".repeat(1_048_577); // 1MB + 1 byte
        let result = validate_value(&too_large_value);
        assert!(matches!(result, Err(StorageError::InvalidValue(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid value: Value too large (max 1MB)"
        );
    }

    #[test]
    fn test_value_validation_allows_any_utf8() {
        // Special characters should be OK in values
        assert!(validate_value("value\nwith\nnewlines").is_ok());
        assert!(validate_value("value\0with\0nulls").is_ok());
        assert!(validate_value("value\twith\ttabs").is_ok());
        assert!(validate_value("value with spaces").is_ok());
        assert!(validate_value("value..with..dots").is_ok());
        assert!(validate_value("__zephyrite_internal_value").is_ok());
        assert!(validate_value("🚀emoji values 中文").is_ok());
    }
}
