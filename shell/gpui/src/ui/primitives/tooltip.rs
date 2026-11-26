use std::time::Duration;

use gpui::{
    App, AppContext, BoxShadow, Context, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Window, div, hsla, px, rgb,
};

use crate::app::theme::{Theme, theme, ui_size};
use crate::repo::window::RepoWindow;
use crate::windows::bookmark_manager::BookmarkManagerView;
use crate::windows::evolog::EvologView;
use crate::windows::overview::OverviewView;

/// About an AppKit help tag's delay; gpui's half second fires while the pointer is still travelling.
const HELP_TAG_DELAY: Duration = Duration::from_millis(1000);

pub(crate) trait HelpTag: StatefulInteractiveElement {
    fn help(self, label: impl Into<SharedString>) -> Self {
        self.help_tag(label.into(), true)
    }

    /// For a row of an open menu, which the menu must not hide.
    fn menu_help(self, label: impl Into<SharedString>) -> Self {
        self.help_tag(label.into(), false)
    }

    fn help_tag(self, label: SharedString, hides_under_menu: bool) -> Self {
        #[allow(clippy::disallowed_methods)]
        self.tooltip(move |_, cx| {
            cx.new(|_| TextTooltip {
                label: label.clone(),
                hides_under_menu,
            })
            .into()
        })
        .tooltip_show_delay(HELP_TAG_DELAY)
    }
}

impl<E: StatefulInteractiveElement> HelpTag for E {}

/// A window whose own menu covers what lies under it.
pub(crate) trait MenuHost: Render {
    fn menu_open(&self) -> bool;
}

fn menu_open(window: &Window, cx: &App) -> bool {
    fn open<V: MenuHost>(window: &Window, cx: &App) -> bool {
        window
            .root::<V>()
            .flatten()
            .is_some_and(|root| root.read(cx).menu_open())
    }
    open::<RepoWindow>(window, cx)
        || open::<EvologView>(window, cx)
        || open::<BookmarkManagerView>(window, cx)
        || open::<OverviewView>(window, cx)
}

struct TextTooltip {
    label: SharedString,
    hides_under_menu: bool,
}

impl Render for TextTooltip {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // gpui shows a pending tooltip even under a menu opened during its delay.
        if self.hides_under_menu && menu_open(window, cx) {
            return div();
        }
        let theme = theme(cx);
        div()
            .debug_selector(|| "help-tag".to_owned())
            .pl(ui_size(6.))
            .pt(ui_size(8.))
            .child(
                div()
                    .px(ui_size(8.))
                    .py(ui_size(4.))
                    .rounded_sm()
                    .bg(rgb(theme.detail_bg))
                    .border_1()
                    .border_color(rgb(theme.border))
                    .shadow(tooltip_shadow(theme))
                    .max_w(ui_size(480.))
                    .text_size(ui_size(11.))
                    .text_color(rgb(theme.fg))
                    .child(self.label.clone()),
            )
    }
}

fn tooltip_shadow(theme: &Theme) -> Vec<BoxShadow> {
    let (wide, tight) = if theme.is_dark {
        (0.26, 0.2)
    } else {
        (0.12, 0.06)
    };
    vec![
        BoxShadow::new(px(0.), px(6.), hsla(0., 0., 0., wide)).blur_radius(px(18.)),
        BoxShadow::new(px(0.), px(1.), hsla(0., 0., 0., tight)),
    ]
}
