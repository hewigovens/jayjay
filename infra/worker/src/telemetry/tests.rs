use super::*;
use url::Url;

fn event(query: &str, channel: &'static str) -> Option<Event> {
    Event::from_url(
        &Url::parse(&format!("https://worker.test/?{query}")).unwrap(),
        channel,
    )
}

#[test]
fn filters_probes_and_non_release_versions() {
    for version in [
        "",
        "test",
        "unknown",
        "0.3",
        "0.3.1-dev",
        "0.3.1.4",
        "1..3",
        "１.2.3",
    ] {
        assert!(event(&format!("version={version}"), "gpui").is_none());
    }
    for probe in ["1", "true", "TRUE", "+true+"] {
        assert!(event(&format!("version=0.3.1&probe={probe}"), "gpui").is_none());
    }
    for version in ["0.3.1", "10.20.300", "+0.3.1+"] {
        assert!(event(&format!("version={version}"), "gpui").is_some());
    }
}

#[test]
fn parses_client_payload_and_normalizes_identity() {
    let event = event(
        &format!("version=0.3.1&build=+42+&platform=gpui&os=linux&osver=6.12&arch=x86_64&model=Mac+Studio&daily_id=+{}+&monthly_id={}", "A".repeat(64), "b".repeat(64)),
        "gpui",
    ).unwrap();
    assert_eq!(
        (
            event.channel,
            event.platform.as_str(),
            event.os.as_str(),
            event.os_version.as_str(),
            event.arch.as_str()
        ),
        ("gpui", "gpui", "linux", "6.12", "x86_64")
    );
    assert_eq!(
        (
            event.version.as_str(),
            event.build.as_str(),
            event.model.as_str()
        ),
        ("0.3.1", "42", "Mac Studio")
    );
    let now = "2026-07-15T12:00:00Z".parse().unwrap();
    assert_eq!(
        event.identity_keys("203.0.113.1", now, "secret"),
        ("a".repeat(64), "b".repeat(64), "client")
    );
}

#[test]
fn preserves_sparkle_aliases_defaults_and_cpu_types() {
    for (cpu, arch) in [
        ("16777228", "arm64"),
        ("16777223", "x86_64"),
        ("7", "x86"),
        ("other", "other"),
    ] {
        let event = event(
            &format!("appVersionShort=0.3.1&appVersion=42&osVersion=15.5&cputype={cpu}"),
            "swiftui",
        )
        .unwrap();
        assert_eq!(
            (
                event.version.as_str(),
                event.platform.as_str(),
                event.os.as_str(),
                event.os_version.as_str(),
                event.arch.as_str()
            ),
            ("0.3.1", "macos", "macos", "15.5", arch)
        );
    }
    let event = event("appVersion=0.3.1", "swiftui").unwrap();
    assert_eq!(event.version, "0.3.1");
}

#[test]
fn preserves_query_precedence_and_first_duplicate() {
    assert!(event("version=+&appVersionShort=0.3.1", "swiftui").is_none());
    let event = event("version=1.2.3&version=4.5.6&appVersionShort=7.8.9&os=linux&platform=gpui&osver=6.12&osVersion=15.5&arch=arm64&cputype=16777223", "swiftui").unwrap();
    assert_eq!(
        (
            event.version.as_str(),
            event.os.as_str(),
            event.platform.as_str(),
            event.os_version.as_str(),
            event.arch.as_str()
        ),
        ("1.2.3", "linux", "gpui", "6.12", "arm64")
    );
}

#[test]
fn legacy_and_invalid_identities_keep_exact_network_hashes() {
    let valid = "a".repeat(64);
    let now = "2026-07-15T12:00:00Z".parse().unwrap();
    for identity in [
        String::new(),
        format!("daily_id={valid}"),
        format!("daily_id=not-hex&monthly_id={valid}"),
        format!("daily_id={valid}&monthly_id=short"),
    ] {
        let event = event(&format!("version=0.3.1&{identity}"), "gpui").unwrap();
        assert_eq!(
            event.identity_keys("203.0.113.1", now, "secret"),
            (
                "62b4319c65396906c6688389".into(),
                "617ee4b1080fb87f8a6ff2c5".into(),
                "network"
            )
        );
    }
    let event = event("version=0.3.1", "gpui").unwrap();
    assert!(event.platform.is_empty() && event.os.is_empty());
    let (daily, monthly, _) = event.identity_keys(
        "203.0.113.1",
        "2026-07-16T00:00:00Z".parse().unwrap(),
        "secret",
    );
    assert_ne!(daily, "62b4319c65396906c6688389");
    assert_eq!(monthly, "617ee4b1080fb87f8a6ff2c5");
    let (_, next_month, _) = event.identity_keys(
        "203.0.113.1",
        "2026-08-01T00:00:00Z".parse().unwrap(),
        "secret",
    );
    assert_ne!(next_month, monthly);
}
