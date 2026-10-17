use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Bounds, Context, Entity, Focusable, InteractiveElement, IntoElement,
    KeyDownEvent, ParentElement, Render, SharedString, StatefulInteractiveElement, Styled,
    TitlebarOptions, Window, WindowBounds, WindowOptions, div, px, rgb, size,
};
use jayjay_core::UserIdentity;

use crate::app::actions::{CloseWindow, Dismiss};
use crate::app::fonts::CodeText as _;
use crate::app::theme::{Theme, observe_window_appearance, ui_font_size};
use crate::repo::RepoWindow;
use crate::ui::primitives::button;
use crate::ui::text_area::{Newline, TextArea};

type OnSaved = Box<dyn FnOnce(&UserIdentity, &mut App)>;

pub struct IdentityDialog {
    name: Entity<TextArea>,
    email: Entity<TextArea>,
    /// Set once the prefill lands; the fields stay read-only until then.
    path: Option<SharedString>,
    error: Option<SharedString>,
    saving: bool,
    on_saved: Vec<OnSaved>,
}

impl IdentityDialog {
    /// Focuses the dialog when it is already open; every opener's `on_saved` runs after a save.
    pub(crate) fn open<V: 'static>(
        cx: &mut Context<V>,
        on_saved: fn(&mut V, &UserIdentity, &mut Context<V>),
    ) {
        let opener = cx.entity().downgrade();
        let on_saved: OnSaved = Box::new(move |identity, cx| {
            let _ = opener.update(cx, |view, cx| {
                on_saved(view, identity, cx);
                cx.notify();
            });
        });
        if let Some(open) = cx
            .windows()
            .into_iter()
            .find_map(|handle| handle.downcast::<Self>())
        {
            let _ = open.update(cx, |dialog, window, _| {
                dialog.on_saved.push(on_saved);
                window.activate_window();
            });
            return;
        }
        let bounds = Bounds::centered(None, size(px(440.), px(250.)), cx);
        let handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Set Up Identity".into()),
                        ..Default::default()
                    }),
                    ..crate::app::window_options()
                },
                |_, cx| cx.new(|cx| Self::new(on_saved, cx)),
            )
            .ok();
        if let Some(handle) = handle {
            let _ = handle.update(cx, |dialog, window, cx| {
                observe_window_appearance(window, cx);
                window.focus(&dialog.name.read(cx).focus_handle(cx), cx);
            });
        }
    }

    fn new(on_saved: OnSaved, cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        let field = |placeholder: &'static str, cx: &mut Context<Self>| {
            let input = cx.new(|cx| {
                let mut input = TextArea::new("", placeholder, false, 28., cx);
                input.set_read_only(true, cx);
                input
            });
            TextArea::subscribe_updates(&input, cx);
            input
        };
        cx.spawn(async move |this, cx| {
            let loaded = cx
                .background_spawn(async {
                    Ok::<_, jayjay_core::JayError>((
                        UserIdentity::suggested()?,
                        UserIdentity::config_path()?,
                    ))
                })
                .await;
            let _ = this.update(cx, |dialog, cx| {
                match loaded {
                    Ok((UserIdentity { name, email }, path)) => {
                        for (input, text) in [(&dialog.name, name), (&dialog.email, email)] {
                            input.update(cx, |input, cx| {
                                input.set_text(text, cx);
                                input.set_read_only(false, cx);
                            });
                        }
                        dialog.path = Some(path.into());
                    }
                    Err(error) => dialog.error = Some(error.to_string().into()),
                }
                cx.notify();
            });
        })
        .detach();
        Self {
            name: field("Your Name", cx),
            email: field("you@example.com", cx),
            path: None,
            error: None,
            saving: false,
            on_saved: vec![on_saved],
        }
    }

    fn identity(&self, cx: &App) -> Option<UserIdentity> {
        let name = self.name.read(cx).text().trim().to_owned();
        let email = self.email.read(cx).text().trim().to_owned();
        (self.path.is_some() && !name.is_empty() && !email.is_empty())
            .then_some(UserIdentity { name, email })
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.saving {
            return;
        }
        let Some(identity) = self.identity(cx) else {
            return;
        };
        self.saving = true;
        self.error = None;
        cx.notify();
        let window = window.window_handle();
        let on_saved = std::mem::take(&mut self.on_saved);
        cx.spawn(async move |this, cx| {
            let saved = {
                let identity = identity.clone();
                cx.background_spawn(async move { identity.save() }).await
            };
            cx.update(|cx| match saved {
                // Runs in the app context, so closing the dialog mid-save skips none of it.
                Ok(()) => {
                    // A refresh reloads each repo's settings too, so views such as `mine()` follow the new identity.
                    for handle in cx.windows() {
                        if let Some(repo) = handle.downcast::<RepoWindow>() {
                            let _ = repo.update(cx, |view, _, cx| {
                                view.view_model().update(cx, |vm, cx| vm.refresh(true, cx));
                            });
                        }
                    }
                    let added_meanwhile = this
                        .update(cx, |dialog, _| std::mem::take(&mut dialog.on_saved))
                        .unwrap_or_default();
                    for on_saved in on_saved.into_iter().chain(added_meanwhile) {
                        on_saved(&identity, cx);
                    }
                    let _ = window.update(cx, |_, window, _| window.remove_window());
                }
                Err(error) => {
                    let _ = this.update(cx, |dialog, cx| {
                        dialog.on_saved.splice(0..0, on_saved);
                        dialog.saving = false;
                        dialog.error = Some(error.to_string().into());
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    fn handle_tab(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let modifiers = &ev.keystroke.modifiers;
        if ev.keystroke.key != "tab" || modifiers.platform || modifiers.alt || modifiers.control {
            return;
        }
        let next = if self.name.read(cx).focus_handle(cx).is_focused(window) {
            &self.email
        } else {
            &self.name
        };
        window.focus(&next.read(cx).focus_handle(cx), cx);
        cx.stop_propagation();
    }
}

impl Render for IdentityDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = crate::app::theme::theme_for_window(window, cx).clone();
        let field = |id: &'static str, input: &Entity<TextArea>| {
            div()
                .id(id)
                .debug_selector(move || id.to_owned())
                .w_full()
                .child(input.clone())
        };
        let save =
            button("identity-save", "Save", &t, true).debug_selector(|| "identity-save".to_owned());
        let save = if !self.saving && self.identity(cx).is_some() {
            save.on_click(cx.listener(|dialog, _, window, cx| dialog.save(window, cx)))
        } else {
            save.opacity(0.45)
        };
        div()
            .font_family(crate::app::fonts::interface())
            .key_context("IdentityDialog")
            .on_action(cx.listener(|_, _: &CloseWindow, window, _| window.remove_window()))
            .on_action(cx.listener(|_, _: &Dismiss, window, _| window.remove_window()))
            .on_action(cx.listener(|dialog, _: &Newline, window, cx| dialog.save(window, cx)))
            .capture_key_down(cx.listener(Self::handle_tab))
            .flex()
            .flex_col()
            .size_full()
            .gap(px(10.))
            .p(px(20.))
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .child(
                div()
                    .text_size(ui_font_size(12.))
                    .text_color(rgb(t.fg_dim))
                    .child("jj records this name and email on the changes you create."),
            )
            .child(field("identity-name", &self.name))
            .child(field("identity-email", &self.email))
            .when_some(self.path.clone(), |root, path| {
                root.child(
                    div()
                        .truncate()
                        .code_font()
                        .text_size(ui_font_size(10.))
                        .text_color(rgb(t.fg_faint))
                        .child(path),
                )
            })
            .when_some(self.error.clone(), |root, error| {
                root.child(
                    div()
                        .text_size(ui_font_size(11.))
                        .text_color(rgb(t.error_fg))
                        .child(error),
                )
            })
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_end()
                    .gap(px(8.))
                    .child(
                        button("identity-cancel", "Cancel", &t, false)
                            .on_click(cx.listener(|_, _, window, _| window.remove_window())),
                    )
                    .child(save),
            )
    }
}
