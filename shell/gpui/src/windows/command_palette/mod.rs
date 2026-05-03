mod actions;
mod render;

use gpui::{
    App, AppContext, Bounds, Context, FocusHandle, Focusable, InteractiveElement, IntoElement,
    KeyDownEvent, ParentElement, Render, SharedString, Styled, TitlebarOptions, Window,
    WindowBounds, WindowKind, WindowOptions, div, px, rgb, size,
};

use crate::app::config::AppConfigStore;
use crate::app::theme::{Theme, theme};

use actions::{ACTIONS, PaletteCtx};
use render::{action_list, divider, query_box};

pub struct CommandPalette {
    query: String,
    selected: usize,
    focus_handle: FocusHandle,
    repo_path: SharedString,
}

impl CommandPalette {
    pub fn open(repo_path: SharedString, cx: &mut App) {
        let bounds = Bounds::centered(None, size(px(560.), px(420.)), cx);
        let handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("JayJay Command Palette".into()),
                        ..Default::default()
                    }),
                    kind: WindowKind::PopUp,
                    ..Default::default()
                },
                |_, cx| {
                    let repo_path = repo_path.clone();
                    cx.new(|cx| {
                        cx.observe_global::<AppConfigStore>(|_, cx| cx.notify())
                            .detach();
                        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                        Self {
                            query: String::new(),
                            selected: 0,
                            focus_handle: cx.focus_handle(),
                            repo_path,
                        }
                    })
                },
            )
            .ok();
        if let Some(h) = handle {
            let _ = h.update(cx, |view, window, cx| {
                let f = view.focus_handle(cx);
                window.focus(&f, cx);
            });
        }
    }

    fn matches(&self) -> Vec<usize> {
        let q = self.query.trim().to_lowercase();
        if q.is_empty() {
            return (0..ACTIONS.len()).collect();
        }
        ACTIONS
            .iter()
            .enumerate()
            .filter(|(_, a)| {
                a.name.to_lowercase().contains(&q)
                    || a.keywords.iter().any(|k| k.to_lowercase().contains(&q))
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn on_key(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let key = ev.keystroke.key.as_str();
        let visible = self.matches();

        match key {
            "escape" => {
                window.remove_window();
                return;
            }
            "down" => {
                if !visible.is_empty() {
                    self.selected = (self.selected + 1).min(visible.len() - 1);
                }
            }
            "up" => {
                self.selected = self.selected.saturating_sub(1);
            }
            "enter" => {
                if let Some(&action_ix) = visible.get(self.selected) {
                    let dispatch = ACTIONS[action_ix].dispatch;
                    let ctx = PaletteCtx {
                        repo_path: self.repo_path.as_ref(),
                    };
                    window.remove_window();
                    dispatch(&ctx, cx);
                    return;
                }
            }
            "backspace" => {
                self.query.pop();
                self.selected = 0;
            }
            _ => {
                if let Some(c) = ev.keystroke.key_char.as_ref() {
                    let m = &ev.keystroke.modifiers;
                    if !m.platform && !m.control && !m.alt {
                        self.query.push_str(c);
                        self.selected = 0;
                    }
                }
            }
        }
        cx.notify();
    }
}

impl Focusable for CommandPalette {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for CommandPalette {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx).clone();
        let visible = self.matches();
        let selected = self.selected.min(visible.len().saturating_sub(1));

        div()
            .track_focus(&self.focus_handle)
            .key_context("CommandPalette")
            .on_key_down(cx.listener(Self::on_key))
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .child(query_box(&self.query, &t))
            .child(divider(&t))
            .child(action_list(&visible, selected, &t))
    }
}
