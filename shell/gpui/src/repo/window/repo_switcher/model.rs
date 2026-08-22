use gpui::{Pixels, Point};

use crate::repo::window::picker::PickerQuery;

pub(crate) struct RepoSwitcherState {
    pub(super) anchor: Point<Pixels>,
    pub(super) current: String,
    pub(super) open: Vec<String>,
    pub(super) pinned: Vec<String>,
    pub(super) query: PickerQuery,
}

#[derive(Clone)]
pub(super) enum RepoSwitcherAction {
    Activate(String),
    Open(String),
    ShowRepositoryList,
    OpenRepository,
}
