use gpui::{
    App, AppContext, Bounds, Context, Entity, Focusable, KeyDownEvent, SharedString,
    TitlebarOptions, Window, WindowBounds, WindowKind, WindowOptions, px, size,
};

use super::actions::{ACTIONS, PaletteCtx};
use super::state::{CommandOutput, CommandPalette};
use crate::app::config::AppConfigStore;
use crate::app::theme::Theme;
use crate::log::LogView;

impl CommandPalette {
    pub fn open(repo_path: SharedString, log_view: Option<Entity<LogView>>, cx: &mut App) {
        let bounds = Bounds::centered(None, size(px(640.), px(480.)), cx);
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
                    let log_view = log_view.clone();
                    cx.new(|cx| {
                        cx.observe_global::<AppConfigStore>(|_, cx| cx.notify())
                            .detach();
                        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                        Self {
                            query: String::new(),
                            selected: 0,
                            focus_handle: cx.focus_handle(),
                            repo_path,
                            log_view,
                            output: CommandOutput::Idle,
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

    // `!` is a shorthand alias for `jj `, matching SwiftUI behavior.
    pub(super) fn parse_command(&self) -> Option<String> {
        let q = self.query.as_str();
        let body_after = |rest: &str| rest.trim_start().to_string();
        if q == "jj" || q == "!" {
            return Some(String::new());
        }
        if let Some(rest) = q.strip_prefix("jj ") {
            return Some(body_after(rest));
        }
        if let Some(rest) = q.strip_prefix('!') {
            return Some(body_after(rest));
        }
        None
    }

    pub(super) fn matches(&self) -> Vec<usize> {
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

    pub(super) fn on_key(
        &mut self,
        ev: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = ev.keystroke.key.as_str();
        let visible = self.matches();

        match key {
            "escape" => {
                if matches!(self.output, CommandOutput::Done { .. }) {
                    self.output = CommandOutput::Idle;
                    self.query.clear();
                } else {
                    window.remove_window();
                    return;
                }
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
                if let Some(body) = self.parse_command() {
                    self.run_command(body, cx);
                    return;
                }
                if let Some(&action_ix) = visible.get(self.selected) {
                    let dispatch = ACTIONS[action_ix].dispatch;
                    let ctx = PaletteCtx {
                        repo_path: self.repo_path.as_ref(),
                        log_view: self.log_view.clone(),
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
