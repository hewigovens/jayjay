use gpui::{
    AnyView, App, AppContext, BoxShadow, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, hsla, px, rgb,
};

use crate::app::theme::{Theme, theme, ui_font_size};

pub(crate) fn text_tooltip(
    label: impl Into<SharedString>,
) -> impl Fn(&mut Window, &mut App) -> AnyView {
    let label = label.into();
    move |_, cx| {
        cx.new(|_| TextTooltip {
            label: label.clone(),
        })
        .into()
    }
}

struct TextTooltip {
    label: SharedString,
}

impl Render for TextTooltip {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = theme(cx);
        div().pl(px(6.)).pt(px(8.)).child(
            div()
                .px(px(8.))
                .py(px(4.))
                .rounded_sm()
                .bg(rgb(theme.detail_bg))
                .border_1()
                .border_color(rgb(theme.border))
                .shadow(tooltip_shadow(theme))
                .max_w(px(480.))
                .text_size(ui_font_size(11.))
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
