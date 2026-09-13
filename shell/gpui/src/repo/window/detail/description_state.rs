#[derive(Default)]
pub(crate) struct DescriptionState {
    pub revision: Option<String>,
    pub expanded: bool,
    pub overflows: bool,
    auto_expand: bool,
}

impl DescriptionState {
    pub fn sync_selection(&mut self, revision: Option<String>, auto_expand: bool) {
        if self.revision != revision {
            *self = Self {
                revision,
                expanded: auto_expand,
                overflows: false,
                auto_expand,
            };
        }
    }

    pub fn apply_preference(&mut self, auto_expand: bool) {
        if self.auto_expand == auto_expand {
            return;
        }
        self.auto_expand = auto_expand;
        self.expanded = auto_expand;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_selection_uses_the_auto_expand_preference() {
        let mut state = DescriptionState::default();
        state.sync_selection(Some("a".into()), true);
        assert!(state.expanded);

        state.expanded = false;
        state.sync_selection(Some("a".into()), true);
        assert!(
            !state.expanded,
            "the same selection must keep a manual collapse"
        );

        state.sync_selection(Some("b".into()), true);
        assert!(state.expanded);
    }

    #[test]
    fn preference_changes_apply_to_the_current_description() {
        let mut state = DescriptionState::default();
        state.sync_selection(Some("a".into()), false);
        assert!(!state.expanded);

        state.apply_preference(true);
        assert!(state.expanded);
        state.expanded = false;
        state.apply_preference(true);
        assert!(
            !state.expanded,
            "an unchanged preference must not override a manual collapse"
        );

        state.apply_preference(false);
        assert!(!state.expanded);
    }
}
