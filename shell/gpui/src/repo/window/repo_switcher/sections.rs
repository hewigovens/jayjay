use jayjay_core::WorkspaceInfo;

use super::model::{RepoSwitcherAction, RepoSwitcherState};
use crate::app::repositories;
use crate::repo::window::picker::{PickerRow, PickerSection, sections_by_best_match};
use crate::ui::icons::glyph;

pub(super) enum RowContent {
    Workspace(WorkspaceInfo),
    Repository {
        path: String,
        glyph: &'static str,
        current: bool,
    },
    Action {
        label: &'static str,
        glyph: &'static str,
    },
}

pub(super) struct SwitcherRow {
    pub(super) id: String,
    search_text: String,
    pub(super) height: f32,
    pub(super) content: RowContent,
    pub(super) action: Option<RepoSwitcherAction>,
}

impl PickerRow for SwitcherRow {
    type Action = RepoSwitcherAction;

    fn action(&self) -> Option<RepoSwitcherAction> {
        self.action.clone()
    }
}

pub(super) fn switcher_sections(
    state: &RepoSwitcherState,
    workspaces: &[WorkspaceInfo],
) -> Vec<PickerSection<SwitcherRow>> {
    let mut sections = Vec::new();
    if workspaces.len() > 1 {
        let mut workspaces = workspaces.to_vec();
        workspaces.sort_by(|left, right| {
            right
                .is_current
                .cmp(&left.is_current)
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
        });
        let rows = workspaces
            .into_iter()
            .map(|workspace| SwitcherRow {
                id: format!("repo-switcher-workspace-{}", workspace.name),
                search_text: format!(
                    "{} {} {}",
                    workspace.name,
                    workspace.description,
                    if workspace.is_path_resolved {
                        ""
                    } else {
                        "path unavailable"
                    }
                ),
                height: 46.,
                action: (!workspace.is_current && workspace.is_path_resolved)
                    .then(|| RepoSwitcherAction::Open(workspace.path.clone())),
                content: RowContent::Workspace(workspace),
            })
            .collect();
        sections.extend(filtered_section(
            "repo-switcher-workspaces",
            Some("Workspaces"),
            rows,
            state.query.input.text(),
        ));
    }

    let mut repository_rows = Vec::new();
    for (index, path) in state.open.iter().enumerate() {
        let current = path == &state.current;
        repository_rows.push(SwitcherRow {
            id: format!("repo-switcher-open-{index}"),
            search_text: format!("{} {path}", repositories::repository_name(path)),
            height: 30.,
            action: (!current).then(|| RepoSwitcherAction::Activate(path.clone())),
            content: RowContent::Repository {
                path: path.clone(),
                glyph: if current {
                    glyph::CHECK
                } else {
                    glyph::COLUMNS
                },
                current,
            },
        });
    }
    for (index, path) in state.pinned.iter().enumerate() {
        repository_rows.push(SwitcherRow {
            id: format!("repo-switcher-pinned-{index}"),
            search_text: format!("{} {path}", repositories::repository_name(path)),
            height: 30.,
            action: Some(RepoSwitcherAction::Open(path.clone())),
            content: RowContent::Repository {
                path: path.clone(),
                glyph: glyph::PIN,
                current: false,
            },
        });
    }
    sections.extend(filtered_section(
        "repo-switcher-repositories",
        Some("Repositories"),
        repository_rows,
        state.query.input.text(),
    ));

    let globals = vec![
        SwitcherRow {
            id: "repo-switcher-list".to_owned(),
            search_text: "Repository List".to_owned(),
            height: 28.,
            content: RowContent::Action {
                label: "Repository List…",
                glyph: glyph::LIST,
            },
            action: Some(RepoSwitcherAction::ShowRepositoryList),
        },
        SwitcherRow {
            id: "repo-switcher-open-repository".to_owned(),
            search_text: "Open Repository".to_owned(),
            height: 28.,
            content: RowContent::Action {
                label: "Open Repository…",
                glyph: glyph::FOLDER,
            },
            action: Some(RepoSwitcherAction::OpenRepository),
        },
    ];
    sections.extend(filtered_section(
        "repo-switcher-global",
        None,
        globals,
        state.query.input.text(),
    ));
    sections_by_best_match(sections)
}

fn filtered_section(
    id: &'static str,
    title: Option<&'static str>,
    rows: Vec<SwitcherRow>,
    query: &str,
) -> Option<PickerSection<SwitcherRow>> {
    PickerSection::filtered(id, title, rows, query, |row| row.search_text.clone())
}
