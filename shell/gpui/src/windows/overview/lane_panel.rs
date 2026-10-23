use gpui::{
    AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::overview::OverviewSnapshot;
use jayjay_core::{OverviewBase, OverviewBaseKind, OverviewChange, OverviewLane};

use super::OverviewView;
use super::canvas::{description_color, workspace_label};
use super::menu::OverviewAction;
use super::panel::{
    note, panel_actions, panel_frame, panel_header, panel_meta, section, show_in_graph_button,
};
use crate::app::theme::{Theme, ui_font_size};
use crate::repo::window::{compact_id, format_relative, id_cell};
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::{capsule, small_button};

impl OverviewView {
    pub(super) fn render_lane_panel(
        &mut self,
        lane_ix: usize,
        lane: &OverviewLane,
        snapshot: &OverviewSnapshot,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let trunk_name = snapshot.trunk_name();
        let title = div()
            .text_color(rgb(description_color(&lane.head().description, t)))
            .child(SharedString::from(lane.title().to_owned()));
        let header = panel_header(title, t, cx, |view, cx| {
            view.selection.lane_panel = false;
            cx.notify();
        });
        let count = lane.changes.len();
        let meta = panel_meta(
            [
                (jayjay_i18n::repo::change_count(count), t.fg_dim),
                (
                    format!("updated {}", format_relative(lane.latest_timestamp_millis)),
                    t.fg_dim,
                ),
                (
                    base_sentence(&lane.base, trunk_name),
                    if lane.is_behind_trunk() {
                        t.warning_fg
                    } else {
                        t.fg_dim
                    },
                ),
            ],
            t,
        );
        let sections = [
            Some(header),
            Some(meta),
            Some(self.lane_actions(lane_ix, lane, trunk_name, t, cx)),
            attention_list(lane, t),
            self.workspace_section(lane, snapshot, t, cx),
            bookmark_section(lane, t),
            Some(self.change_section(lane_ix, &lane.changes, t, cx)),
        ];
        panel_frame("overview-lane-panel", t, sections.into_iter().flatten())
    }

    fn lane_actions(
        &self,
        lane_ix: usize,
        lane: &OverviewLane,
        trunk_name: &str,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let show = show_in_graph_button(t, cx, move |view, cx| {
            view.show_in_graph(lane_ix, None, cx);
        });
        let rebase = lane
            .changes
            .last()
            .filter(|_| lane.is_behind_trunk())
            .map(|root| {
                let action = OverviewAction::RebaseOntoTrunk(root.commit_id.id.clone());
                small_button(
                    "overview-lane-rebase",
                    jayjay_i18n::overview::rebase_onto(trunk_name),
                    t,
                )
                .on_click(cx.listener(move |view, _: &ClickEvent, _, cx| {
                    view.dispatch(action.clone(), cx);
                }))
                .into_any_element()
            });
        panel_actions(std::iter::once(show).chain(rebase))
    }

    fn workspace_section(
        &self,
        lane: &OverviewLane,
        snapshot: &OverviewSnapshot,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if lane.workspaces.is_empty() {
            return None;
        }
        let rows = lane.workspaces.iter().enumerate().map(|(ix, workspace)| {
            let position = match workspace.changes_above {
                0 => jayjay_i18n::overview::at_the_head(),
                above => jayjay_i18n::overview::below_the_head(above),
            };
            let open = snapshot
                .workspace(&workspace.name)
                .filter(|info| info.is_path_resolved && !workspace.is_current)
                .map(|info| {
                    let action = OverviewAction::OpenWorkspace(info.path.clone());
                    small_button(
                        format!("overview-lane-open-{ix}"),
                        jayjay_i18n::common::open(),
                        t,
                    )
                    .on_click(cx.listener(
                        move |view, _: &ClickEvent, _, cx| {
                            view.dispatch(action.clone(), cx);
                        },
                    ))
                });
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(capsule(
                    workspace_label(workspace),
                    t.tag_wc_bg,
                    t.tag_wc_fg,
                    10.,
                ))
                .child(div().flex_1().min_w_0().child(note(position, t.fg_dim)))
                .children(open)
                .into_any_element()
        });
        Some(section(
            jayjay_i18n::overview::workspaces(),
            t,
            rows.collect::<Vec<_>>(),
        ))
    }

    fn change_section(
        &self,
        lane_ix: usize,
        changes: &[OverviewChange],
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let rows = changes.iter().map(|change| {
            let commit_id = change.commit_id.id.clone();
            div()
                .id(SharedString::from(format!(
                    "overview-lane-change-{commit_id}"
                )))
                .flex()
                .flex_row()
                .items_start()
                .gap(px(8.))
                .mx(px(-6.))
                .px(px(6.))
                .py(px(3.))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(|row| row.bg(rgb(t.row_alt_bg)))
                .child(id_cell(
                    &compact_id(&change.change_id),
                    change.change_id.short_len,
                    t.change_id_prefix,
                    11.,
                    t,
                ))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(ui_font_size(12.))
                        .text_color(rgb(description_color(&change.description, t)))
                        .child(SharedString::from(change.title().to_owned())),
                )
                .on_click(cx.listener(move |view, _: &ClickEvent, _, cx| {
                    view.select_change(lane_ix, commit_id.clone(), cx);
                }))
                .into_any_element()
        });
        section(
            jayjay_i18n::overview::changes(),
            t,
            rows.collect::<Vec<_>>(),
        )
    }
}

fn attention_list(lane: &OverviewLane, t: &Theme) -> Option<AnyElement> {
    (!lane.attention.is_empty()).then(|| {
        div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .children(lane.attention.iter().map(|sentence| {
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.))
                    .child(icons::icon(glyph::WARNING, 11., t.warning_fg))
                    .child(note(sentence.clone(), t.warning_fg))
            }))
            .into_any_element()
    })
}

fn bookmark_section(lane: &OverviewLane, t: &Theme) -> Option<AnyElement> {
    let mut bookmarks = lane
        .changes
        .iter()
        .flat_map(|change| &change.bookmarks)
        .peekable();
    bookmarks.peek()?;
    let chips = div().flex().flex_row().flex_wrap().gap(px(4.)).children(
        bookmarks
            .map(|bookmark| capsule(bookmark.clone(), t.tag_bookmark_bg, t.tag_bookmark_fg, 10.)),
    );
    Some(section(
        jayjay_i18n::overview::bookmarks(),
        t,
        [chips.into_any_element()],
    ))
}

fn base_sentence(base: &OverviewBase, trunk_name: &str) -> String {
    let short = || base.change_id.unique_prefix();
    match base.kind {
        OverviewBaseKind::Trunk => jayjay_i18n::overview::on_trunk(trunk_name),
        OverviewBaseKind::OlderTrunk => {
            jayjay_i18n::overview::commits_behind(base.behind_trunk, trunk_name)
        }
        OverviewBaseKind::Mutable => match base.description.as_str() {
            "" => jayjay_i18n::overview::forked_from(&short()),
            description => jayjay_i18n::overview::forked_from(description),
        },
        OverviewBaseKind::Other => jayjay_i18n::overview::on_off_trunk(
            &base.bookmarks.first().cloned().unwrap_or_else(short),
            trunk_name,
        ),
    }
}
