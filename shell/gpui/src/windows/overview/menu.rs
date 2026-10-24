use jayjay_core::overview::OverviewSnapshot;
use jayjay_core::{OverviewChange, OverviewLane};

use crate::ui::icons::glyph;
use crate::ui::popup_menu::PopupMenuEntry;

#[derive(Clone)]
pub(super) enum OverviewAction {
    ShowInGraph { lane: usize, change: Option<String> },
    OpenWorkspace(String),
    RevealWorkspace(String),
    RebaseOntoTrunk(String),
    Abandon(AbandonRequest),
    ForgetWorkspace { name: String, path: Option<String> },
    DeleteWorkspace { name: String, path: String },
    Copy(String),
}

#[derive(Clone)]
pub(super) struct AbandonRequest {
    pub title: String,
    pub commit_ids: Vec<String>,
    pub lane_id: Option<String>,
}

impl AbandonRequest {
    fn lane(lane: &OverviewLane, lane_id: &str) -> Self {
        Self {
            title: lane.title().to_owned(),
            commit_ids: lane
                .changes
                .iter()
                .map(|change| change.commit_id.id.clone())
                .collect(),
            lane_id: Some(lane_id.to_owned()),
        }
    }

    fn change(change: &OverviewChange) -> Self {
        Self {
            title: match change.description.as_str() {
                "" => change.change_id.unique_prefix(),
                description => description.to_owned(),
            },
            commit_ids: vec![change.commit_id.id.clone()],
            lane_id: None,
        }
    }

    pub(super) fn message(&self) -> String {
        match self.commit_ids.len() {
            1 => jayjay_i18n::overview::abandon_this_change_its_descendants_are(),
            count => jayjay_i18n::overview::abandon_lane_message(count),
        }
    }

    pub(super) fn is_current(&self, snapshot: &OverviewSnapshot, lane_ids: &[String]) -> bool {
        let lanes = &snapshot.overview.lanes;
        match &self.lane_id {
            Some(lane_id) => lane_ids
                .iter()
                .position(|id| id == lane_id)
                .is_some_and(|ix| {
                    lanes[ix]
                        .changes
                        .iter()
                        .map(|change| &change.commit_id.id)
                        .eq(&self.commit_ids)
                }),
            None => lanes.iter().any(|lane| {
                lane.changes
                    .iter()
                    .any(|change| self.commit_ids.contains(&change.commit_id.id))
            }),
        }
    }
}

pub(super) fn lane_menu(
    snapshot: &OverviewSnapshot,
    lane_ix: usize,
    lane_id: &str,
) -> Vec<PopupMenuEntry<OverviewAction>> {
    let lane = &snapshot.overview.lanes[lane_ix];
    let mut entries = vec![PopupMenuEntry::item(
        jayjay_i18n::repo::show_in_graph(),
        glyph::ARROW_CIRCLE_RIGHT,
        OverviewAction::ShowInGraph {
            lane: lane_ix,
            change: None,
        },
    )];
    for workspace in &lane.workspaces {
        let Some(info) = snapshot
            .workspace(&workspace.name)
            .filter(|info| info.is_path_resolved)
        else {
            continue;
        };
        if !workspace.is_current {
            entries.push(PopupMenuEntry::item(
                jayjay_i18n::overview::open_in_window(&workspace.name),
                glyph::COLUMNS,
                OverviewAction::OpenWorkspace(info.path.clone()),
            ));
        }
        entries.push(PopupMenuEntry::item(
            jayjay_i18n::overview::show_in(&workspace.name, &crate::platform::file_manager_name()),
            glyph::FOLDER,
            OverviewAction::RevealWorkspace(info.path.clone()),
        ));
    }
    entries.push(PopupMenuEntry::Separator);
    if lane.is_behind_trunk()
        && let Some(root) = lane.changes.last()
    {
        entries.push(PopupMenuEntry::item(
            jayjay_i18n::overview::rebase_lane_onto(snapshot.trunk_name()),
            glyph::GIT_BRANCH,
            OverviewAction::RebaseOntoTrunk(root.commit_id.id.clone()),
        ));
    }
    entries.push(PopupMenuEntry::item(
        jayjay_i18n::overview::abandon_lane_ellipsis(),
        glyph::X_CIRCLE,
        OverviewAction::Abandon(AbandonRequest::lane(lane, lane_id)),
    ));
    for workspace in lane
        .workspaces
        .iter()
        .filter(|workspace| !workspace.is_current)
    {
        let Some(info) = snapshot.workspace(&workspace.name) else {
            continue;
        };
        entries.push(PopupMenuEntry::Separator);
        entries.push(PopupMenuEntry::item(
            jayjay_i18n::palette::forget_workspace_named(&workspace.name),
            glyph::FOLDER_MINUS,
            OverviewAction::ForgetWorkspace {
                name: workspace.name.clone(),
                path: info.is_path_resolved.then(|| info.path.clone()),
            },
        ));
        if info.is_path_resolved {
            entries.push(PopupMenuEntry::item(
                jayjay_i18n::overview::forget_delete_from_disk(&workspace.name),
                glyph::X_CIRCLE,
                OverviewAction::DeleteWorkspace {
                    name: workspace.name.clone(),
                    path: info.path.clone(),
                },
            ));
        }
    }
    entries.push(PopupMenuEntry::Separator);
    entries.push(PopupMenuEntry::item(
        jayjay_i18n::overview::copy_head_change_id(),
        glyph::COPY,
        OverviewAction::Copy(lane.head().change_id.id.clone()),
    ));
    entries
}

pub(super) fn change_menu(
    lane_ix: usize,
    change: &OverviewChange,
) -> Vec<PopupMenuEntry<OverviewAction>> {
    vec![
        PopupMenuEntry::item(
            jayjay_i18n::repo::show_in_graph(),
            glyph::ARROW_CIRCLE_RIGHT,
            OverviewAction::ShowInGraph {
                lane: lane_ix,
                change: Some(change.commit_id.id.clone()),
            },
        ),
        PopupMenuEntry::Separator,
        PopupMenuEntry::item(
            jayjay_i18n::overview::abandon_change_ellipsis(),
            glyph::X_CIRCLE,
            OverviewAction::Abandon(AbandonRequest::change(change)),
        ),
        PopupMenuEntry::Separator,
        PopupMenuEntry::item(
            jayjay_i18n::menu::copy_change_id(),
            glyph::COPY,
            OverviewAction::Copy(change.change_id.id.clone()),
        ),
    ]
}
