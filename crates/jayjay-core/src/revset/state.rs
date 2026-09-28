use super::default::DEFAULT_REVSET;
use super::expressions::ancestors_revset;
use super::recent::remember_revset;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevsetFilterState {
    pub revset: String,
    pub previous: Option<String>,
    pub recent: Vec<String>,
}

impl RevsetFilterState {
    pub fn new(revset: &str) -> Self {
        Self {
            revset: revset.to_owned(),
            previous: None,
            recent: Vec::new(),
        }
    }

    pub fn apply(&mut self, revset: &str) {
        let revset = revset.trim();
        self.revset = if revset.is_empty() {
            DEFAULT_REVSET.to_owned()
        } else {
            revset.to_owned()
        };
        self.previous = None;
        self.recent = remember_revset(&self.recent, &self.revset);
    }

    pub fn show_ancestors(&mut self, change_id: &str) {
        let previous = self.previous.take().unwrap_or_else(|| self.revset.clone());
        self.apply(&ancestors_revset(change_id));
        self.previous = Some(previous);
    }

    pub fn back(&mut self) {
        if let Some(previous) = self.previous.take() {
            self.apply(&previous);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn show_ancestors_returns_to_the_filter_it_first_replaced() {
        let mut state = RevsetFilterState::new(DEFAULT_REVSET);
        state.apply("mine()");
        state.show_ancestors("aaa");
        state.show_ancestors("bbb");
        assert_eq!(state.revset, ancestors_revset("bbb"));
        assert_eq!(state.previous.as_deref(), Some("mine()"));

        state.back();
        assert_eq!(state.revset, "mine()");
        assert_eq!(state.previous, None);

        state.show_ancestors("aaa");
        state.apply("  ");
        assert_eq!(state.revset, DEFAULT_REVSET);
        assert_eq!(
            state.previous, None,
            "an explicit filter drops the way back"
        );
    }
}
