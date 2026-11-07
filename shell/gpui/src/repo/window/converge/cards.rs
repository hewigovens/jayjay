use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, Context, Div, FontWeight, InteractiveElement, IntoElement, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::ConvergeVersion;
use jayjay_core::converge::{ConvergeItem, ConvergeItemStatus, ConvergeReview};

use super::body::ConvergeBody;
use super::render::{caption, card_frame, link, loaded};
use crate::app::actions::{MergeUseLeftHunk, MergeUseRightHunk};
use crate::app::fonts::CodeText as _;
use crate::app::key_caps::KeyCaps;
use crate::app::theme::{Theme, ui_font_size};
use crate::diff::{file_status, read_only_diff};
use crate::repo::window::{RepoWindow, first_line, format_when};
use crate::ui::icons::{glyph, icon};

const MAX_DIFF_LINES: usize = 16;

impl ConvergeBody<'_> {
    pub(super) fn choice_card(
        &self,
        item: ConvergeItem,
        cx: &mut Context<RepoWindow>,
    ) -> AnyElement {
        let (review, t) = (self.review, self.t);
        let focused = review.focused() == Some(item);
        let status = review.status(item);
        let status_color = match status {
            ConvergeItemStatus::FollowsParents => t.fg_dim,
            status if status.needs_attention() => t.warning_fg,
            _ => t.success_fg,
        };
        let changed_files = self
            .sheet
            .contents
            .files
            .as_ref()
            .and_then(|files| files.as_ref().ok())
            .map(|files| files.len());
        let shortcuts: [&dyn gpui::Action; 2] = [&MergeUseLeftHunk, &MergeUseRightHunk];
        let mut actions: Vec<_> = review
            .versions()
            .iter()
            .enumerate()
            .map(|(index, version)| {
                let shortcut = (shortcuts.get(index))
                    .filter(|_| focused)
                    .map(|action| KeyCaps::label(*action, cx));
                self.use_link(item, version, shortcut, cx)
            })
            .collect();
        actions.extend(self.extra_actions(item, cx));
        let content = match (item, &self.sheet.description) {
            (ConvergeItem::Contents, _) => self.contents(cx),
            (ConvergeItem::Description, Some(editor)) => {
                div().p(px(8.)).child(editor.clone()).into_any_element()
            }
            _ => self.values(item),
        };
        card_frame(t)
            .id(("converge-card", item as usize))
            .debug_selector(move || format!("converge-card-{}", item.slug()))
            .overflow_hidden()
            .when(focused, |card| card.border_color(rgb(t.selected_accent)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .px(px(10.))
                    .py(px(7.))
                    .child(title(item))
                    .child(
                        div()
                            .text_size(ui_font_size(11.))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(status_color))
                            .child(status.label()),
                    )
                    .child(div().flex_1())
                    .children(
                        review
                            .note(item, changed_files)
                            .map(|note| caption(note, t)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(14.))
                    .px(px(10.))
                    .py(px(5.))
                    .border_t_1()
                    .border_b_1()
                    .border_color(rgb(t.row_border))
                    .children(actions),
            )
            .child(content)
            .when(self.sheet.is_interactive(), |card| {
                card.on_click(cx.listener(move |view, _, _, cx| {
                    view.update_converge_review(cx, |review| review.focus(item));
                }))
            })
            .into_any_element()
    }

    fn use_link(
        &self,
        item: ConvergeItem,
        version: &ConvergeVersion,
        shortcut: Option<String>,
        cx: &mut Context<RepoWindow>,
    ) -> AnyElement {
        let t = self.t;
        let letter = self.review.letter(&version.commit_id);
        let selected = self.review.selection(item) == Some(version.commit_id.as_str());
        let commit_id = version.commit_id.clone();
        link(
            format!("converge-use-{}-{letter}", item.slug()),
            jayjay_i18n::converge::use_version(&letter.to_string()),
            self.sheet.is_interactive(),
            t,
            cx,
            move |view, cx| {
                view.update_converge_review(cx, |review| review.use_version(&commit_id, item));
            },
        )
        .when(selected, |link| {
            link.font_weight(FontWeight::SEMIBOLD)
                .child(icon(glyph::CHECK, 10., t.selected_accent))
        })
        .children(shortcut.map(|shortcut| div().text_color(rgb(t.fg_faint)).child(shortcut)))
        .into_any_element()
    }

    fn extra_actions(&self, item: ConvergeItem, cx: &mut Context<RepoWindow>) -> Vec<AnyElement> {
        let (review, t) = (self.review, self.t);
        let versions = review.versions();
        match item {
            ConvergeItem::Description => vec![
                link(
                    "converge-edit-description",
                    jayjay_i18n::converge::edit(),
                    self.sheet.is_interactive() && review.custom_description().is_none(),
                    t,
                    cx,
                    |view, cx| view.update_converge_review(cx, ConvergeReview::edit_description),
                )
                .into_any_element(),
            ],
            ConvergeItem::Contents if versions.len() > 2 => {
                let from = review.letter(&versions[0].commit_id);
                let mut targets =
                    vec![caption(jayjay_i18n::converge::compare_with(), t).into_any_element()];
                targets.extend(versions[1..].iter().map(|version| {
                    let letter = review.letter(&version.commit_id);
                    let active = self.sheet.contents.target.as_ref() == Some(&version.commit_id);
                    let commit_id = version.commit_id.clone();
                    link(
                        format!("converge-compare-{letter}"),
                        format!("{from} → {letter}"),
                        self.sheet.is_interactive() && !active,
                        t,
                        cx,
                        move |view, cx| view.compare_converge_with(commit_id.clone(), cx),
                    )
                    .when(active, |link| link.opacity(1.).text_color(rgb(t.fg)))
                    .into_any_element()
                }));
                targets
            }
            _ => Vec::new(),
        }
    }

    fn values(&self, item: ConvergeItem) -> AnyElement {
        let (review, t) = (self.review, self.t);
        let tinted = review.versions().len() == 2;
        div()
            .flex()
            .flex_col()
            .children(
                review
                    .versions()
                    .iter()
                    .enumerate()
                    .map(|(index, version)| {
                        let tint = if index == 0 {
                            t.diff_removed_bg
                        } else {
                            t.diff_added_bg
                        };
                        div()
                            .flex()
                            .items_baseline()
                            .gap(px(10.))
                            .px(px(10.))
                            .py(px(5.))
                            .when(tinted, |row| row.bg(rgb(tint)))
                            .child(
                                div()
                                    .code_text(11., t)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(t.fg_dim))
                                    .child(review.letter(&version.commit_id).to_string()),
                            )
                            .child(
                                value_text(version_value(version, item), item, t)
                                    .flex_1()
                                    .min_w_0()
                                    .whitespace_normal(),
                            )
                    }),
            )
            .into_any_element()
    }

    fn contents(&self, cx: &mut Context<RepoWindow>) -> AnyElement {
        let t = self.t;
        let contents = &self.sheet.contents;
        let files = loaded(&contents.files, t, |files| {
            div()
                .flex()
                .flex_col()
                .children(files.iter().map(|file| {
                    let path = file.path.clone();
                    let caret = if contents.selected_path.as_ref() == Some(&path) {
                        glyph::CARET_DOWN
                    } else {
                        glyph::CARET_RIGHT
                    };
                    let selector = format!("converge-file-{path}");
                    div()
                        .id(SharedString::from(selector.clone()))
                        .debug_selector(move || selector.clone())
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .px(px(10.))
                        .py(px(4.))
                        .child(icon(caret, 9., t.fg_dim))
                        .child(file_status::badge(file.hunk_type, 12., t))
                        .child(
                            div()
                                .min_w_0()
                                .truncate()
                                .code_text(11., t)
                                .child(path.clone()),
                        )
                        .when(self.sheet.is_interactive(), |row| {
                            row.cursor_pointer()
                                .hover(|style| style.bg(rgb(t.row_alt_bg)))
                                .on_click(cx.listener(move |view, _, _, cx| {
                                    cx.stop_propagation();
                                    view.toggle_converge_file(path.clone(), cx);
                                }))
                        })
                }))
                .into_any_element()
        });
        let diff = contents.selected_path.as_ref().map(|_| {
            loaded(&contents.diff, t, |diff| {
                let lines = diff.lines.len().clamp(1, MAX_DIFF_LINES);
                div()
                    .flex()
                    .h(px(t.code_line_height() * lines as f32))
                    .border_t_1()
                    .border_color(rgb(t.row_border))
                    .child(read_only_diff("converge-diff", diff.clone(), t, cx))
                    .into_any_element()
            })
        });
        let conflicts = self.review.expected_conflicts().map(|paths| {
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .p(px(10.))
                .bg(rgb(t.tag_conflict_bg))
                .child(
                    div()
                        .code_text(11., t)
                        .whitespace_normal()
                        .child(paths.join("\n")),
                )
                .child(caption(jayjay_i18n::converge::conflicts_followup(), t))
        });
        div()
            .flex()
            .flex_col()
            .child(files)
            .children(diff)
            .children(conflicts)
            .into_any_element()
    }

    pub(super) fn same_rows(&self) -> Option<AnyElement> {
        let (review, t) = (self.review, self.t);
        let items = review.same_items();
        let first = review.versions().first()?;
        if items.is_empty() {
            return None;
        }
        let label_width = t.scaled_font_size(80.);
        let rows = items.into_iter().enumerate().map(|(index, item)| {
            let value = match item {
                ConvergeItem::Description => first_line(&first.description),
                _ => version_value(first, item),
            };
            div()
                .debug_selector(move || format!("converge-same-{}", item.slug()))
                .flex()
                .items_center()
                .gap(px(10.))
                .px(px(10.))
                .py(px(7.))
                .when(index > 0, |row| {
                    row.border_t_1().border_color(rgb(t.row_border))
                })
                .child(icon(glyph::CHECK, 11., t.success_fg))
                .child(title(item).w(px(label_width)).flex_none())
                .child(value_text(value, item, t).flex_1().min_w_0().truncate())
                .child(caption(
                    jayjay_i18n::converge::same(review.versions().len()),
                    t,
                ))
        });
        Some(card_frame(t).children(rows).into_any_element())
    }
}

fn title(item: ConvergeItem) -> Div {
    div()
        .text_size(ui_font_size(12.))
        .font_weight(FontWeight::SEMIBOLD)
        .child(item.title())
}

fn version_value(version: &ConvergeVersion, item: ConvergeItem) -> String {
    match item {
        ConvergeItem::Description => version.description.trim().to_owned(),
        ConvergeItem::Author => format!(
            "{} <{}> · {}",
            version.author.name,
            version.author.email,
            format_when(version.author.timestamp_millis)
        ),
        ConvergeItem::Parents => version
            .parents
            .iter()
            .map(|id| id.chars().take(12).collect::<String>())
            .collect::<Vec<_>>()
            .join(", "),
        ConvergeItem::Contents => String::new(),
    }
}

pub(super) fn value_text(value: String, item: ConvergeItem, t: &Theme) -> Div {
    let text = div()
        .text_size(ui_font_size(12.))
        .when(item == ConvergeItem::Parents, |text| text.code_text(12., t));
    if value.is_empty() {
        text.text_color(rgb(t.fg_dim))
            .child(jayjay_i18n::converge::no_description())
    } else {
        text.child(value)
    }
}
