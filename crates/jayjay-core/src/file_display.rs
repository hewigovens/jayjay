use jayjay_primitives::file_kind::IMAGE_EXTENSIONS;

pub const MAX_IMAGE_BYTES: usize = 16 * 1024 * 1024;
pub(crate) const MAX_DIFF_BYTES: usize = 32 * 1024 * 1024;

pub(crate) fn is_image_path(path: &str) -> bool {
    extension(path).is_some_and(|ext| {
        IMAGE_EXTENSIONS
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(ext))
    })
}

/// The extension of the file name, so a bare `assets/png` has none.
pub(crate) fn extension(path: &str) -> Option<&str> {
    let name = path.rsplit(['/', '\\']).next()?;
    name.rsplit_once('.').map(|(_, extension)| extension)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_paths_need_an_image_extension_on_the_file_name() {
        assert!(is_image_path("docs/imgs/hero.PNG"));
        assert!(!is_image_path("assets/png"));
        assert!(!is_image_path("v1.png/readme"));
    }
}
