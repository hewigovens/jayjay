use crate::image_cache::ImageCache;
use crate::{JayError, JayResult};
use base64::Engine;
use serde_json::Value;
use std::collections::HashMap;

fn for_each_image_path(
    value: &mut Value,
    visit: &mut impl FnMut(&mut String) -> JayResult<()>,
) -> JayResult<()> {
    match value {
        Value::Object(object) => {
            if let Some(Value::Object(image)) = object.get_mut("Image")
                && let Some(Value::String(path)) = image.get_mut("path")
            {
                visit(path)?;
            }
            // A host can add fields beside an image, so every sibling is still searched for previews.
            object
                .values_mut()
                .try_for_each(|value| for_each_image_path(value, visit))
        }
        Value::Array(values) => values
            .iter_mut()
            .try_for_each(|value| for_each_image_path(value, visit)),
        _ => Ok(()),
    }
}

pub(super) fn materialize(value: &mut Value, images: &HashMap<String, String>) -> JayResult<()> {
    for_each_image_path(value, &mut |path| {
        let encoded = images
            .get(path.as_str())
            .ok_or_else(|| JayError::internal("The SSH image preview is missing."))?;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(JayError::internal)?;
        let destination = ImageCache::shared()?.store(&bytes, path)?;
        *path = destination.to_string_lossy().into_owned();
        Ok(())
    })
}

pub(super) fn collect(value: &mut Value, images: &mut HashMap<String, String>) -> JayResult<()> {
    for_each_image_path(value, &mut |path| {
        let bytes = std::fs::read(path.as_str()).map_err(JayError::internal)?;
        if bytes.len() > crate::MAX_IMAGE_BYTES {
            return Err(JayError::refused(
                "The remote image exceeds the preview limit.",
            ));
        }
        images.insert(
            path.clone(),
            base64::engine::general_purpose::STANDARD.encode(bytes),
        );
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[cfg(unix)]
    use sha2::{Digest, Sha256};
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn nested_previews_transfer_bytes_and_replace_remote_paths() {
        let temporary = tempfile::tempdir().unwrap();
        let source = temporary.path().join("remote-image.png");
        let bytes = b"remote image fixture";
        std::fs::write(&source, bytes).unwrap();
        let mut value = json!({"diff": [{"new": {"Image": {"path": source}}}]});
        let mut images = HashMap::new();
        collect(&mut value, &mut images).unwrap();
        std::fs::remove_file(&source).unwrap();
        materialize(&mut value, &images).unwrap();
        let cached = value["diff"][0]["new"]["Image"]["path"].as_str().unwrap();
        assert_ne!(cached, source.to_str().unwrap());
        assert_eq!(std::fs::read(cached).unwrap(), bytes);
        assert!(materialize(&mut json!({"Image": {"path": "/missing"}}), &images).is_err());
        let mut shadowed = json!({"Image": {}, "new": {"Image": {"path": "/dev/zero"}}});
        assert!(materialize(&mut shadowed, &images).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn preview_materialization_ignores_an_attacker_precreated_shared_cache() {
        const CHILD: &str = "JAYJAY_PREVIEW_CACHE_FIXTURE";
        if let Some(root) = std::env::var_os(CHILD) {
            let root = std::path::PathBuf::from(root);
            let legacy = std::env::temp_dir().join("jayjay-images");
            std::fs::create_dir(&legacy).unwrap();
            std::fs::set_permissions(&legacy, std::fs::Permissions::from_mode(0o777)).unwrap();
            let bytes = b"remote preview fixture";
            let target = root.join("outside");
            let planted = legacy.join(format!("{}.png", hex::encode(Sha256::digest(bytes))));
            std::os::unix::fs::symlink(&target, planted).unwrap();
            let mut value = json!({"Image": {"path": "/remote/preview.png"}});
            let images = HashMap::from([(
                "/remote/preview.png".into(),
                base64::engine::general_purpose::STANDARD.encode(bytes),
            )]);
            materialize(&mut value, &images).unwrap();
            assert!(
                !target.exists(),
                "preview bytes escaped through a planted cache symlink"
            );
            let cached = std::path::Path::new(value["Image"]["path"].as_str().unwrap());
            assert_eq!(std::fs::read(cached).unwrap(), bytes);
            assert_ne!(cached.parent().unwrap(), legacy);
            for path in [cached, cached.parent().unwrap()] {
                assert_eq!(
                    std::fs::metadata(path).unwrap().permissions().mode() & 0o077,
                    0
                );
            }
            return;
        }
        let fixture = tempfile::tempdir().unwrap();
        let shared = fixture.path().join("shared");
        std::fs::create_dir(&shared).unwrap();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "ssh::images::tests::preview_materialization_ignores_an_attacker_precreated_shared_cache", "--nocapture"])
            .env(CHILD, fixture.path())
            .env("TMPDIR", shared)
            .status().unwrap();
        assert!(status.success());
    }
}
