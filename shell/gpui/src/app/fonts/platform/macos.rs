pub(crate) fn platform_default_mono() -> String {
    ".AppleSystemUIFontMonospaced".to_owned()
}

/// font-kit's CoreText source loads every font into memory and has no file paths; GPUI ships on Linux and Windows only.
pub(crate) fn monospaced_families() -> Vec<String> {
    Vec::new()
}

pub(crate) fn user_families() -> Vec<String> {
    Vec::new()
}
