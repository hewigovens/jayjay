use super::*;
use url::Url;

fn event(query: &str, channel: &'static str) -> Option<Event> {
    raw_event(
        &format!(
            "{query}&daily_id={}&monthly_id={}",
            "c".repeat(64),
            "d".repeat(64)
        ),
        channel,
    )
}

fn raw_event(query: &str, channel: &'static str) -> Option<Event> {
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
    assert_eq!(
        (event.daily_id, event.monthly_id),
        ("a".repeat(64), "b".repeat(64))
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
fn drops_events_without_a_valid_identity() {
    let valid = "a".repeat(64);
    for identity in [
        String::new(),
        format!("daily_id={valid}"),
        format!("daily_id=not-hex&monthly_id={valid}"),
        format!("daily_id={valid}&monthly_id=short"),
    ] {
        assert!(raw_event(&format!("version=0.3.1&{identity}"), "gpui").is_none());
    }
    let event = event("version=0.3.1", "gpui").unwrap();
    assert!(event.platform.is_empty() && event.os.is_empty());
}
