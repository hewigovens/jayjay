use jayjay_core::ChangeInfo;

#[derive(Default)]
pub(crate) struct DescriptionState {
    pub revision: Option<String>,
    pub expanded: bool,
    pub overflows: bool,
}

impl DescriptionState {
    pub fn sync_selection(&mut self, change: Option<&ChangeInfo>) {
        let revision = change.map(crate::repo::revset::change_revision);
        if self.revision != revision {
            *self = Self {
                revision,
                ..Self::default()
            };
        }
    }
}
