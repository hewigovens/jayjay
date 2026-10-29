use std::fs;
use std::io::{ErrorKind, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::LazyLock;

use sha2::{Digest, Sha256};
use tempfile::{NamedTempFile, TempDir};

use crate::file_kind::IMAGE_EXTENSIONS;
use crate::{JayError, JayResult, MAX_IMAGE_BYTES};

static CACHE: LazyLock<std::io::Result<ImageCache>> = LazyLock::new(ImageCache::new);

pub(crate) struct ImageCache {
    directory: TempDir,
}

impl ImageCache {
    fn new() -> std::io::Result<Self> {
        let mut builder = tempfile::Builder::new();
        builder.prefix("jayjay-images-");
        #[cfg(unix)]
        builder.permissions(fs::Permissions::from_mode(0o700));
        Ok(Self {
            directory: builder.tempdir()?,
        })
    }

    pub(crate) fn shared() -> JayResult<&'static Self> {
        CACHE.as_ref().map_err(JayError::internal)
    }

    pub(crate) fn store(&self, bytes: &[u8], source_path: &str) -> JayResult<PathBuf> {
        if bytes.len() > MAX_IMAGE_BYTES {
            return Err(JayError::refused("The image exceeds the preview limit."));
        }
        let extension = source_path
            .rsplit(['/', '\\'])
            .next()
            .and_then(|name| name.rsplit_once('.'))
            .map(|(_, extension)| extension.to_ascii_lowercase())
            .filter(|extension| IMAGE_EXTENSIONS.contains(&extension.as_str()))
            .ok_or_else(|| JayError::refused("Unsupported image preview extension."))?;
        let destination = self.directory.path().join(format!(
            "{}.{extension}",
            hex::encode(Sha256::digest(bytes))
        ));
        let mut temporary =
            NamedTempFile::new_in(self.directory.path()).map_err(JayError::internal)?;
        temporary.write_all(bytes).map_err(JayError::internal)?;
        if let Err(error) = temporary.persist_noclobber(&destination) {
            if error.error.kind() != ErrorKind::AlreadyExists {
                return Err(JayError::internal(error.error));
            }
            let metadata = fs::symlink_metadata(&destination).map_err(JayError::internal)?;
            if !metadata.is_file() || metadata.len() != bytes.len() as u64 {
                return Err(JayError::refused(
                    "The image preview cache entry is unsafe.",
                ));
            }
        }
        Ok(destination)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrent_previews_reuse_complete_files_and_reject_unsafe_hints() {
        let cache = ImageCache::new().unwrap();
        let bytes = vec![42; 8192];
        std::thread::scope(|scope| {
            let tasks: Vec<_> = (0..8)
                .map(|_| scope.spawn(|| cache.store(&bytes, "remote/Preview.PNG").unwrap()))
                .collect();
            let paths: Vec<_> = tasks.into_iter().map(|task| task.join().unwrap()).collect();
            assert!(paths.iter().all(|path| path == &paths[0]));
            assert_eq!(fs::read(&paths[0]).unwrap(), bytes);
        });
        for source in [
            "preview.exe",
            "preview.png:stream",
            "preview.png/../../program.sh",
            "preview.png\\..\\program.sh",
            "preview",
        ] {
            assert!(cache.store(&bytes, source).is_err(), "{source}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn planted_cache_entries_cannot_redirect_or_replace_preview_bytes() {
        let cache = ImageCache::new().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let bytes = b"preview fixture";
        let entry = cache
            .directory
            .path()
            .join(format!("{}.png", hex::encode(Sha256::digest(bytes))));
        let target = outside.path().join("outside");
        std::os::unix::fs::symlink(&target, &entry).unwrap();
        assert!(cache.store(bytes, "preview.png").is_err());
        assert!(!target.exists());
        fs::write(&target, "unchanged").unwrap();
        assert!(cache.store(bytes, "preview.png").is_err());
        assert_eq!(fs::read_to_string(&target).unwrap(), "unchanged");
    }
}
