use super::error::{StorageError, StorageResult};

pub fn validate_key(key: &str) -> StorageResult<()> {
    /// Helper functions for key validation
    pub fn validate_key(key: &str) -> StorageResult<()> {
        // 1. Basic checks
        if key.is_empty() {
            return Err(StorageError::InvalidKey("Key cannot be empty".to_string()));
        }

        if key.len() > 1024 {
            return Err(StorageError::InvalidKey(
                "Key too long (max 1024 bytes)".to_string(),
            ));
        }

        // 2. Whitespace checks
        if key.starts_with(' ') || key.ends_with(' ') {
            return Err(StorageError::InvalidKey(
                "Key cannot start or end with spaces".to_string(),
            ));
        }

        if key.starts_with('\t') || key.ends_with('\t') {
            return Err(StorageError::InvalidKey(
                "Key cannot start or end with tabs".to_string(),
            ));
        }

        // 3. Dangerous characters
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

        // 4. Control characters (ASCII 0-31, 127)
        if key.chars().any(|c| c.is_ascii_control()) {
            return Err(StorageError::InvalidKey(
                "Key cannot contain control characters".to_string(),
            ));
        }

        // 5. Reserved patterns for internal use
        if key.starts_with("__zephyrite_") {
            return Err(StorageError::InvalidKey(
                "Keys cannot start with '__zephyrite_' (reserved prefix)".to_string(),
            ));
        }

        // 6. Path traversal prevention
        if key.contains("..") {
            return Err(StorageError::InvalidKey(
                "Key cannot contain '..' (security risk)".to_string(),
            ));
        }

        // 7. Some systems have issues with these
        if key.contains('\x7F') {
            return Err(StorageError::InvalidKey(
                "Key cannot contain DEL character".to_string(),
            ));
        }

        Ok(())
    }
}

/// Validate key with configurable strictness
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
