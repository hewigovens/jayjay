use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use jayjay_core::MONO_FONT_FALLBACK_NAMES;

pub(super) fn platform_default_mono() -> String {
    default_mono_from(MONO_FONT_FALLBACK_NAMES)
}

fn default_mono_from(candidates: &[&str]) -> String {
    super::super::pick(candidates)
        .or_else(system_monospace)
        .unwrap_or_else(|| "monospace".to_owned())
}

/// GPUI loads families by exact name, so the fontconfig `monospace` alias only helps once it is resolved to an installed face; otherwise text falls back to a proportional font while widths are still measured as monospace.
fn system_monospace() -> Option<String> {
    SystemSource::new()
        .select_best_match(&[FamilyName::Monospace], &Properties::new())
        .ok()?
        .load()
        .ok()
        .map(|font| font.family_name())
}

#[cfg(test)]
mod tests {
    use font_kit::source::SystemSource;

    use super::default_mono_from;

    #[test]
    fn missing_candidates_resolve_to_an_installed_family() {
        let family = default_mono_from(&["JayJay Missing Mono"]);
        assert_ne!(family, "monospace");
        assert!(
            SystemSource::new().select_family_by_name(&family).is_ok(),
            "{family} is not an installed family"
        );
    }
}
