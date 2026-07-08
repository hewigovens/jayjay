use jayjay_core::MONO_FONT_FALLBACK_NAMES;

pub(super) fn platform_default_mono() -> String {
    super::super::pick(MONO_FONT_FALLBACK_NAMES, "monospace")
}
