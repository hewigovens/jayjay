/// Tab stops in cycle order; a stop that is not on screen is skipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusStop {
    Dag,
    FileList,
    TreeToggle,
    FilterToggle,
    ReviewMode,
    ExpandDescription,
    EditDescription,
    UnlistedFilesAction,
    MarkReviewed,
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
    CommitSummary,
    CommitDescription,
}

impl FocusStop {
    pub(super) const CYCLE: [Self; 24] = [
        Self::Dag,
        Self::FileList,
        Self::TreeToggle,
        Self::FilterToggle,
        Self::ReviewMode,
        Self::ExpandDescription,
        Self::EditDescription,
        Self::UnlistedFilesAction,
        Self::MarkReviewed,
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
