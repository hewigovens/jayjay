use std::hash::{Hash, Hasher};

use jj_lib::backend::MergedTreeValueExt as _;
use jj_lib::conflicts::{
    ConflictMarkerStyle, ConflictMaterializeOptions, MaterializedFileConflictValue,
    MaterializedFileValue, MaterializedTreeValue, materialize_merge_result_to_bytes,
};
use jj_lib::files::FileMergeHunkLevel;
use jj_lib::merge::SameChange;
use jj_lib::object_id::ObjectId;
use jj_lib::tree_merge::MergeOptions;

pub(super) use crate::file_display::is_image_path;
use crate::file_display::{MAX_DIFF_BYTES, MAX_IMAGE_BYTES, bytes_to_display};
use crate::repo::support::block_on_result;
use crate::types::*;

/// Reads at most `limit + 1` bytes, capping peak allocation. Returns `(bytes, truncated)`;
/// on truncation the buffer is cleared (the content gets a placeholder instead).
async fn read_capped(
    file: &mut MaterializedFileValue,
    limit: usize,
) -> std::io::Result<(Vec<u8>, bool)> {
    crate::filesystem::read_to_limit(&mut file.reader, limit).await
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
) -> CoreResult<(ImagePreviewResult, bool)> {
    let MaterializedTreeValue::File(mut file) = value else {
        return Ok((ImagePreviewResult::None, false));
    };
    let (bytes, truncated) = block_on_result(
        &format!("read image {}", path.as_internal_file_string()),
        read_capped(&mut file, MAX_IMAGE_BYTES),
    )?;
    if truncated {
        return Ok((ImagePreviewResult::None, false));
    }
    let supports_file_editor = !bytes.contains(&0) && std::str::from_utf8(&bytes).is_ok();
    if bytes.is_empty() {
        return Ok((ImagePreviewResult::None, supports_file_editor));
    }

    if let Some(pointer) = detect_git_lfs_pointer_bytes(&bytes) {
        return Ok((
            ImagePreviewResult::GitLfsPointer(pointer),
            supports_file_editor,
        ));
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

    Ok((
        ImagePreviewResult::Image(DiffPreview::Image {
            path: cache_path.to_string_lossy().into_owned(),
        }),
        false,
    ))
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
    oid: String,
    pub(super) size: usize,
}

pub(super) enum MaterializedContent {
    Absent,
    File(Vec<u8>),
    Display(String),
}

impl MaterializedContent {
    pub(super) fn raw_string(&self) -> Option<String> {
        match self {
            MaterializedContent::Absent => None,
            MaterializedContent::File(bytes) => Some(bytes_to_display(bytes)),
            MaterializedContent::Display(text) => Some(text.clone()),
        }
    }

    pub(super) fn file_bytes(&self) -> Option<&[u8]> {
        match self {
            MaterializedContent::File(bytes) => Some(bytes),
            MaterializedContent::Absent | MaterializedContent::Display(_) => None,
        }
    }

    pub(super) fn supports_file_editor(&self) -> bool {
        matches!(self, Self::File(bytes) if !bytes.contains(&0) && std::str::from_utf8(bytes).is_ok())
    }
}

pub(super) fn materialized_to_content(
    path: &jj_lib::repo_path::RepoPath,
    value: MaterializedTreeValue,
) -> CoreResult<MaterializedContent> {
    match value {
        MaterializedTreeValue::Absent => Ok(MaterializedContent::Absent),
        MaterializedTreeValue::AccessDenied(err) => Ok(MaterializedContent::Display(format!(
            "<access denied: {err}>"
        ))),
        MaterializedTreeValue::File(mut file) => {
            let (bytes, truncated) = block_on_result(
                &format!("read file {}", path.as_internal_file_string()),
                read_capped(&mut file, MAX_DIFF_BYTES),
            )?;
            if truncated {
                return Ok(MaterializedContent::Display(format!(
                    "<file too large to display (over {MAX_DIFF_BYTES} bytes)>"
                )));
            }
            Ok(MaterializedContent::File(bytes))
        }
        MaterializedTreeValue::Symlink { target, .. } => {
            Ok(MaterializedContent::Display(format!("symlink -> {target}")))
        }
        MaterializedTreeValue::FileConflict(file) => Ok(MaterializedContent::Display(
            materialized_file_conflict(file),
        )),
        MaterializedTreeValue::OtherConflict { id, labels } => Ok(MaterializedContent::Display(
            format!("<conflict>\n{}", id.describe(&labels)),
        )),
        MaterializedTreeValue::GitSubmodule(id) => Ok(MaterializedContent::Display(format!(
            "<git submodule {}>",
            id.hex()
        ))),
        MaterializedTreeValue::Tree(_) => {
            Ok(MaterializedContent::Display("<directory>".to_owned()))
        }
    }
}

fn materialized_file_conflict(file: MaterializedFileConflictValue) -> String {
    // jj-lib already holds each side in memory; just avoid a second merged copy when oversized.
    let total: usize = file.contents.iter().map(|side| side.len()).sum();
    if total > MAX_DIFF_BYTES {
        return format!("<conflict too large to display (over {MAX_DIFF_BYTES} bytes)>");
    }
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
    bytes_to_display(&bytes)
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
