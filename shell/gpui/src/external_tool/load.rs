use gpui::{AppContext as _, Context};

use crate::ui::text_area::TextArea;

use super::ExternalToolInvocation;
use super::diff::ExternalDiffSession;
use super::merge::ExternalMergeSession;
use super::view::{ExternalToolState, ExternalToolWindow};

enum LoadedExternalTool {
    Diff(ExternalDiffSession),
    Merge(Box<ExternalMergeSession>),
}

impl ExternalToolWindow {
    pub(super) fn load(&mut self, cx: &mut Context<Self>) {
        let invocation = self.invocation.clone();
        cx.spawn(async move |this, cx| {
            let loaded = cx
                .background_spawn(async move { load_invocation(invocation) })
                .await;
            let _ = this.update(cx, move |view, cx| {
                if !matches!(view.state, ExternalToolState::Loading) {
                    return;
                }
                match loaded {
                    Ok(LoadedExternalTool::Diff(session)) => {
                        view.state = ExternalToolState::Diff(session);
                    }
                    Ok(LoadedExternalTool::Merge(session)) => {
                        view.state = merge_state(session, cx);
                    }
                    Err(error) => {
                        view.exit_state.fail();
                        view.state = ExternalToolState::Error;
                        view.error_message = Some(error);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}

fn load_invocation(invocation: ExternalToolInvocation) -> Result<LoadedExternalTool, String> {
    match invocation {
        ExternalToolInvocation::Diff {
            left,
            right,
            editable,
        } => ExternalDiffSession::load(left.into(), right.into(), editable)
            .map(LoadedExternalTool::Diff)
            .map_err(|error| error.to_string()),
        ExternalToolInvocation::Merge {
            left,
            base,
            right,
            output,
            path,
            marker_length,
            output_is_initialized,
        } => ExternalMergeSession::load(
            left.into(),
            base.into(),
            right.into(),
            output.into(),
            path,
            output_is_initialized,
            marker_length as usize,
        )
        .map(Box::new)
        .map(LoadedExternalTool::Merge)
        .map_err(|error| error.to_string()),
    }
}

fn merge_state(
    session: Box<ExternalMergeSession>,
    cx: &mut Context<ExternalToolWindow>,
) -> ExternalToolState {
    let path = session.repo_path.clone();
    let sources = [
        (session.left.clone(), Some(session.base.clone())),
        (session.base.clone(), None),
        (session.right.clone(), Some(session.base.clone())),
    ]
    .map(|(content, base)| {
        let path = path.clone();
        cx.new(move |cx| match base {
            Some(base) => {
                TextArea::diff_highlighted_code_block(content, path, base, cx).full_bleed_pane()
            }
            None => TextArea::highlighted_code_block(content, path, cx).full_bleed_pane(),
        })
    });
    let result = cx.new(|cx| {
        TextArea::code_editor(
            session.initial_result.clone(),
            session.repo_path.clone(),
            "Merge result",
            360.,
            cx,
        )
        .full_bleed_pane()
        .starting_at_top()
    });
    TextArea::subscribe_updates(&result, cx);
    ExternalToolState::Merge {
        session,
        sources,
        result,
    }
}
