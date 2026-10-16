use std::sync::Arc;

use gpui::{
    AnyElement, AppContext, Context, InteractiveElement, IntoElement, ParentElement, Styled,
    Window, div, px, rgb,
};
use jayjay_core::DiffHunk;
use jayjay_core::diff::{FileDiff, compute_file_diff};

use super::FileHistoryView;
use crate::app::theme::{Theme, ui_font_size};
use crate::diff::{hunk_is_image, image_diff_view, read_only_diff};
use crate::ui::loading::loading_status;
use crate::ui::primitives::{placeholder, placeholder_err};

#[derive(Clone, PartialEq, Eq)]
struct ComparisonRequest {
    from: Option<String>,
    to: String,
    path: String,
}

impl FileHistoryView {
    pub(super) fn load_comparison(&mut self, cx: &mut Context<Self>) {
        self.current_hunk = None;
        self.current_diff = None;
        self.diff_error = None;
        let request = self.comparison_request();
        cx.notify();
        let Some(request) = request else {
            return;
        };
        let repo = self.repo.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn({
                    let request = request.clone();
                    async move { load_request(&repo, &request) }
                })
                .await;
            let _ = this.update(cx, move |view, cx| {
                // A slower load for a pair the user has already left must not paint under the new labels.
                if view.comparison_request().as_ref() != Some(&request) {
                    return;
                }
                match result {
                    Ok((hunk, diff)) => {
                        view.current_hunk = Some(hunk);
                        view.current_diff = Some(Arc::new(diff));
                    }
                    Err(error) => view.diff_error = Some(crate::app::error_text(error)),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn comparison_request(&self) -> Option<ComparisonRequest> {
        if self.comparison.is_same_revision() {
            return None;
        }
        Some(ComparisonRequest {
            from: self.comparison.compare_from().map(str::to_owned),
            to: self.comparison.compare_to()?.to_owned(),
            path: self.path.to_string(),
        })
    }
}

fn load_request(
    repo: &jayjay_core::Repo,
    request: &ComparisonRequest,
) -> jayjay_core::JayResult<(DiffHunk, FileDiff)> {
    let hunk = match request.from.as_deref() {
        Some(from) => repo.interdiff_file(from, &request.to, &request.path)?,
        None => repo.show_file(&request.to, &request.path)?,
    };
    let old = hunk.old.content.clone().unwrap_or_default();
    let new = hunk.new.content.clone().unwrap_or_default();
    let diff = compute_file_diff(&hunk.path, &old, &new, false);
    Ok((hunk, diff))
}

pub(super) fn comparison_diff(
    view: &FileHistoryView,
    theme: &Theme,
    window: &mut Window,
    cx: &mut Context<FileHistoryView>,
) -> AnyElement {
    let body = if view.comparison.is_same_revision() {
        same_revision_note(theme)
    } else if let Some(error) = view.diff_error.as_ref() {
        placeholder_err(error, theme)
    } else if let Some(hunk) = view
        .current_hunk
        .as_ref()
        .filter(|hunk| hunk_is_image(hunk))
    {
        image_diff_view(hunk, theme, window, cx)
    } else if let Some(diff) = view.current_diff.clone() {
        if diff.lines.is_empty() {
            placeholder("No textual diff (binary, identical, or empty)", theme)
        } else {
            div()
                .flex_1()
                .min_h_0()
                .h_full()
                .debug_selector(|| "file-history-diff".to_owned())
                .child(read_only_diff("file-history-diff", diff, theme, cx))
                .into_any_element()
        }
    } else if view.comparison.target().is_none() {
        placeholder(
            "Pick a revision above to see its change to this file.",
            theme,
        )
    } else {
        loading_status("Loading diff…", theme)
    };

    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .min_w_0()
        .child(body)
        .into_any_element()
}

fn same_revision_note(theme: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_1()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(4.))
        .px(px(24.))
        .debug_selector(|| "file-history-same-revision".to_owned())
        .child(
            div()
                .text_size(ui_font_size(13.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(theme.fg))
                .child("Same Revision as the Base"),
        )
        .child(
            div()
                .text_size(ui_font_size(12.))
                .text_color(rgb(theme.fg_dim))
                .child("Choose a different revision to compare against the base."),
        )
        .into_any_element()
}
