use crate::{JayError, JayResult};
use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use std::path::Path;

const PATH_ESCAPES: &AsciiSet = &CONTROLS.add(b' ').add(b'%').add(b'#').add(b'?');

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteLocation {
    pub machine: String,
    pub path: String,
}

impl RemoteLocation {
    pub fn new(machine: &str, path: &str) -> JayResult<Self> {
        if machine.is_empty()
            || machine.starts_with('-')
            || !machine
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._@:-[]".contains(&b))
        {
            return Err(JayError::refused(jayjay_i18n::ssh::error_host()));
        }
        if !(path.starts_with('/') || path == "~" || path.starts_with("~/"))
            || path.contains(['\0', '\r', '\n'])
        {
            return Err(JayError::refused(jayjay_i18n::ssh::error_path()));
        }
        Ok(Self {
            machine: machine.into(),
            path: path.into(),
        })
    }

    pub fn parse(value: &str) -> JayResult<Option<Self>> {
        let Some(rest) = value.strip_prefix("ssh://") else {
            return Ok(None);
        };
        let (machine, path) = rest
            .split_once('/')
            .ok_or_else(|| JayError::refused("Invalid SSH repository location."))?;
        let path = percent_decode_str(path)
            .decode_utf8()
            .map_err(JayError::internal)?;
        Self::new(machine, &path).map(Some)
    }

    pub fn from_path(path: impl AsRef<Path>) -> Option<Self> {
        Self::parse(path.as_ref().to_str()?).ok().flatten()
    }

    pub fn encode(&self) -> String {
        format!(
            "ssh://{}/{}",
            self.machine,
            utf8_percent_encode(&self.path, PATH_ESCAPES)
        )
    }

    pub(crate) fn qualify(&self, path: &str) -> String {
        Self {
            machine: self.machine.clone(),
            path: path.into(),
        }
        .encode()
    }

    pub(crate) fn unqualify(&self, value: &str) -> JayResult<String> {
        let path = match Self::parse(value)? {
            Some(other) if other.machine != self.machine => {
                return Err(JayError::refused(jayjay_i18n::ssh::error_other_machine()));
            }
            Some(other) => other.path,
            None => value.into(),
        };
        if !path.starts_with('/') {
            return Err(JayError::refused(jayjay_i18n::ssh::error_absolute_path()));
        }
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn locations_preserve_host_identity_and_remote_paths() {
        for path in [
            "/home/alice/project with spaces/#100%",
            "~/src/project",
            "/a?b",
        ] {
            let location = RemoteLocation::new("DevBox", path).unwrap();
            assert_eq!(
                RemoteLocation::parse(&location.encode()).unwrap(),
                Some(location)
            );
        }
        for host in ["-F", "host name", "host;touch", "$(command)"] {
            assert!(RemoteLocation::new(host, "/repo").is_err());
        }
        assert!(RemoteLocation::new("devbox", "relative").is_err());
        assert!(
            RemoteLocation::new("devbox", "/repo")
                .unwrap()
                .unqualify("ssh://other//repo")
                .is_err()
        );
        let location = RemoteLocation::new("devbox", "/repo").unwrap();
        for path in ["relative", "~/repo", "ssh://devbox/~/repo"] {
            assert!(location.unqualify(path).is_err());
        }
        assert_eq!(location.unqualify("ssh://devbox//work").unwrap(), "/work");
    }
}
