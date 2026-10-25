use crate::messages::LOCALES;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Language {
    pub tag: &'static str,
    pub name: &'static str,
}

impl Language {
    pub fn all() -> impl Iterator<Item = Self> {
        LOCALES.iter().map(|(language, _)| *language)
    }
}
