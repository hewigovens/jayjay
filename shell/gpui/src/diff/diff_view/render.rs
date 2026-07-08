use gpui::{
    AnyElement, Context, FontWeight, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled, UniformListScrollHandle, div, px, rgb,
};
use jayjay_core::{DiffProjection, diff::ConflictLineKind};

use super::find_bar::render_find_bar;
use super::header::{
    FileHeaderState, ProjectionHeaderState, file_header, hunk_is_git_lfs, hunk_is_submodule,
};
use super::placeholders::{placeholder, placeholder_card, placeholder_inner};
use super::sbs_body::side_by_side_body;
use super::state::{DetailMode, DiffViewMode, DiffViewState, FindState};
use super::unified_body::unified_body;
use crate::app::theme::theme;
use crate::diff::image_diff::{hunk_is_image, image_diff_view};
use crate::diff::projection;
use crate::diff::svg_diff::{SvgDiffContent, svg_diff_view};
use crate::repo::window::RepoWindow;
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::button;

pub fn diff_view(
    state: DiffViewState<'_>,
    find: FindState<'_>,
    scroll: UniformListScrollHandle,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let t = theme(cx).clone();

    let Some(hunk) = state.hunk else {
        return placeholder("Select a file", &t).into_any_element();
    };

    let is_annotating = matches!(state.detail_mode, DetailMode::Annotate);
    let view_mode = state.view_mode.effective_for_diff(state.file_diff);
    let header = file_header(
        FileHeaderState {
            hunk,
            view_mode,
            projection: ProjectionHeaderState {
                projection: state.effective_projection(),
                active: state.active_projection_preview,
            },
            active_svg_preview: state.active_svg_preview,
            can_render_svg_preview: projection::can_render_svg_preview(hunk),
            is_annotating,
            just_copied: state.path_just_copied,
            html_external_url: state.html_external_url,
        },
        &t,
        cx,
    );

    let query = find
        .query
        .map(|q| q.text())
        .filter(|q| !q.is_empty())
        .map(|q| q.to_owned());

    let body: AnyElement = if is_annotating {
        if state.loading_annotate {
            placeholder_inner("Loading annotations…", &t).into_any_element()
        } else if let Some(lines) = state.annotate_lines.as_ref() {
            if lines.is_empty() {
                placeholder_inner("No annotations available", &t).into_any_element()
            } else {
                crate::diff::annotate_view::annotate_body(
                    hunk.path.clone(),
                    lines.clone(),
                    t.clone(),
                    scroll.clone(),
                )
            }
        } else {
            placeholder_inner("Annotations unavailable", &t).into_any_element()
        }
    } else if hunk_is_image(hunk) {
        image_diff_view(hunk, &t)
    } else if state.active_svg_preview && projection::can_render_svg_preview(hunk) {
        svg_diff_view(
            SvgDiffContent {
                old: state.svg_preview.and_then(|preview| preview.old),
                new: state.svg_preview.and_then(|preview| preview.new),
            },
            hunk.hunk_type,
            &t,
        )
    } else if hunk_is_submodule(hunk) {
        placeholder_card(
            glyph::PACKAGE,
            "Git submodule",
            "This submodule has working-copy changes, but JayJay does not render an inline text diff for submodule contents. Open or commit the submodule in its own repository.",
            &t,
        )
        .into_any_element()
    } else if hunk_is_git_lfs(hunk) {
        placeholder_card(
            glyph::HARD_DRIVE,
            "Git LFS-backed file",
            "This file is tracked through Git LFS. JayJay does not render an inline text diff between the committed pointer and the local binary object.",
            &t,
        )
        .into_any_element()
    } else if hunk.is_content_free_rename() {
        placeholder_card(
            glyph::ARROW_CIRCLE_RIGHT,
            "No content changes",
            "This file was renamed; its contents are identical.",
            &t,
        )
        .into_any_element()
    } else {
        match (state.file_diff, view_mode) {
            (None, _) => placeholder_inner("Loading diff…", &t).into_any_element(),
            (Some(fd), _) if fd.lines.is_empty() => {
                placeholder_inner("No textual diff (binary, identical, or empty)", &t)
                    .into_any_element()
            }
            (Some(fd), DiffViewMode::Unified) => unified_body(
                fd,
                t.clone(),
                query.clone(),
                scroll.clone(),
                state.unified_bounds.clone(),
                &state.wrap_cache,
                cx,
            ),
            (Some(fd), DiffViewMode::SideBySide) => side_by_side_body(
                fd,
                t.clone(),
                query.clone(),
                scroll.clone(),
                state.sbs_old_bounds.clone(),
                state.sbs_new_bounds.clone(),
                &state.wrap_cache,
                cx,
            ),
        }
    };

    let find_bar = find
        .query
        .map(|q| render_find_bar(q, find.match_count, find.match_current, &t, cx));
    let show_conflict_bar = state.can_resolve_conflict
        && state.file_diff.is_some_and(|fd| {
            fd.lines
                .iter()
                .any(|line| line.conflict_kind != ConflictLineKind::None)
        });
    let projection_banner = state.effective_projection().and_then(|projection| {
        projection::shows_banner(projection, state.active_projection_preview)
            .then(|| render_projection_banner(projection, &t))
    });

    let mut root = div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .bg(rgb(t.detail_bg));
    if show_conflict_bar {
        root = root.child(conflict_banner(&t, cx));
    }
    root = root.child(header);
    if let Some(bar) = find_bar {
        root = root.child(bar);
    }
    if let Some(bar) = projection_banner {
        root = root.child(bar);
    }
    root.child(div().flex().flex_col().flex_1().min_h_0().child(body))
        .into_any_element()
}

fn render_projection_banner(
    projection: &DiffProjection,
    t: &crate::app::theme::Theme,
) -> AnyElement {
    let has_diagnostics = !projection.diagnostics.is_empty();
    let (bg, fg, icon) = if has_diagnostics {
        (t.tag_conflict_bg, t.tag_conflict_fg, glyph::WARNING)
    } else {
        (
            t.toggle_active_bg,
            t.toggle_active_fg,
            projection::icon(Some(projection)),
        )
    };
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(12.))
        .py(px(6.))
        .bg(rgb(bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(icons::icon(icon, 14., fg))
        .child(
            div()
                .text_size(px(12.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(fg))
                .child(projection::title(projection)),
        );
    if has_diagnostics {
        row = row.child(div().flex_1()).child(
            div()
                .min_w_0()
                .truncate()
                .text_size(px(11.))
                .text_color(rgb(fg))
                .child(projection.diagnostics.join("; ")),
        );
    }
    row.into_any_element()
}

fn conflict_banner(t: &crate::app::theme::Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    let cfg = crate::app::config::current(cx);
    let editor_id = cfg.tools.external_editor.as_str();
    let editor_title = crate::app::tools::editor_title(cx);
    let merge_tool = matches!(editor_id, "vscode" | "zed").then(|| editor_id.to_owned());
    let mut bar = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(12.))
        .py(px(7.))
        .bg(rgb(t.tag_conflict_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(icons::icon(glyph::WARNING, 14., t.tag_conflict_fg))
        .child(
            div()
                .text_size(px(12.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(t.tag_conflict_fg))
                .child("Conflict"),
        )
        .child(div().flex_1())
        .child(
            button("conflict-use-ours", "Use Ours", t, false).on_click(cx.listener(
                |view, _, _, cx| {
                    view.resolve_selected_file_with_tool(":ours".to_owned(), cx);
                },
            )),
        )
        .child(
            button("conflict-use-theirs", "Use Theirs", t, false).on_click(cx.listener(
                |view, _, _, cx| {
                    view.resolve_selected_file_with_tool(":theirs".to_owned(), cx);
                },
            )),
        );

    if let Some(tool) = merge_tool {
        bar = bar.child(
            button(
                "conflict-resolve-editor",
                format!("Resolve in {editor_title}"),
                t,
                true,
            )
            .on_click(cx.listener(move |view, _, _, cx| {
                view.resolve_selected_file_with_tool(tool.clone(), cx);
            })),
        );
    } else {
        bar = bar.child(
            button(
                "conflict-open-editor",
                format!("Open in {editor_title}"),
                t,
                true,
            )
            .on_click(cx.listener(|view, _, _, cx| {
                view.open_selected_file_in_editor(cx);
            })),
        );
    }

    bar.into_any_element()
}
