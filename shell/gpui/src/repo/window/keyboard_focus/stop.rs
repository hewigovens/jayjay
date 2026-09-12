/// Tab stops in cycle order; a stop that is not on screen is skipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusStop {
    Dag,
    FileList,
    TreeToggle,
    FilterToggle,
    ExpandDescription,
    DiffLayout,
    EditDescription,
    EditDiff,
    RevsetFilter,
    RevsetInput,
    Refresh,
    Pull,
    Push,
    Editor,
    Terminal,
    Settings,
    CommitSummary,
    CommitDescription,
}

impl FocusStop {
    pub(super) const CYCLE: [Self; 18] = [
        Self::Dag,
        Self::FileList,
        Self::TreeToggle,
        Self::FilterToggle,
        Self::ExpandDescription,
        Self::DiffLayout,
        Self::EditDescription,
        Self::EditDiff,
        Self::RevsetFilter,
        Self::RevsetInput,
        Self::Refresh,
        Self::Pull,
        Self::Push,
        Self::Editor,
        Self::Terminal,
        Self::Settings,
        Self::CommitSummary,
        Self::CommitDescription,
    ];

    /// Text inputs take real text focus on arrival, so Space, Return and Escape stay with the text.
    pub(super) fn is_text_input(self) -> bool {
        matches!(
            self,
            Self::RevsetInput | Self::CommitSummary | Self::CommitDescription
        )
    }
}
