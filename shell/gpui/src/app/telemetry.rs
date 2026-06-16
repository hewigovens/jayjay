//! Anonymous, opt-in daily ping: app version + OS + arch only. No personal
//! data, no IP stored server-side (see infra/worker). Fire-and-forget
//! on a background thread; never blocks startup, never surfaces errors.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const ENDPOINT: &str = "https://jayjay.hewigovens.workers.dev/ping";
const INTERVAL_SECS: u64 = 24 * 60 * 60;

/// Send one ping if enabled and the daily interval has elapsed.
pub fn maybe_ping(enabled: bool) {
    if !enabled {
        return;
    }
    let Some(stamp) = stamp_path() else { return };
    let now = unix_now();
    if !due(last_ping(&stamp), now) {
        return;
    }
    write_stamp(&stamp, now);
    std::thread::spawn(move || {
        let url = format!(
            "{ENDPOINT}?platform=gpui&app=jayjay&version={version}&os={os}&arch={arch}",
            version = env!("CARGO_PKG_VERSION"),
            os = std::env::consts::OS,
            arch = std::env::consts::ARCH,
        );
        let _ = jayjay_network::get_text(&url);
    });
}

/// True when no ping has been sent or the interval has elapsed.
fn due(last: Option<u64>, now: u64) -> bool {
    match last {
        None => true,
        Some(t) => now.saturating_sub(t) >= INTERVAL_SECS,
    }
}

fn stamp_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("dev", "hewig", "jayjay")
        .map(|d| d.cache_dir().join("last_ping"))
}

fn last_ping(path: &PathBuf) -> Option<u64> {
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn write_stamp(path: &PathBuf, now: u64) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, now.to_string());
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{INTERVAL_SECS, due};

    #[test]
    fn first_ping_is_due() {
        assert!(due(None, 1000));
    }

    #[test]
    fn within_interval_is_not_due() {
        assert!(!due(Some(1000), 1000 + INTERVAL_SECS - 1));
    }

    #[test]
    fn after_interval_is_due() {
        assert!(due(Some(1000), 1000 + INTERVAL_SECS));
    }

    #[test]
    fn clock_skew_backwards_is_not_due() {
        assert!(!due(Some(5000), 1000));
    }
}
