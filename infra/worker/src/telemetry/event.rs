use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use url::Url;

pub(crate) struct Event {
    pub(crate) channel: &'static str,
    pub(crate) platform: String,
    pub(crate) os: String,
    pub(crate) os_version: String,
    pub(crate) arch: String,
    pub(crate) version: String,
    pub(crate) build: String,
    pub(crate) model: String,
    identity: Option<(String, String)>,
}

impl Event {
    pub(crate) fn from_url(url: &Url, channel: &'static str) -> Option<Self> {
        let query: Vec<_> = url.query_pairs().collect();
        let get = |key: &str| {
            query
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value.as_ref())
                .unwrap_or("")
        };
        let first = |keys: &[&str]| {
            keys.iter()
                .map(|key| get(key))
                .find(|value| !value.is_empty())
                .unwrap_or("")
        };
        let version = first(&["version", "appVersionShort", "appVersion"]).trim();
        if matches!(
            get("probe").trim().to_ascii_lowercase().as_str(),
            "1" | "true"
        ) || !is_release_version(version)
        {
            return None;
        }

        let default = if channel == "swiftui" { "macos" } else { "" };
        let or_default = |value: &str| if value.is_empty() { default } else { value }.to_owned();
        let arch = match get("arch") {
            "" => match get("cputype") {
                "16777223" => "x86_64",
                "16777228" => "arm64",
                "7" => "x86",
                value => value,
            },
            value => value,
        };
        Some(Self {
            channel,
            platform: or_default(get("platform")),
            os: or_default(get("os")),
            os_version: first(&["osver", "osVersion"]).to_owned(),
            arch: arch.to_owned(),
            version: version.to_owned(),
            build: get("build").trim().to_owned(),
            model: get("model").to_owned(),
            identity: period_id(get("daily_id")).zip(period_id(get("monthly_id"))),
        })
    }

    pub(crate) fn identity_keys(
        &self,
        ip: &str,
        now: DateTime<Utc>,
        secret: &str,
    ) -> (String, String, &'static str) {
        if let Some((daily, monthly)) = &self.identity {
            return (daily.clone(), monthly.clone(), "client");
        }
        let period_key = |period| {
            let hash = Sha256::digest(format!("{ip}|{period}|{secret}"));
            hex::encode(&hash[..12])
        };
        (
            period_key((now.timestamp() / 86400).to_string()),
            period_key(now.format("%Y-%m").to_string()),
            "network",
        )
    }
}

fn period_id(value: &str) -> Option<String> {
    let value = value.trim().to_ascii_lowercase();
    (value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())).then_some(value)
}

fn is_release_version(version: &str) -> bool {
    let parts: Vec<_> = version.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}
