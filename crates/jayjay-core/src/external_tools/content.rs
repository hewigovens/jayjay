use std::fs::{self, File};
use std::io::Read as _;
use std::path::Path;

use crate::CoreResult;
use crate::file_display::{bytes_to_display, text_content};
use crate::filesystem::io_error;

pub(super) struct ExternalContent {
    pub text: String,
    pub is_text: bool,
}

pub(super) fn external_content(path: &Path, limit: usize) -> CoreResult<ExternalContent> {
    let metadata = fs::symlink_metadata(path).map_err(|error| io_error("read", path, error))?;
    if metadata.file_type().is_symlink() {
        return fs::read_link(path)
            .map(|target| ExternalContent {
                text: format!("symlink -> {}", target.to_string_lossy()),
                is_text: false,
            })
            .map_err(|error| io_error("read symlink", path, error));
    }
    if !metadata.is_file() {
        return Ok(ExternalContent {
            text: "<not a regular file>".to_owned(),
            is_text: false,
        });
    }
    let file = File::open(path).map_err(|error| io_error("open", path, error))?;
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| io_error("read", path, error))?;
    if bytes.len() > limit {
        return Ok(ExternalContent {
            text: format!("<file too large to display (over {limit} bytes)>"),
            is_text: false,
        });
    }
    Ok(ExternalContent {
        text: bytes_to_display(&bytes),
        is_text: text_content(&bytes).is_some(),
    })
}
