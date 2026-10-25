#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

/// Milliseconds since the Unix epoch; 0 if the system clock reads earlier than the epoch.
#[cfg(not(target_arch = "wasm32"))]
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
pub fn now_ms() -> i64 {
    js_sys::Date::now().max(0.0) as i64
}
