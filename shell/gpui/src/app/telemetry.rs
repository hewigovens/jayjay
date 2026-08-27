//! Anonymous activity ping. A random installation secret stays on the
//! device and derives unlinkable UTC-day and UTC-month identifiers, allowing
//! DAU and MAU counts without sending a permanent installation identifier.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use sha2::{Digest, Sha256};
use sysinfo::System;
use uuid::Uuid;

const ENDPOINT: &str = "https://jayjay.hewigovens.workers.dev/ping";

/// Send at most one successful ping per UTC day when anonymous stats are enabled.
pub fn maybe_ping(enabled: bool) {
    if !enabled || !release_telemetry_enabled() {
        return;
    }
    let version = env!("CARGO_PKG_VERSION");
    if !is_release_version(version) {
        return;
    }

    let periods = Periods::at(Utc::now());
    let Some(stamp) = stamp_path() else { return };
    if last_sent_day(&stamp).as_deref() == Some(periods.day.as_str()) {
        return;
    }
    let Some(secret) = load_or_create_secret() else {
        return;
    };

    let daily_id = period_id(&secret, "day", &periods.day);
    let monthly_id = period_id(&secret, "month", &periods.month);
    let os_version = System::os_version().unwrap_or_default();
    let url = format!(
        "{ENDPOINT}?platform=gpui&app=jayjay&version={version}&os={os}&osver={os_version}&arch={arch}&daily_id={daily_id}&monthly_id={monthly_id}",
        version = query_value(version),
        os = query_value(std::env::consts::OS),
        os_version = query_value(&os_version),
        arch = query_value(std::env::consts::ARCH),
    );
    let day = periods.day;
    std::thread::spawn(move || {
        if jayjay_network::HttpClient::default().get_text(&url).is_ok() {
            let _ = write_stamp(&stamp, &day);
        }
    });
}

fn release_telemetry_enabled() -> bool {
    !cfg!(debug_assertions)
}

fn is_release_version(version: &str) -> bool {
    let mut parts = version.trim().split('.');
    let Some(major) = parts.next() else {
        return false;
    };
    let Some(minor) = parts.next() else {
        return false;
    };
    let Some(patch) = parts.next() else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }
    [major, minor, patch]
        .into_iter()
        .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

struct Periods {
    day: String,
    month: String,
}

impl Periods {
    fn at(now: DateTime<Utc>) -> Self {
        Self {
            day: now.format("%Y-%m-%d").to_string(),
            month: now.format("%Y-%m").to_string(),
        }
    }
}

fn period_id(secret: &str, scope: &str, period: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update([0]);
    hasher.update(scope.as_bytes());
    hasher.update([0]);
    hasher.update(period.as_bytes());
    hex::encode(hasher.finalize())
}

fn query_value(value: &str) -> String {
    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}

fn project_dirs() -> Option<directories::ProjectDirs> {
    directories::ProjectDirs::from("dev", "hewig", "jayjay")
}

fn identity_path() -> Option<PathBuf> {
    project_dirs().map(|dirs| dirs.data_local_dir().join("telemetry_install_secret"))
}

fn stamp_path() -> Option<PathBuf> {
    project_dirs().map(|dirs| dirs.cache_dir().join("last_telemetry_day"))
}

fn load_or_create_secret() -> Option<String> {
    load_or_create_secret_at(&identity_path()?)
}

fn load_or_create_secret_at(path: &Path) -> Option<String> {
    if let Ok(secret) = std::fs::read_to_string(path)
        && !secret.trim().is_empty()
    {
        return Some(secret.trim().to_string());
    }
    let secret = Uuid::new_v4().simple().to_string();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok()?;
    }
    std::fs::write(path, &secret).ok()?;
    Some(secret)
}

fn last_sent_day(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn write_stamp(path: &Path, day: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, day)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::{
        Periods, is_release_version, last_sent_day, load_or_create_secret_at, period_id,
        release_telemetry_enabled, write_stamp,
    };

    #[test]
    fn daily_and_monthly_ids_rotate_at_their_utc_boundaries() {
        let secret = "local-install-secret";
        let first = Periods::at(Utc.with_ymd_and_hms(2026, 7, 14, 23, 59, 59).unwrap());
        let next_day = Periods::at(Utc.with_ymd_and_hms(2026, 7, 15, 0, 0, 0).unwrap());
        let next_month = Periods::at(Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap());

        assert_ne!(
            period_id(secret, "day", &first.day),
            period_id(secret, "day", &next_day.day)
        );
        assert_eq!(
            period_id(secret, "month", &first.month),
            period_id(secret, "month", &next_day.month)
        );
        assert_ne!(
            period_id(secret, "month", &first.month),
            period_id(secret, "month", &next_month.month)
        );
    }

    #[test]
    fn installation_secret_and_successful_day_survive_restart() {
        let dir = tempfile::tempdir().unwrap();
        let identity = dir.path().join("identity");
        let stamp = dir.path().join("stamp");

        let first = load_or_create_secret_at(&identity).unwrap();
        let second = load_or_create_secret_at(&identity).unwrap();
        assert_eq!(first, second);

        write_stamp(&stamp, "2026-07-15").unwrap();
        assert_eq!(last_sent_day(&stamp).as_deref(), Some("2026-07-15"));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn debug_builds_do_not_send_telemetry() {
        assert!(!release_telemetry_enabled());
    }

    #[test]
    fn release_versions_are_three_numeric_components() {
        assert!(is_release_version("0.3.1"));
        assert!(is_release_version("10.20.300"));
        assert!(!is_release_version(""));
        assert!(!is_release_version("test"));
        assert!(!is_release_version("unknown"));
        assert!(!is_release_version("0.3"));
        assert!(!is_release_version("0.3.1-dev"));
        assert!(!is_release_version("0.3.1.4"));
    }
}
