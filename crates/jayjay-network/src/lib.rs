//! Shared blocking HTTP helpers for JayJay.
//!
//! One configured `ureq` agent so every caller shares the same timeout and
//! redirect policy. Calls block; run them on a background executor.

use std::io::Read;
use std::sync::LazyLock;
use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// Body cap for text responses (JSON APIs): 2 MiB.
const DEFAULT_TEXT_CAP: u64 = 2 * 1024 * 1024;

static AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| {
    ureq::Agent::config_builder()
        .timeout_global(Some(REQUEST_TIMEOUT))
        .build()
        .into()
});

/// GET `url` as text. None on transport error, non-200, oversize body, or invalid UTF-8.
pub fn get_text(url: &str) -> Option<String> {
    let bytes = get_bytes(url, DEFAULT_TEXT_CAP)?;
    String::from_utf8(bytes).ok()
}

/// GET `url`, returning up to `max_bytes` of the body. None on transport error or non-200.
pub fn get_bytes(url: &str, max_bytes: u64) -> Option<Vec<u8>> {
    let mut response = AGENT.get(url).call().ok()?;
    if response.status().as_u16() != 200 {
        return None;
    }
    let mut bytes = Vec::new();
    response
        .body_mut()
        .as_reader()
        .take(max_bytes)
        .read_to_end(&mut bytes)
        .ok()?;
    Some(bytes)
}
