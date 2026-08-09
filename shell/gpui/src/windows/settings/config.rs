use std::path::Path;

mod model;

use gpui::{
    AnyElement, App, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, div, px, rgb,
};

use super::SettingsView;
use super::shared::{detail_row, feedback_copy_icon_button, row_container, section_title};
use crate::app::theme::Theme;
use crate::ui::icons::glyph;
use model::{JjConfigEntry, JjConfigSection};

const CONFIG_PATH_COPY_ID: &str = "jj-config-copy-path";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct JjConfigSnapshot(model::JjConfigSnapshot);

pub(super) fn load_jj_config_snapshot() -> JjConfigSnapshot {
    JjConfigSnapshot(model::load_jj_config_snapshot())
}

impl SettingsView {
    /// Snapshot injection seam for component tests; an in-flight config load will preserve it.
    pub fn set_jj_config_path(&mut self, path: String, cx: &mut Context<Self>) {
        self.jj_config = Some(JjConfigSnapshot(model::JjConfigSnapshot {
            path,
            sections: Vec::new(),
            error: None,
        }));
        self.jj_config_loading = false;
        cx.notify();
    }
}

pub(super) fn jujutsu_section(
    snapshot: Option<&JjConfigSnapshot>,
    loading: bool,
    recently_copied: Option<&SharedString>,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let mut root = div()
        .debug_selector(|| "settings-jujutsu-section".to_owned())
        .flex()
        .flex_col()
        .w_full()
        .gap(px(16.))
        .child(section_title("Jujutsu", t));

    if loading {
        return root
            .child(status_message("Loading jj config...", t))
            .into_any_element();
    }
    let Some(snapshot) = snapshot else {
        return root
            .child(status_message("jj config has not been loaded.", t))
            .into_any_element();
    };
    let snapshot = &snapshot.0;

    if let Some(error) = snapshot.error.as_ref() {
        return root
            .child(status_message(error.as_str(), t))
            .into_any_element();
    }

    if !snapshot.path.is_empty() {
        root = root.child(config_path_row(
            &snapshot.path,
            recently_copied.is_some_and(|id| id.as_ref() == CONFIG_PATH_COPY_ID),
            t,
            cx,
        ));
    }
    for section in &snapshot.sections {
        root = root.child(config_section(section, t));
    }
    root.into_any_element()
}

fn config_path_row(
    path: &str,
    copied: bool,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    row_container(t)
        .debug_selector(|| "jj-config-path-row".to_owned())
        .py(px(6.))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .font_family(crate::app::fonts::mono())
                .text_size(px(11.))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(path.to_owned())),
        )
        .child(open_button(path.to_owned(), t))
        .child(feedback_copy_icon_button(
            CONFIG_PATH_COPY_ID,
            path.to_owned(),
            copied,
            t,
            cx,
        ))
        .into_any_element()
}

fn config_section(section: &JjConfigSection, t: &Theme) -> AnyElement {
    let mut group = div().flex().flex_col().w_full().gap(px(4.)).child(
        div()
            .w_full()
            .pt(px(4.))
            .text_size(px(11.))
            .text_color(rgb(t.fg_faint))
            .child(SharedString::from(section.name.clone())),
    );
    for entry in &section.entries {
        group = group.child(config_row(entry, t));
    }
    group.into_any_element()
}

fn config_row(entry: &JjConfigEntry, t: &Theme) -> AnyElement {
    detail_row(
        entry_icon(&entry.key),
        SharedString::from(entry.key.clone()),
        SharedString::from(entry.value.clone()),
        12.,
        t.fg_dim,
        t,
    )
    .debug_selector(|| "jj-config-row".to_owned())
    .into_any_element()
}

fn status_message(message: &str, t: &Theme) -> AnyElement {
    div()
        .debug_selector(|| "jj-config-status".to_owned())
        .w_full()
        .px(px(8.))
        .py(px(8.))
        .rounded_sm()
        .bg(rgb(t.row_alt_bg))
        .text_size(px(12.))
        .text_color(rgb(t.fg_dim))
        .child(SharedString::from(message.to_owned()))
        .into_any_element()
}

fn open_button(path: String, t: &Theme) -> AnyElement {
    div()
        .id(SharedString::from("jj-config-open"))
        .px(px(10.))
        .py(px(4.))
        .rounded_md()
        .bg(rgb(t.toggle_inactive_bg))
        .text_size(px(11.))
        .text_color(rgb(t.toggle_inactive_fg))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(t.row_alt_bg)))
        .on_click(move |_: &ClickEvent, _, cx: &mut App| {
            let cwd = Path::new(&path)
                .parent()
                .and_then(Path::to_str)
                .unwrap_or(".");
            if !crate::app::tools::open_in_editor(cwd, &path, cx) {
                cx.open_url(&format!("file://{path}"));
            }
        })
        .child("Open")
        .into_any_element()
}

fn entry_icon(key: &str) -> &'static str {
    match key {
        "name" => glyph::INFO,
        "email" => glyph::TAG,
        "hostname" => glyph::TERMINAL,
        "username" => glyph::INFO,
        "backend" => glyph::PACKAGE,
        "behavior" => glyph::PENCIL_CIRCLE,
        "key" => glyph::GEAR,
        _ if key.contains("command") => glyph::TERMINAL,
        _ if key.contains("pattern") => glyph::SEARCH,
        _ if key.contains("sign") => glyph::PENCIL_CIRCLE,
        _ => glyph::GEAR,
    }
}
