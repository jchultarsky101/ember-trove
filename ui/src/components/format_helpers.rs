//! Shared formatting helpers used by multiple UI components.
//!
//! Centralises locale-aware date/time formatting so every panel speaks
//! the same language without duplicating JS interop boilerplate.

/// Format a UTC timestamp as a concise local date-time string using JS's
/// `Intl.DateTimeFormat` (avoids a full `chrono-tz` / `time` WASM dependency).
///
/// Example output: `"Mar 25, 2026, 14:32"`
pub fn format_timestamp(ts: &chrono::DateTime<chrono::Utc>) -> String {
    let iso = ts.to_rfc3339();
    let js = format!(
        "new Intl.DateTimeFormat(undefined, {{year:'numeric',month:'short',day:'numeric',\
         hour:'2-digit',minute:'2-digit'}}).format(new Date('{iso}'))"
    );
    js_sys::eval(&js)
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| iso[..16].replace('T', " "))
}
