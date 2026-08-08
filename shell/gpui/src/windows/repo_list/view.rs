use std::path::{Path, PathBuf};

use gpui::{
    Div, FontWeight, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use crate::app::config;
use crate::app::repositories;
use crate::app::theme::Theme;
use crate::ui::icons;
use crate::ui::logo::Logo;
use crate::ui::primitives::button;

pub(super) fn header(logo: &Logo, t: &Theme) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(12.))
        .child(logo.image(80.))
        .child(
            div()
                .text_size(px(28.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(t.fg))
                .child("JayJay"),
        )
        .child(
            div()
                .text_size(px(14.))
                .text_color(rgb(t.fg_dim))
                .child("A native GUI for Jujutsu"),
        )
        .child(
            button("repo-list-open", "Open Repository...", t, true)
                .debug_selector(|| "repo-list-open".to_owned())
                .on_click(|_, _, cx| crate::app::menus::prompt_open_repository(cx)),
        )
}

pub(super) fn repository_sections(
    pinned: Vec<String>,
    recent: Vec<String>,
    t: &Theme,
) -> impl IntoElement {
    let mut sections = div().flex().flex_col().gap(px(18.)).px(px(30.)).py(px(18.));
    if !pinned.is_empty() {
        sections = sections.child(repository_section("Pinned", pinned, RowKind::Pinned, t));
    }
    if !recent.is_empty() {
        sections = sections.child(repository_section(
            "Recent Repositories",
            recent,
            RowKind::Recent,
            t,
        ));
    }

    div()
        .id("repo-list-scroll")
        .flex()
        .flex_1()
        .min_h_0()
        .flex_col()
        .overflow_y_scroll()
        .scrollbar_width(px(0.))
        .child(sections)
}

#[derive(Clone, Copy)]
enum RowKind {
    Pinned,
    Recent,
}

fn repository_section(title: &'static str, paths: Vec<String>, kind: RowKind, t: &Theme) -> Div {
    let mut rows = div().flex().flex_col().gap(px(10.));
    for (index, path) in paths.into_iter().enumerate() {
        rows = rows.child(repository_row(index, path, kind, t));
    }

    let mut header = div().flex().items_center().pb(px(2.)).child(
        div()
            .flex_1()
            .text_size(px(13.))
            .font_weight(FontWeight::SEMIBOLD)
            .child(title),
    );
    if matches!(kind, RowKind::Recent) {
        header = header.child(
            button("repo-list-clear", "Clear", t, false)
                .debug_selector(|| "repo-list-clear".to_owned())
                .on_click(|_, _, cx| config::update(cx, |cfg| cfg.clear_recent_repos())),
        );
    }
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(header)
        .child(rows)
}

fn repository_row(index: usize, path: String, kind: RowKind, t: &Theme) -> impl IntoElement {
    let name = repositories::repository_name(&path);
    let prefix = match kind {
        RowKind::Pinned => "repo-list-pinned",
        RowKind::Recent => "repo-list",
    };
    let row_id = SharedString::from(format!("{prefix}-row-{index}"));
    let open_path = path.clone();
    let pin_path = path.clone();

    let mut row = div()
        .id(row_id.clone())
        .debug_selector(move || row_id.to_string())
        .flex()
        .items_center()
        .gap(px(8.))
        .p(px(8.))
        .rounded_lg()
        .bg(rgb(t.row_alt_bg))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(t.selected_bg)))
        .on_click(move |_, _, cx| {
            crate::repo::open_repo_window(PathBuf::from(&open_path), cx);
        })
        .child(
            div()
                .flex()
                .flex_1()
                .min_w_0()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .truncate()
                        .text_size(px(12.))
                        .font_weight(FontWeight::MEDIUM)
                        .child(name),
                )
                .child(
                    div()
                        .truncate()
                        .text_size(px(10.))
                        .text_color(rgb(t.fg_dim))
                        .child(path.clone()),
                ),
        );

    let pin_id = SharedString::from(format!("{prefix}-pin-{index}"));
    let pinned = matches!(kind, RowKind::Pinned);
    row = row.child(
        row_action(
            pin_id,
            if pinned {
                icons::glyph::PIN_OFF
            } else {
                icons::glyph::PIN
            },
            t,
        )
        .on_click(move |_, _, cx| {
            cx.stop_propagation();
            repositories::set_pinned(cx, Path::new(&pin_path), !pinned);
        }),
    );

    if matches!(kind, RowKind::Recent) {
        let remove_path = path;
        let remove_id = SharedString::from(format!("repo-list-remove-{index}"));
        row = row.child(row_action(remove_id, icons::glyph::X_CIRCLE, t).on_click(
            move |_, _, cx| {
                cx.stop_propagation();
                config::update(cx, |cfg| cfg.remove_recent_repo(&remove_path));
            },
        ));
    }
    row
}

fn row_action(id: SharedString, glyph: &'static str, t: &Theme) -> gpui::Stateful<Div> {
    div()
        .id(id.clone())
        .debug_selector(move || id.to_string())
        .flex()
        .items_center()
        .justify_center()
        .w(px(24.))
        .h(px(24.))
        .rounded_full()
        .cursor_pointer()
        .hover(|style| style.bg(rgb(t.selected_bg)))
        .child(icons::icon(glyph, 14., t.fg_faint))
}
