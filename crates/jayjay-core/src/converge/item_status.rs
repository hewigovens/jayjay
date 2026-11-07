#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvergeItemStatus {
    Edited,
    Using(char),
    Unresolved,
    WillConflict(usize),
    FollowsParents,
    Merged,
}

impl ConvergeItemStatus {
    pub fn label(self) -> String {
        match self {
            Self::Edited => jayjay_i18n::converge::edited(),
            Self::Using(letter) => jayjay_i18n::converge::using(&letter.to_string()),
            Self::Unresolved => jayjay_i18n::converge::unresolved(),
            Self::WillConflict(count) => jayjay_i18n::converge::will_conflict(count),
            Self::FollowsParents => jayjay_i18n::converge::follows_parents(),
            Self::Merged => jayjay_i18n::converge::merged(),
        }
    }

    pub fn needs_attention(self) -> bool {
        matches!(self, Self::Unresolved | Self::WillConflict(_))
    }
}
