use std::collections::HashSet;

use gpui::{AppContext, Context};
use jayjay_core::repositories::{RepoGroup, RepoListGroups, group_repositories};

use super::window::RepoListWindow;

impl RepoListWindow {
    pub(super) fn show(
        &mut self,
        pinned: Vec<String>,
        recent: Vec<String>,
        cx: &mut Context<Self>,
    ) {
        if self.pinned == pinned && self.recent == recent {
            return;
        }
        let pinned_set = pinned.iter().collect::<HashSet<_>>();
        self.groups = RepoListGroups {
            pinned: pinned.iter().map(|path| flat_group(path)).collect(),
            recent: recent
                .iter()
                .filter(|path| !pinned_set.contains(path))
                .map(|path| flat_group(path))
                .collect(),
        };
        self.pinned = pinned;
        self.recent = recent;
        self.regroup(cx);
    }

    pub(super) fn regroup(&mut self, cx: &mut Context<Self>) {
        self.grouping_generation = self.grouping_generation.wrapping_add(1);
        if self.pinned.is_empty() && self.recent.is_empty() {
            self.groups = RepoListGroups::default();
            return;
        }
        let generation = self.grouping_generation;
        let pinned = self.pinned.clone();
        let recent = self.recent.clone();
        cx.spawn(async move |this, cx| {
            let groups = cx
                .background_spawn(async move { group_repositories(&pinned, &recent) })
                .await;
            let _ = this.update(cx, move |view, cx| {
                if view.grouping_generation == generation {
                    view.groups = groups;
                    cx.notify();
                }
            });
        })
        .detach();
    }
}

fn flat_group(path: &str) -> RepoGroup {
    RepoGroup {
        path: path.to_owned(),
        workspaces: Vec::new(),
    }
}
