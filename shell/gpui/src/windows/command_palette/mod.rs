mod actions;
mod render;

use std::process::Command;

use gpui::{
    App, AppContext, Bounds, Context, FocusHandle, Focusable, InteractiveElement, IntoElement,
    KeyDownEvent, ParentElement, Render, SharedString, Styled, TitlebarOptions, Window,
    WindowBounds, WindowKind, WindowOptions, div, px, rgb, size,
};

use crate::app::config::AppConfigStore;
use crate::app::theme::{Theme, theme};

use actions::{ACTIONS, PaletteCtx};
use render::{action_list, command_view, divider, query_box};

pub struct CommandPalette {
    query: String,
    selected: usize,
    focus_handle: FocusHandle,
    repo_path: SharedString,
    output: CommandOutput,
}

#[derive(Clone)]
pub(super) enum CommandOutput {
    Idle,
    Running { display: String },
    Done {
        display: String,
        stdout: String,
        stderr: String,
        success: bool,
    },
}

impl CommandPalette {
    pub fn open(repo_path: SharedString, cx: &mut App) {
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
                    cx.new(|cx| {
                        cx.observe_global::<AppConfigStore>(|_, cx| cx.notify())
                            .detach();
                        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                        Self {
                            query: String::new(),
                            selected: 0,
                            focus_handle: cx.focus_handle(),
                            repo_path,
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

    /// Body of a command-mode query. Returns the args after a `jj` or `!`
    /// prefix; `None` when the palette is in action-search mode. Both prefixes
    /// resolve to a `jj` subcommand — `!` is a typing-shorthand alias, not a
    /// generic shell escape (matches the SwiftUI app).
    fn parse_command(&self) -> Option<String> {
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

    fn run_command(&mut self, body: String, cx: &mut Context<Self>) {
        if body.is_empty() {
            return;
        }
        let display = format!("jj {body}");
        self.output = CommandOutput::Running {
            display: display.clone(),
        };
        cx.notify();
        let cwd = self.repo_path.to_string();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { execute(body, &cwd, display) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.output = result;
                cx.notify();
            });
        })
        .detach();
    }

    fn on_key(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
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

        let body = match (&self.output, self.parse_command()) {
            (CommandOutput::Idle, None) => action_list(&visible, selected, &t).into_any_element(),
            (CommandOutput::Idle, Some(body)) => {
                let cmd = format!("jj {body}");
                command_view(None, cmd.trim_end(), &t).into_any_element()
            }
            (out, _) => command_view(Some(out), "", &t).into_any_element(),
        };

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
            .child(body)
    }
}

fn execute(body: String, cwd: &str, display: String) -> CommandOutput {
    let mut cmd = Command::new("/bin/sh");
    cmd.arg("-c").arg(format!("jj {body}")).current_dir(cwd);
    match cmd.output() {
        Ok(out) => CommandOutput::Done {
            display,
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            success: out.status.success(),
        },
        Err(e) => CommandOutput::Done {
            display,
            stdout: String::new(),
            stderr: format!("failed to spawn: {e}"),
            success: false,
        },
    }
}
