#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConvergeItem {
    Description,
    Contents,
    Author,
    Parents,
}

impl ConvergeItem {
    pub const ALL: [Self; 4] = [
        Self::Description,
        Self::Contents,
        Self::Author,
        Self::Parents,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Self::Description => "description",
            Self::Contents => "contents",
            Self::Author => "author",
            Self::Parents => "parents",
        }
    }

    pub fn title(self) -> String {
        match self {
            Self::Description => jayjay_i18n::converge::description(),
            Self::Contents => jayjay_i18n::converge::contents(),
            Self::Author => jayjay_i18n::converge::author(),
            Self::Parents => jayjay_i18n::converge::parents(),
        }
    }
}
