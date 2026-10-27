use super::RemoteLocation;
use crate::{JayError, JayResult};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub(crate) struct Response {
    pub id: u64,
    pub result: JayResult<Value>,
    #[serde(default)]
    pub images: HashMap<String, String>,
}

impl Response {
    pub(super) fn decode<T: DeserializeOwned>(
        self,
        method: &str,
        location: &RemoteLocation,
        qualify: bool,
    ) -> JayResult<T> {
        let mut value = self.result?;
        validate_web_urls(method, &value)?;
        super::images::materialize(&mut value, &self.images)?;
        if qualify {
            match &mut value {
                Value::String(path) => *path = location.qualify(path),
                value => qualify_paths(value, location),
            }
        }
        serde_json::from_value(value).map_err(JayError::internal)
    }
}

fn validate_web_urls(method: &str, value: &Value) -> JayResult<()> {
    match method {
        "pull_request_open_url" => validate_web_url(value, false),
        "remote_web_url" if !value.is_null() => validate_web_url(value, false),
        "pull_request_info" if !value.is_null() => validate_web_url(&value["url"], false),
        "submit_stack" => {
            let layers = value["layers"].as_array().ok_or_else(invalid_web_url)?;
            let open_urls = value["open_urls"].as_array().ok_or_else(invalid_web_url)?;
            for layer in layers {
                validate_web_url(&layer["pr_url"], true)?;
            }
            for target in open_urls {
                validate_web_url(target, false)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn invalid_web_url() -> JayError {
    JayError::refused("The SSH host returned an invalid web URL.")
}

fn validate_web_url(value: &Value, allow_empty: bool) -> JayResult<()> {
    let raw = value.as_str().ok_or_else(invalid_web_url)?;
    if raw.is_empty() && allow_empty {
        return Ok(());
    }
    if raw.chars().any(|ch| ch.is_control() || ch.is_whitespace()) || raw.contains('\\') {
        return Err(invalid_web_url());
    }
    let (scheme, authority) = raw.split_once("://").ok_or_else(invalid_web_url)?;
    if !(scheme.eq_ignore_ascii_case("https") || scheme.eq_ignore_ascii_case("http"))
        || authority.starts_with('/')
    {
        return Err(invalid_web_url());
    }
    let url = url::Url::parse(raw).map_err(|_| invalid_web_url())?;
    if !url.has_host() {
        return Err(invalid_web_url());
    }
    Ok(())
}

fn qualify_paths(value: &mut Value, location: &RemoteLocation) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                if matches!(key.as_str(), "path" | "pinnable_path" | "dest") {
                    if let Value::String(path) = value
                        && path.starts_with('/')
                    {
                        *path = location.qualify(path);
                    }
                } else {
                    qualify_paths(value, location);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                qualify_paths(value, location);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn decode(method: &str, value: Value) -> JayResult<Value> {
        Response {
            id: 1,
            result: Ok(value),
            images: HashMap::new(),
        }
        .decode(
            method,
            &RemoteLocation::new("host", "/repo").unwrap(),
            false,
        )
    }

    #[test]
    fn web_destinations_reject_system_targets_in_every_remote_result_slot() {
        for target in [
            r"\\attacker\share\payload.exe",
            r"C:\payload.exe",
            "file:///tmp/program",
            "javascript:alert(1)",
            "ssh://host/repo",
            "https://host/\nfile",
            "https:host/path",
            "https://",
        ] {
            for (method, value) in [
                ("pull_request_open_url", json!(target)),
                ("remote_web_url", json!(target)),
                ("pull_request_info", json!({"url": target})),
                (
                    "submit_stack",
                    json!({"layers": [{"pr_url": target}], "open_urls": []}),
                ),
                (
                    "submit_stack",
                    json!({"layers": [{"pr_url": ""}], "open_urls": [target]}),
                ),
            ] {
                assert!(
                    decode(method, value).is_err(),
                    "accepted {target:?} in {method}"
                );
            }
        }
    }

    #[test]
    fn web_destinations_preserve_forge_links_optional_results_and_non_web_operations() {
        for target in [
            "https://forge.example/repo/pulls/1?tab=files#diff",
            "http://forge.internal:3000/repo",
        ] {
            for (method, value) in [
                ("pull_request_open_url", json!(target)),
                ("remote_web_url", json!(target)),
                ("pull_request_info", json!({"url": target})),
                (
                    "submit_stack",
                    json!({"layers": [{"pr_url": target}, {"pr_url": ""}], "open_urls": [target]}),
                ),
            ] {
                assert_eq!(decode(method, value.clone()).unwrap(), value);
            }
        }
        for method in ["pull_request_info", "remote_web_url"] {
            assert_eq!(decode(method, Value::Null).unwrap(), Value::Null);
        }
        let import = json!({"url": "ssh://host/repo"});
        assert_eq!(
            decode("pull_request_import_preview", import.clone()).unwrap(),
            import
        );
    }
}
