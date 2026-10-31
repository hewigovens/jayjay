use crate::{AppDirs, JayError, JayResult};
use flate2::read::GzDecoder;
use jayjay_network::{Auth, Fetch, HttpClient, Request};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;
use tempfile::NamedTempFile;

const MAX_DOWNLOAD: u32 = 64 * 1024 * 1024;
const MAX_HELPER: u64 = 256 * 1024 * 1024;
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);

static DIRECTORY: OnceLock<PathBuf> = OnceLock::new();

pub fn set_helper_directory(path: PathBuf) {
    let _ = DIRECTORY.set(path);
}

/// A release lists each target's helper by URL and hash in `manifest.json` instead of bundling them.
#[derive(Deserialize)]
struct Download {
    url: String,
    sha256: String,
}

pub(super) struct Helper {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
    pub hash: String,
}

impl Helper {
    pub fn locate(target: &str) -> JayResult<Self> {
        let name = if target.ends_with("windows-msvc") {
            "jayjay-ssh.exe"
        } else {
            "jayjay-ssh"
        };
        let directory = Self::directory();
        let bundled = directory.join(target).join(name);
        if bundled.is_file() {
            return Self::read(bundled);
        }
        if let Some(sibling) = Self::native_sibling(target, name) {
            return Self::read(sibling);
        }
        let manifest = directory.join("manifest.json");
        if !manifest.is_file() {
            return Err(JayError::internal(format!(
                "The SSH helper for {target} is missing. Build it with `just ssh-helper {target}` or install a release containing it."
            )));
        }
        let download = std::fs::read(&manifest)
            .and_then(|json| Ok(serde_json::from_slice::<HashMap<String, Download>>(&json)?))
            .map_err(|error| {
                JayError::internal(format!("Could not read the SSH helper manifest: {error}."))
            })?
            .remove(target)
            .ok_or_else(|| {
                JayError::internal(format!("This release has no SSH helper for {target}."))
            })?;
        let cache = AppDirs::new()
            .ok_or_else(|| JayError::internal("No cache directory for the SSH helper."))?
            .cache
            .join("ssh-helpers")
            .join(target)
            .join(&download.sha256);
        Self::cached(
            &cache.join(name),
            &download,
            &HttpClient::with_timeout(DOWNLOAD_TIMEOUT),
        )
    }

    fn directory() -> PathBuf {
        let bundled = if cfg!(target_os = "macos") {
            "../Resources/ssh-helpers"
        } else {
            "../share/jayjay/ssh-helpers"
        };
        DIRECTORY
            .get()
            .cloned()
            .or_else(|| Some(std::env::current_exe().ok()?.parent()?.join(bundled)))
            .unwrap_or_default()
    }

    /// Development builds and non-release AppImages carry the helper for their own platform beside the executable.
    fn native_sibling(target: &str, name: &str) -> Option<PathBuf> {
        let native = ((cfg!(target_os = "macos") && target.ends_with("apple-darwin"))
            || (cfg!(target_os = "linux") && target.ends_with("linux-musl"))
            || (cfg!(windows) && target.ends_with("windows-msvc")))
            && target.starts_with(std::env::consts::ARCH);
        let sibling = std::env::current_exe().ok()?.parent()?.join(name);
        (native && sibling.is_file()).then_some(sibling)
    }

    fn cached(path: &Path, download: &Download, fetch: &impl Fetch) -> JayResult<Self> {
        if let Ok(helper) = Self::read(path.to_owned())
            && helper.hash == download.sha256
        {
            return Ok(helper);
        }
        let compressed = fetch
            .fetch(&Request::get(&download.url, MAX_DOWNLOAD, &Auth::default()))
            .and_then(|response| response.into_bytes(MAX_DOWNLOAD))
            .map_err(|error| {
                JayError::internal(format!("Could not download the SSH helper: {error}."))
            })?;
        let mut bytes = Vec::new();
        GzDecoder::new(compressed.as_slice())
            .take(MAX_HELPER)
            .read_to_end(&mut bytes)
            .map_err(|error| {
                JayError::internal(format!("Could not unpack the SSH helper: {error}."))
            })?;
        let hash = hex::encode(Sha256::digest(&bytes));
        if hash != download.sha256 {
            return Err(JayError::internal(
                "The downloaded SSH helper does not match this version of JayJay.",
            ));
        }
        Self::install(path, &bytes).map_err(|error| {
            JayError::internal(format!("Could not save the SSH helper: {error}."))
        })?;
        Ok(Self {
            path: path.to_owned(),
            bytes,
            hash,
        })
    }

    /// Writes the helper atomically and drops the copies earlier releases left for the same target.
    fn install(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        let directory = path.parent().unwrap_or(path);
        std::fs::create_dir_all(directory)?;
        let mut temporary = NamedTempFile::new_in(directory)?;
        temporary.write_all(bytes)?;
        #[cfg(unix)]
        temporary
            .as_file()
            .set_permissions(std::fs::Permissions::from_mode(0o755))?;
        temporary.persist(path)?;
        if let Some(versions) = directory.parent().and_then(|target| target.read_dir().ok()) {
            for old in versions.flatten().map(|entry| entry.path()) {
                if old != directory {
                    let _ = std::fs::remove_dir_all(old);
                }
            }
        }
        Ok(())
    }

    fn read(path: PathBuf) -> JayResult<Self> {
        let bytes = std::fs::read(&path).map_err(JayError::internal)?;
        let hash = hex::encode(Sha256::digest(&bytes));
        Ok(Self { path, bytes, hash })
    }
}

#[cfg(test)]
mod tests {
    use super::{Download, Helper};
    use flate2::{Compression, write::GzEncoder};
    use jayjay_network::{MockFetch, NetError, Response};
    use sha2::{Digest, Sha256};
    use std::io::Write;

    #[test]
    fn downloads_are_checked_against_the_manifest_and_cached() {
        let mut gzip = GzEncoder::new(Vec::new(), Compression::fast());
        gzip.write_all(b"helper").unwrap();
        let compressed = gzip.finish().unwrap();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("target/new/jayjay-ssh");
        let old = directory.path().join("target/old");
        std::fs::create_dir_all(&old).unwrap();
        let serve = MockFetch::new(|_| Ok(Response::new(200, compressed.clone())));
        let wrong = Download {
            url: "https://example.invalid/helper.gz".into(),
            sha256: "0".repeat(64),
        };
        assert!(Helper::cached(&path, &wrong, &serve).is_err());
        assert!(!path.exists());

        let download = Download {
            sha256: hex::encode(Sha256::digest(b"helper")),
            ..wrong
        };
        let helper = Helper::cached(&path, &download, &serve).unwrap();
        assert_eq!(
            (helper.bytes.as_slice(), helper.hash.as_str()),
            (&b"helper"[..], download.sha256.as_str())
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"helper");
        assert!(!old.exists());

        let offline = MockFetch::new(|_| Err(NetError::Transport));
        assert_eq!(
            Helper::cached(&path, &download, &offline).unwrap().bytes,
            b"helper"
        );
        assert!(offline.requests().is_empty());
    }
}
