use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use super::state::{JjCheckState, OnboardingPage, OnboardingState};
use super::widgets::{command_row, mono_line, tip};
use crate::app::theme::Theme;
use crate::repo::window::RepoWindow;
use crate::ui::icons::{self, glyph};
use crate::ui::logo::Logo;
use crate::ui::primitives::button;

pub(super) fn onboarding_pane(
    state: &OnboardingState,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    div()
        .id("onboarding-pane")
        .debug_selector(|| "onboarding-pane".to_owned())
        .flex()
        .flex_1()
        .size_full()
        .flex_col()
        .items_center()
        .justify_between()
        .bg(rgb(t.detail_bg))
        .child(
            div()
                .flex()
                .flex_1()
                .w_full()
                .items_center()
                .justify_center()
                .child(page_content(state, t, cx)),
        )
        .child(page_indicator(state.page, t, cx))
        .child(footer(state.page, t, cx))
        .into_any_element()
}

fn page_content(state: &OnboardingState, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    match state.page {
        OnboardingPage::Welcome => welcome_page(&state.logo, t),
        OnboardingPage::JjCheck => jj_check_page(&state.jj, t, cx),
        OnboardingPage::Ready => ready_page(t),
    }
}

fn welcome_page(logo: &Logo, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(14.))
        .max_w(px(380.))
        .px(px(24.))
        .text_align(gpui::TextAlign::Center)
        .child(logo.image(88.))
        .child(
            div()
                .text_size(px(28.))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(t.fg))
                .child("Welcome to JayJay"),
        )
        .child(
            div()
                .text_size(px(14.))
                .line_height(px(20.))
                .text_color(rgb(t.fg_dim))
                .child("A native GUI for Jujutsu version control. Browse history, review diffs, and manage changes from one window."),
        )
        .into_any_element()
}

fn jj_check_page(jj: &JjCheckState, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    let mut root = div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(12.))
        .max_w(px(420.))
        .px(px(24.))
        .text_align(gpui::TextAlign::Center);

    match jj {
        JjCheckState::Checking => {
            root = root
                .child(icons::icon(glyph::ARROW_CLOCKWISE, 44., t.fg_dim))
                .child(
                    div()
                        .text_size(px(18.))
                        .text_color(rgb(t.fg))
                        .child("Checking for jj..."),
                );
        }
        JjCheckState::Loaded(status) if status.is_installed => {
            root = root
                .child(icons::icon(glyph::CHECK, 48., t.success_fg))
                .child(
                    div()
                        .text_size(px(22.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(t.fg))
                        .child("Jujutsu is installed"),
                );
            if !status.version.is_empty() {
                root = root.child(mono_line(status.version.clone(), 13., t.fg_dim));
            }
            if !status.path.is_empty() {
                root = root.child(mono_line(status.path.clone(), 11., t.fg_faint));
            }
        }
        JjCheckState::Loaded(_) => {
            root = root
                .child(icons::icon(glyph::WARNING, 48., t.compare_accent))
                .child(
                    div()
                        .text_size(px(22.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(t.fg))
                        .child("Jujutsu not found"),
                )
                .child(
                    div()
                        .text_size(px(14.))
                        .line_height(px(20.))
                        .text_color(rgb(t.fg_dim))
                        .child("JayJay requires jj to be installed. Install it with Homebrew or Cargo:"),
                )
                .child(command_row("brew install jj", t))
                .child(command_row("cargo install --locked jj-cli", t))
                .child(
                    button("onboarding-check-again", "Check Again", t, false)
                        .on_click(cx.listener(|view, _, _, cx| {
                            view.check_jj_for_onboarding(cx);
                        })),
                );
        }
    }

    root.into_any_element()
}

fn ready_page(t: &Theme) -> AnyElement {
    let multi_select_tip = if cfg!(target_os = "macos") {
        "Shift-click selects a range; ⌘-click toggles changes"
    } else {
        "Shift-click selects a range; Ctrl-click toggles changes"
    };
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(14.))
        .max_w(px(420.))
        .px(px(24.))
        .child(icons::icon(glyph::CHECK, 48., t.success_fg))
        .child(
            div()
                .text_size(px(22.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(t.fg))
                .child("You're all set"),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(10.))
                .pt(px(6.))
                .child(tip(
                    glyph::FOLDER,
                    "Open any jj repository to get started",
                    t,
                ))
                .child(tip(
                    glyph::SEARCH,
                    "Use the command palette to find actions",
                    t,
                ))
                .child(tip(
                    glyph::CHECK,
                    "Press Space to mark the selected file reviewed",
                    t,
                ))
                .child(tip(glyph::GIT_BRANCH, multi_select_tip, t))
                .child(tip(
                    glyph::WARNING,
                    "Close GitHub Desktop when working in jj repos",
                    t,
                )),
        )
        .into_any_element()
}

fn page_indicator(
    current: OnboardingPage,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> impl IntoElement {
    let mut row = div()
        .id("onboarding-page-indicator")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.));
    for page in OnboardingPage::all() {
        let active = page == current;
        row = row.child(
            div()
                .id(SharedString::from(format!("onboarding-dot-{}", page.id())))
                .w(px(if active { 8. } else { 6. }))
                .h(px(if active { 8. } else { 6. }))
                .rounded_full()
                .bg(rgb(if active {
                    t.selected_accent
                } else {
                    t.fg_faint
                }))
                .cursor_pointer()
                .on_click(cx.listener(move |view, _, _, cx| {
                    view.set_onboarding_page(page, cx);
                })),
        );
    }
    row
}

fn footer(page: OnboardingPage, t: &Theme, cx: &mut Context<RepoWindow>) -> impl IntoElement {
    let mut left = div().w(px(80.));
    if let Some(previous) = page.previous() {
        left = left.child(
            button("onboarding-back", "Back", t, false)
                .debug_selector(|| "onboarding-back".to_owned())
                .on_click(cx.listener(move |view, _, _, cx| {
                    view.set_onboarding_page(previous, cx);
                })),
        );
    }

    let right = if let Some(next) = page.next() {
        button("onboarding-next", "Next", t, true)
            .debug_selector(|| "onboarding-next".to_owned())
            .on_click(cx.listener(move |view, _, _, cx| {
                view.set_onboarding_page(next, cx);
            }))
    } else {
        button("onboarding-finish", "Get Started", t, true)
            .debug_selector(|| "onboarding-finish".to_owned())
            .on_click(cx.listener(|view, _, _, cx| {
                view.finish_onboarding(cx);
            }))
    };

    div()
        .id("onboarding-footer")
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .px(px(20.))
        .py(px(18.))
        .child(left)
        .child(right)
}
