use std::hash::{Hash, Hasher};

use jj_lib::conflicts::{
    ConflictMarkerStyle, ConflictMaterializeOptions, MaterializedFileConflictValue,
    MaterializedTreeValue, materialize_merge_result_to_bytes,
};
use jj_lib::files::FileMergeHunkLevel;
use jj_lib::merge::SameChange;
use jj_lib::object_id::ObjectId;
use jj_lib::tree_merge::MergeOptions;

use crate::repo::support::block_on_result;
use crate::types::*;

/// Max inline image size; larger files fall back to the text placeholder.
const MAX_IMAGE_BYTES: usize = 16 * 1024 * 1024;

const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "heic", "bmp", "tiff", "tif", "ico", "icns",
];

pub(super) fn is_image_path(path: &str) -> bool {
    path.rsplit('.')
        .next()
        .map(|ext| IMAGE_EXTENSIONS.iter().any(|e| e.eq_ignore_ascii_case(ext)))
        .unwrap_or(false)
}

pub(super) enum ImagePreviewResult {
    Image(DiffPreview),
    GitLfsPointer(GitLfsPointerInfo),
    None,
}

/// Sniffs for LFS pointer bytes before caching; otherwise writes a content-addressed temp file.
pub(super) fn extract_image_preview(
    path: &jj_lib::repo_path::RepoPath,
    value: MaterializedTreeValue,
) -> CoreResult<ImagePreviewResult> {
    let MaterializedTreeValue::File(mut file) = value else {
        return Ok(ImagePreviewResult::None);
    };
    let bytes = block_on_result(
        &format!("read image {}", path.as_internal_file_string()),
        file.read_all(path),
    )?;
    if bytes.is_empty() || bytes.len() > MAX_IMAGE_BYTES {
        return Ok(ImagePreviewResult::None);
    }

    if let Some(pointer) = detect_git_lfs_pointer_bytes(&bytes) {
        return Ok(ImagePreviewResult::GitLfsPointer(pointer));
    }

    let path_str = path.as_internal_file_string();
    let ext = path_str
        .rsplit('.')
        .next()
        .unwrap_or("img")
        .to_ascii_lowercase();

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    let hash = hasher.finish();

    let cache_dir = std::env::temp_dir().join("jayjay-images");
    if let Err(err) = std::fs::create_dir_all(&cache_dir) {
        return Err(CoreError::Internal {
            message: format!("create image cache dir: {err}"),
        });
    }
    let cache_path = cache_dir.join(format!("{hash:016x}.{ext}"));

    if !cache_path.exists()
        && let Err(err) = std::fs::write(&cache_path, &bytes)
    {
        return Err(CoreError::Internal {
            message: format!("write image cache {}: {err}", cache_path.display()),
        });
    }

    Ok(ImagePreviewResult::Image(DiffPreview::Image {
        path: cache_path.to_string_lossy().into_owned(),
    }))
}

fn detect_git_lfs_pointer_bytes(bytes: &[u8]) -> Option<GitLfsPointerInfo> {
    let text = std::str::from_utf8(bytes).ok()?;
    parse_git_lfs_pointer(text)
}

/// Text placeholder — needed so rename detection and hunk iteration see the entry.
pub(super) fn preview_placeholder(preview: &DiffPreview) -> String {
    match preview {
        DiffPreview::Image { path } => {
            let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            format!("<image ({size} bytes)>")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GitLfsPointerInfo {
    pub(super) oid: String,
    pub(super) size: usize,
}

pub(super) fn materialized_to_string(
    path: &jj_lib::repo_path::RepoPath,
    value: MaterializedTreeValue,
) -> CoreResult<Option<String>> {
    match value {
        MaterializedTreeValue::Absent => Ok(None),
        MaterializedTreeValue::AccessDenied(err) => Ok(Some(format!("<access denied: {err}>"))),
        MaterializedTreeValue::File(mut file) => {
            let read = file.read_all(path);
            let bytes = block_on_result(
                &format!("read file {}", path.as_internal_file_string()),
                read,
            )?;
            if bytes.contains(&0) {
                return Ok(Some(format!("<binary file ({} bytes)>", bytes.len())));
            }
            match String::from_utf8(bytes) {
                Ok(text) => Ok(Some(text)),
                Err(err) => Ok(Some(format!(
                    "<binary file ({} bytes)>",
                    err.into_bytes().len()
                ))),
            }
        }
        MaterializedTreeValue::Symlink { target, .. } => Ok(Some(format!("symlink -> {target}"))),
        MaterializedTreeValue::FileConflict(file) => Ok(Some(materialized_file_conflict(file))),
        MaterializedTreeValue::OtherConflict { id, labels } => Ok(Some(id.describe(&labels))),
        MaterializedTreeValue::GitSubmodule(id) => {
            Ok(Some(format!("<git submodule {}>", id.hex())))
        }
        MaterializedTreeValue::Tree(_) => Ok(Some("<directory>".to_owned())),
    }
}

fn materialized_file_conflict(file: MaterializedFileConflictValue) -> String {
    let options = ConflictMaterializeOptions {
        marker_style: ConflictMarkerStyle::Diff,
        marker_len: None,
        merge: MergeOptions {
            hunk_level: FileMergeHunkLevel::Line,
            same_change: SameChange::Accept,
        },
    };
    let bytes: Vec<u8> =
        materialize_merge_result_to_bytes(&file.contents, &file.labels, &options).into();
    if bytes.contains(&0) {
        return format!("<binary file ({} bytes)>", bytes.len());
    }
    String::from_utf8(bytes)
        .unwrap_or_else(|err| format!("<binary file ({} bytes)>", err.into_bytes().len()))
}

pub(super) fn parse_git_lfs_pointer(text: &str) -> Option<GitLfsPointerInfo> {
    let mut lines = text.lines();
    if lines.next()? != "version https://git-lfs.github.com/spec/v1" {
        return None;
    }

    let mut oid = None;
    let mut size = None;
    for line in lines {
        if let Some(value) = line.strip_prefix("oid sha256:") {
            oid = Some(value.to_owned());
        } else if let Some(value) = line.strip_prefix("size ") {
            size = value.parse::<usize>().ok();
        }
    }

    Some(GitLfsPointerInfo {
        oid: oid?,
        size: size?,
    })
}

pub(super) fn parse_binary_placeholder_size(text: &str) -> Option<usize> {
    text.strip_prefix("<binary file (")?
        .strip_suffix(" bytes)>")?
        .parse::<usize>()
        .ok()
}

pub(super) fn git_lfs_pointer_placeholder(pointer: &GitLfsPointerInfo) -> String {
    format!(
        "<git lfs pointer sha256:{} ({} bytes)>",
        short_oid(&pointer.oid),
        pointer.size
    )
}

pub(super) fn git_lfs_object_placeholder(pointer: &GitLfsPointerInfo) -> String {
    format!(
        "<git lfs object sha256:{} ({} bytes)>",
        short_oid(&pointer.oid),
        pointer.size
    )
}

fn short_oid(oid: &str) -> &str {
    oid.get(..12).unwrap_or(oid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_git_lfs_pointer_text() {
        let pointer = parse_git_lfs_pointer(
            "version https://git-lfs.github.com/spec/v1\n\
             oid sha256:496634778d7b9bdbdb4b98b43a08a00ce8d794ed135a0cb1f345bf6febc5b9b4\n\
             size 742800\n",
        )
        .unwrap();

        assert_eq!(pointer.size, 742800);
        assert_eq!(
            pointer.oid,
            "496634778d7b9bdbdb4b98b43a08a00ce8d794ed135a0cb1f345bf6febc5b9b4"
        );
        assert_eq!(
            git_lfs_pointer_placeholder(&pointer),
            "<git lfs pointer sha256:496634778d7b (742800 bytes)>"
        );
    }

    #[test]
    fn parses_binary_placeholder_size() {
        assert_eq!(
            parse_binary_placeholder_size("<binary file (742800 bytes)>"),
            Some(742800)
        );
    }

    #[test]
    fn is_image_path_recognizes_common_formats() {
        assert!(is_image_path("foo.png"));
        assert!(is_image_path("foo.jpg"));
        assert!(is_image_path("foo.jpeg"));
        assert!(is_image_path("path/to/icon.heic"));
        assert!(is_image_path("Assets/logo.webp"));
        assert!(is_image_path("favicon.icns"));
    }

    #[test]
    fn is_image_path_is_case_insensitive() {
        assert!(is_image_path("Screenshot.PNG"));
        assert!(is_image_path("photo.JPEG"));
        assert!(is_image_path("sprite.Gif"));
    }

    #[test]
    fn detects_git_lfs_pointer_bytes() {
        let pointer_text = b"version https://git-lfs.github.com/spec/v1\n\
             oid sha256:496634778d7b9bdbdb4b98b43a08a00ce8d794ed135a0cb1f345bf6febc5b9b4\n\
             size 742800\n";
        let pointer = detect_git_lfs_pointer_bytes(pointer_text).expect("should detect pointer");
        assert_eq!(pointer.size, 742800);

        // PNG magic bytes → not a pointer.
        let png_magic = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert!(detect_git_lfs_pointer_bytes(&png_magic).is_none());

        // Random binary noise → not a pointer.
        let garbage = [0xFFu8; 32];
        assert!(detect_git_lfs_pointer_bytes(&garbage).is_none());

        // Empty → not a pointer.
        assert!(detect_git_lfs_pointer_bytes(&[]).is_none());
    }

    #[test]
    fn is_image_path_rejects_non_images() {
        assert!(!is_image_path("main.rs"));
        assert!(!is_image_path("readme.md"));
        assert!(!is_image_path("logo.svg")); // SVG is text — handled via opt-in rich view.
        assert!(!is_image_path("noextension"));
        assert!(!is_image_path(""));
    }
}
