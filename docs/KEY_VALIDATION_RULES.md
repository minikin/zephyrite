# Key Validation Rules

Zephyrite enforces validation rules on keys to ensure data integrity, security, and compatibility across different systems and protocols.

- [Key Validation Rules](#key-validation-rules)
  - [Valid Key Patterns](#valid-key-patterns)
    - [Basic Requirements](#basic-requirements)
    - [Allowed Characters](#allowed-characters)
    - [Common Patterns](#common-patterns)
  - [❌ Invalid Key Patterns](#-invalid-key-patterns)
    - [1. Empty or Size Issues](#1-empty-or-size-issues)
    - [2. Whitespace Issues](#2-whitespace-issues)
    - [3. Control Characters](#3-control-characters)
    - [4. Reserved Patterns](#4-reserved-patterns)
    - [5. Security Risks](#5-security-risks)
  - [🔧 Implementation Details](#-implementation-details)
    - [Standard Validation](#standard-validation)
    - [Strict Validation](#strict-validation)
    - [Performance](#performance)
  - [🛡️ Security Considerations](#️-security-considerations)
    - [Why These Rules Matter](#why-these-rules-matter)
    - [Attack Prevention](#attack-prevention)
  - [📋 Validation Error Messages](#-validation-error-messages)
  - [🚀 Best Practices](#-best-practices)
    - [Recommended Key Patterns](#recommended-key-patterns)
    - [What to Avoid](#what-to-avoid)
    - [Validation in Your App](#validation-in-your-app)
  - [🔮 Future Enhancements](#-future-enhancements)
  - [📚 Related Documentation](#-related-documentation)

## Valid Key Patterns

### Basic Requirements

- **Length**: 1-1024 bytes
- **Encoding**: Valid UTF-8 strings
- **Character set**: Printable characters (no control characters)

### Allowed Characters

```txt
✅ Letters: a-z, A-Z
✅ Numbers: 0-9
✅ Common symbols: - _ . : / @
✅ Unicode: 🚀, 中文, العربية, Ελληνικά
```

### Common Patterns

```bash
✅ user:123                    # Namespaced IDs
✅ session_abc123              # Underscore separation
✅ config.database.host        # Dot notation
✅ cache/user/profile/123      # Path-like structure
✅ metrics:cpu.usage.percent   # Mixed separators
✅ api_key_v2_production       # Descriptive names
✅ 🚀emoji_keys                # Unicode support
```

## ❌ Invalid Key Patterns

### 1. Empty or Size Issues

```bash
❌ ""                          # Empty key
❌ "a".repeat(1025)           # Too long (>1024 bytes)
```

**Reason**: Empty keys cause lookup issues, oversized keys waste memory and may break protocols.

### 2. Whitespace Issues

```bash
❌ " leading_space"           # Leading space
❌ "trailing_space "          # Trailing space
```

**Reason**: Often accidental, causes confusion in logs, breaks some protocols.

**Note**: Tabs are rejected as control characters (see section 3).

### 3. Control Characters

```bash
❌ "key\0with\0null"          # Null bytes (\0)
❌ "key\nwith\nnewline"       # Line breaks (\n, \r)
❌ "key\x01control"           # ASCII control chars (0-31, 127)
```

**Reason**:

- **Null bytes**: Break C-style string handling, cause truncation
- **Line breaks**: Break line-based protocols, corrupt logs
- **Control chars**: Display issues, protocol problems (includes DEL character)

### 4. Reserved Patterns

```bash
❌ "__zephyrite_internal"     # Reserved prefix
❌ "__zephyrite_config"       # System metadata
❌ "__zephyrite_*"            # Any __zephyrite_ prefix
```

**Reason**: Reserved for internal Zephyrite system operations and metadata.

### 5. Security Risks

```bash
❌ "path/../../etc/passwd"    # Path traversal
❌ "../config"                # Relative paths
❌ "data/../secrets"          # Directory traversal
❌ "any..where"               # Double dots anywhere
```

**Reason**: Prevent path traversal attacks, even if not directly used for file paths.

## 🔧 Implementation Details

### Standard Validation

The default `validate_key()` function applies all the rules above:

```rust
// This will validate according to all rules
storage.put("my_key", "value")?;
```

### Strict Validation

For high-security environments, additional restrictions:

```rust
// Configurable validation with additional checks
validate_key_strict(key, allow_slashes: false, allow_dots: false)?;
```

**Additional strict checks:**

- **Path separators**: Blocks `/` and `\` when `allow_slashes: false`
- **Dots**: Blocks `.` when `allow_dots: false`
- **Consecutive characters**: Blocks `::`, `--`, `__` patterns

### Performance

- Validation runs in **O(n)** where n = key length
- Optimized for common cases
- Minimal overhead (~microseconds for typical keys)

## 🛡️ Security Considerations

### Why These Rules Matter

1. **Data Integrity**: Prevent keys that could cause data corruption
2. **Protocol Safety**: Ensure compatibility with HTTP, network protocols
3. **Logging Safety**: Prevent log injection and parsing issues
4. **Future Compatibility**: Work with persistence layers, clustering
5. **User Experience**: Catch common mistakes early

### Attack Prevention

- **Path Traversal**: `..` sequences blocked
- **Control Character Injection**: All control chars blocked
- **Null Byte Injection**: Explicit null byte detection
- **Reserved Namespace**: System internals protected

## 📋 Validation Error Messages

Clear, actionable error messages help debugging:

```rust
StorageError::InvalidKey("Key cannot be empty")
StorageError::InvalidKey("Key too long (max 1024 bytes)")
StorageError::InvalidKey("Key cannot contain null bytes")
StorageError::InvalidKey("Key cannot start or end with spaces")
StorageError::InvalidKey("Key cannot contain line breaks")
StorageError::InvalidKey("Key cannot contain control characters")
StorageError::InvalidKey("Keys cannot start with '__zephyrite_' (reserved prefix)")
StorageError::InvalidKey("Key cannot contain '..' (security risk)")

// Strict validation additional errors:
StorageError::InvalidKey("Key cannot contain path separators")
StorageError::InvalidKey("Key cannot contain dots")
StorageError::InvalidKey("Key cannot contain consecutive special characters")
```

## 🚀 Best Practices

### Recommended Key Patterns

1. **Namespacing**: `user:123`, `session:abc`, `config:prod`
2. **Hierarchical**: `cache/user/profile/123`
3. **Dot notation**: `metrics.cpu.usage`
4. **Descriptive**: `api_key_production_v2`
5. **Timestamped**: `backup_20240101_123456`

### What to Avoid

1. **Leading/trailing whitespace**: Easy to miss, causes bugs
2. **Control characters**: Often invisible, cause weird behavior
3. **Very long keys**: Waste memory, may hit limits
4. **Reserved prefixes**: Will conflict with system operations
5. **Path traversal patterns**: Security risk, blocked anyway

### Validation in Your App

```rust
// Good: Handle validation errors gracefully
match storage.put(&user_provided_key, &value) {
    Ok(_) => println!("Stored successfully"),
    Err(StorageError::InvalidKey(msg)) => {
        eprintln!("Invalid key: {}", msg);
        // Show user-friendly error message
    }
    Err(e) => eprintln!("Storage error: {}", e),
}

// Good: Validate early if building keys programmatically
let key = format!("user:{}", user_id);
// user_id from trusted source, format is safe

// Better: Use helper functions for common patterns
fn make_user_key(user_id: u32) -> String {
    format!("user:{}", user_id)  // Known safe pattern
}
```

## 🔮 Future Enhancements

Planned improvements:

1. **Key normalization**: Automatic cleanup of common issues
2. **Custom validators**: User-defined validation functions
3. **Performance optimizations**: Faster validation for hot paths
4. **Validation profiles**: Multiple predefined rule sets
5. **Per-storage validation**: Different rules per storage instance

## 📚 Related Documentation

- [Storage API Documentation](../examples/storage_basic.rs)
- [HTTP API Guide](../README.md#api-endpoints)
- [Security Best Practices](./SECURITY.md) _(coming soon)_
