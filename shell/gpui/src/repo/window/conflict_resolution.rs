use gpui::{App, Context, SharedString};

use super::RepoWindow;
use crate::ui::icons::glyph;

#[derive(Clone)]
pub enum ConflictResolution {
    MergeEditor(SharedString),
    InJayJay,
    EditorTool {
        tool: SharedString,
        editor: SharedString,
    },
    OpenInEditor(SharedString),
}

impl ConflictResolution {
    pub(crate) fn options(
        merge_editor_name: Option<&str>,
        supports_conflict_editor: bool,
        cx: &App,
    ) -> Vec<Self> {
        let editor_id = crate::app::config::current(cx)
            .tools
            .external_editor
            .clone();
        let editor = SharedString::from(crate::app::tools::editor_title(cx));
        let mut options: Vec<Self> = merge_editor_name
            .map(|name| Self::MergeEditor(name.to_owned().into()))
            .into_iter()
            .collect();
        if supports_conflict_editor {
            options.push(Self::InJayJay);
        }
        options.push(match editor_id.as_str() {
            "vscode" | "zed" => Self::EditorTool {
                tool: editor_id.into(),
                editor,
            },
            _ => Self::OpenInEditor(editor),
        });
        options
    }

    pub(crate) fn label(&self) -> String {
        match self {
            Self::MergeEditor(name) => jayjay_i18n::menu::resolve_in(name),
            Self::InJayJay => jayjay_i18n::repo::edit_in_jayjay(),
            Self::EditorTool { editor, .. } => jayjay_i18n::menu::resolve_in(editor),
            Self::OpenInEditor(editor) => jayjay_i18n::menu::open_in(editor),
        }
    }

    pub(crate) fn debug_id(&self) -> &'static str {
        match self {
            Self::MergeEditor(_) => "conflict-resolve-merge-editor",
            Self::InJayJay => "conflict-resolve-jayjay",
            Self::EditorTool { .. } => "conflict-resolve-editor",
            Self::OpenInEditor(_) => "conflict-open-editor",
        }
    }

    pub(crate) fn glyph(&self) -> &'static str {
        match self {
            Self::InJayJay => glyph::PENCIL,
            _ => glyph::EXTERNAL_LINK,
        }
    }
}

impl RepoWindow {
    pub(crate) fn resolve_conflict(
        &mut self,
        resolution: ConflictResolution,
        cx: &mut Context<Self>,
    ) {
        match resolution {
            ConflictResolution::MergeEditor(_) => self.resolve_selected_file(
                |repo, rev, path| repo.resolve_with_merge_editor(rev, path),
                cx,
            ),
            ConflictResolution::InJayJay => self.enter_selected_conflict_editor(cx),
            ConflictResolution::EditorTool { tool, .. } => self.resolve_selected_file(
                move |repo, rev, path| repo.resolve_with_tool(rev, path, &tool),
                cx,
            ),
            ConflictResolution::OpenInEditor(_) => self.open_selected_file_in_editor(cx),
        }
    }
}
