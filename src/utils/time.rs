use chrono::Utc;

/// Returns the current timestamp in ISO 8601 format (YYYY-MM-DDTHH:mm:ss.sssZ)
#[must_use]
pub fn current_timestamp() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}
