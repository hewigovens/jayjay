use crate::file_kind::IMAGE_EXTENSIONS;

pub const MAX_IMAGE_BYTES: usize = 16 * 1024 * 1024;
pub(crate) const MAX_DIFF_BYTES: usize = 32 * 1024 * 1024;

pub(crate) fn is_image_path(path: &str) -> bool {
    path.rsplit('.')
        .next()
        .map(|ext| {
            IMAGE_EXTENSIONS
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(ext))
        })
        .unwrap_or(false)
}

pub(crate) fn bytes_to_display(bytes: &[u8]) -> String {
    if let Some(text) = text_content(bytes) {
        return text.to_owned();
    }
    format!("<binary file ({} bytes)>", bytes.len())
}

pub(crate) fn optional_bytes_to_display<T: AsRef<[u8]>>(content: Option<&T>) -> String {
    content
        .map(|content| bytes_to_display(content.as_ref()))
        .unwrap_or_default()
}

pub(crate) fn text_content(bytes: &[u8]) -> Option<&str> {
    (!bytes.contains(&0))
        .then(|| std::str::from_utf8(bytes).ok())
        .flatten()
}
