use super::activation::Activation;
use super::artwork::Artwork;
use super::icon::FlockIcon;
use crate::app::theme::{Theme, ui_font_size};
use crate::app::{config, links};
use crate::ui::icons::{self, glyph};
use crate::ui::overlay::{overlay_actions, overlay_card, overlay_layer};
use crate::ui::{logo::Logo, text_area::TextArea};
use gpui::{
    Animation, AnimationExt, AnyElement, App, AppContext, ClipboardItem, Context, Div, Entity,
    Focusable, FontWeight, InteractiveElement, IntoElement, KeyDownEvent, ParentElement, Render,
    Role, SharedString, Stateful, StatefulInteractiveElement, Styled, Window, div, img,
    prelude::FluentBuilder as _, px, rgb,
};
use jayjay_core::flock::{FlockMember, FlockStore};
use jayjay_i18n::{common, settings};
use std::{rc::Rc, time::Duration};

const FEATHER_BG: u32 = 0xFAF2DE;
const FEATHER_FG: u32 = 0x1F3861;

pub(super) enum Sheet {
    Code,
    Activation(Activation),
}

pub(crate) struct FlockView {
    pub(super) member: Option<FlockMember>,
    loaded: bool,
    pub(super) sheet: Option<Sheet>,
    pub(super) message: Option<String>,
    pub(super) busy: bool,
    pub(super) code: Entity<TextArea>,
    pub(super) name: Entity<TextArea>,
    artwork: Artwork,
    logo: Logo,
    replay: usize,
}

impl FlockView {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let code = cx.new(|cx| TextArea::new("", settings::flock_code_field(), false, 30., cx));
        let name = cx.new(|cx| TextArea::new("", settings::flock_code_name(), false, 30., cx));
        TextArea::subscribe_updates(&code, cx);
        TextArea::subscribe_updates(&name, cx);
        cx.spawn(async |this, cx| {
            let member = cx
                .background_spawn(async { FlockStore::load().member() })
                .await;
            let _ = this.update(cx, |view, cx| {
                view.member = member;
                view.loaded = true;
                cx.notify();
            });
        })
        .detach();
        Self {
            member: None,
            loaded: false,
            sheet: None,
            message: None,
            busy: false,
            code,
            name,
            artwork: Artwork::load(cx),
            logo: Logo::load(cx),
            replay: 0,
        }
    }

    fn unlocked(&self) -> bool {
        self.loaded
            && (self.member.is_some()
                || cfg!(debug_assertions) && std::env::var_os("JAYJAY_FLOCK_LOCKED").is_none())
    }

    fn active_icon(&self, cx: &App) -> FlockIcon {
        if self.unlocked() {
            FlockIcon::from_id(&config::current(cx).flock_icon)
        } else {
            FlockIcon::Classic
        }
    }

    pub(crate) fn icon(&self, size: f32, t: &Theme, cx: &App) -> AnyElement {
        let icon = self.active_icon(cx);
        div()
            .debug_selector(move || format!("flock-active-{}", icon.id()))
            .child(self.bird(icon, size, t))
            .into_any_element()
    }

    fn bird(&self, icon: FlockIcon, size: f32, t: &Theme) -> AnyElement {
        self.artwork
            .bird(icon, t.is_dark, size)
            .unwrap_or_else(|| self.logo.app_icon(size, t))
    }

    pub(super) fn code_ready(&self, cx: &App) -> bool {
        !self.code.read(cx).text().trim().is_empty() && !self.name.read(cx).text().trim().is_empty()
    }

    fn open_code(&mut self, cx: &mut Context<Self>) {
        self.sheet = Some(Sheet::Code);
        self.message = None;
        cx.notify();
    }

    pub(super) fn close_sheet(&mut self, cx: &mut Context<Self>) {
        if matches!(self.sheet, Some(Sheet::Code)) {
            self.message = None;
            self.code.update(cx, |field, cx| field.clear(cx));
            self.name.update(cx, |field, cx| field.clear(cx));
        }
        self.sheet = None;
        cx.notify();
    }

    pub(crate) fn dismiss(&mut self, cx: &mut Context<Self>) -> bool {
        if self.sheet.is_none() || self.busy {
            return self.busy;
        }
        self.close_sheet(cx);
        true
    }

    pub(crate) fn handle_tab(&self, backward: bool, window: &mut Window, cx: &mut App) {
        if matches!(self.sheet, Some(Sheet::Code)) {
            let next = if self.code.focus_handle(cx).is_focused(window) {
                &self.name
            } else {
                &self.code
            };
            window.focus(&next.focus_handle(cx), cx);
        } else if backward {
            window.focus_prev(cx);
        } else {
            window.focus_next(cx);
        }
    }

    fn action(
        control: Stateful<Div>,
        cx: &Context<Self>,
        action: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> Stateful<Div> {
        let action = Rc::new(action);
        let click = action.clone();
        control
            .tab_index(0)
            .role(Role::Button)
            .on_click(cx.listener(move |view, _, _, cx| click(view, cx)))
            .on_key_down(cx.listener(move |view, event: &KeyDownEvent, _, cx| {
                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                    cx.stop_propagation();
                    action(view, cx);
                }
            }))
    }

    fn display_name(&self) -> String {
        match &self.member {
            Some(member) => member
                .name
                .as_deref()
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .unwrap_or(&member.login)
                .to_owned(),
            None if self.unlocked() => settings::flock_debug_build(),
            None => settings::flock_your_feather(),
        }
    }

    fn header(&self, t: &Theme, cx: &Context<Self>) -> Div {
        let mut row = div().flex().items_center().gap(px(8.)).child(
            div()
                .flex_1()
                .text_size(ui_font_size(24.))
                .font_weight(FontWeight::BOLD)
                .child(settings::flock_title()),
        );
        if self.member.is_some() {
            row = row.child(Self::action(
                link("flock-forget", settings::flock_forget(), t.fg_dim)
                    .debug_selector(|| "flock-forget".to_owned()),
                cx,
                |view, cx| view.forget(cx),
            ));
        } else if self.unlocked() {
            row = row
                .child(Self::action(
                    small_button("flock-verify", settings::flock_verify(), t),
                    cx,
                    |view, cx| view.open_activation(cx),
                ))
                .child(Self::action(
                    small_button("flock-code", settings::flock_code_enter(), t)
                        .debug_selector(|| "flock-code".to_owned()),
                    cx,
                    |view, cx| view.open_code(cx),
                ));
        }
        let mut header = div().flex().flex_col().gap(px(6.)).child(row);
        if self.unlocked() {
            header = header.child(dim(
                if self.member.is_some() {
                    settings::flock_thanks()
                } else {
                    settings::flock_debug_detail()
                },
                12.,
                t,
            ));
        }
        if self.sheet.is_none()
            && let Some(message) = &self.message
        {
            header = header.child(error(message.clone(), t));
        }
        header
    }

    fn feather(&self, interactive: bool, cx: &mut Context<Self>) -> AnyElement {
        let name = self.display_name();
        let card = div()
            .id("flock-feather")
            .aria_label(name.clone())
            .aria_description(settings::flock_replay())
            .debug_selector(|| "flock-feather".to_owned())
            .flex()
            .flex_col()
            .items_center()
            .gap(px(8.))
            .p(px(20.))
            .w(px(230.))
            .max_w_full()
            .rounded_lg()
            .bg(rgb(FEATHER_BG))
            .text_color(rgb(FEATHER_FG))
            .shadow_md()
            .child(
                div()
                    .text_size(ui_font_size(11.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(settings::flock_title()),
            )
            .child(img(self.artwork.feather.clone()).size(px(130.)))
            .child(
                div()
                    .w_full()
                    .min_w_0()
                    .text_size(ui_font_size(19.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_center()
                    .line_clamp(2)
                    .child(name),
            )
            .child(
                div()
                    .text_size(ui_font_size(9.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(settings::flock_feather_title()),
            );
        if !interactive {
            return card.into_any_element();
        }
        Self::action(card.cursor_pointer(), cx, |view, cx| {
            view.replay += 1;
            cx.notify();
        })
        .with_animation(
            ("flock-replay", self.replay),
            Animation::new(crate::app::motion::animation_duration(
                cx,
                Duration::from_millis(650),
            )),
            |card, delta| card.opacity(1. - 0.2 * (delta * std::f32::consts::PI).sin()),
        )
        .into_any_element()
    }

    fn picker(&self, interactive: bool, t: &Theme, cx: &mut Context<Self>) -> Div {
        let active = self.active_icon(cx);
        let tiles = FlockIcon::ALL.into_iter().map(|icon| {
            let bird = icon.id();
            let selected = icon == active;
            let tile =
                div()
                    .id(SharedString::from(format!("flock-icon-{bird}")))
                    .debug_selector(move || format!("flock-icon-{bird}"))
                    .aria_label(icon.title())
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(6.))
                    .w(px(88.))
                    .p(px(6.))
                    .rounded_lg()
                    .when(selected, |tile| tile.bg(rgb(t.selected_bg)))
                    .child(div().relative().child(self.bird(icon, 72., t)).when(
                        selected,
                        |image| {
                            image.child(
                                div()
                                    .absolute()
                                    .top(px(-2.))
                                    .right(px(-2.))
                                    .child(icons::icon(glyph::CHECK, 14., t.selected_accent)),
                            )
                        },
                    ))
                    .child(
                        div()
                            .text_size(ui_font_size(11.))
                            .text_center()
                            .child(icon.title()),
                    );
            if interactive {
                Self::action(tile.cursor_pointer(), cx, move |_, cx| {
                    config::update(cx, |cfg| cfg.flock_icon = bird.to_owned());
                    cx.notify();
                })
            } else {
                tile
            }
        });
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .w_full()
            .p(px(16.))
            .rounded_lg()
            .bg(rgb(t.row_alt_bg))
            .child(
                div()
                    .text_size(ui_font_size(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(settings::flock_about_icon()),
            )
            .child(div().flex().flex_wrap().gap(px(12.)).children(tiles))
            .child(dim(settings::flock_about_detail(), 11., t))
    }

    pub(crate) fn gift(&self, t: &Theme, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.loaded || self.unlocked() {
            return None;
        }
        let overlay = div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .flex()
            .items_center()
            .justify_center()
            .p(px(24.))
            .child(
                div()
                    .debug_selector(|| "flock-gift".to_owned())
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(12.))
                    .w(px(360.))
                    .max_w_full()
                    .p(px(24.))
                    .rounded(px(16.))
                    .border_1()
                    .border_color(rgb(t.border))
                    .bg(rgb(t.header_bg))
                    .shadow_lg()
                    .text_center()
                    .child(icons::icon(glyph::GIFT, 24., t.selected_accent))
                    .child(
                        div()
                            .text_size(ui_font_size(19.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(settings::flock_claim()),
                    )
                    .child(dim(settings::flock_gift_detail(), 12., t))
                    .child(Self::action(
                        button("flock-code", settings::flock_code_enter(), t, true)
                            .debug_selector(|| "flock-code".to_owned()),
                        cx,
                        |view, cx| view.open_code(cx),
                    ))
                    .child(Self::action(
                        link("flock-verify", settings::flock_unlock(), t.selected_accent),
                        cx,
                        |view, cx| view.open_activation(cx),
                    )),
            );
        Some(overlay.into_any_element())
    }

    pub(crate) fn sheet(&self, t: &Theme, cx: &mut Context<Self>) -> Option<AnyElement> {
        let card = match self.sheet.as_ref()? {
            Sheet::Code => self.code_sheet(t, cx),
            Sheet::Activation(activation) => self.activation_sheet(activation, t, cx),
        };
        Some(overlay_layer().child(card).into_any_element())
    }

    fn code_sheet(&self, t: &Theme, cx: &mut Context<Self>) -> Div {
        let unlock = button("flock-redeem", settings::flock_code_unlock(), t, true)
            .debug_selector(|| "flock-redeem".to_owned());
        let unlock = if self.code_ready(cx) && !self.busy {
            Self::action(unlock, cx, |view, cx| view.redeem(cx))
        } else {
            unlock.opacity(0.45).cursor_default()
        };
        let mut card = sheet_card(settings::flock_code_title(), t)
            .child(dim(settings::flock_code_detail(), 12., t))
            .child(field("flock-code-input", &self.code, cx))
            .child(field("flock-name-input", &self.name, cx));
        if let Some(message) = &self.message {
            card = card.child(error(message.clone(), t));
        }
        card.child(sponsor_link(t, cx)).child(overlay_actions(
            Self::action(
                button("flock-code-cancel", common::cancel(), t, false)
                    .debug_selector(|| "flock-code-cancel".to_owned()),
                cx,
                |view, cx| view.close_sheet(cx),
            ),
            unlock,
        ))
    }

    fn activation_sheet(&self, activation: &Activation, t: &Theme, cx: &mut Context<Self>) -> Div {
        let mut card = sheet_card(settings::flock_claim(), t)
            .child(dim(settings::flock_gift_detail(), 12., t))
            .child(dim(settings::flock_privacy_detail(), 11., t));
        if let Some((code, _)) = &activation.sign_in {
            card = card.child(
                div()
                    .text_size(ui_font_size(22.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .font_family(crate::app::fonts::mono())
                    .child(code.clone()),
            );
        }
        if activation.busy {
            card = card.child(dim(
                if activation.sign_in.is_some() {
                    settings::waiting_for_approval()
                } else {
                    settings::flock_checking()
                },
                11.,
                t,
            ));
        }
        if let Some(message) = &activation.message {
            card = card.child(div().whitespace_normal().child(message.clone()));
        }
        if let Some(message) = &activation.error {
            card = card.child(error(message.clone(), t));
        }
        if !activation.busy {
            card = card
                .child(Self::action(
                    small_button("flock-other-account", settings::flock_another_account(), t),
                    cx,
                    |view, cx| view.verify(true, cx),
                ))
                .child(sponsor_link(t, cx));
        }
        let confirm = match &activation.sign_in {
            Some((code, uri)) => {
                let (code, uri) = (code.clone(), uri.clone());
                Self::action(
                    button("flock-open", settings::copy_code_and_open(), t, true),
                    cx,
                    move |_, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(code.clone()));
                        links::open_url(cx, &uri);
                    },
                )
            }
            None => {
                let verify = button("flock-verify-confirm", settings::flock_verify(), t, true);
                if activation.busy {
                    verify.opacity(0.45).cursor_default()
                } else {
                    Self::action(verify, cx, |view, cx| view.verify(false, cx))
                }
            }
        };
        card.child(overlay_actions(
            Self::action(
                button("flock-cancel", common::cancel(), t, false),
                cx,
                |view, cx| view.close_sheet(cx),
            ),
            confirm,
        ))
    }
}

impl Render for FlockView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = crate::app::theme::theme_for_window(window, cx).clone();
        let page = div()
            .debug_selector(|| "settings-flock".to_owned())
            .text_size(ui_font_size(12.));
        if !self.loaded {
            return page.child(common::checking());
        }
        let unlocked = self.unlocked();
        let mut content = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(20.))
            .child(self.header(&t, cx).w_full())
            .child(self.feather(unlocked, cx));
        if self.member.is_some() {
            content = content.child(dim(settings::flock_keep(), 11., &t));
        }
        content = content.child(self.picker(unlocked, &t, cx));
        page.child(if unlocked {
            content
        } else {
            content.opacity(0.35)
        })
    }
}

fn button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    t: &Theme,
    primary: bool,
) -> Stateful<Div> {
    let label = label.into();
    crate::ui::primitives::button(id, label.clone(), t, primary).aria_label(label)
}

fn small_button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    t: &Theme,
) -> Stateful<Div> {
    let label = label.into();
    crate::ui::primitives::small_button(id, label.clone(), t).aria_label(label)
}

fn link(id: impl Into<SharedString>, label: impl Into<SharedString>, color: u32) -> Stateful<Div> {
    let label = label.into();
    div()
        .id(id.into())
        .aria_label(label.clone())
        .text_size(ui_font_size(11.))
        .text_color(rgb(color))
        .cursor_pointer()
        .hover(|s| s.underline())
        .child(label)
}

fn sponsor_link(t: &Theme, cx: &Context<FlockView>) -> Stateful<Div> {
    FlockView::action(
        link(
            "flock-sponsor",
            settings::flock_sponsor(),
            t.selected_accent,
        ),
        cx,
        |_, cx| links::open_url(cx, links::SPONSOR_URL),
    )
}

fn sheet_card(title: impl Into<SharedString>, t: &Theme) -> Div {
    overlay_card(t, 420.).child(
        div()
            .text_size(ui_font_size(14.))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(rgb(t.fg))
            .child(title.into()),
    )
}

fn field(selector: &'static str, input: &Entity<TextArea>, cx: &App) -> Stateful<Div> {
    div()
        .id(selector)
        .track_focus(&input.focus_handle(cx))
        .debug_selector(move || selector.to_owned())
        .child(input.clone())
}

fn dim(text: impl Into<SharedString>, size: f32, t: &Theme) -> Div {
    div()
        .text_size(ui_font_size(size))
        .text_color(rgb(t.fg_dim))
        .whitespace_normal()
        .child(text.into())
}

fn error(text: String, t: &Theme) -> Div {
    div()
        .debug_selector(|| "flock-error".to_owned())
        .text_size(ui_font_size(11.))
        .text_color(rgb(t.error_fg))
        .whitespace_normal()
        .child(text)
}
