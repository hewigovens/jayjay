use gpui::{
    AnyElement, AppContext, ClickEvent, Context, Entity, FontWeight, InteractiveElement,
    IntoElement, ParentElement, SharedString, StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::{FileDiffStats, OverviewChange};

use super::OverviewView;
use super::canvas::description_color;
use crate::app::theme::{Theme, ui_font_size};
use crate::repo::window::{compact_id, format_relative};
use crate::ui::icons::glyph;
use crate::ui::primitives::{capsule, icon_button, small_button};
use crate::ui::text_area::{TextArea, Tint};

pub(super) struct PanelText {
    commit_id: String,
    title: Entity<TextArea>,
    ids: Entity<TextArea>,
    body: Option<Entity<TextArea>>,
}

impl PanelText {
    fn new(change: &OverviewChange, cx: &mut Context<OverviewView>) -> Self {
        let body = change
            .full_description
            .split_once('\n')
            .map(|(_, rest)| rest.trim().to_owned())
            .filter(|body| !body.is_empty());
        let change_id = compact_id(&change.change_id);
        let commit_start = change_id.len() + 1;
        let tints = vec![
            Tint {
                range: 0..change.change_id.short_len as usize,
                color: |theme| theme.change_id_prefix,
            },
            Tint {
                range: commit_start..commit_start + change.commit_id.short_len as usize,
                color: |theme| theme.commit_id_prefix,
            },
        ];
        let ids = format!(
            "{change_id} {} · {}",
            compact_id(&change.commit_id),
            format_relative(change.timestamp_millis)
        );
        let title = change.title().to_owned();
        Self {
            commit_id: change.commit_id.id.clone(),
            title: cx.new(|cx| TextArea::label(title, cx)),
            ids: cx.new(|cx| TextArea::label(ids, cx).with_tints(tints)),
            body: body.map(|body| cx.new(|cx| TextArea::label(body, cx))),
        }
    }
}

impl OverviewView {
    pub(super) fn render_change_panel(
        &mut self,
        lane_ix: usize,
        change: &OverviewChange,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if self
            .selection
            .text
            .as_ref()
            .is_none_or(|text| text.commit_id != change.commit_id.id)
        {
            self.selection.text = Some(PanelText::new(change, cx));
        }
        let Some(text) = self.selection.text.as_ref() else {
            return div().into_any_element();
        };
        let commit_id = change.commit_id.id.clone();
        let title = div()
            .text_color(rgb(description_color(&change.description, t)))
            .child(text.title.clone());
        let ids = div()
            .font_family(crate::app::fonts::mono())
            .text_size(ui_font_size(11.))
            .text_color(rgb(t.fg_dim))
            .child(text.ids.clone())
            .into_any_element();
        let chips = (!change.workspaces.is_empty() || change.has_conflict).then(|| {
            div()
                .flex()
                .flex_row()
                .gap(px(4.))
                .children(
                    change
                        .workspaces
                        .iter()
                        .map(|name| capsule(format!("{name}@"), t.tag_wc_bg, t.tag_wc_fg, 9.)),
                )
                .children(
                    change
                        .has_conflict
                        .then(|| capsule("conflict", t.tag_conflict_bg, t.tag_conflict_fg, 9.)),
                )
                .into_any_element()
        });
        let body = text.body.clone().map(|body| {
            div()
                .id("overview-panel-body")
                .debug_selector(|| "overview-panel-body".to_owned())
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.fg_dim))
                .child(body)
                .into_any_element()
        });
        let header = panel_header(title, t, cx, |view, cx| {
            view.clear_change();
            cx.notify();
        });
        let actions = panel_actions([show_in_graph_button(t, cx, move |view, cx| {
            view.show_in_graph(lane_ix, Some(commit_id.clone()), cx);
        })]);
        let files = file_list(self.selection.files.as_deref(), t);
        panel_frame(
            "overview-change-panel",
            t,
            [
                Some(header),
                Some(ids),
                chips,
                Some(actions),
                body,
                Some(files),
            ]
            .into_iter()
            .flatten(),
        )
    }
}

pub(super) fn panel_frame(
    id: &'static str,
    t: &Theme,
    sections: impl IntoIterator<Item = AnyElement>,
) -> AnyElement {
    div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .flex_none()
        .w(px(400.))
        .h_full()
        .overflow_y_scroll()
        .border_l_1()
        .border_color(rgb(t.border))
        .bg(rgb(t.detail_bg))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(14.))
                .p(px(16.))
                .children(sections),
        )
        .into_any_element()
}

pub(super) fn panel_header(
    title: gpui::Div,
    t: &Theme,
    cx: &mut Context<OverviewView>,
    close: impl Fn(&mut OverviewView, &mut Context<OverviewView>) + 'static,
) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_start()
        .gap(px(8.))
        .child(
            title
                .flex_1()
                .min_w_0()
                .text_size(ui_font_size(13.))
                .font_weight(FontWeight::SEMIBOLD),
        )
        .child(
            icon_button("overview-panel-close", glyph::X, 11., 20., 20., t.fg_dim, t)
                .on_click(cx.listener(move |view, _: &ClickEvent, _, cx| close(view, cx))),
        )
        .into_any_element()
}

pub(super) fn show_in_graph_button(
    t: &Theme,
    cx: &mut Context<OverviewView>,
    show: impl Fn(&mut OverviewView, &mut Context<OverviewView>) + 'static,
) -> AnyElement {
    small_button("overview-panel-show-in-graph", "Show in Graph", t)
        .on_click(cx.listener(move |view, _: &ClickEvent, _, cx| show(view, cx)))
        .into_any_element()
}

pub(super) fn panel_actions(buttons: impl IntoIterator<Item = AnyElement>) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .gap(px(6.))
        .children(buttons)
        .into_any_element()
}

pub(super) fn panel_meta(parts: impl IntoIterator<Item = (String, u32)>, t: &Theme) -> AnyElement {
    let mut row = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(px(5.))
        .text_size(ui_font_size(11.));
    for (ix, (text, color)) in parts.into_iter().enumerate() {
        if ix > 0 {
            row = row.child(div().text_color(rgb(t.fg_faint)).child("·"));
        }
        row = row.child(div().text_color(rgb(color)).child(SharedString::from(text)));
    }
    row.into_any_element()
}

pub(super) fn note(text: impl Into<SharedString>, color: u32) -> gpui::Div {
    div()
        .text_size(ui_font_size(11.))
        .text_color(rgb(color))
        .child(text.into())
}

pub(super) fn section(
    title: &'static str,
    t: &Theme,
    rows: impl IntoIterator<Item = AnyElement>,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(section_title(title, t))
        .children(rows)
        .into_any_element()
}

fn section_title(title: impl Into<SharedString>, t: &Theme) -> gpui::Div {
    div()
        .text_size(ui_font_size(10.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(t.fg_faint))
        .child(title.into())
}

pub(super) fn file_list(files: Option<&[FileDiffStats]>, t: &Theme) -> AnyElement {
    let note = |text: &'static str| {
        div()
            .text_size(ui_font_size(11.))
            .text_color(rgb(t.fg_faint))
            .child(text)
            .into_any_element()
    };
    let Some(files) = files else {
        return note("Loading files…");
    };
    if files.is_empty() {
        return note("No file changes");
    }
    let count = files.len();
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(section_title(
            format!("{count} FILE{}", if count == 1 { "" } else { "S" }),
            t,
        ))
        .children(files.iter().map(|file| {
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(ui_font_size(11.))
                        .text_color(rgb(t.fg))
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .text_ellipsis_middle()
                        .child(SharedString::from(file.path.clone())),
                )
                .children(
                    (file.insertions > 0)
                        .then(|| stat(format!("+{}", file.insertions), t.file_added_color)),
                )
                .children(
                    (file.deletions > 0)
                        .then(|| stat(format!("−{}", file.deletions), t.file_removed_color)),
                )
        }))
        .into_any_element()
}

fn stat(text: String, color: u32) -> impl IntoElement {
    div()
        .flex_none()
        .font_family(crate::app::fonts::mono())
        .text_size(ui_font_size(10.))
        .text_color(rgb(color))
        .child(SharedString::from(text))
}
