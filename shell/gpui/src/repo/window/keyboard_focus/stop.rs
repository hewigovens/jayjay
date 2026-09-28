/// Tab stops in cycle order; a stop that is not on screen is skipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusStop {
    Dag,
    FileList,
    TreeToggle,
    FilterToggle,
    ExpandDescription,
    EditDescription,
    DiffLayout,
    EditDiff,
    EditFile,
    SidebarToggle,
    Refresh,
    Pull,
    Push,
    RevsetBack,
    RevsetPresets,
    RevsetFilter,
    RevsetReset,
    Editor,
    Terminal,
    Settings,
    CommitSummary,
    CommitDescription,
}

impl FocusStop {
    pub(super) const CYCLE: [Self; 22] = [
        Self::Dag,
        Self::FileList,
        Self::TreeToggle,
        Self::FilterToggle,
        Self::ExpandDescription,
        Self::EditDescription,
        Self::DiffLayout,
        Self::EditDiff,
        Self::EditFile,
        Self::SidebarToggle,
        Self::Refresh,
        Self::Pull,
        Self::Push,
        Self::RevsetBack,
        Self::RevsetPresets,
        Self::RevsetFilter,
        Self::RevsetReset,
        Self::Editor,
        Self::Terminal,
        Self::Settings,
        Self::CommitSummary,
        Self::CommitDescription,
    ];

    pub(crate) fn is_in_sidebar(self) -> bool {
        matches!(
            self,
            Self::Dag | Self::CommitSummary | Self::CommitDescription
        )
    }

    /// Text inputs take real text focus on arrival, so Space, Return and Escape stay with the text.
    pub(super) fn is_text_input(self) -> bool {
        matches!(self, Self::CommitSummary | Self::CommitDescription)
    }
}
