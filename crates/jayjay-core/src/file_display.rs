const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "heic", "bmp", "tiff", "tif", "ico", "icns",
];

pub(crate) const MAX_IMAGE_BYTES: usize = 16 * 1024 * 1024;
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
    if bytes.contains(&0) {
        return format!("<binary file ({} bytes)>", bytes.len());
    }
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .unwrap_or_else(|_| format!("<binary file ({} bytes)>", bytes.len()))
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
