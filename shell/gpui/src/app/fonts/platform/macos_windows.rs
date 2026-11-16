use font_kit::source::SystemSource;

pub(crate) fn pick(candidates: &[&str]) -> Option<String> {
    let source = SystemSource::new();
    candidates
        .iter()
        .find(|name| source.select_family_by_name(name).is_ok())
        .map(|name| (*name).to_owned())
}
