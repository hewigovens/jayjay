use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb, rgba,
};

use crate::app::theme::{Theme, ui_font_size, with_alpha};
use crate::repo::window::{PrImportState, RepoWindow};
use crate::ui::icons::glyph;
use crate::ui::overlay::{overlay_actions, overlay_card, overlay_header, overlay_layer};
use crate::ui::primitives::button;

pub(crate) fn pr_import_overlay(
    state: &PrImportState,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let mut card = overlay_card(t, 440.).child(overlay_header(
        glyph::GIT_MERGE,
        t.fg_dim,
        "New Workspace from Pull Request",
        "",
        t,
    ));
    if let Some(preview) = state.preview.as_ref() {
        card = card.child(pr_import_preview(state, preview, t, cx));
    } else {
        card = card.child(
            div()
                .id("pr-import-url")
                .debug_selector(|| "pr-import-url".to_owned())
                .child(state.url_input.clone()),
        );
    }
    if let Some(error) = state.error.as_ref() {
        card = card.child(
            div()
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.error_fg))
                .whitespace_normal()
                .child(SharedString::from(error.clone())),
        );
    }
    let (primary_label, primary_selector) = pr_import_primary(state);
    let primary = button("pr-import-primary", primary_label, t, true);
    let primary = if state.can_submit(cx) {
        primary
            .debug_selector(|| primary_selector.to_owned())
            .on_click(cx.listener(|view, _, _, cx| view.submit_pr_import(cx)))
    } else {
        primary.opacity(0.45)
    };
    overlay_layer()
        .child(
            card.child(overlay_actions(
                button("pr-import-cancel", "Cancel", t, false)
                    .debug_selector(|| "pr-import-cancel".to_owned())
                    .on_click(cx.listener(|view, _, _, cx| view.cancel_pr_import(cx))),
                primary,
            )),
        )
        .into_any_element()
}

fn pr_import_primary(state: &PrImportState) -> (&'static str, &'static str) {
    if state.preview.is_none() {
        let label = if state.resolving {
            "Resolving…"
        } else {
            "Resolve"
        };
        return (label, "pr-import-resolve");
    }
    if state.existing_workspace().is_some() {
        return ("Open Workspace", "pr-import-open");
    }
    let busy = state.importing || state.resolving;
    let label = if busy {
        "Creating…"
    } else {
        "Fetch & Create Workspace"
    };
    (label, "pr-import-create")
}

fn pr_import_preview(
    state: &PrImportState,
    preview: &jayjay_core::PullRequestImportPreview,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> gpui::Div {
    let pr = &preview.pull_request;
    let mut body = div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.))
                .child(pr_state_badge(pr.state, t))
                .child(
                    div()
                        .flex_none()
                        .text_size(ui_font_size(12.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(format!("#{}", pr.number)),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_size(ui_font_size(12.))
                        .child(SharedString::from(pr.title.clone())),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(summary_row(
                    "Repository",
                    format!("{} · {}", pr.host, pr.base_repo),
                    None,
                    t,
                ))
                .child(summary_row(
                    "Remote",
                    preview.remote.name.clone(),
                    Some(if preview.remote.exists {
                        "existing remote"
                    } else {
                        "will be added"
                    }),
                    t,
                ))
                .child(summary_row(
                    "Bookmark",
                    preview.remote.bookmark.clone(),
                    None,
                    t,
                ))
                .child(summary_row(
                    "Head",
                    preview.head_commit_id.chars().take(8).collect::<String>(),
                    None,
                    t,
                )),
        );
    if let Some(existing) = state.existing_workspace() {
        body = body.child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(summary_row(
                    "Workspace",
                    existing.name.clone(),
                    Some("already imported"),
                    t,
                ))
                .child(
                    button("pr-import-again", "Import Again", t, false)
                        .debug_selector(|| "pr-import-again".to_owned())
                        .on_click(cx.listener(|view, _, _, cx| {
                            if let Some(state) = view.pr_import.as_mut() {
                                state.import_again = true;
                                cx.notify();
                            }
                        })),
                ),
        );
    } else {
        body = body
            .child(
                div()
                    .id("pr-import-name")
                    .debug_selector(|| "pr-import-name".to_owned())
                    .child(state.name_input.clone()),
            )
            .child(
                div()
                    .id("pr-import-dest")
                    .debug_selector(|| "pr-import-dest".to_owned())
                    .child(state.dest_input.clone()),
            );
    }
    body
}

fn summary_row(
    label: &'static str,
    value: String,
    note: Option<&'static str>,
    t: &Theme,
) -> gpui::Div {
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .w_full()
        .min_w_0()
        .child(
            div()
                .flex_none()
                .w(px(70.))
                .text_size(ui_font_size(11.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(t.fg_dim))
                .child(label),
        )
        .child(
            div()
                .min_w_0()
                .truncate()
                .font_family(crate::app::fonts::mono())
                .text_size(ui_font_size(11.))
                .child(SharedString::from(value)),
        );
    if let Some(note) = note {
        row = row.child(
            div()
                .flex_none()
                .text_size(ui_font_size(10.))
                .text_color(rgb(t.fg_faint))
                .child(note),
        );
    }
    row
}

fn pr_state_badge(state: jayjay_core::PrState, t: &Theme) -> gpui::Div {
    let (label, color) = match state {
        jayjay_core::PrState::Open => ("Open", t.success_fg),
        jayjay_core::PrState::Closed => ("Closed", t.error_fg),
        jayjay_core::PrState::Merged => ("Merged", t.note_accent),
    };
    div()
        .flex_none()
        .px(px(5.))
        .py(px(1.))
        .rounded_full()
        .bg(rgba(with_alpha(color, if t.is_dark { 0x2a } else { 0x22 })))
        .text_size(ui_font_size(9.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(color))
        .child(label)
}
