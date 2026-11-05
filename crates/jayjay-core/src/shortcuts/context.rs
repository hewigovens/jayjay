#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShortcutContext {
    App,
    RepositoryWindow,
    ChangesList,
    FileList,
    Diff,
    Overview,
    MergeEditor,
    DiffEditor,
    TextEditor,
}

impl ShortcutContext {
    pub fn title(self) -> String {
        match self {
            Self::App => jayjay_i18n::shortcuts::context_app(),
            Self::RepositoryWindow => jayjay_i18n::shortcuts::context_repository_window(),
            Self::ChangesList => jayjay_i18n::shortcuts::context_changes_list(),
            Self::FileList => jayjay_i18n::shortcuts::context_file_list(),
            Self::Diff => jayjay_i18n::shortcuts::context_diff(),
            Self::Overview => jayjay_i18n::shortcuts::context_overview(),
            Self::MergeEditor => jayjay_i18n::shortcuts::context_merge_editor(),
            Self::DiffEditor => jayjay_i18n::shortcuts::context_diff_editor(),
            Self::TextEditor => jayjay_i18n::shortcuts::context_text_editor(),
        }
    }

    pub fn contains(self, other: Self) -> bool {
        if self == other {
            return true;
        }
        match self {
            Self::App => true,
            Self::RepositoryWindow => {
                matches!(other, Self::ChangesList | Self::FileList | Self::Diff)
            }
            _ => false,
        }
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.contains(other) || other.contains(self)
    }
}
