use font_kit::family_name::FamilyName;
use font_kit::font::Font;
use font_kit::handle::Handle;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use jayjay_core::is_user_font_path;

/// With no code font saved, the first installed coding font wins over fontconfig's pick.
pub(super) fn platform_default_mono() -> String {
    default_mono_from(&jayjay_core::coding_font_families().collect::<Vec<_>>())
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

pub(crate) fn monospaced_families() -> Vec<String> {
    families_where(|handle| handle.load().is_ok_and(|font| has_fixed_advances(&font)))
}

pub(crate) fn user_families() -> Vec<String> {
    families_where(|handle| matches!(handle, Handle::Path { path, .. } if is_user_font_path(path)))
}

fn families_where(include: impl Fn(&Handle) -> bool) -> Vec<String> {
    let source = SystemSource::new();
    source
        .all_families()
        .unwrap_or_default()
        .into_iter()
        .filter(|family| {
            source
                .select_family_by_name(family)
                .is_ok_and(|handles| handles.fonts().first().is_some_and(&include))
        })
        .collect()
}

/// The fixed-pitch flag misses some mono fonts (Nerd Font patches), so compare advances.
fn has_fixed_advances(font: &Font) -> bool {
    let mut advances = "iMW.0".chars().map(|ch| {
        let glyph = font.glyph_for_char(ch)?;
        font.advance(glyph).ok().map(|advance| advance.x())
    });
    let Some(Some(first)) = advances.next() else {
        return false;
    };
    advances.all(|advance| advance.is_some_and(|width| (width - first).abs() < 0.01))
}

#[cfg(test)]
mod tests {
    use font_kit::family_name::FamilyName;
    use font_kit::properties::Properties;
    use font_kit::source::SystemSource;

    use super::{default_mono_from, has_fixed_advances};

    #[test]
    fn missing_candidates_resolve_to_an_installed_family() {
        let family = default_mono_from(&["JayJay Missing Mono"]);
        assert_ne!(family, "monospace");
        assert!(
            SystemSource::new().select_family_by_name(&family).is_ok(),
            "{family} is not an installed family"
        );
    }

    #[test]
    fn fixed_advances_separate_monospace_from_sans_serif() {
        let load = |family| {
            SystemSource::new()
                .select_best_match(&[family], &Properties::new())
                .expect("generic family resolves")
                .load()
                .expect("generic family loads")
        };
        assert!(has_fixed_advances(&load(FamilyName::Monospace)));
        assert!(!has_fixed_advances(&load(FamilyName::SansSerif)));
    }
}
